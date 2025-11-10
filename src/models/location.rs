use serde::{Deserialize, Serialize};

use super::{item::Item, skill::Skill};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Location {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub recommended_level: i64,
    pub teleport_cost: i64,
    pub distance: i64,
    pub enemies: Vec<Item>,
    pub dungeons: Vec<Item>,
    pub skill_items: Vec<Skill>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum TravelMode {
    Walk,
    Teleport,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct LocationResponseData {
    #[serde(default)]
    pub status: String,
    pub message: String,
}
