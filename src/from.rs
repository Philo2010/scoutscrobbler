use rocket::form::Form;
use serde::Deserialize;

#[derive(Debug, FromForm, Deserialize)]
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
