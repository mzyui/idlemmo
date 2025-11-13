use crate::{
    error::{AppError, Result},
    lazy_regex,
};
use enum_iterator::Sequence;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error as SerdeDeError;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

#[derive(
    Default,
    Debug,
    Clone,
    Deserialize_enum_str,
    Serialize_enum_str,
    Sequence,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum SkillType {
    #[default]
    Woodcutting,
    Mining,
    Fishing,
    Alchemy,
    Smelting,
    Cooking,
    Forge,
    Meditation,
    Travelling,
}

impl SkillType {
    pub fn from_str(input_string: &str) -> Result<Self> {
        for skill_type in enum_iterator::all::<Self>() {
            if skill_type.to_string().to_lowercase() == input_string.to_lowercase() {
                return Ok(skill_type);
            }
        }
        Err(AppError::Parse("Failed to parse skill type.".into()))
    }
}

#[allow(clippy::unnecessary_wraps)]
fn extract_requirements_item<'de, D>(deserializer: D) -> Result<Vec<SkillItem>>
where
    D: Deserializer<'de>,
{
    let mut required_items = vec![];
    if let Some(json_object) = serde_json::Value::deserialize(deserializer)
        .map_err(|e| AppError::SerdeJson(SerdeDeError::custom(e.to_string())))?
        .as_object()
    {
        for item_value in json_object.values() {
            let skill_item = SkillItem::deserialize(item_value)
                .map_err(|e| AppError::SerdeJson(SerdeDeError::custom(e.to_string())))?;
            required_items.push(skill_item);
        }
    }
    Ok(required_items)
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, PartialOrd, Ord, Eq)]
pub struct SkillItem {
    #[serde(alias = "item_id")]
    pub id: i64,
    pub name: String,
    #[serde(rename = "skill")]
    pub skill_type: SkillType,
    pub level_required: i64,
    #[serde(default)]
    pub wait_length_ms: i64,
    #[serde(default)]
    pub requirements: Vec<SkillItem>,
    #[serde(default)]
    pub quantity_requirement: i64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SkillData {
    pub skill_item_id: u64,
    pub quantity: u64,
    #[serde(default)]
    pub essence_crystal: Option<u64>,
    #[serde(default)]
    pub auto_purchase: bool,
}

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

#[derive(Debug, Default)]
pub struct SkillConfig {
    pub skill_type: SkillType,
    pub essence_crystal: Option<u64>,
    pub auto_purchase: bool,
    pub filter_by: FilterBy,
}
