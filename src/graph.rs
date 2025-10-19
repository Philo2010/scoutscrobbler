use rocket::{data, form::{self, Form}, http::CookieJar};
use rocket_dyn_templates::{context, Template};
use sqlx::{types::chrono, Error, FromRow, PgPool};

use crate::check_if_read;

#[derive(Debug, FromForm)]
struct TeamSearch {
    pub team: Vec<i32>,
}

#[derive(FromRow, serde::Serialize)]
pub struct DataNodeTeam {
    total_score: i32,
    matchid: i32,
    created_at: chrono::NaiveDateTime,
    auto_total: i32,
    teleop_total: i32,
    defense: i32,
}

pub async fn get_team_data(team: &i32, pool: &PgPool) -> Result<Vec<DataNodeTeam>, Error> {
    sqlx::query_as::<_, DataNodeTeam>(r#"
        SELECT 
            se.total_score,
            se.created_at,
            se.matchid,
            e.defense_rating as defense,

            -- Auto total with weights
            COALESCE(a.L4, 0) * 7 +
            COALESCE(a.L3, 0) * 6 +
            COALESCE(a.L2, 0) * 4 +
            COALESCE(a.L1, 0) * 3 +
            COALESCE(a.algae_processor, 0) * 2 +
            COALESCE(a.algae_barge, 0) * 4 AS auto_total,

            -- Teleop total with weights
            COALESCE(t.L4, 0) * 5 +
            COALESCE(t.L3, 0) * 4 +
            COALESCE(t.L2, 0) * 3 +
            COALESCE(t.L1, 0) * 2 +
            COALESCE(t.algae_processor, 0) * 2 +
            COALESCE(t.algae_barge, 0) * 4 AS teleop_total

        FROM scouting_entry se
        LEFT JOIN auto_data a ON a.scouting_id = se.id
        LEFT JOIN teleop_data t ON t.scouting_id = se.id
        LEFT JOIN endgame_data e ON e.scouting_id = se.id
        WHERE se.team = $1
        ORDER BY se.created_at ASC;
    "#)
    .bind(team)
    .fetch_all(pool)
    .await
}


#[post("/graph_team", data = "<form_data>")]
pub async fn graph(pool: &rocket::State<PgPool>,  form_data: Form<TeamSearch>) -> Template {

    let mut team_data: Vec<Vec<DataNodeTeam>> = Vec::new();
    for team_number in form_data.team.iter() {
        let data = match get_team_data(team_number, pool.inner()).await {
            Err(a) => {
                println!("{a}");
                continue;
            },
            Ok(a) => a,
        };
        team_data.push(data);
    }

    Template::render("graph_render", context! {
        teams: team_data,
        team_numbers: &form_data.team
    })
}