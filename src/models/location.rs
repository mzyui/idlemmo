// location.rs
use chrono::Duration;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Result;
use std::fmt;

use crate::models::action::SkillType;
use crate::utils::duration_from_any;

#[allow(dead_code)]
#[derive(Debug)]
pub enum TravelMode {
    Walk,
    Teleport,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub id: u64,
    pub key: String,
    pub name: String,
    pub description: String,
    pub distance: u64,
    pub dungeons: Vec<Dungeon>,
    pub enemies: Vec<Entity>,
    #[serde(deserialize_with = "duration_from_any")]
    pub length: Duration,
    pub points_of_interest: PointsOfInterest,
    pub skill_items: Vec<SkillItem>,
    pub teleport_cost: u64,
    pub recommended_level: u64,
    pub world_bosses: Vec<Entity>,
    pub disabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dungeon {
    pub id: u64,
    pub level: u64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entity {
    pub id: u64,
    pub level: u64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillItem {
    pub id: u64,
    pub level_required: u64,
    pub name: String,
    pub skill: SkillType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PointsOfInterest {
    pub has_bank: bool,
    pub has_dungeons: bool,
    pub has_enemies: bool,
    pub has_shrine: bool,
    pub has_world_bosses: bool,
}
