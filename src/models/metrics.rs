use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metrics {
    #[serde(deserialize_with = "de_u64_from_string")]
    pub items_gathered: u64,

    #[serde(deserialize_with = "de_duration_from_string")]
    pub time_spent: Duration,

    #[serde(deserialize_with = "de_u64_from_string")]
    pub total_experience: u64,
}

fn de_u64_from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let clean = s.replace(",", "");
    clean.parse::<u64>().map_err(serde::de::Error::custom)
}

fn de_duration_from_string<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let mut days = 0;
    let mut hours = 0;
    let mut minutes = 0;

    for part in s.split_whitespace() {
        if let Some(stripped) = part.strip_suffix('d') {
            days = stripped.parse::<i64>().unwrap_or(0);
        } else if let Some(stripped) = part.strip_suffix('h') {
            hours = stripped.parse::<i64>().unwrap_or(0);
        } else if let Some(stripped) = part.strip_suffix('m') {
            minutes = stripped.parse::<i64>().unwrap_or(0);
        }
    }

    Ok(Duration::days(days) + Duration::hours(hours) + Duration::minutes(minutes))
}
