use std::fmt::format;
use std::str::FromStr;

use reqwest::header::HeaderMap;
use reqwest::{Client, Method};
use rocket::{form::Form, http::CookieJar, State};
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use sqlx::types::Uuid;



#[derive(Debug, FromForm)]
struct QueueForm {
    #[field(name = "tournament_level")] tournament_level: String, //Must be ether "qual", or "Playoff" (Bootleg enum)
    #[field(name = "event")] event: String,
    #[field(name = "season")] season: i32,
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

    let url = format(format_args!("https://frc-api.firstinspires.org/v3.0/{}/schedule/{}?tournamentLevel={}", 
        &form_data.season,
        &form_data.event,
        &form_data.tournament_level));

    let request = client.inner()
    .request(Method::GET, url)
    .headers(headers.inner().clone());
    //Send that shit!
    let body = match request.send().await {
        Ok(a) => a,
        Err(_) => {
            return Template::render("error", context![error: "Form is invaild or api does not have this value yet"]);
        },
    };
    let data: ScheduleData = match body.json().await {
        Ok(a) => a,
        Err(_) => {
            return Template::render("error", context![error: "Responce was invaild"]);
        },
    };

    match insert_schedule(pool.inner(), &form_data.event, data).await {
        Ok(_) => {},
        Err(a) => {
            println!("{:?}", a);
            return Template::render("error", context![error: "Database error!"]);
        },
    }

    Template::render("error", context![error: "This is not an error: it worked!"])
}