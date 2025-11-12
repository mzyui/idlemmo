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
    async fn switch_character(&mut self, char: Character) -> Result<()>;
}

#[async_trait]
impl CharacterApi for IdleMMOClient {
    #[tracing::instrument(skip(self))]
    async fn get_character_information(&mut self) -> Result<CharacterInfo> {
        let api_url = Parser::CharacterInformationApiEndpoint.get_value(&self.cache.html)?;
        debug!(url = %api_url, "Calling API: Get Character Information");

        let api_response = self.client.post(&api_url).json(&json!({})).send().await?;
        let mut char_info = api_response.json::<CharacterInfo>().await?;

        std::fs::write("@val.html", &self.cache.html)?;
        for skill_type in enum_iterator::all::<SkillType>() {
            if let Ok(value) = Parser::SkillData(skill_type.clone()).get_value(&self.cache.html) {
                char_info.update_skill(skill_type, &value)?;
            }
        }

        info!(
            name = %char_info.name,
            id = char_info.id,
            "Character information fetched."
        );
        Ok(char_info)
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_characters(&self) -> Result<Vec<Character>> {
        let api_url = Parser::CharactersAllApiEndpoint.get_value(&self.cache.html)?;
        debug!(url = %api_url, "Calling API: Get All Characters");

        let api_response = self.client.post(&api_url).json(&json!({})).send().await?;
        let raw_value = api_response.json::<Value>().await?;

        let mut chars = vec![];
        if let Some(characters) = raw_value.get("characters").and_then(|v| v.as_array()) {
            for char in characters.clone() {
                chars.push(serde_json::from_value::<Character>(char)?);
            }
        }
        info!(count = chars.len(), "All characters fetched.");

        Ok(chars)
    }

    #[tracing::instrument(skip(self, char))]
    async fn switch_character(&mut self, char: Character) -> Result<()> {
        if char.is_current {
            info!("Target character is already currently active. Skipping switch.");
            return Ok(());
        }

        self.client
            .post(format!(
                "{}user/character/switch/{}",
                self.base_url, char.id
            ))
            .form(&json!({
                "_token": self.cache.csrf_token,
                "return_to_current_page": false
            }))
            .send()
            .await?;

        info!(
            name = %char.name,
            id = char.id,
            "Character switched.");
        self.update_current_data().await?;
        Ok(())
    }
}
