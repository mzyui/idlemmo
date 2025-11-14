use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Item {
    pub id: u64,
    pub name: String,
    pub level: u64,
}
