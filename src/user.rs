use rocket::{form::Form, State};
use sqlx::{Error, PgPool};
use uuid::Uuid;

const bcrypt_cost: u32 = 12;

#[derive(Debug, FromForm)]
pub struct UserRequestCreate {
    #[field(name = "passcreate")] pub passcreate: String,
    #[field(name = "username")] pub username: String,
    #[field(name = "can_write")] pub can_write: String,
    #[field(name = "can_read")] pub can_read: String,
    #[field(name = "is_admin")] pub is_admin: String
}



#[post("/new_user", data = "<form_data>")]
pub async fn new_user(
    pool: &State<PgPool>,        // Pool extracted from Rocket's State
    form_data: Form<UserRequestCreate>  // Form data from the request
) -> &'static str {
    // Validate admin password
    if form_data.passcreate != "VerySexyPass" {
        return "Invalid admin password, please contact Philip for password.";
    }

    // Parse can_read
    let can_read: bool = match form_data.can_read.as_str() {
        "true" => true,
        "false" => false,
        _ => return "Invalid form: can_read must be 'true' or 'false'.",
    };

    // Parse can_write
    let can_write: bool = match form_data.can_write.as_str() {
        "true" => true,
        "false" => false,
        _ => return "Invalid form: can_write must be 'true' or 'false'.",
    };

    //Parse is_admin
    let is_admin: bool = match form_data.is_admin.as_str() {
        "true" => true,
        "false" => false,
        _ => return "Invalid form: is admin must be 'true' or 'false'.",
    };

    let uuid = Uuid::new_v4();

    // Execute the SQL query
    let result = sqlx::query("
        INSERT INTO user_list (id, username, can_write, can_read, is_admin)
        VALUES ($1, $2, $3, $4, $5)
    ")
    .bind(uuid)  // UUID as bytes for the BLOB column
    .bind(&form_data.username)       // Bind username
    .bind(can_write)                  // Bind can_read as i32 (1 for true, 0 for false)
    .bind(can_read)                 // Bind can_write as i32 (1 for true, 0 for false)
    .bind(is_admin)
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
