use crate::error::{AppError, Result};
use crate::models::metadata::MetaData;
use chrono::{DateTime, Duration, Utc};
use enum_iterator::Sequence;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use url::Url;

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
    pub metadata: MetaData,
    #[serde(default)]
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
            metadata: MetaData,
            #[serde(default, deserialize_with = "seconds_to_duration")]
            expires_in: Duration,
            #[serde(default, deserialize_with = "seconds_or_millis_to_duration")]
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
    #[serde(default, deserialize_with = "seconds_to_duration")]
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
            _ => Err(AppError::Parse(format!("Failed to parse skill type: {}", input_string))),
        }
    }
}

impl fmt::Display for SkillType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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


/// Deserialize tolerant: accepts lowercase, UPPERCASE, Capitalized, returns Other(...) if unknown
fn deserialize_skill_type<'de, D>(deserializer: D) -> std::result::Result<SkillType, D::Error>
where
    D: Deserializer<'de>,
{
    let s_opt: Option<String> = Option::deserialize(deserializer)?;
    match s_opt {
        Some(raw) => {
            let norm = raw.trim().to_lowercase();
            let kt = match norm.as_str() {
                "woodcutting" => SkillType::Woodcutting,
                "mining" => SkillType::Mining,
                "fishing" => SkillType::Fishing,
                "alchemy" => SkillType::Alchemy,
                "smelting" => SkillType::Smelting,
                "cooking" => SkillType::Cooking,
                "forge" => SkillType::Forge,
                "meditation" => SkillType::Meditation,
                other => SkillType::Other(other.to_string()),
            };
            Ok(kt)
        }
        None => Ok(SkillType::None),
    }
}

/// Convert Option<number> (seconds or integer) -> Option<chrono::Duration>
/// Accepts integer or float seconds; returns None if JSON null/missing.
fn seconds_to_duration<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    // accept numbers or null
    let opt: Option<serde_json::Number> = Option::deserialize(deserializer)?;
    if let Some(num) = opt {
        // try integer or float
        if let Some(i) = num.as_i64() {
            return Ok(Duration::seconds(i));
        } else if let Some(f) = num.as_f64() {
            // convert fractional seconds to milliseconds
            let ms = (f * 1000.0).round() as i64;
            return Ok(Duration::milliseconds(ms));
        }
    }
    Err(de::Error::custom("invalid numeric value for duration"))
}

fn seconds_or_millis_to_duration<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<serde_json::Number> = Option::deserialize(deserializer)?;
    if let Some(num) = opt {
        if let Some(i) = num.as_i64() {
            return if i.abs() >= 10_000 {
                Ok(Duration::milliseconds(i))
            } else {
                Ok(Duration::seconds(i))
            };
        } else if let Some(f) = num.as_f64() {
            return if f.abs() >= 10_000.0 {
                Ok(Duration::milliseconds(f.round() as i64))
            } else {
                let ms = (f * 1000.0).round() as i64;
                Ok(Duration::milliseconds(ms))
            };
        }
    }
    Err(de::Error::custom("invalid numeric value for duration"))
}
