//HEY ANYONE READING THIS CODE!!!!
//This feature as been scraped by request
//I am keeping this here in case I am in need of reviving this feature onemore
//Until then... You won't be veifying shit.



use std::error::Error;

use reqwest::{header::HeaderMap, Client};
use rocket::{http::CookieJar, State};
use rocket_dyn_templates::{context, Template};
use sqlx::{Row, PgPool};
use crate::blue::*;

pub async fn get_unq_event_codes(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let event_codes: Vec<String> = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT event_code
        FROM scouting_entry
        WHERE is_verified = 'Unverified'
        AND event_code IS NOT NULL
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .flatten()
    .collect();

    Ok(event_codes)
}

//Struct for returning match data
struct ScoreScout {
    score: i32,
    scout_id: i32,
}
struct ScoreMatch {
    red: [ScoreScout, 3],
    blue: [ScoreScout, 3],
}


async fn get_match_by_code_and_id(pool: &PgPool, event: &str, matchnumber: i32) -> Result<
ScoreMatch, sqlx::Error> {

}



#[get("/verify")]
pub async fn verify(pool: &State<PgPool>, client: &State<Client>, headers: &State<HeaderMap>, jar: &CookieJar<'_>) -> Template {
    //Get every event that needs to be matched
    let event_codes = match get_unq_event_codes(pool.inner()).await {
        Ok(a) =>  {
            a
        },
        Err(a) => {
            return Template::render("error", context! { error: "Database error" });
        },
    };



    //TODO: 
    //For now, asumming season 2025 will be ok, but as we enter next season we need to fix this, but product must go out NOWWWW

    //Great, now we have event codes, time to use them!
    for code in event_codes {
        let data = match pull_from_blue(client, headers, &code).await {
            Ok(a) => a,
            Err(_) => {
                //We could not find the data for this event, don't do anything and just move on
                continue;
            },
        };

        //Loop and verify all of our matches
        for gamematch in data {

        }
    }

    Template::render("error", context![error: "Done"])
}