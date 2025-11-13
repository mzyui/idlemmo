use anyhow::Result;
use chrono::TimeDelta;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Deserializer, Serialize};

use crate::models::SkillData;

use super::skill::SkillType;

fn deserialize_capitalize<'de, D>(deserializer: D) -> Result<SkillType, D::Error>
where
    D: Deserializer<'de>,
{
    let input_string = String::deserialize(deserializer)?;

    let mut chars_iterator = input_string.chars();
    let capitalized_string = match chars_iterator.next() {
        None => String::new(),
        Some(first_char) => first_char.to_uppercase().collect::<String>() + chars_iterator.as_str(),
    };
    SkillType::deserialize(capitalized_string.into_deserializer())
}

#[derive(Deserialize, Default, Debug)]
struct InnerItem {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    percentage: f64,
    #[serde(default)]
    data: Option<SkillData>,
}

#[allow(clippy::unnecessary_wraps)]
fn extract_item_name<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let inner_item = InnerItem::deserialize(deserializer).unwrap_or_default();
    Ok(inner_item.name)
}

fn extract_refresh_data<'de, D>(deserializer: D) -> Result<Option<SkillData>, D::Error>
where
    D: Deserializer<'de>,
{
    let inner_item = InnerItem::deserialize(deserializer)?;
    Ok(inner_item.data)
}

fn extract_percentage<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let inner_item = InnerItem::deserialize(deserializer)?;
    Ok(inner_item.percentage)
}

fn deserialize_timedelta_from_milliseconds<'de, D>(deserializer: D) -> Result<TimeDelta, D::Error>
where
    D: Deserializer<'de>,
{
    let milliseconds_value = i64::deserialize(deserializer)?;
    Ok(TimeDelta::milliseconds(milliseconds_value))
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
    pub refresh_data: Option<SkillData>,
}
