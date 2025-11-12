use std::fmt::Debug;

use super::{character::CharacterInfo, location::Location};

#[derive(Default)]
pub struct CachedData {
    pub locations: Vec<Location>,
    pub character_info: CharacterInfo,
    pub csrf_token: String,
    pub html: String,
}

impl Debug for CachedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedData")
            .field("locations", &self.locations)
            .field("character_info", &self.character_info)
            .field("csrf_token", &self.csrf_token)
            .finish()
    }
}
