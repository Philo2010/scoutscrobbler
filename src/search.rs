use rocket::{form::Form, http::CookieJar, State};
use rocket_dyn_templates::{context, Template};
use sqlx::SqlitePool;

use crate::{check_if_read, ScoutingEntryBasic};

#[derive(Debug, FromForm)]
pub struct SearchForm {
    #[field(name = "event")] pub event: String,
    #[field(name = "team")] pub team: i32,
}


#[post("/search", data = "<form_data>")]
pub async fn search(pool: &State<SqlitePool>,
    form_data: Form<SearchForm>  // Form data from the request)
) -> Template {


    let list = sqlx::query_as::<_, ScoutingEntryBasic>(r#"
        SELECT id, user, team, created_at
        FROM scouting_entry
        WHERE team = ?
        AND event_code = ?
        ORDER BY created_at DESC;
    "#)
    .bind(&form_data.team)
    .bind(&form_data.event)
    .fetch_all(pool.inner())
    .await;

    match list {
        Ok(a) => Template::render("search", context! {entries: a}),
        Err(a) => {
            println!("{a}");
            return Template::render("error", context! { error: "Unkown error" });
        },
    }
}
