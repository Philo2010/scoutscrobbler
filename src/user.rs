use rocket::{form::Form, State};
use sqlx::{Error, SqlitePool};
use uuid::Uuid;

const bcrypt_cost: u32 = 12;

#[derive(Debug, FromForm)]
pub struct UserRequestCreate {
    #[field(name = "passcreate")] pub passcreate: String,
    #[field(name = "username")] pub username: String,
    #[field(name = "password")] pub password: String,
    #[field(name = "can_write")] pub can_write: String,
    #[field(name = "can_read")] pub can_read: String,
}



#[post("/new_user", data = "<form_data>")]
pub async fn new_user(
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
    let uuid = Uuid::new_v4();
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
