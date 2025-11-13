use async_trait::async_trait;

use crate::error::Result;
use crate::models::{Character, CharacterInfo};

#[allow(dead_code)]
#[async_trait]
pub trait CharacterApi {
    async fn get_character_information(&mut self) -> Result<CharacterInfo>;
    async fn get_all_characters(&self) -> Result<Vec<Character>>;
    async fn switch_character(&mut self, character_to_switch: Character) -> Result<()>;
}
