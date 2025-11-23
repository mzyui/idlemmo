use chrono::{DateTime, Duration, FixedOffset};
use serde::{
    Deserialize, Deserializer,
    de::{Error as DeError, IntoDeserializer},
};
use serde_json::Value;
use std::collections::HashMap;

use crate::models::{game_action::SkillType, item::Requirement};

pub(crate) fn deserialize_optional_duration<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_value = Option::<Value>::deserialize(deserializer).map_err(DeError::custom)?;
    match opt_value {
        Some(value) => {
            let duration =
                deserialize_duration(value.into_deserializer()).map_err(DeError::custom)?;
            Ok(Some(duration))
        }
        None => Ok(None),
    }
}

pub(crate) fn deserialize_duration<'de, D>(
    deserializer: D,
) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer).map_err(|e| DeError::custom(e.to_string()))?;

    match v {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                return if i.abs() >= 10_000 {
                    Ok(Duration::milliseconds(i))
                } else {
                    Ok(Duration::seconds(i))
                };
            } else if let Some(f) = n.as_f64() {
                return if f.abs() >= 10_000.0 {
                    Ok(Duration::milliseconds(f.round() as i64))
                } else {
                    let ms = (f * 1000.0).round() as i64;
                    Ok(Duration::milliseconds(ms))
                };
            }
            Err(DeError::custom("unsupported numeric value for duration"))
        }
        Value::String(s) => {
            if let Some(dur) = parse_hms(&s) {
                return Ok(dur);
            }
            if let Some(dur) = parse_dhms(&s) {
                return Ok(dur);
            }
            if let Ok(i) = s.parse::<i64>() {
                return Ok(Duration::seconds(i));
            }
            if let Ok(f) = s.parse::<f64>() {
                return Ok(Duration::milliseconds((f * 1000.0) as i64));
            }
            Err(DeError::custom(format!(
                "could not parse string '{}' as duration",
                s
            )))
        }
        other => Err(DeError::custom(format!(
            "unsupported type for duration: {:?}",
            other
        ))),
    }
}

fn parse_dhms(input: &str) -> Option<Duration> {
    let mut s = input.trim();

    // dukung minus di depan, misalnya "-1d 2h"
    let mut negative = false;
    if let Some(rest) = s.strip_prefix('-') {
        negative = true;
        s = rest.trim();
    }

    let mut total = Duration::zero();

    for token in s.split_whitespace() {
        if token.is_empty() {
            continue;
        }

        // minimal panjang 2: "1d", "5h", dst
        if token.len() < 2 {
            return None;
        }

        let (num_str, unit_str) = token.split_at(token.len() - 1);
        let value: i64 = num_str.parse().ok()?;

        let part = match unit_str {
            "d" => Duration::days(value),
            "h" => Duration::hours(value),
            "m" => Duration::minutes(value),
            "s" => Duration::seconds(value),
            _ => return None, // ada unit yang gak dikenal
        };

        total = total + part;
    }

    if negative {
        total = -total;
    }

    Some(total)
}

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

fn u64_to_i64_clamped(v: u64) -> i64 {
    if v > i64::MAX as u64 {
        i64::MAX
    } else {
        v as i64
    }
}

pub(crate) fn deserialize_skill_type<'de, D>(
    deserializer: D,
) -> std::result::Result<SkillType, D::Error>
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

pub(crate) fn deserialize_u64_str<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let clean = s.replace(',', "");
    clean.parse::<u64>().map_err(DeError::custom)
}

pub(crate) fn deserialize_rfc3339<'de, D>(
    deserializer: D,
) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    DateTime::parse_from_rfc3339(&s)
        .map_err(|e| DeError::custom(format!("invalid datetime: {}", e)))
}

pub(crate) fn deserialize_name_from_obj<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    if let Some(s) = value
        .as_object()
        .and_then(|obj| obj.get("name").and_then(|v| v.as_str()))
    {
        return Ok(s.to_string());
    }
    Err(DeError::custom("Cannot extract name"))
}

pub(crate) fn deserialize_requirements<'de, D>(des: D) -> Result<Vec<Requirement>, D::Error>
where
    D: Deserializer<'de>,
{
    let maybe_map: Option<HashMap<String, Requirement>> = Option::deserialize(des)?;
    if let Some(map) = maybe_map {
        let mut pairs: Vec<(Option<u64>, Requirement)> = map
            .into_iter()
            .map(|(k, v)| match k.parse::<u64>() {
                Ok(n) => (Some(n), v),
                Err(_) => (None, v),
            })
            .collect();
        pairs.sort_by(|a, b| match (a.0, b.0) {
            (Some(x), Some(y)) => x.cmp(&y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        let values = pairs.into_iter().map(|(_, v)| v).collect();
        Ok(values)
    } else {
        Ok(Vec::new())
    }
}
