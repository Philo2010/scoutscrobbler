#[macro_use] extern crate rocket;

use core::hash;
use std::default;
use std::str::{Bytes, FromStr};
use std::sync::atomic::{AtomicUsize, Ordering};

use rocket::fs::FileServer;
use rocket::serde::Serialize;
use rocket::{serde::Deserialize, State};
use rocket::form::Form;
use rocket_dyn_templates::{context, Template};
use sqlx::{Error, FromRow, Row};
use rocket::fs::relative;
use rocket_db_pools::sqlx::{self, SqlitePool};
use sqlx::types::Uuid;
use rocket::http::{Cookie, CookieJar};

const bcrypt_cost: u32 = 12;

#[derive(Debug, FromForm)]
pub struct ScoutingForm {
    // Auto
    #[field(name = "auto-L1")] pub auto_l1: i32,
    #[field(name = "auto-L2")] pub auto_l2: i32,
    #[field(name = "auto-L3")] pub auto_l3: i32,
    #[field(name = "auto-L4")] pub auto_l4: i32,
    #[field(name = "auto-alpro")] pub auto_algae_processor: i32,
    #[field(name = "auto-albar")] pub auto_algae_barge: i32,
    #[field(name = "auto-alrem")] pub auto_algae_remove: i32,

    // Teleop
    #[field(name = "teleop-L1")] pub teleop_l1: i32,
    #[field(name = "teleop-L2")] pub teleop_l2: i32,
    #[field(name = "teleop-L3")] pub teleop_l3: i32,
    #[field(name = "teleop-L4")] pub teleop_l4: i32,
    #[field(name = "teleop-alpro")] pub teleop_algae_processor: i32,
    #[field(name = "teleop-albar")] pub teleop_algae_barge: i32,
    #[field(name = "teleop-alrem")] pub teleop_algae_remove: i32,

    // Endgame
    #[field(name = "rating")] pub defense_rating: i32,
    #[field(name = "climb")] pub climb_type: String,
    #[field(name = "comment")] pub comment: String,
}
#[derive(Debug, sqlx::FromRow, Serialize)]
struct ScoutingEntry {
    id: i64,
    created_at: String,

    auto_l1: Option<i32>,
    auto_l2: Option<i32>,
    auto_l3: Option<i32>,
    auto_l4: Option<i32>,
    auto_algae_processor: Option<i32>,
    auto_algae_barge: Option<i32>,
    auto_algae_remove: Option<i32>,

    teleop_l1: Option<i32>,
    teleop_l2: Option<i32>,
    teleop_l3: Option<i32>,
    teleop_l4: Option<i32>,
    teleop_algae_processor: Option<i32>,
    teleop_algae_barge: Option<i32>,
    teleop_algae_remove: Option<i32>,

    defense_rating: Option<i32>,
    climb_type: Option<String>,
    comment: Option<String>,
}

#[derive(Debug, FromForm)]
struct UserRequestLogin {
    #[field(name = "username")] pub username: String,
    #[field(name = "password")] pub password: String,
}

#[derive(Debug, FromForm)]
struct UserRequestCreate {
    #[field(name = "passcreate")] pub passcreate: String,
    #[field(name = "username")] pub username: String,
    #[field(name = "password")] pub password: String,
    #[field(name = "can_write")] pub can_write: String,
    #[field(name = "can_read")] pub can_read: String,
}


#[derive(Debug, sqlx::FromRow, Serialize)]
struct User<'a> {
    pub id: &'a [u8],
    pub username: String,
    pub passhash: String,
    pub can_write: bool,
    pub can_read: bool,
}

