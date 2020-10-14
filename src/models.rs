use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GroceryItem {
    pub id: i32,
    pub name: String,
    pub checked_off: bool,
    pub position: i32,

    pub checked_off_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewGroceryItem {
    pub name: String,
    #[serde(default)]
    pub checked_off: bool,
    pub position: u32,
}
