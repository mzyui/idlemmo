// meta_data.rs
use chrono::Duration;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaData {
    pub exp_per_second: f64,
    pub exp_per_enemy: Option<f64>,
    pub total_bonus_enemies: Option<u64>,
    pub enemy_queue: Option<EnemyQueue>,
    pub bonus_enemy: Option<BonusEnemy>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnemyQueue {
    pub preview: Vec<EnemyPreview>,
    pub offset: Option<u64>,
    pub total: Option<u64>,
    pub chunk_size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnemyPreview {
    pub enemy_id: u64,
    pub name: String,
    pub image_url: String,
    pub position: u64,
    pub is_current: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BonusEnemy {
    pub active: bool,
    pub id: u64,
    #[serde(deserialize_with = "duration_from_any_option")]
    pub enemy_available_in: Option<Duration>,
    #[serde(deserialize_with = "duration_from_any_option")]
    pub enemy_shown_for: Option<Duration>,
    #[serde(deserialize_with = "duration_from_any_option")]
    pub wait_between_ui_cycles: Option<Duration>,
    #[serde(deserialize_with = "duration_from_any_option")]
    pub expires_in: Option<Duration>,
    pub exp_per_enemy: Option<u64>,
    pub length_per_power_hunt: Option<u64>,
    #[serde(deserialize_with = "duration_from_any_option")]
    pub cooldown_expires_in: Option<Duration>,
}

fn duration_from_any_option<'de, D>(
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

fn duration_from_any<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
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
            if let Ok(i) = s.parse::<i64>() {
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

/// Helper to clamp u64 -> i64 safely (to avoid overflow into negative)
fn u64_to_i64_clamped(v: u64) -> i64 {
    if v > i64::MAX as u64 {
        i64::MAX
    } else {
        v as i64
    }
}
