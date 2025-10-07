//This file is used in a few ways, mainly with 

use reqwest::Client;
use serde::Deserialize;
use reqwest::header::HeaderMap;

#[derive(Debug, Deserialize)]
pub struct TbaMatch {
    pub comp_level: String,
    pub match_number: i32,
    pub set_number: i32,
    pub alliances: Alliances,
}

#[derive(Debug, Deserialize)]
pub struct Alliances {
    pub red: Alliance,
    pub blue: Alliance,
}

#[derive(Debug, Deserialize)]
pub struct Alliance {
    pub score: Option<i32>,
    pub team_keys: Vec<String>,
}

pub async fn pull_from_blue(client: &Client, headers: &HeaderMap, event_code: &String) -> Result<Vec<TbaMatch>, reqwest::Error> {
    //https://www.thebluealliance.com/api/v3/event/2025tacy/matches/simple

    //Make a request to get the major data
    let request = client.get(format!("https://www.thebluealliance.com/api/v3/event/{}/matches/simple", event_code))
        .headers(headers.clone()).send().await?;

    let body: Vec<TbaMatch> = request.json().await?;
    

    Ok(body)
}

