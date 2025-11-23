use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::utils::serde::{deserialize_duration, deserialize_u64_str};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metrics {
    #[serde(deserialize_with = "deserialize_u64_str")]
    pub items_gathered: u64,

    #[serde(deserialize_with = "deserialize_duration")]
    pub time_spent: Duration,

    #[serde(deserialize_with = "deserialize_u64_str")]
    pub total_experience: u64,
}
