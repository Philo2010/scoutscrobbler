use rocket::form::Form;
use sqlx::{PgPool, Row};

//like submit.rs but for non logined people

use crate::submit::*;


#[derive(Debug, FromForm)]
pub struct ScoutingFormBad {
    //Metadata
    #[field(name = "username")] pub username: String,
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

pub fn calculate_final_score(form: &ScoutingFormBad) -> i32 {
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

#[post("/submit_bad", data = "<form_data>")]
pub async fn submit_page(pool: &rocket::State<PgPool>, form_data: Form<ScoutingFormBad>) -> &'static str {
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
    let row = sqlx::query(
        r#"
        INSERT INTO scouting_entry 
            (user, team, matchid, total_score, is_verified, event_code, tournament_level, station)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#)
    .bind(&form.username)
    .bind(form.team)
    .bind(form.matchid)
    .bind(calculate_final_score(&form))
    .bind("Unverified") // corrected typo too
    .bind(&form.event_code)
    .bind(&level)
    .bind(&station)
    .fetch_one(pool.inner())
    .await
    .expect("Insert failed");

    let scouting_id: i64 = match row.try_get("id") {
        Ok(a) => a,
        Err(a) => {
            return "Failed";
        }
    };


    let moved = match form.moved.as_str() {
        "yes" => true,
        _ => false,
    };

    // Insert auto data
    sqlx::query("
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
    .await
    .expect("Failed to insert into auto_data");

    // Insert teleop data
    sqlx::query("
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
    .await
    .expect("Failed to insert into teleop_data");

    let died = match form.died.as_str() {
        "yes" => true,
        _ => false
    };

    // Insert endgame
    sqlx::query("
        INSERT INTO endgame_data (died, scouting_id, defense_rating, climb_type, comment)
        VALUES ($1, $2, $3, $4, $5)
    ")
    .bind(died)
    .bind(scouting_id)
    .bind(form.defense_rating)
    .bind(&form.climb_type)
    .bind(&form.comment)
    .execute(pool.inner())
    .await
    .expect("Failed to insert endgame data");

    //Get match id (uni)
    let matchuniid: (i32,) = sqlx::query_as::<_, (i32,)>("
        SELECT match_id
        FROM match_teams
        WHERE id = $1
    ")
    .bind(form.id)
    .fetch_one(pool.inner())
    .await
    .expect("Failed to get match id");

    sqlx::query("
        DELETE FROM match_teams
        WHERE id = $1
    ")
    .bind(form.id)
    .execute(pool.inner())
    .await
    .expect("Failed to insert match teams");

    let remaining: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM match_teams WHERE match_id = $1")
    .bind(matchuniid.0) // store match_id in your form
    .fetch_one(pool.inner())
    .await
    .expect("Failed to get match ID");

    // If no teams left, delete the match
    if remaining.0 == 0 {
        sqlx::query("DELETE FROM matches WHERE id = $1")
            .bind(matchuniid.0)
            .execute(pool.inner())
            .await.expect("Remove matches failed!");
    }
    "Scouting data submitted successfully"
}