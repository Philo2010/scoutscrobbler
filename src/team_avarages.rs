use rocket::State;
use rocket_dyn_templates::{context, Template};


pub async fn get_team_avarage(pool: &sqlx::PgPool) -> Result<Vec<(i32, f64)>, sqlx::Error> {
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

    let mut avarages: Vec<(i32, f64)> = Vec::with_capacity(teams.len());

    for t in teams {
        let avarage = sqlx::query_scalar!(r#"
        SELECT AVG(total_score)::float
        FROM scouting_entry
        WHERE team = $1"#, t)
        .fetch_one(pool)
        .await;

        match avarage {
            Ok(Some(a)) => {
                avarages.push((t, a));
            },
            Ok(None) => {continue;}
            Err(_) => {continue;},
        }
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
    avarages.sort_by(|a,b| b.1.total_cmp(&a.1));

    Template::render("teams", context! [entries: avarages])
}