use std::f64::consts::E;

use rocket_dyn_templates::{Template, context};
use sqlx::PgPool;
use rocket::State;

use crate::ScoutingEntry;




#[get("/entries")]
pub async fn get_entries(pool: &State<PgPool>) -> Template {
    let entries = match sqlx::query_as::<_,ScoutingEntry>(r#"
    SELECT 
        s.id,
        s."user",
        s.team,
        s.matchid,
        s.total_score,
        s.event_code,
        s.created_at,

        a.moved,
        a.L1 AS auto_l1,
        a.L2 AS auto_l2,
        a.L3 AS auto_l3,
        a.L4 AS auto_l4,
        a.algae_processor AS auto_algae_processor,
        a.algae_barge AS auto_algae_barge,
        a.algae_remove AS auto_algae_remove,

        t.L1 AS teleop_l1,
        t.L2 AS teleop_l2,
        t.L3 AS teleop_l3,
        t.L4 AS teleop_l4,
        t.algae_processor AS teleop_algae_processor,
        t.algae_barge AS teleop_algae_barge,
        t.algae_remove AS teleop_algae_remove,

        e.died,
        e.defense_rating,
        e.climb_type,
        e.comment
    FROM scouting_entry s
    LEFT JOIN auto_data a ON s.id = a.scouting_id
    LEFT JOIN teleop_data t ON s.id = t.scouting_id
    LEFT JOIN endgame_data e ON s.id = e.scouting_id
    ORDER BY s.team ASC;
    "#).fetch_all(pool.inner()).await {
        Ok(a) => a,
        Err(a) => {
            println!("{a}");
            return Template::render("error", context! {error: "Database Error!"});},
    };


    let mut fmt_time: Vec<String> = Vec::new();
    let mut total_game_piece: Vec<i32> = Vec::new();

    for game in &entries {
        fmt_time.push(game.created_at.format("%b %d, %Y %I:%M").to_string());
        let game_piece_count: i32 = (
            //Auto
            game.auto_l1.unwrap() +
            game.auto_l2.unwrap() +
            game.auto_l3.unwrap() +
            game.auto_l4.unwrap() +
            game.auto_algae_processor.unwrap() +
            game.auto_algae_barge.unwrap() +
            game.auto_algae_remove.unwrap() +

            //Teleop
            game.teleop_l1.unwrap() +
            game.teleop_l2.unwrap() +
            game.teleop_l3.unwrap() +
            game.teleop_l4.unwrap() +
            game.teleop_algae_processor.unwrap() +
            game.teleop_algae_barge.unwrap() +
            game.teleop_algae_remove.unwrap()
        );
        
        total_game_piece.push(game_piece_count);           
    }
    let zipped: Vec<_> = fmt_time
        .iter()
        .zip(entries.iter())
        .zip(total_game_piece.iter())
        .map(|((time, e), pieces)| (time, e, pieces))
        .collect();


    Template::render("search", context! {entries: zipped})
}