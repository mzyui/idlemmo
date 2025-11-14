use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::skill::SkillType;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CharacterInfo {
    pub id: u64,
    pub name: String,
    pub combat_level: u64,
    #[serde(default)]
    pub skill_level: BTreeMap<SkillType, u64>,
    pub total_level: u64,
    pub gold: u64,
    pub tokens: u64,
    pub shards: u64,
    pub health: u64,
    pub max_health: u64,
    pub location_id: u64,
}

impl CharacterInfo {
    pub fn update_skill(&mut self, skill_type: SkillType, value: &str) -> Result<()> {
        let value_int = value.parse::<u64>()?;
        *self.skill_level.entry(skill_type).or_default() = value_int;
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Character {
    pub id: u64,
    pub name: String,
    pub class_name: String,
    pub level: u64,
    pub is_current: bool,
}
