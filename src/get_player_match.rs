use rocket::http::CookieJar;
use rocket_dyn_templates::{context, Template};
use sqlx::{PgPool, Postgres};

use crate::{check_if_read, ScoutingEntry, ScoutingEntryBasic};

#[get("/get_player_match/<id>")]
pub async fn get_player_match(pool: &rocket::State<PgPool>, id: i32) -> Template {

    let entry = match sqlx::query_as::<_, ScoutingEntry>(r#"
        SELECT 
        se.id,
        se.team,
        se.user,
        se.matchid,
        se.total_score,
        se.created_at,
        se.event_code,
        ad.L1 AS auto_l1, ad.L2 AS auto_l2, ad.L3 AS auto_l3, ad.L4 AS auto_l4,
        ad.moved,
        ad.algae_processor AS auto_algae_processor,
        ad.algae_barge AS auto_algae_barge,
        ad.algae_remove AS auto_algae_remove,
        td.L1 AS teleop_l1, td.L2 AS teleop_l2, td.L3 AS teleop_l3, td.L4 AS teleop_l4,
        td.algae_processor AS teleop_algae_processor,
        td.algae_barge AS teleop_algae_barge,
        td.algae_remove AS teleop_algae_remove,
        eg.died,
        eg.defense_rating,
        eg.climb_type,
        eg.comment
        FROM scouting_entry se
        LEFT JOIN auto_data ad ON ad.scouting_id = se.id
        LEFT JOIN teleop_data td ON td.scouting_id = se.id
        LEFT JOIN endgame_data eg ON eg.scouting_id = se.id
        WHERE se.id = $1
        LIMIT 1
    "#)
    .bind(id)  // <-- bind the specific id here
    .fetch_one(pool.inner())
    .await {
        Ok(a) => a,
        Err(a) => {
            println!("{a}");
            return Template::render("error", context! { error: "Database Error" });
        },
    };

    //Get the pit data
    let comment_query = sqlx::query_scalar!(
        r#"
        SELECT comment FROM pit_data
        WHERE team = $1 AND event_code = $2
        "#,
        entry.team,
        entry.event_code
    )
    .fetch_optional(pool.inner())
    .await;

    let comment = match comment_query {
        Ok(a) => {
            match a {
                Some(a) => a,
                None => {
                    "No pit data".to_string()
                },
            }
        },
        Err(_) => {
            return Template::render("error", context! { error: "Database Error" });
        },
    };


    return Template::render("get_player_match", context! { entry, pit_data: comment });
}
