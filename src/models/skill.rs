use crate::error::{AppError, Result};
use crate::models::{Metrics, action_model::SkillType};
use chrono::Duration;
use enum_iterator::Sequence;
use serde::de::Error as SerdeDeError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use tracing::debug;

#[allow(clippy::unnecessary_wraps)]
fn extract_requirements_item<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<SkillItem>, D::Error>
where
    D: Deserializer<'de>,
{
    debug!("Extracting skill item requirements.");
    let mut required_items = vec![];
    let opt = Option::<Value>::deserialize(deserializer).map_err(D::Error::custom)?;

    if let Some(value) = opt
        && let Some(requirements_json_object) = value.as_object()
    {
        for skill_item_value in requirements_json_object.values() {
            let skill_item = SkillItem::deserialize(skill_item_value).map_err(D::Error::custom)?;
            required_items.push(skill_item);
        }
    }
    debug!(?required_items, "Extracted skill item requirements.");
    Ok(required_items)
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, PartialOrd, Ord, Eq)]
pub struct SkillItem {
    #[serde(alias = "item_id", default)]
    pub id: u64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "skill", default)]
    pub skill_type: SkillType,
    #[serde(default)]
    pub level_required: u64,
    pub wait_length_ms: Option<u64>,
    #[serde(default, deserialize_with = "extract_requirements_item")]
    pub requirements: Vec<SkillItem>,
    pub quantity_requirement: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkillData {
    pub items: Vec<SkillItem>,
    pub metrics: Metrics,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SkillRequestData {
    pub skill_item_id: u64,
    pub quantity: u64,
    #[serde(default)]
    pub essence_crystal: u64,
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
    pub essence_crystal: u64,
    pub auto_purchase: bool,
    pub filter_by: FilterBy,
}

