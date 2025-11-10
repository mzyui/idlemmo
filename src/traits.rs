use async_trait::async_trait;

use crate::models::{Action, Character, CharacterInfo, User};
use crate::models::location::{Location, TravelMode};
use crate::error::Result;

#[allow(dead_code)]
#[async_trait]
pub trait CharacterApi {
    async fn get_character_information(&mut self) -> Result<CharacterInfo>;
    async fn get_all_characters(&self) -> Result<Vec<Character>>;
    async fn switch_character(&mut self, char: Character) -> Result<()>;
}

#[allow(dead_code)]
#[async_trait]
pub trait LocationApi {
    async fn get_locations(&mut self) -> Result<Vec<Location>>;
    async fn move_location(&mut self, travel_mode: TravelMode, location: Location) -> Result<()>;
}

#[allow(dead_code)]
#[async_trait]
pub trait ActionSkillApi {
    async fn start_skill(&self) -> Result<()>;
    async fn get_active_action(&self) -> Result<Option<Action>>;
}

#[allow(dead_code)]
#[async_trait]
pub trait AccountManagement {
    async fn load_account(&mut self, user: User) -> Result<()>;
    async fn get_users(&self) -> Result<Vec<User>>;
    async fn add_account(&mut self, email: &str, password: &str) -> Result<()>;
    async fn post_login(&mut self, email: &str, password: &str) -> Result<()>;
}
