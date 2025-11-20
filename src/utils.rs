use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STD;
use chrono::Duration;
use once_cell::sync::OnceCell;
use regex::Regex;
use reqwest::Response;
use serde::{Deserialize, Deserializer, de::IntoDeserializer};
use serde_json::Value;

use crate::error::{AppError, Result};
use crate::models::{
    config::{FilterBy, SkillConfig},
    location::{Location, SkillItem},
};

pub const API_VERSION: &str = "1.0.0.1";

#[macro_export]
macro_rules! lazy_regex {
    ($regex_str:expr) => {{
        static REGEX: ::once_cell::sync::OnceCell<::regex::Regex> =
            ::once_cell::sync::OnceCell::new();
        REGEX.get_or_init(|| ::regex::Regex::new($regex_str).unwrap())
    }};
}

pub(crate) async fn debug_deserialize<T: serde::de::DeserializeOwned>(
    response: Response,
) -> Result<T> {
    let value = response.json::<Value>().await?;
    dbg!(&value);
    serde_json::from_value::<T>(value).map_err(AppError::SerdeJson)
}

pub fn generate_obfuscated_data(encryption_key_option: Option<&str>) -> String {
    let encryption_key = encryption_key_option.unwrap_or("fair-maiden");
    let random_text = fastrand::u64(400..600).to_string();
    let key_char_codes: Vec<u32> = encryption_key.chars().map(|c| c as u32).collect();
    let mut encrypted_bytes: Vec<u8> = Vec::with_capacity(random_text.len());

    for (i, text_char) in random_text.chars().enumerate() {
        let text_char_code = text_char as u32;
        let key_char_code = key_char_codes[i % key_char_codes.len()];
        let xor_result = (text_char_code ^ key_char_code) & 0xFF;
        encrypted_bytes.push(xor_result as u8);
    }

    BASE64_STD.encode(encrypted_bytes)
}

pub fn obfuscate_email(email: &str) -> String {
    let mut email_parts = email.split('@');
    let local_part = email_parts.next().unwrap_or("");
    let domain_part = email_parts.next().unwrap_or("");
    format!(
        "{}{}.@{}",
        &local_part[..local_part.len().min(3)],
        "*".repeat(local_part.len() - 3),
        domain_part
    )
}

fn find_best_skill_for_location<'a>(
    location: &'a Location,
    config: &SkillConfig,
) -> Option<&'a SkillItem> {
    let skills = &location.skill_items;

    let skills_of_type = skills
        .iter()
        .filter(|skill_item| skill_item.skill == config.skill_type);

    match config.filter_by {
        FilterBy::HighestLevelRequired => {
            skills_of_type.max_by_key(|skill_item| skill_item.level_required)
        }
        FilterBy::LowestLevelRequired => {
            skills_of_type.min_by_key(|skill_item| skill_item.level_required)
        }
        _ => unimplemented!(),
    }
}

pub fn find_best_skill<'a>(
    locations: &'a [Location],
    config: &SkillConfig,
) -> Option<(&'a Location, &'a SkillItem)> {
    let best_skills_per_location = locations.iter().filter_map(|location| {
        find_best_skill_for_location(location, config).map(|skill_item| (location, skill_item))
    });

    match config.filter_by {
        FilterBy::HighestLevelRequired => {
            best_skills_per_location.max_by_key(|(_, skill_item)| skill_item.level_required)
        }
        FilterBy::LowestLevelRequired => {
            best_skills_per_location.min_by_key(|(_, skill_item)| skill_item.level_required)
        }
        _ => unimplemented!(),
    }
}

pub(crate) fn duration_from_any_option<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error as DeError;
    let opt_value = Option::<Value>::deserialize(deserializer).map_err(D::Error::custom)?;
    match opt_value {
        Some(value) => {
            let duration =
                duration_from_any(value.into_deserializer()).map_err(D::Error::custom)?;
            Ok(Some(duration))
        }
        None => Ok(None),
    }
}
pub(crate) fn duration_from_any<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error as DeError;

    let v = Value::deserialize(deserializer).map_err(|e| DeError::custom(e.to_string()))?;

    match v {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Duration::seconds(i))
            } else if let Some(u) = n.as_u64() {
                let sec = u64_to_i64_clamped(u);
                Ok(Duration::seconds(sec))
            } else if let Some(f) = n.as_f64() {
                Ok(Duration::milliseconds((f * 1000.0) as i64))
            } else {
                Err(DeError::custom("unsupported number for duration"))
            }
        }
        Value::String(s) => {
            if let Some(dur) = parse_hms(&s) {
                Ok(dur)
            } else if let Ok(i) = s.parse::<i64>() {
                Ok(Duration::seconds(i))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(Duration::milliseconds((f * 1000.0) as i64))
            } else {
                Err(DeError::custom(format!(
                    "could not parse string '{}' as duration",
                    s
                )))
            }
        }
        other => Err(DeError::custom(format!(
            "unsupported type for duration: {:?}",
            other
        ))),
    }
}

/// Parse "HH:MM:SS" (or "H:MM:SS") into Duration. Returns None if parse fails.
fn parse_hms(s: &str) -> Option<Duration> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let h = parts[0].parse::<i64>().ok()?;
    let m = parts[1].parse::<i64>().ok()?;
    let sec = parts[2].parse::<i64>().ok()?;
    Some(Duration::hours(h) + Duration::minutes(m) + Duration::seconds(sec))
}

/// Helper to clamp u64 -> i64 safely (to avoid overflow into negative)
fn u64_to_i64_clamped(v: u64) -> i64 {
    if v > i64::MAX as u64 {
        i64::MAX
    } else {
        v as i64
    }
}
