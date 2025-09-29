use rocket::{form::Form, State};
use rocket_dyn_templates::{context, Template};
use sqlx::{SqlitePool, Row};
use uuid::Uuid;

#[derive(Debug, FromForm)]
pub struct UserRequestLogin {
    #[field(name = "username")] pub username: String,
}



#[post("/login", data = "<form_data>")]
pub async fn login(pool: &State<SqlitePool>, form_data: Form<UserRequestLogin>) -> Template {
    let user_result = sqlx::query(r#"
        SELECT id, can_read, can_write, is_admin
        FROM user_list
        WHERE username = ?
    "#)
    .bind(&form_data.username) // Bind the username to the query
    .fetch_optional(pool.inner()) // Fetch one or none
    .await;

    // Handle the query result
    let (user_id, can_read, can_write, is_admin) = match user_result {
        Ok(Some(row)) => {
            // Extract id (UUID) and passhash from the row
            let user_id: Uuid = row.get("id");
            let can_read: String = match row.get("can_read") {
                true => "true".to_string(),
                false => "false".to_string()
            };
            let can_write: String = match row.get("can_write") {
                true => "true".to_string(),
                false => "false".to_string()
            };
            let is_admin: String = match row.get("is_admin") {
                true => "true".to_string(),
                false => "false".to_string()
            };
            (user_id, can_read, can_write, is_admin)
        },
        Ok(None) => {return Template::render("login", context! [state: "No user found", uuid: "", can_read: "", can_write: "", username: "", is_admin: ""]);}, // No user found
        Err(_) => {return Template::render("login", context! [state: "Database Error", uuid: "", can_read: "", can_write: "", username: "", is_admin: ""]);}, // Database error occurred
    };
    


    Template::render("login", context! [state: "Logined in!", 
    uuid: user_id,
    username: &form_data.username,
    can_read: can_read,
    can_write: can_write, 
    is_admin: is_admin
        ])
}
