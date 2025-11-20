use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct ResponseData {
    #[serde(default, alias = "result")]
    pub status: String,
    pub message: String,
}

pub mod action;
pub mod cached;
pub mod config;
pub mod location;
pub mod metadata;
pub mod metrics;
pub mod profile;
pub mod user;
