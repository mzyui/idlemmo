use async_trait::async_trait;

use crate::error::Result;
use crate::models::{Action, SkillConfig};

#[allow(dead_code)]
#[async_trait]
pub trait ActionSkillApi {
    async fn start_skill(&mut self, config: SkillConfig) -> Result<()>;
    async fn get_active_action(&self) -> Result<Option<Action>>;
}
