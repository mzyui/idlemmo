
use chrono::TimeDelta;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::debug;

use crate::models::SkillRequestData;

use crate::models::action_model::SkillType;

fn deserialize_capitalize<'de, D>(deserializer: D) -> std::result::Result<SkillType, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_skill_type_string = String::deserialize(deserializer)?;
    debug!(
        ?raw_skill_type_string,
        "Deserializing and capitalizing skill type."
    );

    let mut chars_iterator = raw_skill_type_string.chars();
    let capitalized_skill_type_string = match chars_iterator.next() {
        None => String::new(),
        Some(first_char) => first_char.to_uppercase().collect::<String>() + chars_iterator.as_str(),
    };
    debug!(
        ?capitalized_skill_type_string,
        "Capitalized skill type string."
    );
    SkillType::deserialize(capitalized_skill_type_string.into_deserializer())
}

#[derive(Deserialize, Default, Debug)]
struct InnerItem {
    #[serde(default)]
    name: String,
    percentage: f64,
}

#[allow(clippy::unnecessary_wraps)]
fn extract_item_name<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let action_item_data = InnerItem::deserialize(deserializer).unwrap_or_default();
    if action_item_data.name.is_empty() {
        return Ok(None);
    }
    Ok(Some(action_item_data.name))
}

fn extract_percentage<'de, D>(deserializer: D) -> std::result::Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let action_item_data = InnerItem::deserialize(deserializer)?;
    Ok(action_item_data.percentage)
}

fn deserialize_timedelta_from_milliseconds<'de, D>(deserializer: D) -> std::result::Result<TimeDelta, D::Error>
where
    D: Deserializer<'de>,
{
    debug!("Deserializing timedelta from milliseconds.");
    let milliseconds_value = u64::deserialize(deserializer)?;
    debug!(?milliseconds_value, "Deserialized milliseconds value.");
    Ok(TimeDelta::milliseconds(milliseconds_value as i64))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActiveAction {
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
}
