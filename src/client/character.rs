use async_trait::async_trait;
use serde_json::{Value, json};
use tracing::{debug, info};

use crate::{
    client::IdleMMOClient,
    error::Result,
    models::{Character, CharacterInfo, SkillType},
    parser::Parser,
};

#[allow(dead_code)]
#[async_trait]
pub trait CharacterApi {
    async fn get_character_information(&mut self) -> Result<CharacterInfo>;
    async fn get_all_characters(&self) -> Result<Vec<Character>>;
    async fn switch_character(&mut self, character_to_switch: Character) -> Result<()>;
}

#[async_trait]
impl CharacterApi for IdleMMOClient {
    #[tracing::instrument(skip(self))]
    async fn get_character_information(&mut self) -> Result<CharacterInfo> {
        let character_info_api_url = Parser::CharacterInformationApiEndpoint.get_value(&self.cache.html)?;
        debug!(url = %character_info_api_url, "Calling API: Get Character Information");

        let http_api_response = self.client.post(&character_info_api_url).json(&json!({})).send().await?;
        let mut character_details = http_api_response.json::<CharacterInfo>().await?;

        std::fs::write("@val.html", &self.cache.html)?;
        let skill_data_regex = Parser::SkillData.to_regex();
        for capture in skill_data_regex.captures_iter(&self.cache.html) {
            let (_, [skill_level_str, skill_type_str]) = capture.extract();
            let parsed_skill_type = SkillType::from_str(skill_type_str)?;
            character_details.update_skill(parsed_skill_type, skill_level_str)?;
        }

        info!(
            name = %character_details.name,
            id = character_details.id,
            "Character information fetched."
        );
        Ok(character_details)
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_characters(&self) -> Result<Vec<Character>> {
        let all_characters_api_url = Parser::CharactersAllApiEndpoint.get_value(&self.cache.html)?;
        debug!(url = %all_characters_api_url, "Calling API: Get All Characters");

        let http_api_response = self.client.post(&all_characters_api_url).json(&json!({})).send().await?;
        let raw_json_response = http_api_response.json::<Value>().await?;

        let mut character_list = vec![];
        if let Some(json_characters_array) = raw_json_response.get("characters").and_then(|v| v.as_array()) {
            for json_character_value in json_characters_array.clone() {
                character_list.push(serde_json::from_value::<Character>(json_character_value)?);
            }
        }
        info!(count = character_list.len(), "All characters fetched.");

        Ok(character_list)
    }

    #[tracing::instrument(skip(self, character_to_switch))]
    async fn switch_character(&mut self, character_to_switch: Character) -> Result<()> {
        if character_to_switch.is_current {
            info!("Target character is already currently active. Skipping switch.");
            return Ok(());
        }

        self.client
            .post(format!(
                "{}user/character/switch/{}",
                self.base_url, character_to_switch.id
            ))
            .form(&json!({
                "_token": self.cache.csrf_token,
                "return_to_current_page": false
            }))
            .send()
            .await?;

        info!(
            name = %character_to_switch.name,
            id = character_to_switch.id,
            "Character switched.");
        self.update_current_data().await?;
        Ok(())
    }
}
