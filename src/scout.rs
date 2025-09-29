use std::str::FromStr;

use rocket::http::CookieJar;
use rocket_dyn_templates::{context, Template};
use serde::Deserialize;
use serde::Serialize;
use sqlx::{SqlitePool, Row};
use uuid::Uuid;
use std::collections::BTreeMap;

use crate::queue::Match;
use crate::queue::Team;

use sqlx::FromRow;

#[derive(Debug, FromRow)]
struct FlatMatchRow {
    match_id: i64,
    description: String,
    match_number: i32,
    event_code: String,
    tournament_level: String,
    team_number: i32,
    team_id: i32,
    station: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MatchFull {
    // We omit `matchNumber` since we don't care about it
    pub id: i64,
    pub description: String,
    pub event_code: String,
    pub matchNumber: i32,
    pub tournamentLevel: String,
    pub teams: Vec<TeamFull>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeamFull {
    pub id: i32,
    pub teamNumber: u32,
    pub station: String,
}


pub async fn get_matches(pool: &SqlitePool) -> Result<Vec<MatchFull>, sqlx::Error> {
    let rows: Vec<FlatMatchRow> = sqlx::query_as::<_, FlatMatchRow>(
        r#"
        SELECT 
            m.id AS match_id,
            m.description,
            m.match_number,
            m.event_code,
            m.tournament_level,
            t.team_number,
            t.station,
            t.id as team_id
        FROM matches m
        JOIN match_teams t ON t.match_id = m.id
        ORDER BY m.match_number, t.station
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut grouped = BTreeMap::<i64, MatchFull>::new();

    for row in rows {
        let entry = grouped.entry(row.match_id).or_insert_with(|| MatchFull {
            id: row.match_id,
            description: row.description.clone(),
            matchNumber: row.match_number,
            event_code: row.event_code,
            tournamentLevel: row.tournament_level.clone(),
            teams: Vec::new(),
        });

        entry.teams.push(TeamFull {
            teamNumber: row.team_number as u32,
            station: row.station,
            id: row.team_id
        });
    }

    Ok(grouped.into_values().collect())
}


#[get("/scout_b")]
pub async fn scout(pool: &rocket::State<SqlitePool>, jar: &CookieJar<'_>) -> Template {
    let userid_string = match jar.get("uuid") {
        Some(a) =>  a.value(),
        None => {
            return Template::render("error", context![error: "Not logined in"]);
        },
    };

    let userid = match Uuid::from_str(userid_string) {
        Ok(a) => a,
        Err(_) => {
            return Template::render("error", context![error: "Not logined in"]);
        },
    };


    let user_request = sqlx::query(r#"
        SELECT can_write, username
        FROM user_list
        WHERE id = ?
    "#)
    .bind(userid)
    .fetch_optional(pool.inner())
    .await; //TODO: fix make new user to get perms right


    let (can_write, username) = match user_request {
        Ok(Some(a)) => {
            (a.get::<bool, _>(0), a.get::<String, _>(1))
        },
        Ok(None) => {
            return Template::render("error", context![error: "Can't find user"]);
        }
        Err(_) => {
            return Template::render("error", context![error: "Database Error"]);
        },
    };

    if !can_write {
        return Template::render("error", context![error: "You don't have writing perms!"]);
    }

    let data = match get_matches(pool.inner()).await {
        Ok(a) => a,
        Err(a)=> {
            println!("{:?}", a);
            return Template::render("error", context![error: "Database Error"]);
        }
    };

    

    Template::render("selectscrob", context![
        matches: data
    ])
}