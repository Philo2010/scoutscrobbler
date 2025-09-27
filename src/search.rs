use rocket::{form::Form, http::CookieJar, State};
use rocket_dyn_templates::{context, Template};
use sqlx::SqlitePool;

use crate::{check_if_read, ScoutingEntryBasic};

#[derive(Debug, FromForm)]
pub struct SearchForm {
    #[field(name = "startdate")] pub startdate: String,
    #[field(name = "stopdate")] pub stopdate: String,
    #[field(name = "team")] pub team: i32,
}


#[post("/search", data = "<form_data>")]
pub async fn search(pool: &State<SqlitePool>, jar: &CookieJar<'_>,
    form_data: Form<SearchForm>  // Form data from the request)
) -> Template {
   
    //Get cookie
    let userid_string = match jar.get("uuid") {
        Some(a) =>  a.value(),
        None => {
            let entries: Vec<ScoutingEntryBasic> = Vec::new();
            return Template::render("entries", context! { entries });
        },
    };

    match check_if_read(userid_string, pool).await {
        Some(a) => {
            return a;
        },
        None => {},
    };

    let list = sqlx::query_as::<_, ScoutingEntryBasic>(r#"
        SELECT id, user, team, created_at
        FROM scouting_entry
        WHERE team = ?
        AND DATE(created_at) BETWEEN DATE(?) AND DATE(?)
        ORDER BY created_at DESC;
    "#)
    .bind(&form_data.team)
    .bind(&form_data.startdate)
    .bind(&form_data.stopdate)
    .fetch_all(pool.inner())
    .await;

    match list {
        Ok(a) => Template::render("search", context! {entries: a}),
        Err(_) => {
            let entries: Vec<ScoutingEntryBasic> = Vec::new();
            return Template::render("entries", context! { entries });
        },
    }
}
