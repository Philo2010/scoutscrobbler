use rocket::http::CookieJar;
use rocket_dyn_templates::{context, Template};
use sqlx::PgPool;

use crate::queue::check_if_admin;


#[get("/queue_remove")]
pub async fn queue_remove(pool: &rocket::State<PgPool>, jar: &CookieJar<'_>) -> Template {

    let userid_string = match jar.get("uuid") {
        Some(a) =>  a.value(),
        None => {
            return Template::render("error", context![error: "You are not logined in"]);
        },
    };

    match check_if_admin(userid_string, pool).await {
        Some(a) => {
            return a;
        },
        None => {},
    };

    //Remove everything from queue
    let result = sqlx::query(r#"
    DELETE FROM scouting_entry
    "#).execute(pool.inner()).await;
    
    match result {
        Ok(_) => {
        },
        Err(a) => {
            println!("{a}");
            return Template::render("error", context![error: "Database error"]);
        },
    }

    let result = sqlx::query(r#"
    ALTER SEQUENCE scouting_entry_id_seq RESTART WITH 1
    "#).execute(pool.inner()).await;

    match result {
        Ok(_) => {
            return Template::render("yippy", context! {});
        },
        Err(a) => {
            println!("{a}");
            return Template::render("error", context![error: "Database error"]);
        },
    }
}