#[post("/new_user", data = "<form_data>")]
async fn new_user(
    pool: &State<SqlitePool>,        // Pool extracted from Rocket's State
    form_data: Form<UserRequestCreate>  // Form data from the request
) -> &'static str {
    // Validate admin password
    if form_data.passcreate != "VerySexyPass" {
        return "Invalid admin password, please contact Philip for password.";
    }

    // Parse can_read
    let can_read: i32 = match form_data.can_read.as_str() {
        "true" => 1,
        "false" => 0,
        _ => return "Invalid form: can_read must be 'true' or 'false'.",
    };

    // Parse can_write
    let can_write: i32 = match form_data.can_write.as_str() {
        "true" => 1,
        "false" => 0,
        _ => return "Invalid form: can_write must be 'true' or 'false'.",
    };

    // Hash the password
    let passhash = match bcrypt::hash(&form_data.password, 12) {
        Ok(hash) => hash,
        Err(_) => return "Hash failed while processing password.",
    };
    let uuid =Uuid::new_v4();
    let id: Vec<u8> = uuid.as_bytes().to_vec();

    // Execute the SQL query
    let result = sqlx::query("
        INSERT INTO user_list (id, username, passhash, can_write, can_read)
        VALUES (?, ?, ?, ?, ?)
    ")
    .bind(id)  // UUID as bytes for the BLOB column
    .bind(&form_data.username)       // Bind username
    .bind(passhash)                  // Bind hashed password
    .bind(can_write)                  // Bind can_read as i32 (1 for true, 0 for false)
    .bind(can_read)                 // Bind can_write as i32 (1 for true, 0 for false)
    .execute(pool.inner())          // Execute query on the pool
    .await;

    match result {
        Ok(_) => "User created successfully.",
        Err(e) => match e {
            Error::Database(db_error) => {
                // Check for the specific error code or message for unique constraint violations
                if db_error.to_string().contains("UNIQUE constraint failed") {
                    "Username already taken. Please choose another one."
                } else {
                    eprintln!("Database error: {}", db_error);
                    "Database error occurred. Please try again."
                }
            },
            _ => {
                eprintln!("Unexpected error: {}", e);
                "Unexpected error occurred. Please try again."
            }
        },
    }
}

