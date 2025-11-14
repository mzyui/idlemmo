use crate::{
    error::{AppError, Result},
    lazy_regex,
};
use chrono::Duration;
use enum_iterator::Sequence;
use regex::Regex;
use serde::de::Error as SerdeDeError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use tracing::{debug, info};

#[derive(Default, Serialize, Debug, Clone, Sequence, PartialEq, Eq, PartialOrd, Ord)]
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

impl std::fmt::Display for SkillType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'de> Deserialize<'de> for SkillType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer).map_err(D::Error::custom)?;
        SkillType::from_str(&s).map_err(D::Error::custom)
    }
}

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
    pub level_required: u64,
    pub wait_length_ms: Option<u64>,
    #[serde(default, deserialize_with = "extract_requirements_item")]
    pub requirements: Vec<SkillItem>,
    pub quantity_requirement: Option<u64>,
}

fn duration_from_str<'de, D, E>(deserializer: D) -> std::result::Result<Duration, E>
where
    D: Deserializer<'de>,
    E: SerdeDeError,
{
    let duration_string = String::deserialize(deserializer).map_err(SerdeDeError::custom)?;
    debug!(?duration_string, "Deserializing duration from string.");

    let mut days = 0;
    let mut hours = 0;
    let mut minutes = 0;

    for part in duration_string.split_whitespace() {
        if let Some(days_str) = part.strip_suffix("d") {
            days = days_str.parse::<u64>().map_err(SerdeDeError::custom)?;
            debug!(?days_str, "Parsed days.");
        } else if let Some(hours_str) = part.strip_suffix("h") {
            hours = hours_str.parse::<u64>().map_err(SerdeDeError::custom)?;
            debug!(?hours_str, "Parsed hours.");
        } else if let Some(minutes_str) = part.strip_suffix("m") {
            minutes = minutes_str.parse::<u64>().map_err(SerdeDeError::custom)?;
            debug!(?minutes_str, "Parsed minutes.");
        }
    }

    let duration = Duration::days(days as i64)
        + Duration::hours(hours as i64)
        + Duration::minutes(minutes as i64);
    debug!(?duration, "Constructed duration.");
    Ok(duration)
}

fn number_from_string<'de, D, E>(deserializer: D) -> std::result::Result<u64, E>
where
    D: Deserializer<'de>,
    E: SerdeDeError,
{
    let v = Value::deserialize(deserializer).map_err(SerdeDeError::custom)?;
    match v {
        Value::Number(n) => Ok(n.as_u64().unwrap_or_default()),
        Value::String(s) => {
            let cleaned = s.replace(',', "");
            cleaned
                .parse::<u64>()
                .map_err(|e| SerdeDeError::custom(e.to_string()))
        }
        _ => Err(SerdeDeError::custom("Not a valid number".to_string())),
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Metrics {
    #[serde(deserialize_with = "number_from_string", default)]
    pub items_gathered: u64,
    #[serde(deserialize_with = "duration_from_str", default)]
    pub time_spent: Duration,
    #[serde(deserialize_with = "number_from_string", default)]
    pub total_experience: u64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SkillData {
    pub skill_type: SkillType,
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
