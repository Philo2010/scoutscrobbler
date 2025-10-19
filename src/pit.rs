//Handles pit scouting data

use rocket::{form::{self, Form}, State};
use rocket_dyn_templates::{context, Template};
use sqlx::PgPool;

#[derive(Debug, FromForm)]
pub struct Pit_Submit_data {
    pub team: i32,
    pub event_code: String,

    pub algae_processor: String,
    pub algae_barge: String,
    pub algae_remove: String,
    pub auto_align: String,
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

#[derive(Debug, FromForm)]
pub struct pitAutoSubmit {
    pub team: i32,
    pub event_code: String,

    //Auto
    pub left_auto: String,
    pub center_auto: String,
    pub right_auto: String,
    
    pub amount_of_sides: Option<String>,
    pub amount_of_combo_sides: Option<String>,
    pub coral_amount: String,
    pub algae_amount: String,
}

fn parse_out_bool(val: &str) -> bool {
    match val {
        "yes" => true,
        "no" => false,
        _ => false,
    }
}


#[post("/pit_auto_submit", data = "<form_data>")]
pub async fn pit_auto_submit(form_data: Form<pitAutoSubmit>, pool: &State<PgPool>) -> Template {

    let id: i32 = match sqlx::query_scalar!("
    SELECT id FROM pit_data WHERE team = $1 AND event_code = $2", &form_data.team, &form_data.event_code).fetch_one(pool.inner()).await {
        Ok(a) => a,
        Err(_) => {
            return Template::render("error", context![error: "Database error"]);
        },
    };

    let left_auto = parse_out_bool(&form_data.left_auto);
    let center_auto = parse_out_bool(&form_data.center_auto);
    let right_auto = parse_out_bool(&form_data.right_auto);
    let amount_of_sides = match &form_data.amount_of_sides {
        Some(a) => a.clone(),
        None => "".to_string(),
    };
    let amount_of_combo_sides = match &form_data.amount_of_combo_sides {
        Some(a) => a.clone(),
        None => "".to_string(),
    };

    let result = sqlx::query!(r#"
        INSERT INTO pit_auto_data (
        pit_id,
        left_auto,
        center_auto,
        right_auto,
        amount_of_sides,
        amount_of_combo_sides,
        coral_amount,
        algae_amount)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    "#, id,
     left_auto, 
     center_auto,
      right_auto, 
      amount_of_sides, 
      amount_of_combo_sides,
    &form_data.coral_amount,
    &form_data.algae_amount).execute(pool.inner()).await;


    return Template::render("yippy", context! {});
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
    let defence = parse_out_bool(&form_data.defence);
    let ground_intake = parse_out_bool(&form_data.ground_intake);
    let climber = parse_out_bool(&form_data.climber);
    let auto_align = parse_out_bool(&form_data.auto_align);
    let comment = match &form_data.comment {
        Some(a) => a.clone(),
        None => "".to_string(),
    };
    
    
    
    


    let result = sqlx::query!(r#"
    INSERT INTO pit_data (
    auto_align,
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
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
    "#, 
    auto_align,
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
    defence,
    &form_data.driver_years_experience,
    comment,
)
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