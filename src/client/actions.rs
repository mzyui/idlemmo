use async_trait::async_trait;
use serde_json::{Value, json};
use tracing::{debug, info};

use crate::{
    client::{IdleMMOClient, LocationApi},
    error::{AppError, Result},
    models::{
        config::SkillConfig,
        game_action::{ActiveAction, SkillType},
        item::SkillData,
        world::TravelMode,
    },
    utils::{
        API_VERSION, obfuscation::generate_obfuscated_data, parser::Parser, skills::find_best_skill,
    },
};

#[allow(dead_code)]
#[async_trait]
pub trait ActionSkillApi {
    async fn start_skill(&mut self, config: SkillConfig) -> Result<()>;
    async fn get_skill_data(&self, skill_type: &SkillType) -> Result<SkillData>;
    async fn get_active_action(&self) -> Result<Option<ActiveAction>>;
}

#[async_trait]
impl ActionSkillApi for IdleMMOClient {
    #[tracing::instrument(skip_all)]
    async fn start_skill(&mut self, config: SkillConfig) -> Result<()> {
        let available_locations = self.get_locations(true).await?;

        let (selected_location, selected_skill_item) =
            find_best_skill(&available_locations, &config)
                .ok_or_else(|| AppError::Application("No suitable skill found".to_string()))?;

        if self.state.character_info.location_id != selected_location.id {
            self.move_location(TravelMode::Teleport, selected_location)
                .await?;
        }

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

        let http_response = self
            .client
            .post(start_skill_api_url)
            .json(&request_payload)
            .send()
            .await?;
        dbg!(&http_response.text().await?[..100]);
        Ok(())
    }

    #[tracing::instrument(skip(self, skill_type))]
    async fn get_skill_data(&self, skill_type: &SkillType) -> Result<SkillData> {
        info!(skill_type = ?skill_type, "Fetching skill data.");
        let http_response = self
            .client
            .get(format!("{}skills/view/{}", self.base_url, skill_type).to_lowercase())
            .send()
            .await?;
        let response_html = http_response.text().await?;
        let api_url = Parser::SkillsDataApiEndpoint.get_value(&response_html)?;
        debug!(url = %api_url, skill_type = ?skill_type, "Calling API: Get Skill Data");

        let http_response = self
            .client
            .post(api_url)
            .json(&json!({
                "filter": {}
            }))
            .send()
            .await?;
        let skill_data = http_response.json::<SkillData>().await?;
        info!(
            total_items = skill_data.items.len(),
            items_gathered = %skill_data.metrics.items_gathered,
            time_spent = %skill_data.metrics.time_spent,
            total_experience = %skill_data.metrics.total_experience,
            "Skill data retrieved.");
        Ok(skill_data)
    }

    #[tracing::instrument(skip(self))]
    async fn get_active_action(&self) -> Result<Option<ActiveAction>> {
        let active_action_api_url = Parser::ActionActiveApiEndpoint.get_value(&self.state.html)?;
        debug!(url = %active_action_api_url, "Calling API: Get Active Action");
        let http_api_response = self
            .client
            .post(&active_action_api_url)
            .json(&json!({
                "character_id": self.state.character_info.id,
                "v": API_VERSION
            }))
            .send()
            .await?;

        let json_response_data = http_api_response.json::<Value>().await?;
        if json_response_data.is_array() {
            info!("No active action found for current character.");
            Ok(None)
        } else {
            let active_action = serde_json::from_value::<ActiveAction>(json_response_data)?;
            info!(skill_type = ?active_action.kind, item = ?active_action.item , "Active action found.");
            Ok(Some(active_action))
        }
    }
}
