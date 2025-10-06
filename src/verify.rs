
/* 
use reqwest::{header::HeaderMap, Client};
use rocket::{http::CookieJar, State};
use rocket_dyn_templates::{context, Template};
use sqlx::{Row};

pub async fn get_unq_event_codes(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    let a = sqlx::query(" 
    SELECT DISTINCT event_code
    FROM scouting_entry
    WHERE is_verified = 'Unverifed'")
    .fetch_all(pool)
    .await;
    let mut event_codes: Vec<String> = Vec::new();
    let mut types: Vec<String> = Vec::new();
    match a {
        Ok(a) => {
            for thing in a {
                event_codes.push(thing.get(0));
                let a = sqlx::query(" 
                SELECT DISTINCT 
                FROM scouting_entry
                WHERE is_verified = 'Unverifed'")
                .fetch_all(pool)
                .await; //TODO: Get types and use them for api request
            }
        },
        Err(a) => {
            return Err(a);
        },
    }

    Ok(event_codes)
}


#[get("/verify")]
pub async fn verify(pool: &State<SqlitePool>, client: &State<Client>, headers: &State<HeaderMap>, jar: &CookieJar<'_>) -> Template {
    //Get every event that needs to be matched
    let event_codes = match get_unq_event_codes(pool).await {
        Ok(a) => a,
        Err(a) => {
            return Template::render("error", context! { error: "Database error" });
        },
    };



    //TODO: 
    //For now, asumming season 2025 will be ok, but as we enter next season we need to fix this, but product must go out NOWWWW

    //Great, now we have event codes, time to use them!
    for code in event_codes {
        let request = client.request(reqwest::Method::GET, "https://frc-api.firstinspires.org/v3.0/2025/scores/MNST/qual")
        .headers(headers.inner().clone());
    }

    Template::render("error", context![error: "Done"])
}

    */