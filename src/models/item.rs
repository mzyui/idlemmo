use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub level: i64,
}
