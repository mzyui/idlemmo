pub mod action;
pub mod cached_data;
pub mod character;
pub mod item;
pub mod location;
pub mod metrics;
pub mod skill;
pub mod user;

pub use action::*;
pub use cached_data::*;
pub use character::*;
pub use metrics::*;
use serde::{Deserialize, Serialize};
pub use skill::*;
pub use user::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct ResponseData {
    #[serde(default, alias = "result")]
    pub status: String,
    pub message: String,
}

pub mod action_model;
pub mod metadata_model;
pub mod profile_model;
