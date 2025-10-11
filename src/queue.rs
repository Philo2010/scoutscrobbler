use std::fmt::format;
use std::str::FromStr;

use reqwest::header::HeaderMap;
use reqwest::{Client, Method};
use rocket::{data, form};
use rocket::{form::Form, http::CookieJar, State};
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::{PgPool, Row, Error};
use crate::blue::*;



#[derive(Debug, FromForm)]
struct QueueForm {
    #[field(name = "event")] event: String,
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
    pub teamNumber: i32,
    pub station: String,
}

pub async fn check_if_admin(userid_string: &str, pool: &State<PgPool>) -> Option<Template> {
    let userid = match Uuid::from_str(userid_string) {
        Ok(a) => a,
        Err(_) => {
            return Some(Template::render("error", context![error: "Your not logined!"]));
        },
    };


    let user_request = sqlx::query(r#"
        SELECT is_admin
        FROM user_list
        WHERE id = $1
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
    pool: &PgPool,
    event_code: &str,
    schedule: ScheduleData,
) -> Result<(), Error> {
    for m in schedule.Schedule {
        let row = sqlx::query(
            "INSERT INTO matches (event_code, match_number, description, tournament_level)
             VALUES ($1, $2, $3, $4)
             RETURNING id"
        )
        .bind(event_code)
        .bind(m.matchNumber)
        .bind(&m.description)
        .bind(&m.tournamentLevel)
        .fetch_one(pool)
        .await?;

        let match_id: i32 = row.try_get("id")?;

        for t in m.teams {
            sqlx::query(
                "INSERT INTO match_teams (match_id, team_number, station)
                 VALUES ($1, $2, $3)
                 ON CONFLICT DO NOTHING"
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

impl From<&TbaMatch> for Match {
    fn from(tba: &TbaMatch) -> Self {
        let mut teams = Vec::new();

        // Red teams
        for (i, team_key) in tba.alliances.red.team_keys.iter().enumerate() {
            if let Some(num) = team_key.strip_prefix("frc").and_then(|n| n.parse::<i32>().ok()) {
                teams.push(Team {
                    teamNumber: num,
                    station: format!("Red{}", i + 1),
                });
            }
        }

        // Blue teams
        for (i, team_key) in tba.alliances.blue.team_keys.iter().enumerate() {
            if let Some(num) = team_key.strip_prefix("frc").and_then(|n| n.parse::<i32>().ok()) {
                teams.push(Team {
                    teamNumber: num,
                    station: format!("Blue{}", i + 1),
                });
            }
        }

        let (formatted_level, match_number): (String, i32) = match tba.comp_level.as_str() {
            "qm" => ("Qualification".to_string(), tba.match_number),
            "sf" => ("Playoff".to_string(), tba.set_number),
            "f" => ("Playoff/Finals".to_string(), tba.match_number),
            "ef" => ("Playoff/Eighth-Finals".to_string(), tba.match_number),
            "qf" => ("Playoff/Quarterfinals".to_string(), tba.match_number), 
            _ => ("Playoff".to_string(), tba.set_number), // Fallback
        };


        Self {
            description: format!("{} {}", formatted_level, match_number),
            matchNumber: match_number,
            tournamentLevel: formatted_level,
            teams,
        }
    }
}

#[post("/queue", data = "<form_data>")]
pub async fn queue_form(client: &State<Client>, headers: &State<HeaderMap>, pool: &rocket::State<PgPool>, jar: &CookieJar<'_>, form_data: Form<QueueForm>) -> Template {

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

    let request = match pull_from_blue(client, headers, &form_data.event).await {
        Ok(a) => a,
        Err(a) => {
            println!("{:?}", a);
            return Template::render("error", context![error: "Database error!"]);
        },
    };

    let mut data: ScheduleData = ScheduleData {
        Schedule: request.iter().map(Match::from).collect::<Vec<_>>(),
    };

    //Sort
    data.Schedule.sort_by(|a, b| {
    a.tournamentLevel
        .cmp(&b.tournamentLevel)
        .then(a.matchNumber.cmp(&b.matchNumber))
    });
    
    match insert_schedule(pool.inner(), &form_data.event, data).await {
        Ok(_) => {},
        Err(a) => {
            println!("{:?}", a);
            return Template::render("error", context![error: "Database error!"]);
        },
    }

    Template::render("yippy", context! {})
}