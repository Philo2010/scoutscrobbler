use rocket::{form::Form, State};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Serialize)]
struct PitData {
    id: i32,
    team: i32,
    event_code: String,
    algae_processor: bool,
    algae_barge: bool,
    algae_remove: bool,
    auto_align: bool,
    l1: bool,
    l2: bool,
    l3: bool,
    l4: bool,
    ground_intake: bool,
    climber: bool,
    height: String,
    widthxlength: String,
    weight: String,
    defence: bool,
    driver_years_experience: String,
    comment: String,
}

#[derive(Debug, FromForm)]
struct ViewPitForm {
    pub team: i32,
    pub event_code: String,
}


#[get("/viewpit/<event_code>/<team>")]
pub async fn view_pit(pool: &State<PgPool>, event_code: &str, team: i32) -> Template {
    let data = sqlx::query_as!(
        PitData,
        r#"
        SELECT *
        FROM pit_data
        WHERE team = $1 AND event_code = $2
        "#,
        team,
        &event_code
    ).fetch_optional(pool.inner()).await;

    let data_clean = match data {
        Ok(Some(a)) => a,
        Ok(None) => {
            return Template::render("error", context! {error: "No pit data"});
        }
        Err(_) => {
            return Template::render("error", context! {error: "Database Error!"});
        },
    };


    Template::render("pit", context! {entry: data_clean})
}