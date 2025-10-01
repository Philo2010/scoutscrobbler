use rocket::{data, form::Form, http::CookieJar};
use rocket_dyn_templates::{context, Template};
use sqlx::{FromRow, SqlitePool};

use crate::check_if_read;

#[derive(Debug, FromForm)]
struct TeamSearch {
    #[field(name = "team")] pub team: i32,
}

#[derive(FromRow, serde::Serialize)]
pub struct DataNodeTeam {
    total_score: i32,
    matchid: i32,
    created_at: String,
    auto_total: i32,
    teleop_total: i32,
}

#[post("/graph_team", data = "<form_data>")]
pub async fn graph(pool: &rocket::State<SqlitePool>, jar: &CookieJar<'_>, form_data: Form<TeamSearch>) -> Template {
    
    //Check if user has perms
    let userid_string = match jar.get("uuid") {
        Some(a) =>  a.value(),
        None => {
            return Template::render("error", "Not login!");
        },
    };

    match check_if_read(userid_string, pool).await {
        Some(a) => {
            return Template::render("error", "Dont have perms");
        },
        None => {},
    };

    //We should precaluate this value the read it, but im not changeing structs yet again!
    let dataquery = sqlx::query_as::<_, DataNodeTeam>(r#"
        SELECT 
        se.total_score,
        se.created_at,
        se.matchid,

        -- Auto total
        COALESCE(a.L1, 0) +
        COALESCE(a.L2, 0) +
        COALESCE(a.L3, 0) +
        COALESCE(a.L4, 0) +
        COALESCE(a.algae_processor, 0) +
        COALESCE(a.algae_barge, 0) +
        COALESCE(a.algae_remove, 0) AS auto_total,

        -- Teleop total
        COALESCE(t.L1, 0) +
        COALESCE(t.L2, 0) +
        COALESCE(t.L3, 0) +
        COALESCE(t.L4, 0) +
        COALESCE(t.algae_processor, 0) +
        COALESCE(t.algae_barge, 0) +
        COALESCE(t.algae_remove, 0) AS teleop_total

        FROM scouting_entry se
        LEFT JOIN auto_data a ON a.scouting_id = se.id
        LEFT JOIN teleop_data t ON t.scouting_id = se.id
        WHERE se.team = ?
        ORDER BY se.created_at ASC;
    "#)
    .bind(&form_data.team)
    .fetch_all(pool.inner())
    .await;



    let data = match dataquery {
        Ok(a) => {
            a
        },
        Err(a) => {
            println!("{a}");
            return Template::render("error", context! {error: "Database error"});
        },
    };
    println!("{}", data.len());

    


    Template::render("graph_render",context![team_data: data])
}