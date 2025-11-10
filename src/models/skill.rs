use enum_iterator::Sequence;
use once_cell_regex::regex;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

#[derive(
    Debug, Clone, Deserialize_enum_str, Serialize_enum_str, Sequence, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum SkillType {
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
    pub fn to_regex(&self) -> &'static Regex {
        match *self {
            Self::Woodcutting => regex!(r"(?s)Woodcutting.*?Lv\.\s*(\d+)"),
            Self::Mining => regex!(r"(?s)Mining.*?Lv\.\s*(\d+)"),
            Self::Fishing => regex!(r"(?s)Fishing.*?Lv\.\s*(\d+)"),
            Self::Alchemy => regex!(r"(?s)Alchemy.*?Lv\.\s*(\d+)"),
            Self::Smelting => regex!(r"(?s)Smelting.*?Lv\.\s*(\d+)"),
            Self::Cooking => regex!(r"(?s)Cooking.*?Lv\.\s*(\d+)"),
            Self::Forge => regex!(r"(?s)Forge.*?Lv\.\s*(\d+)"),
            Self::Meditation => regex!(r"(?s)Meditation.*?Lv\.\s*(\d+)"),
            Self::Travelling => regex!(r"(?s)Travelling.*?Lv\.\s*(\d+)"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Skill {
    pub id: i64,
    pub name: String,
    #[serde(rename = "skill")]
    pub r#type: SkillType,
    pub level_required: i64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SkillData {
    pub skill_item_id: u64,
    pub quantity: u64,
}
