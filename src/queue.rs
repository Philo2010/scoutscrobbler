use std::fmt::format;
use std::str::FromStr;

use reqwest::header::HeaderMap;
use reqwest::{Client, Method};
use rocket::{data, form};
use rocket::{form::Form, http::CookieJar, State};
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use sqlx::types::Uuid;



#[derive(Debug, FromForm)]
struct QueueForm {
    #[field(name = "event")] event: String,
}

#[derive(Debug, Deserialize)]
pub struct TbaMatch {
    pub comp_level: String,
    pub match_number: i32,
    pub alliances: Alliances,
}

#[derive(Debug, Deserialize)]
pub struct Alliances {
    pub red: Alliance,
    pub blue: Alliance,
}

#[derive(Debug, Deserialize)]
pub struct Alliance {
    pub team_keys: Vec<String>,
}

//Serde types for seralziatoin
#[derive(Debug, Deserialize)]
struct ScheduleData {
    Schedule: Vec<Match>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Match {
    // We omit `matchNumber` since we don't care about it
    pub description: String,
    pub matchNumber: i32,
    pub tournamentLevel: String,
    pub teams: Vec<Team>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Team {
    pub teamNumber: u32,
    pub station: String,
}

async fn check_if_admin(userid_string: &str, pool: &State<SqlitePool>) -> Option<Template> {
    let userid = match Uuid::from_str(userid_string) {
        Ok(a) => a,
        Err(_) => {
            return Some(Template::render("error", context![error: "Your not logined!"]));
        },
    };


    let user_request = sqlx::query(r#"
        SELECT is_admin
        FROM user_list
        WHERE id = ?
    "#)
    .bind(userid)
    .fetch_optional(pool.inner())
    .await; //TODO: fix make new user to get perms right


    let is_admin = match user_request {
        Ok(Some(a)) => {
            a.get::<bool, _>(0)
        },
        Ok(None) => {
            return Some(Template::render("error", context![error: "This user no longer exists!"]));
        }
        Err(_) => {
            return Some(Template::render("error", context![error: "This user no longer exists!"]));
        },
    };

    if !is_admin {
        return Some(Template::render("error", context![error: "You don't have perms"]));
    }
    None
}

pub async fn insert_schedule(
    pool: &SqlitePool,
    event_code: &str,
    schedule: ScheduleData,
) -> Result<(), sqlx::Error> {

    for m in schedule.Schedule {
        let row = sqlx::query(
            "INSERT INTO matches (event_code, match_number, description, tournament_level)
             VALUES (?, ?, ?, ?)
             RETURNING id"
        )
        .bind(event_code)
        .bind(m.matchNumber)
        .bind(&m.description)
        .bind(&m.tournamentLevel)
        .fetch_one(pool)
        .await?;

        let match_id: i64 = row.try_get("id")?;

        // 3. Insert teams and match_teams
        for t in m.teams {
            // Insert match-team relationship
            sqlx::query(
                "INSERT OR IGNORE INTO match_teams (match_id, team_number, station)
                 VALUES (?, ?, ?)"
            )
            .bind(match_id)
            .bind(t.teamNumber)
            .bind(&t.station)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

pub async fn pull_from_blue(client: &Client, headers: &HeaderMap, pool: &SqlitePool, event_code: &String) -> Result<Vec<TbaMatch>, reqwest::Error> {
    //https://www.thebluealliance.com/api/v3/event/2025tacy/matches/simple

    //Make a request to get the major data
    let request = client.get(format!("https://www.thebluealliance.com/api/v3/event/{}/matches/simple", event_code))
        .headers(headers.clone()).send().await?;

    let body: Vec<TbaMatch> = request.json().await?;
    

    Ok(body)
}

impl From<&TbaMatch> for Match {
    fn from(tba: &TbaMatch) -> Self {
        let mut teams = Vec::new();

        // Red teams
        for (i, team_key) in tba.alliances.red.team_keys.iter().enumerate() {
            if let Some(num) = team_key.strip_prefix("frc").and_then(|n| n.parse::<u32>().ok()) {
                teams.push(Team {
                    teamNumber: num,
                    station: format!("Red{}", i + 1),
                });
            }
        }

        // Blue teams
        for (i, team_key) in tba.alliances.blue.team_keys.iter().enumerate() {
            if let Some(num) = team_key.strip_prefix("frc").and_then(|n| n.parse::<u32>().ok()) {
                teams.push(Team {
                    teamNumber: num,
                    station: format!("Blue{}", i + 1),
                });
            }
        }
        let formated_level = match tba.comp_level.as_str() {
                "qm" => "Qualification".to_string(),
                "sf" => "Playoff".to_string(),
                _ => "Playoff".to_string() //Fallback
        };

        Self {
            description: format!("{} {}",formated_level, tba.match_number),
            matchNumber: tba.match_number,
            tournamentLevel: formated_level,
            teams,
        }
    }
}

#[post("/queue", data = "<form_data>")]
pub async fn queue_form(client: &State<Client>, headers: &State<HeaderMap>, pool: &rocket::State<SqlitePool>, jar: &CookieJar<'_>, form_data: Form<QueueForm>) -> Template {

    let userid_string = match jar.get("uuid") {
        Some(a) =>  a.value(),
        None => {
            return Template::render("error", context![error: "You are not logined in"]);
        },
    };

    match check_if_admin(userid_string, pool).await {
        Some(a) => {
            return a;
        },
        None => {},
    };

    //We now know the user is a admin
    //Pull data to make a requeast

    let request = match pull_from_blue(client, headers, pool, &form_data.event).await {
        Ok(a) => a,
        Err(_) => {
            return Template::render("error", context![error: "Database error!"]);
        },
    };

    let data: ScheduleData = ScheduleData {
        Schedule: request.iter().map(Match::from).collect::<Vec<_>>(),
    };
    
    match insert_schedule(pool.inner(), &form_data.event, data).await {
        Ok(_) => {},
        Err(a) => {
            println!("{:?}", a);
            return Template::render("error", context![error: "Database error!"]);
        },
    }

    Template::render("yippy", context! {})
}