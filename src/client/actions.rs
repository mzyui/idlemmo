use async_trait::async_trait;
use serde_json::{Value, json};
use tracing::{debug, info};

use crate::{
    client::{IdleMMOClient, LocationApi},
    error::Result,
    models::{Action, FilterBy, ResponseData, Skill, SkillConfig, location::Location},
    parser::Parser,
    utils::generate_obfuscated_data,
};

#[allow(dead_code)]
#[async_trait]
pub trait ActionSkillApi {
    async fn start_skill(&mut self, config: SkillConfig) -> Result<()>;
    async fn get_active_action(&self) -> Result<Option<Action>>;
}

#[async_trait]
impl ActionSkillApi for IdleMMOClient {
    #[tracing::instrument(skip_all)]
    async fn start_skill(&mut self, config: SkillConfig) -> Result<()> {
        let mut locations = self.get_locations(true).await?;

        // let response = self
        //     .client
        //     .get(format!("{}skills/view/{}", self.base_url, config.skill_type).to_lowercase())
        //     .send()
        //     .await?;
        // let html = response.text().await?;
        // let api_url = Parser::SkillsStartApiEndpoint.get_value(&html)?;
        //
        // let payload = json!({
        //     "skill_item_id": item.id,
        //     "quantity": 1,
        //     "essence_crystal": config.essence_crystal,
        //     "auto_purchase": config.auto_purchase,
        //     "ts2mic5ytx": generate_obfuscated_data(None),
        //     "qty6bx4peh": generate_obfuscated_data(None),
        //     "v": "1.0.0.1"
        // });
        //

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_active_action(&self) -> Result<Option<Action>> {
        let api_url = Parser::ActionActiveApiEndpoint.get_value(&self.cache.html)?;
        debug!(url = %api_url, "Calling API: Get Active Action");
        let api_response = self
            .client
            .post(&api_url)
            .json(&json!({
                "character_id": self.cache.character_info.id,
                "v": "1.0.0.1"
            }))
            .send()
            .await?;

        let json = api_response.json::<Value>().await?;
        if json.is_array() {
            info!("No active action found for current character.");
            Ok(None)
        } else {
            let action = serde_json::from_value::<Action>(json)?;
            info!(skill_type = ?action.skill_type,
                item_name = ?action.item_name , "Active action found.");
            Ok(Some(action))
        }
    }
}
