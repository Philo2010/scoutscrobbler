#[macro_use] extern crate rocket;

use std::str::FromStr;
use reqwest::Client;
use rocket::fs::FileServer;
use rocket::serde::Serialize;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use sqlx::Row;
use rocket::fs::relative;
use rocket_db_pools::sqlx::{self, SqlitePool};
use sqlx::types::Uuid;
use base64::{engine::general_purpose, Engine};

mod submit;
mod entries;
mod user;
mod login;
mod search;
mod get_player_match;
mod graph;
mod verify;
mod queue;
mod scout;

//Make the auth header

// username:password = "***REMOVED***"
// Base64-encoded = "REDACTED"
pub const BASIC_AUTH_HEADER: &str = "***REMOVED***";


//Generatic types used all across code

#[derive(Debug, FromForm)]
pub struct ScoutingForm {
    //Metadata
    #[field(name = "team")] pub team: i32,
    #[field(name = "match")] pub matchid: i32,
    #[field(name = "event-code")] pub event_code: String,
    #[field(name = "level")] pub tournament_level: String,
    #[field(name = "station")] pub station: String,
    #[field(name = "id")] pub id: i32,

    // Auto
    #[field(name = "moved")] pub moved: String,
    #[field(name = "auto-L1")] pub auto_l1: i32,
    #[field(name = "auto-L2")] pub auto_l2: i32,
    #[field(name = "auto-L3")] pub auto_l3: i32,
    #[field(name = "auto-L4")] pub auto_l4: i32,
    #[field(name = "auto-alpro")] pub auto_algae_processor: i32,
    #[field(name = "auto-albar")] pub auto_algae_barge: i32,
    #[field(name = "auto-alrem")] pub auto_algae_remove: i32,

    // Teleop
    #[field(name = "teleop-L1")] pub teleop_l1: i32,
    #[field(name = "teleop-L2")] pub teleop_l2: i32,
    #[field(name = "teleop-L3")] pub teleop_l3: i32,
    #[field(name = "teleop-L4")] pub teleop_l4: i32,
    #[field(name = "teleop-alpro")] pub teleop_algae_processor: i32,
    #[field(name = "teleop-albar")] pub teleop_algae_barge: i32,
    #[field(name = "teleop-alrem")] pub teleop_algae_remove: i32,

    // Endgame
    #[field(name = "died")] pub died: String,
    #[field(name = "rating")] pub defense_rating: i32,
    #[field(name = "climb")] pub climb_type: String,
    #[field(name = "comment")] pub comment: String,
}
#[derive(Debug, sqlx::FromRow, Serialize)]
struct ScoutingEntry {
    id: i64,
    team: i32,
    user: Option<String>,
    matchid: i32,
    total_score: i32,
    created_at: String,

    moved: bool,
    auto_l1: Option<i32>,
    auto_l2: Option<i32>,
    auto_l3: Option<i32>,
    auto_l4: Option<i32>,
    auto_algae_processor: Option<i32>,
    auto_algae_barge: Option<i32>,
    auto_algae_remove: Option<i32>,

    teleop_l1: Option<i32>,
    teleop_l2: Option<i32>,
    teleop_l3: Option<i32>,
    teleop_l4: Option<i32>,
    teleop_algae_processor: Option<i32>,
    teleop_algae_barge: Option<i32>,
    teleop_algae_remove: Option<i32>,

    died: bool,
    defense_rating: Option<i32>,
    climb_type: Option<String>,
    comment: Option<String>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
struct ScoutingEntryBasic {
    id: i64,
    user: String,
    team: i32,
    created_at: String,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
struct User<'a> {
    pub id: &'a [u8],
    pub username: String,
    pub passhash: String,
    pub can_write: bool,
    pub can_read: bool,
}

async fn check_if_read(userid_string: &str, pool: &State<SqlitePool>) -> Option<Template> {
    let userid = match Uuid::from_str(userid_string) {
        Ok(a) => a,
        Err(_) => {
           return Some(Template::render("error", context! { error: "Not logined" }));
        },
    };


    let user_request = sqlx::query(r#"
        SELECT can_read
        FROM user_list
        WHERE id = ?
    "#)
    .bind(userid)
    .fetch_optional(pool.inner())
    .await; //TODO: fix make new user to get perms right


    let can_read = match user_request {
        Ok(Some(a)) => {
            a.get::<bool, _>(0)
        },
        Ok(None) => {
            return Some(Template::render("error", context! { error: "No user found" }));
        }
        Err(_) => {
            return Some(Template::render("error", context! { error: "Unkown error" }));
        },
    };

    if !can_read {
        let entries: Vec<ScoutingEntry> = Vec::new();
        return  Some(Template::render("entries", context! { entries }));
    }
    None
}




//Boiler plate
#[launch]
async fn rocket() -> _ {
   //Create a rewqest client for speed and share
   let client = Client::builder()
        .build()
        .expect("Could not build http client! FATAL!!!");
    let mut auth_headers =  reqwest::header::HeaderMap::new();
    auth_headers.insert("Authorization", BASIC_AUTH_HEADER.parse().expect("Error with http header"));
    auth_headers.insert("If-Modified-Since", "".parse().expect("Error with http header"));

    let db_pool = SqlitePool::connect("sqlite:main.sqlite").await.expect("Failed to connect to DB");

    sqlx::query("PRAGMA foreign_keys = ON;")
    .execute(&db_pool)
    .await.expect("Could not enable foreign keys");

    rocket::build()
    .manage(db_pool)
    .manage(client)
    .manage(auth_headers)
    .attach(Template::fairing())
    .mount("/", routes![submit::submit_page, entries::view_entries, user::new_user, login::login, search::search, get_player_match::get_player_match, graph::graph, queue::queue_form, scout::scout, verify::verify])
    .mount("/", FileServer::from(relative!("static")))
}
