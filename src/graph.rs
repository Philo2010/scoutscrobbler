use rocket::{form::Form, http::CookieJar};
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

    let dataquery = sqlx::query_as::<_, DataNodeTeam>(r#"
        SELECT total_score, matchid, created_at
        FROM scouting_entry
        WHERE team = ?
        ORDER BY created_at ASC
    "#)
    .bind(&form_data.team)
    .fetch_all(pool.inner())
    .await;

    let data = match dataquery {
        Ok(a) => {
            a
        },
        Err(_) => {
            return Template::render("error", "Database error");
        },
    };

    Template::render("graph_render",context![team_data: data])
}