// meta_data.rs
use chrono::Duration;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::utils::duration_from_any_option;

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
