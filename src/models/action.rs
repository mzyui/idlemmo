use anyhow::Result;
use chrono::TimeDelta;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::{debug, info};

use crate::models::SkillRequestData;

use super::skill::SkillType;

fn deserialize_capitalize<'de, D>(deserializer: D) -> Result<SkillType, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_skill_type_string = String::deserialize(deserializer)?;
    debug!(?raw_skill_type_string, "Deserializing and capitalizing skill type.");

    let mut chars_iterator = raw_skill_type_string.chars();
    let capitalized_skill_type_string = match chars_iterator.next() {
        None => String::new(),
        Some(first_char) => first_char.to_uppercase().collect::<String>() + chars_iterator.as_str(),
    };
    debug!(?capitalized_skill_type_string, "Capitalized skill type string.");
    SkillType::deserialize(capitalized_skill_type_string.into_deserializer())
}

#[derive(Deserialize, Default, Debug)]
struct InnerItem {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    percentage: f64,
    #[serde(default)]
    data: Option<SkillRequestData>,
}

#[allow(clippy::unnecessary_wraps)]
fn extract_item_name<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    debug!("Extracting item name.");
    let action_item_data = InnerItem::deserialize(deserializer).unwrap_or_default();
    debug!(?action_item_data.name, "Extracted item name.");
    Ok(action_item_data.name)
}

fn extract_refresh_data<'de, D>(deserializer: D) -> Result<Option<SkillRequestData>, D::Error>
where
    D: Deserializer<'de>,
{
    debug!("Extracting refresh data.");
    let action_item_data = InnerItem::deserialize(deserializer)?;
    debug!(?action_item_data.data, "Extracted refresh data.");
    Ok(action_item_data.data)
}

fn extract_percentage<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    debug!("Extracting percentage.");
    let action_item_data = InnerItem::deserialize(deserializer)?;
    debug!(?action_item_data.percentage, "Extracted percentage.");
    Ok(action_item_data.percentage)
}

fn deserialize_timedelta_from_milliseconds<'de, D>(deserializer: D) -> Result<TimeDelta, D::Error>
where
    D: Deserializer<'de>,
{
    debug!("Deserializing timedelta from milliseconds.");
    let milliseconds_value = u64::deserialize(deserializer)?;
    debug!(?milliseconds_value, "Deserialized milliseconds value.");
    Ok(TimeDelta::milliseconds(milliseconds_value as i64))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Action {
    #[serde(rename = "type", deserialize_with = "deserialize_capitalize")]
    pub skill_type: SkillType,
    #[serde(rename = "item", deserialize_with = "extract_item_name")]
    pub item_name: Option<String>,
    #[serde(deserialize_with = "extract_percentage")]
    pub current_progress: f64,
    #[serde(deserialize_with = "deserialize_timedelta_from_milliseconds")]
    pub expires_in: TimeDelta,
    pub quantity: Option<u64>,
    pub max_quantity: Option<u64>,
    #[serde(rename = "refresh", deserialize_with = "extract_refresh_data")]
    pub refresh_data: Option<SkillRequestData>,
}