#[post("/login", data = "<form_data>")]
async fn login(pool: &State<SqlitePool>, form_data: Form<UserRequestLogin>) -> Template {
    let user_result = sqlx::query(r#"
        SELECT id, passhash, can_read, can_write
        FROM user_list
        WHERE username = ?
    "#)
    .bind(&form_data.username) // Bind the username to the query
    .fetch_optional(pool.inner()) // Fetch one or none
    .await;

    // Handle the query result
    let (user_id, passhash, can_read, can_write) = match user_result {
        Ok(Some(row)) => {
            // Extract id (UUID) and passhash from the row
            let user_id: Uuid = row.get("id");
            let passhash: String = row.get("passhash");
            let can_read: bool = row.get("can_read");
            let can_write: bool = row.get("can_write");
            (user_id, passhash, can_read, can_write)
        },
        Ok(None) => {return Template::render("login", context! [state: "No user found", uuid: ""]);}, // No user found
        Err(_) => {return Template::render("login", context! [state: "Database Error",uuid: ""]);}, // Database error occurred
    };
    
    let is_vaild = match bcrypt::verify(&form_data.password, &passhash) {
        Ok(a) => a,
        Err(_) => {return Template::render("login", context! [state: "An unknown problem has occered", uuid: ""]);},
    };
    
    if (is_vaild) {
        Template::render("login", context! [state: "Logined in!", 
        uuid: user_id,
        username: &form_data.username,
        can_read: can_read,
        can_write: can_write
         ])
    } else {
        Template::render("login", context! [state: "Wrong password", uuid: ""])
    }
}

#[get("/entries")]
async fn view_entries(db: &State<SqlitePool>, jar: &CookieJar<'_>) -> Template {

    //Get cookie
    let userid_string = match jar.get("uuid") {
        Some(a) =>  a.value(),
        None => {
            let entries: Vec<ScoutingEntry> = Vec::new();
            return Template::render("entries", context! { entries });
        },
    };

    let userid = match Uuid::from_str(userid_string) {
        Ok(a) => a,
        Err(_) => {
            let entries: Vec<ScoutingEntry> = Vec::new();
            return Template::render("entries", context! { entries });
        },
    };


    let user_request = sqlx::query(r#"
        SELECT can_read
        FROM user_list
        WHERE id = ?
    "#)
    .bind(userid)
    .fetch_optional(db.inner())
    .await; //TODO: fix make new user to get perms right


    let can_read = match user_request {
        Ok(Some(a)) => {
            a.get::<bool, _>(0)
        },
        Ok(None) => {
            let entries: Vec<ScoutingEntry> = Vec::new();
            return Template::render("entries", context! { entries });
        }
        Err(_) => {
            let entries: Vec<ScoutingEntry> = Vec::new();
            return Template::render("entries", context! { entries });
        },
    };
    println!("{}", can_read);

    if !can_read {
        let entries: Vec<ScoutingEntry> = Vec::new();
        return Template::render("entries", context! { entries });
    }

    let entries = sqlx::query_as::<_, ScoutingEntry>(r#"
        SELECT 
          se.id,
          se.created_at,
          ad.L1 AS auto_l1, ad.L2 AS auto_l2, ad.L3 AS auto_l3, ad.L4 AS auto_l4,
          ad.algae_processor AS auto_algae_processor,
          ad.algae_barge AS auto_algae_barge,
          ad.algae_remove AS auto_algae_remove,
          td.L1 AS teleop_l1, td.L2 AS teleop_l2, td.L3 AS teleop_l3, td.L4 AS teleop_l4,
          td.algae_processor AS teleop_algae_processor,
          td.algae_barge AS teleop_algae_barge,
          td.algae_remove AS teleop_algae_remove,
          eg.defense_rating,
          eg.climb_type,
          eg.comment
        FROM scouting_entry se
        LEFT JOIN auto_data ad ON ad.scouting_id = se.id
        LEFT JOIN teleop_data td ON td.scouting_id = se.id
        LEFT JOIN endgame_data eg ON eg.scouting_id = se.id
        ORDER BY se.id DESC
    "#)
    .fetch_all(db.inner())
    .await
    .unwrap_or_default();

    Template::render("entries", context! { entries })
}


#[post("/submit", data = "<form_data>")]
async fn submit(pool: &rocket::State<SqlitePool>, jar: &CookieJar<'_>, form_data: Form<ScoutingForm>) -> &'static str {

    let userid_string = match jar.get("uuid") {
        Some(a) =>  a.value(),
        None => {
            let entries: Vec<ScoutingEntry> = Vec::new();
            "Not logined in"
        },
    };

    let userid = match Uuid::from_str(userid_string) {
        Ok(a) => a,
        Err(_) => {
            let entries: Vec<ScoutingEntry> = Vec::new();
            return "Not logined in";
        },
    };


    let user_request = sqlx::query(r#"
        SELECT can_write
        FROM user_list
        WHERE id = ?
    "#)
    .bind(userid)
    .fetch_optional(pool.inner())
    .await; //TODO: fix make new user to get perms right


    let can_write = match user_request {
        Ok(Some(a)) => {
            a.get::<bool, _>(0)
        },
        Ok(None) => {
            return "Can't find user";
        }
        Err(_) => {
            return "Database Error";
        },
    };

    if !can_write {
        return "You don't have writing perms!";
    }

    let form = form_data.into_inner();

    // Insert into scouting_entry
    let scouting_id = sqlx::query("INSERT INTO scouting_entry DEFAULT VALUES")
        .execute(pool.inner())
        .await
        .expect("Insert failed")
        .last_insert_rowid();

    // Insert auto data
    sqlx::query("
        INSERT INTO auto_data (scouting_id, L1, L2, L3, L4, algae_processor, algae_barge, algae_remove)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    ")
    .bind(scouting_id)
    .bind(form.auto_l1)
    .bind(form.auto_l2)
    .bind(form.auto_l3)
    .bind(form.auto_l4)
    .bind(form.auto_algae_processor)
    .bind(form.auto_algae_barge)
    .bind(form.auto_algae_remove)
    .execute(pool.inner())
    .await
    .unwrap();

    // Insert teleop data
    sqlx::query("
        INSERT INTO teleop_data (scouting_id, L1, L2, L3, L4, algae_processor, algae_barge, algae_remove)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    ")
    .bind(scouting_id)
    .bind(form.teleop_l1)
    .bind(form.teleop_l2)
    .bind(form.teleop_l3)
    .bind(form.teleop_l4)
    .bind(form.teleop_algae_processor)
    .bind(form.teleop_algae_barge)
    .bind(form.teleop_algae_remove)
    .execute(pool.inner())
    .await
    .unwrap();

    // Insert endgame
    sqlx::query("
        INSERT INTO endgame_data (scouting_id, defense_rating, climb_type, comment)
        VALUES (?, ?, ?, ?)
    ")
    .bind(scouting_id)
    .bind(form.defense_rating)
    .bind(&form.climb_type)
    .bind(&form.comment)
    .execute(pool.inner())
    .await
    .unwrap();

    "Scouting data submitted successfully"
}



#[launch]
async fn rocket() -> _ {
    let db_pool = SqlitePool::connect("sqlite:main.sqlite").await.expect("Failed to connect to DB");
    rocket::build()
    .manage(db_pool)
    .attach(Template::fairing())
    .mount("/", routes![submit, view_entries, new_user, login])
    .mount("/", FileServer::from(relative!("static")))
}
