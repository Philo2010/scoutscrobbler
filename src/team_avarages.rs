use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;


#[derive(Serialize)]
struct EntryAvarage {
    team: i32,
    total_score: f64,

    //auto
    auto_l1: f64,
    auto_l2: f64,
    auto_l3: f64,
    auto_l4: f64,
    auto_algae_processor: f64,
    auto_algae_barge: f64,
    auto_algae_remove: f64,

    //teleop
    teleop_l1: f64,
    teleop_l2: f64,
    teleop_l3: f64,
    teleop_l4: f64,
    teleop_algae_processor: f64,
    teleop_algae_barge: f64,
    teleop_algae_remove: f64,

    //Endgame
    climb_deep: f64,
    climb_shallow: f64,
    climb_park: f64,
    game_pieces: f64,
    died: f64
}


pub async fn get_team_avarage(pool: &sqlx::PgPool) -> Result<Vec<EntryAvarage>, sqlx::Error> {
    let teams: Vec<i32> = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT team
        FROM scouting_entry
        WHERE is_verified = 'Unverified'
        AND team IS NOT NULL
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .flatten()
    .collect();

    let mut avarages: Vec<EntryAvarage> = Vec::with_capacity(teams.len());

    for t in teams {
        let avarage = sqlx::query!(r#"
        SELECT
            AVG(se.total_score)::float AS total_score,
            
            AVG(COALESCE(ad.L1, 0))::float AS auto_L1,
            AVG(COALESCE(ad.L2, 0))::float AS auto_L2,
            AVG(COALESCE(ad.L3, 0))::float AS auto_L3,
            AVG(COALESCE(ad.L4, 0))::float AS auto_L4,
            AVG(COALESCE(ad.algae_processor, 0))::float AS auto_algae_processor,
            AVG(COALESCE(ad.algae_barge, 0))::float AS auto_algae_barge,
            AVG(COALESCE(ad.algae_remove, 0))::float AS auto_algae_remove,
            
            AVG(COALESCE(td.L1, 0))::float AS teleop_L1,
            AVG(COALESCE(td.L2, 0))::float AS teleop_L2,
            AVG(COALESCE(td.L3, 0))::float AS teleop_L3,
            AVG(COALESCE(td.L4, 0))::float AS teleop_L4,
            AVG(COALESCE(td.algae_processor, 0))::float AS teleop_algae_processor,
            AVG(COALESCE(td.algae_barge, 0))::float AS teleop_algae_barge,
            AVG(COALESCE(td.algae_remove, 0))::float AS teleop_algae_remove,
            
            100.0 * SUM(CASE WHEN eg.died THEN 1 ELSE 0 END)::float / COUNT(*) AS died,

            100.0 * SUM(CASE WHEN eg.climb_type = 'deep' THEN 1 ELSE 0 END)::float / COUNT(*) AS climb_deep,
            100.0 * SUM(CASE WHEN eg.climb_type = 'shallow' THEN 1 ELSE 0 END)::float / COUNT(*) AS climb_shallow,
            100.0 * SUM(CASE WHEN eg.climb_type = 'park' THEN 1 ELSE 0 END)::float / COUNT(*) AS climb_park,

            AVG(
                COALESCE(ad.L1,0) + COALESCE(ad.L2,0) + COALESCE(ad.L3,0) + COALESCE(ad.L4,0)
                + COALESCE(ad.algae_processor,0) + COALESCE(ad.algae_barge,0) + COALESCE(ad.algae_remove,0)
                + COALESCE(td.L1,0) + COALESCE(td.L2,0) + COALESCE(td.L3,0) + COALESCE(td.L4,0)
                + COALESCE(td.algae_processor,0) + COALESCE(td.algae_barge,0) + COALESCE(td.algae_remove,0)
            )::float AS game_pieces

        FROM scouting_entry se
        LEFT JOIN auto_data ad ON ad.scouting_id = se.id
        LEFT JOIN teleop_data td ON td.scouting_id = se.id
        LEFT JOIN endgame_data eg ON eg.scouting_id = se.id
        WHERE se.team = $1;
    "#, t)
        .fetch_one(pool)
        .await?;
        let data: EntryAvarage = EntryAvarage { 
            team: t, 
            total_score: avarage.total_score.expect("Value not here"),
            auto_l1: avarage.auto_l1.expect("Value not here"),
            auto_l2: avarage.auto_l2.expect("Value not here"),
            auto_l3: avarage.auto_l3.expect("Value not here"),
            auto_l4: avarage.auto_l4.expect("Value not here"),
            auto_algae_processor: avarage.auto_algae_processor.expect("Value not here"),
            auto_algae_barge: avarage.auto_algae_barge.expect("Value not here"),
            auto_algae_remove: avarage.auto_algae_remove.expect("Value not here"),
            teleop_l1: avarage.teleop_l1.expect("Value not here"),
            teleop_l2: avarage.teleop_l2.expect("Value not here"),
            teleop_l3: avarage.teleop_l3.expect("Value not here"),
            teleop_l4: avarage.teleop_l4.expect("Value not here"),
            teleop_algae_processor: avarage.teleop_algae_processor.expect("Value not here"),
            teleop_algae_barge: avarage.teleop_algae_barge.expect("Value not here"),
            teleop_algae_remove: avarage.teleop_algae_remove.expect("Value not here"),
            climb_deep: avarage.climb_deep.expect("Value not here"),
            climb_shallow: avarage.climb_shallow.expect("Value not here"),
            climb_park: avarage.climb_park.expect("Value not here"),
            game_pieces: avarage.game_pieces.expect("Value not here"),
            died: avarage.died.expect("Value not here")
        };

        avarages.push(data);
    }

    Ok(avarages)
}

#[get("/teams")]
pub async fn avarage_team(pool: &State<sqlx::PgPool>) -> Template {

    let mut avarages = match get_team_avarage(pool.inner()).await {
        Ok(a) => a,
        Err(_) => {
            return Template::render("error", context![error: "Database Error"]);
        },
    };

    //sort
    avarages.sort_by(|a,b| b.total_score.total_cmp(&a.total_score));

    Template::render("teams", context! [entries: avarages])
}