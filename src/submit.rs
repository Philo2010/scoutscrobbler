use std::str::FromStr;

use rocket::{form::Form, http::CookieJar};
use sqlx::{query, PgPool, Row};
use uuid::Uuid;

use crate::ScoutingForm;



//Auto scores
pub const auto_l4_amount: i32 = 7;
pub const auto_l3_amount: i32 = 6;
pub const auto_l2_amount: i32 = 4;
pub const auto_l1_amount: i32 = 3;
pub const auto_algae_processor_amount: i32 = 2;
pub const auto_algae_barge_amount: i32 = 4;

//Teleop scores
pub const teleop_l4_amount: i32 = 5;
pub const teleop_l3_amount: i32 = 4;
pub const teleop_l2_amount: i32 = 3;
pub const teleop_l1_amount: i32 = 2;
pub const teleop_algae_processor_amount: i32 = 2;
pub const teleop_algae_barge_amount: i32 = 2;

fn calculate_final_score(form: &ScoutingForm) -> i32 {
    let auto_coral = 
        (auto_l4_amount*form.auto_l4) + 
        (auto_l3_amount*form.auto_l3) + 
        (auto_l2_amount*form.auto_l2) + 
        (auto_l1_amount*form.auto_l1) + 
        (auto_algae_processor_amount*form.auto_algae_processor) + 
        (auto_algae_barge_amount*form.auto_algae_barge);

    let teleop_coral = 
        (teleop_l4_amount*form.teleop_l4) + 
        (teleop_l3_amount*form.teleop_l3) + 
        (teleop_l2_amount*form.teleop_l2) + 
        (teleop_l1_amount*form.teleop_l1) + 
        (teleop_algae_processor_amount*form.teleop_algae_processor) + 
        (teleop_algae_barge_amount*form.teleop_algae_barge);
    
    
    let climb = match form.climb_type.as_str() {
        "deep" => 12,
        "shallow" => 7,
        "park" => 2,
        _ => 0,
    };

    auto_coral + teleop_coral + climb
}

#[post("/submit", data = "<form_data>")]
pub async fn submit_page(pool: &rocket::State<PgPool>, jar: &CookieJar<'_>, form_data: Form<ScoutingForm>) -> &'static str {

    let userid_string = match jar.get("uuid") {
        Some(a) =>  a.value(),
        None => {
            "Not logined in"
        },
    };

    let userid = match Uuid::from_str(userid_string) {
        Ok(a) => a,
        Err(_) => {
            return "Not logined in";
        },
    };


    let user_request = sqlx::query(r#"
        SELECT can_write, username
        FROM user_list
        WHERE id = $1
    "#)
    .bind(userid)
    .fetch_optional(pool.inner())
    .await; //TODO: fix make new user to get perms right


    let (can_write, username) = match user_request {
        Ok(Some(a)) => {
            (a.get::<bool, _>(0), a.get::<String, _>(1))
        },
        Ok(None) => {
            return "Can't find user";
        }
        Err(_) => {
            return "Database Error";
        },
    };

    if !can_write {
        return "You don't have writing perms!";
    }

    let form = form_data.into_inner();

    let level = match form.tournament_level.as_str() {
        "Qualification" => "qual".to_string(),
        _ => "Playoff".to_string()
    };
    let station =  match form.station.as_str() {
        "Red1" => "Red1".to_string(),
        "Red2" => "Red2".to_string(),
        "Red3" => "Red3".to_string(),
        "Blue1" => "Blue1".to_string(),
        "Blue2" => "Blue2".to_string(),
        "Blue3" => "Blue3".to_string(),
        _ => "Red1".to_string(), //Fallback
    };

    // Insert into scouting_entry
    let row = match sqlx::query(
        r#"
        INSERT INTO scouting_entry (
            "user", team, matchid, total_score, is_verified, event_code, tournament_level, station
        ) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#
    )
    .bind(&username)
    .bind(form.team)
    .bind(form.matchid)
    .bind(calculate_final_score(&form))
    .bind("Unverified")
    .bind(&form.event_code)
    .bind(&level)
    .bind(&station)
    .fetch_one(pool.inner())
    .await {
        Ok(a) => {a},
        Err(a) => {return "Database Error";},
    };

    let scouting_id: i32 = row.try_get("id").expect("Could not get ID");

    let moved = match form.moved.as_str() {
        "yes" => true,
        _ => false,
    };

    // Insert auto data
    match sqlx::query("
        INSERT INTO auto_data (moved, scouting_id, L1, L2, L3, L4, algae_processor, algae_barge, algae_remove)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    ")
    .bind(moved)
    .bind(scouting_id)
    .bind(form.auto_l1)
    .bind(form.auto_l2)
    .bind(form.auto_l3)
    .bind(form.auto_l4)
    .bind(form.auto_algae_processor)
    .bind(form.auto_algae_barge)
    .bind(form.auto_algae_remove)
    .execute(pool.inner())
    .await {
        Ok(_) => {},
        Err(a) => {return "Database Error";},
    };

    // Insert teleop data
    match sqlx::query("
        INSERT INTO teleop_data (scouting_id, L1, L2, L3, L4, algae_processor, algae_barge, algae_remove)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    ")
    .bind(scouting_id)
    .bind(form.teleop_l1)
    .bind(form.teleop_l2)
    .bind(form.teleop_l3)
    .bind(form.teleop_l4)
    .bind(form.teleop_algae_processor)
    .bind(form.teleop_algae_barge)
    .bind(form.teleop_algae_remove)
    .execute(pool.inner())
    .await {
        Ok(_) => {},
        Err(a) => {return "Database Error";},
    };

    let died = match form.died.as_str() {
        "yes" => true,
        _ => false
    };

    // Insert endgame
    match sqlx::query("
        INSERT INTO endgame_data (died, scouting_id, defense_rating, climb_type, comment)
        VALUES ($1, $2, $3, $4, $5)
    ")
    .bind(died)
    .bind(scouting_id)
    .bind(form.defense_rating)
    .bind(&form.climb_type)
    .bind(&form.comment)
    .execute(pool.inner())
    .await {
        Ok(_) => {},
        Err(a) => {return "Database Error";},
    };

    //Get match id (uni)
    let matchuniid: (i32,) = match sqlx::query_as::<_, (i32,)>("
        SELECT match_id
        FROM match_teams
        WHERE id = $1
    ")
    .bind(form.id)
    .fetch_one(pool.inner())
    .await {
        Ok(a) => {a},
        Err(a) => {return "Database Error";},
    };

    match sqlx::query("
        DELETE FROM match_teams
        WHERE id = $1
    ")
    .bind(form.id)
    .execute(pool.inner())
    .await {
        Ok(_) => {},
        Err(a) => {return "Database Error";},
    };

    let remaining: (i64,) = match sqlx::query_as("SELECT COUNT(*) FROM match_teams WHERE match_id = $1")
    .bind(matchuniid.0) // store match_id in your form
    .fetch_one(pool.inner())
    .await {
        Ok(a) => {a},
        Err(a) => {return "Database Error";},
    };

    // If no teams left, delete the match
    if remaining.0 == 0 {
        match sqlx::query("DELETE FROM matches WHERE id = $1")
            .bind(matchuniid.0)
            .execute(pool.inner())
            .await {
                Ok(_) => {},
                Err(a) => {return "Database Error";},
            };
    }
    "Scouting data submitted successfully"
}
