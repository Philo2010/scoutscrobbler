//Handles pit scouting data

use rocket::form::Form;
use rocket_dyn_templates::{context, Template};
use sqlx::PgPool;

#[derive(Debug, FromForm)]
pub struct Pit_Submit_data {
    pub team: i32,
    pub event_code: String,

    pub algae_processor: String,
    pub algae_barge: String,
    pub algae_remove: String,
    pub l1: String,
    pub l2: String,
    pub l3: String,
    pub l4: String,
    pub ground_intake: String,
    pub climber: String,

    pub height: String,
    pub widthxlength: String,
    pub weight: String,
    pub defence: String,
    pub driver_years_experience: String,
    pub comment: Option<String>,
}

fn parse_out_bool(val: &str) -> bool {
    match val {
        "yes" => true,
        "no" => false,
        _ => false,
    }
}

#[post("/pit_submit", data = "<form_data>")]
pub async fn pit_submit(form_data: Form<Pit_Submit_data>, pool: &rocket::State<PgPool>) -> Template {
    //TODO: add user basied checks later, get it done now!

    //Parse the strings as bools
    let algae_processor = parse_out_bool(&form_data.algae_processor);
    let algae_barge = parse_out_bool(&form_data.algae_barge);
    let algae_remove = parse_out_bool(&form_data.algae_remove);
    let l1 = parse_out_bool(&form_data.l1);
    let l2 = parse_out_bool(&form_data.l2);
    let l3 = parse_out_bool(&form_data.l3);
    let l4 = parse_out_bool(&form_data.l4);
    let ground_intake = parse_out_bool(&form_data.ground_intake);
    let climber = parse_out_bool(&form_data.climber);
    let comment = match &form_data.comment {
        Some(a) => a.clone(),
        None => "".to_string(),
    };
    
    
    
    
    


    let result = sqlx::query!(r#"
    INSERT INTO pit_data (
    team,
    event_code,
    algae_processor,
    algae_barge,
    algae_remove,
    L1, L2, L3, L4,
    ground_intake,
    climber,
    height,
    widthxlength,
    weight,
    defence,
    driver_years_experience,
    comment)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
    "#, 
    form_data.team,
    form_data.event_code,
    algae_processor,
    algae_barge,
    algae_remove,
    l1,
    l2,
    l3,
    l4,
    ground_intake,
    climber,
    &form_data.height,
    &form_data.widthxlength,
    &form_data.weight,
    &form_data.defence,
    &form_data.driver_years_experience,
    comment)
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