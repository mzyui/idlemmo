use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::skill::SkillType;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CharacterInfo {
    pub id: i64,
    pub name: String,
    pub combat_level: i64,
    #[serde(default)]
    pub skill_level: BTreeMap<SkillType, i64>,
    pub total_level: i64,
    pub gold: i64,
    pub tokens: i64,
    pub shards: i64,
    pub health: i64,
    pub max_health: i64,
}

impl CharacterInfo {
    pub fn update_skill(&mut self, skill_type: SkillType, value: &str) -> Result<()> {
        let value_int = value.parse::<i64>()?;
        *self.skill_level.entry(skill_type).or_default() = value_int;
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Character {
    pub id: i64,
    pub name: String,
    pub class_name: String,
    pub level: i64,
    pub is_current: bool,
}
