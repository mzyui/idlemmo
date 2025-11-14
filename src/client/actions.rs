use async_trait::async_trait;
use serde_json::{Value, json};
use tracing::{debug, info};

use crate::{
    client::{IdleMMOClient, LocationApi},
    error::{AppError, Result},
    models::{Action, FilterBy, SkillConfig, SkillItem, SkillType, location::Location},
    parser::Parser,
    utils::{API_VERSION, find_best_skill, generate_obfuscated_data},
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
        let mut available_locations = self.get_locations(true).await?;

        let (selected_location, selected_skill_item) =
            find_best_skill(&available_locations, &config)
                .ok_or_else(|| AppError::Application("No suitable skill found".to_string()))?;

        if self.cache.character_info.location_id != selected_location.id {
            self.move_location(
                crate::models::location::TravelMode::Teleport,
                selected_location.clone(),
            )
            .await?;
        }

        dbg!(&selected_skill_item);

        let http_response = self
            .client
            .get(format!("{}skills/view/{}", self.base_url, config.skill_type).to_lowercase())
            .send()
            .await?;
        let response_html = http_response.text().await?;
        let start_skill_api_url = Parser::SkillsStartApiEndpoint.get_value(&response_html)?;

        let request_payload = json!({
            "skill_item_id": selected_skill_item.id,
            "quantity": 1,
            "essence_crystal": config.essence_crystal,
            "auto_purchase": config.auto_purchase,
            "ts2mic5ytx": generate_obfuscated_data(None),
            "qty6bx4peh": generate_obfuscated_data(None),
            "v": API_VERSION
        });

        dbg!(&request_payload);
        let http_response = self
            .client
            .post(start_skill_api_url)
            .json(&request_payload)
            .send()
            .await?;
        dbg!(&http_response.text().await?[..100]);
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_active_action(&self) -> Result<Option<Action>> {
        let active_action_api_url = Parser::ActionActiveApiEndpoint.get_value(&self.cache.html)?;
        debug!(url = %active_action_api_url, "Calling API: Get Active Action");
        let http_api_response = self
            .client
            .post(&active_action_api_url)
            .json(&json!({
                "character_id": self.cache.character_info.id,
                "v": API_VERSION
            }))
            .send()
            .await?;

        let json_response_data = http_api_response.json::<Value>().await?;
        if json_response_data.is_array() {
            info!("No active action found for current character.");
            Ok(None)
        } else {
            let active_action = serde_json::from_value::<Action>(json_response_data)?;
            info!(skill_type = ?active_action.skill_type,
                item_name = ?active_action.item_name , "Active action found.");
            Ok(Some(active_action))
        }
    }
}
