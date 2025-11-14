use serde::{Deserialize, Serialize};

use super::{item::Item, skill::SkillItem};

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Location {
    pub id: u64,
    pub key: String,
    pub name: String,
    pub recommended_level: u64,
    pub teleport_cost: u64,
    pub distance: u64,
    pub enemies: Vec<Item>,
    pub dungeons: Vec<Item>,
    pub skill_items: Vec<SkillItem>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum TravelMode {
    Walk,
    Teleport,
}
