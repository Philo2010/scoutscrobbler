use rocket::{form::Form, http::CookieJar, State};
use rocket_dyn_templates::{context, Template};
use sqlx::PgPool;
use sqlx::QueryBuilder;

use crate::{check_if_read, ScoutingEntry};

#[derive(Debug, FromForm)]
pub struct SearchForm {
    #[field(name = "scouter")] pub scouter: Option<String>,
    #[field(name = "team")] pub team: Option<i32>,
    #[field(name = "event")] pub event: Option<String>,
}

impl SearchForm {
    pub fn normalized(&self) -> Self {
        Self {
            scouter: self.scouter.as_ref().filter(|s| !s.trim().is_empty()).cloned(),
            team: self.team,
            event: self.event.as_ref().filter(|s| !s.trim().is_empty()).cloned(),
        }
    }
}



#[post("/search", data = "<form>")]
pub async fn search(pool: &State<PgPool>,
    form: Form<SearchForm>  // Form data from the request)
) -> Template {
    let form_data = form.normalized();

    let mut builder = QueryBuilder::new(
    r#"
    SELECT 
        s.id,
        s."user",
        s.team,
        s.matchid,
        s.total_score,
        s.event_code,
        s.created_at,

        a.moved,
        a.L1 AS auto_l1,
        a.L2 AS auto_l2,
        a.L3 AS auto_l3,
        a.L4 AS auto_l4,
        a.algae_processor AS auto_algae_processor,
        a.algae_barge AS auto_algae_barge,
        a.algae_remove AS auto_algae_remove,

        t.L1 AS teleop_l1,
        t.L2 AS teleop_l2,
        t.L3 AS teleop_l3,
        t.L4 AS teleop_l4,
        t.algae_processor AS teleop_algae_processor,
        t.algae_barge AS teleop_algae_barge,
        t.algae_remove AS teleop_algae_remove,

        e.died,
        e.defense_rating,
        e.climb_type,
        e.comment
    FROM scouting_entry s
    LEFT JOIN auto_data a ON s.id = a.scouting_id
    LEFT JOIN teleop_data t ON s.id = t.scouting_id
    LEFT JOIN endgame_data e ON s.id = e.scouting_id
    WHERE 1=1
    "#
    );


    if let Some(scouter) = &form_data.scouter {
        builder.push(" AND \"user\" = ").push_bind(scouter);
    }
    if let Some(team) = &form_data.team {
        builder.push(" AND team = ").push_bind(team);
    }
    if let Some(event) = &form_data.event {
        builder.push(" AND event_code = ").push_bind(event);
    }

    builder.push(" ORDER BY created_at DESC;");

    let query = builder.build_query_as::<ScoutingEntry>();

    let list = query.fetch_all(pool.inner()).await;


    match list {
        Ok(a) => {
            let mut fmt_time: Vec<String> = Vec::new();
            let mut total_game_piece: Vec<i32> = Vec::new();

            for game in &a {
                fmt_time.push(game.created_at.format("%b %d, %Y %I:%M").to_string());
                let game_piece_count: i32 = (
                    //Auto
                    game.auto_l1.unwrap() +
                    game.auto_l2.unwrap() +
                    game.auto_l3.unwrap() +
                    game.auto_l4.unwrap() +
                    game.auto_algae_processor.unwrap() +
                    game.auto_algae_barge.unwrap() +
                    game.auto_algae_remove.unwrap() +

                    //Teleop
                    game.teleop_l1.unwrap() +
                    game.teleop_l2.unwrap() +
                    game.teleop_l3.unwrap() +
                    game.teleop_l4.unwrap() +
                    game.teleop_algae_processor.unwrap() +
                    game.teleop_algae_barge.unwrap() +
                    game.teleop_algae_remove.unwrap()
                );
                
                total_game_piece.push(game_piece_count);           
            }
            let zipped: Vec<_> = fmt_time
                .iter()
                .zip(a.iter())
                .zip(total_game_piece.iter())
                .map(|((time, e), pieces)| (time, e, pieces))
                .collect();


            Template::render("search", context! {entries: zipped})
        },
        Err(a) => {
            println!("{a}");
            return Template::render("error", context! { error: "Unkown error" });
        },
    }
}
