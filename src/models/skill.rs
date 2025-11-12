use crate::{
    error::{AppError, Result},
    lazy_regex,
};
use enum_iterator::Sequence;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

#[derive(
    Default,
    Debug,
    Clone,
    Deserialize_enum_str,
    Serialize_enum_str,
    Sequence,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum SkillType {
    #[default]
    Woodcutting,
    Mining,
    Fishing,
    Alchemy,
    Smelting,
    Cooking,
    Forge,
    Meditation,
    Travelling,
}

impl SkillType {
    pub fn from_str(s: &str) -> Result<Self> {
        for skill_type in enum_iterator::all::<Self>() {
            if skill_type.to_string().to_lowercase() == s.to_lowercase() {
                return Ok(skill_type);
            }
        }
        Err(AppError::Parse("Failed to parse skill type.".into()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Skill {
    pub id: i64,
    pub name: String,
    #[serde(rename = "skill")]
    pub skill_type: SkillType,
    pub level_required: i64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SkillData {
    pub skill_item_id: u64,
    pub quantity: u64,
    #[serde(default)]
    pub essence_crystal: Option<u64>,
    #[serde(default)]
    pub auto_purchase: bool,
}

#[derive(Debug, Default)]
pub enum FilterBy {
    #[default]
    HighestLevelRequired,
    LowestLevelRequired,
    FastestTime,
    LongestTime,
    HighestExperience,
    LowestExperience,
    ItemName(String),
}

#[derive(Debug, Default)]
pub struct SkillConfig {
    pub skill_type: SkillType,
    pub essence_crystal: Option<u64>,
    pub auto_purchase: bool,
    pub filter_by: FilterBy,
}
