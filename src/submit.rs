use std::str::FromStr;

use rocket::{form::Form, http::CookieJar};
use sqlx::{SqlitePool, Row};
use uuid::Uuid;

use crate::ScoutingForm;



//Auto scores
const auto_l4_amount: i32 = 7;
const auto_l3_amount: i32 = 6;
const auto_l2_amount: i32 = 4;
const auto_l1_amount: i32 = 3;
const auto_algae_processor_amount: i32 = 2;
const auto_algae_barge_amount: i32 = 4;

//Teleop scores
const teleop_l4_amount: i32 = 5;
const teleop_l3_amount: i32 = 4;
const teleop_l2_amount: i32 = 3;
const teleop_l1_amount: i32 = 2;
const teleop_algae_processor_amount: i32 = 2;
const teleop_algae_barge_amount: i32 = 2;

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
pub async fn submit_page(pool: &rocket::State<SqlitePool>, jar: &CookieJar<'_>, form_data: Form<ScoutingForm>) -> &'static str {

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

    // Insert into scouting_entry
    let scouting_id = sqlx::query(
    "INSERT INTO scouting_entry (user, team, matchid, total_score) VALUES (?, ?, ?, ?)")
    .bind(username)
    .bind(form.team)
    .bind(form.matchid)
    .bind(calculate_final_score(&form))
    .execute(pool.inner())
    .await
    .expect("Insert failed")
    .last_insert_rowid();

    let moved = match form.moved.as_str() {
        "yes" => true,
        _ => false,
    };

    // Insert auto data
    sqlx::query("
        INSERT INTO auto_data (moved, scouting_id, L1, L2, L3, L4, algae_processor, algae_barge, algae_remove)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
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
    .await
    .unwrap();

    // Insert teleop data
    sqlx::query("
        INSERT INTO teleop_data (scouting_id, L1, L2, L3, L4, algae_processor, algae_barge, algae_remove)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
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
    .await
    .unwrap();

    let died = match form.died.as_str() {
        "yes" => true,
        _ => false
    };

    // Insert endgame
    sqlx::query("
        INSERT INTO endgame_data (died, scouting_id, defense_rating, climb_type, comment)
        VALUES (?, ?, ?, ?, ?)
    ")
    .bind(died)
    .bind(scouting_id)
    .bind(form.defense_rating)
    .bind(&form.climb_type)
    .bind(&form.comment)
    .execute(pool.inner())
    .await
    .unwrap();

    "Scouting data submitted successfully"
}
