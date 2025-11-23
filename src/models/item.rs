use chrono::{DateTime, Duration, NaiveDateTime, NaiveTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::Value;
use std::{collections::HashMap, str::FromStr};

use crate::{
    models::{game_action::SkillType, player_stats::Metrics},
    utils::serde::{deserialize_duration, deserialize_requirements, deserialize_skill_type},
};

#[derive(Debug, Default)]
pub enum FilterBy {
    #[default]
    HighestLevelRequired,
    LowestLevelRequired,
    // FastestTime,
    // LongestTime,
    // HighestExperience,
    // LowestExperience,
    ItemName(String),
}

#[derive(Debug, Deserialize)]
pub struct SkillData {
    pub items: Vec<Item>,
    pub metrics: Metrics,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    pub id: u64,
    pub name: String,
    pub key: u64,
    #[serde(deserialize_with = "deserialize_skill_type")]
    pub skill: SkillType,
    #[serde(default, deserialize_with = "deserialize_requirements")]
    pub requirements: Vec<Requirement>,
    #[serde(rename = "item")]
    pub item_info: ItemInfo,
}

#[derive(Debug, Deserialize)]
pub struct Requirement {
    pub item_id: u64,
    #[serde(rename = "item")]
    pub item_info: ItemInfo,
    pub player_quantity: Option<u64>,
    pub quantity_requirement: u64,
    pub vendor_price: Option<u64>,
    pub inspect_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ItemInfo {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    #[serde(rename = "meta_data")]
    pub effect: ItemEffect,
    pub quality: Quality,
    pub quantity: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Sword,
    Dagger,
    Bow,
    Helmet,
    Chestplate,
    Gauntlets,
    Shield,
    Greaves,
    Boots,
    Log,
    Fish,
    Bait,
    Food,
    CraftingMaterial,
    PetEgg,
    MetalBar,
    Potion,
    EssenceCrystal,
    EmptyCrystal,
    Ore,
    Recipe,
    Skin,
    CampaignItem,
    Chest,
    FishingRod,
    Pickaxe,
    FellingAxe,
    Special,
    Membership,
    Tokens,
    Collectable,
    UpgradeStone,
    Cake,
    Vial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quality {
    Standard,
    Refined,
    Premium,
    Epic,
    Legendary,
    Mythic,
    Unique,
}

// --- Structs/Enums from src/models/effect.rs ---
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ItemEffect {
    Timed(Vec<TimedEffect>),
    Skill(Vec<SkillEffect>),
    Instant(InstantEffect),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TimedEffect {
    pub attribute: String,
    #[serde(deserialize_with = "deserialize_duration")]
    pub length: Duration,
    pub target: String,
    pub value: u64,
    pub value_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SkillEffect {
    pub attribute: String,
    pub available_uses: u64,
    pub target: String,
    #[serde(rename = "type")]
    pub effect_type: String,
    pub value: u64,
    pub value_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InstantEffect {
    pub health: u64,
    pub hunger: u64,
}
