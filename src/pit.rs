//Handles pit scouting data

use rocket::form::Form;
use rocket_dyn_templates::{context, Template};
use sqlx::PgPool;

#[derive(Debug, FromForm)]
pub struct Pit_Submit_data {
    pub team: i32,
    pub event_code: String,
    pub comment: String
}

#[post("/pit_submit", data = "<form_data>")]
pub async fn pit_submit(form_data: Form<Pit_Submit_data>, pool: &rocket::State<PgPool>) -> Template {
    //TODO: add user basied checks later, get it done now!

    let result = sqlx::query(r#"
    INSERT INTO pit_data (team, event_code, comment)
    VALUES ($1, $2, $3)
    "#)
    .bind(form_data.team)
    .bind(&form_data.event_code)
    .bind(&form_data.comment)
    .execute(pool.inner()).await;

    match result {
        Ok(_) => {
            return Template::render("yippy", context! {});
        },
        Err(_) => {
            return Template::render("error", context![error: "Database error"]);
        },
    }
}