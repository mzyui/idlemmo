use chrono::{DateTime, Duration, Utc};
use enum_iterator::Sequence;
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use url::Url;

use crate::error::{AppError, Result};
use crate::models::world::{Location, SkillItem};
use crate::utils::serde::{
    deserialize_duration, deserialize_optional_duration, deserialize_skill_type,
};

#[derive(Debug)]
pub struct SkillRecommendation<'a> {
    pub skill: SkillType,
    pub level: u64,
    pub location: &'a Location,
    pub item: &'a SkillItem,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ActiveAction {
    pub kind: SkillType,
    pub item: Option<String>,
    #[serde(default)]
    pub quantity: u64,
    #[serde(default)]
    pub max_quantity: u64,
    pub current_progress: Option<CurrentProgress>,
    pub metadata: ActionMetaData,
    #[serde(default, deserialize_with = "deserialize_duration")]
    pub expires_in: Duration,
    pub expires_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub wait_length: Duration,
    // #[serde(flatten)]
    // pub extra: Option<Value>,
}

impl<'de> Deserialize<'de> for ActiveAction {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "snake_case")]
        struct Raw {
            #[serde(rename = "type", deserialize_with = "deserialize_skill_type")]
            kind: SkillType,
            item: Option<Item>,
            #[serde(default)]
            quantity: u64,
            #[serde(default)]
            max_quantity: u64,
            current_progress: Option<CurrentProgress>,
            #[serde(rename = "meta_data")]
            metadata: ActionMetaData,
            #[serde(default, deserialize_with = "deserialize_duration")]
            expires_in: Duration,
            #[serde(default, deserialize_with = "deserialize_duration")]
            wait_length: Duration,
            #[serde(flatten)]
            extra: Option<Value>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let expires_at = Utc::now()
            .checked_add_signed(raw.expires_in)
            .unwrap_or_default();
        let item = raw
            .item
            .map(|i| i.name.as_str().map(|s| s.to_string()))
            .unwrap_or_default();
        Ok(ActiveAction {
            item,
            kind: raw.kind,
            quantity: raw.quantity,
            max_quantity: raw.max_quantity,
            current_progress: raw.current_progress,
            metadata: raw.metadata,
            expires_in: raw.expires_in,
            expires_at,
            wait_length: raw.wait_length,
            // extra: raw.extra,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Item {
    pub name: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CurrentProgress {
    pub percentage: f64,
    #[serde(default, deserialize_with = "deserialize_duration")]
    pub time_remaining_until_next_loop: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Default, Deserialize, Serialize)]
pub enum SkillType {
    #[default]
    None,
    Woodcutting,
    Mining,
    Fishing,
    Alchemy,
    Smelting,
    Cooking,
    Forge,
    Meditation,
    Other(String),
}

impl FromStr for SkillType {
    type Err = AppError;

    fn from_str(input_string: &str) -> std::result::Result<Self, Self::Err> {
        let normalized_input = input_string.to_lowercase();
        match normalized_input.as_str() {
            "none" => Ok(SkillType::None),
            "woodcutting" => Ok(SkillType::Woodcutting),
            "mining" => Ok(SkillType::Mining),
            "fishing" => Ok(SkillType::Fishing),
            "alchemy" => Ok(SkillType::Alchemy),
            "smelting" => Ok(SkillType::Smelting),
            "cooking" => Ok(SkillType::Cooking),
            "forge" => Ok(SkillType::Forge),
            "meditation" => Ok(SkillType::Meditation),
            _ => Err(AppError::Parse(format!(
                "Failed to parse skill type: {}",
                input_string
            ))),
        }
    }
}

impl fmt::Display for SkillType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SkillType::None => write!(f, "none"),
            SkillType::Woodcutting => write!(f, "woodcutting"),
            SkillType::Mining => write!(f, "mining"),
            SkillType::Fishing => write!(f, "fishing"),
            SkillType::Alchemy => write!(f, "alchemy"),
            SkillType::Smelting => write!(f, "smelting"),
            SkillType::Cooking => write!(f, "cooking"),
            SkillType::Forge => write!(f, "forge"),
            SkillType::Meditation => write!(f, "meditation"),
            SkillType::Other(s) => write!(f, "{}", s),
        }
    }
}

// --- Structs from src/models/metadata.rs ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillMetaData {
    #[serde(default)]
    pub kind: String,
    pub value: u64,
    #[serde(deserialize_with = "deserialize_duration", default)]
    pub length: Duration,
    #[serde(deserialize_with = "deserialize_skill_type")]
    pub target: SkillType,
    pub attribute: String,
    pub value_type: String,
    #[serde(default)]
    pub available_uses: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionMetaData {
    pub exp_per_second: f64,
    pub exp_per_enemy: Option<f64>,
    pub total_bonus_enemies: Option<u64>,
    pub enemy_queue: Option<EnemyQueue>,
    pub bonus_enemy: Option<BonusEnemy>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnemyQueue {
    pub preview: Vec<EnemyPreview>,
    pub offset: Option<u64>,
    pub total: Option<u64>,
    pub chunk_size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnemyPreview {
    pub enemy_id: u64,
    pub name: String,
    pub image_url: String,
    pub position: u64,
    pub is_current: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BonusEnemy {
    pub active: bool,
    pub id: u64,
    #[serde(deserialize_with = "deserialize_optional_duration")]
    pub enemy_available_in: Option<Duration>,
    #[serde(deserialize_with = "deserialize_optional_duration")]
    pub enemy_shown_for: Option<Duration>,
    #[serde(deserialize_with = "deserialize_optional_duration")]
    pub wait_between_ui_cycles: Option<Duration>,
    #[serde(deserialize_with = "deserialize_optional_duration")]
    pub expires_in: Option<Duration>,
    pub exp_per_enemy: Option<u64>,
    pub length_per_power_hunt: Option<u64>,
    #[serde(deserialize_with = "deserialize_optional_duration")]
    pub cooldown_expires_in: Option<Duration>,
}
