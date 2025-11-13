use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub level: i64,
}
