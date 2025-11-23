use crate::error::{AppError, Result};
use crate::models::game_action::SkillType;
use crate::models::item::FilterBy;

#[derive(Debug, Clone)]
pub struct Config {
    pub supabase_url: String,
    pub supabase_key: String,
}

impl Config {
    #[tracing::instrument]
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let supabase_url = std::env::var("SUPABASE_URL")
            .map_err(|_| AppError::Config("SUPABASE_URL env var not set".to_string()))?;
        let supabase_key = std::env::var("SUPABASE_KEY")
            .map_err(|_| AppError::Config("SUPABASE_KEY env var not set".to_string()))?;

        Ok(Self {
            supabase_url,
            supabase_key,
        })
    }
}

#[derive(Debug, Default)]
pub struct SkillConfig {
    pub skill_type: SkillType,
    pub essence_crystal: u64,
    pub auto_purchase: bool,
    pub filter_by: FilterBy,
}
