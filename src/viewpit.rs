use rocket::{form::Form, State};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use sqlx::PgPool;

//TODO: Rewrite this without the horrable Op workaround, needed for comp times

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


#[derive(sqlx::FromRow, Debug, Serialize)]
struct PitAutoData {
    id: i32,
    pit_id: Option<i32>,
    left_auto: i32,
    center_auto: i32,
    right_auto: i32,
    amount_of_sides: String,
    amount_of_combo_sides: String,
    coral_amount: String,
    algae_amount: String,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
struct PitAutoData2 {
    id: i32,
    pit_id: i32,
    left_auto: i32,
    center_auto: i32,
    right_auto: i32,
    amount_of_sides: String,
    amount_of_combo_sides: String,
    coral_amount: String,
    algae_amount: String,
}

impl From<PitAutoData> for PitAutoData2 {
    fn from(data: PitAutoData) -> Self {
        PitAutoData2 {
            id: data.id,
            pit_id: data.pit_id.unwrap(),
            left_auto: data.left_auto,
            center_auto: data.center_auto,
            right_auto: data.right_auto,
            amount_of_sides: data.amount_of_sides,
            amount_of_combo_sides: data.amount_of_combo_sides,
            coral_amount: data.coral_amount,
            algae_amount: data.algae_amount,
        }
    }
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
    let auto_data =  sqlx::query_as!(
        PitAutoData,
        r#"
        SELECT *
        FROM pit_auto_data
        WHERE pit_id = $1
        "#,
        &data_clean.id
    ).fetch_optional(pool.inner()).await;

    match auto_data {
        Ok(Some(a)) => {
            let pd: PitAutoData2 = a.into();

            Template::render("pitandauto", context! {entry: data_clean, auto: pd})
        },
        _ => {
            return Template::render("pit", context! {entry: data_clean});
        }
    }
}