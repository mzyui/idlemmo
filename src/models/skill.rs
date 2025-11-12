use crate::lazy_regex;
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
    pub fn to_regex(&self) -> &'static Regex {
        match *self {
            Self::Woodcutting => lazy_regex!(
                r#"<li[^>]*x-data="\{\s*level\s*:\s*(\d+)\s*\}"[\s\S]*?<a[^>]+href=['"][^'"]*/skills/view/woodcutting"#
            ),
            Self::Mining => lazy_regex!(
                r#"<li[^>]*x-data="\{\s*level\s*:\s*(\d+)\s*\}"[\s\S]*?<a[^>]+href=['"][^'"]*/skills/view/mining"#
            ),
            Self::Fishing => lazy_regex!(
                r#"<li[^>]*x-data="\{\s*level\s*:\s*(\d+)\s*\}"[\s\S]*?<a[^>]+href=['"][^'"]*/skills/view/fishing"#
            ),
            Self::Alchemy => lazy_regex!(
                r#"<li[^>]*x-data="\{\s*level\s*:\s*(\d+)\s*\}"[\s\S]*?<a[^>]+href=['"][^'"]*/skills/view/alchemy"#
            ),
            Self::Smelting => lazy_regex!(
                r#"<li[^>]*x-data="\{\s*level\s*:\s*(\d+)\s*\}"[\s\S]*?<a[^>]+href=['"][^'"]*/skills/view/melting"#
            ),
            Self::Cooking => lazy_regex!(
                r#"<li[^>]*x-data="\{\s*level\s*:\s*(\d+)\s*\}"[\s\S]*?<a[^>]+href=['"][^'"]*/skills/view/cooking"#
            ),
            Self::Forge => lazy_regex!(
                r#"<li[^>]*x-data="\{\s*level\s*:\s*(\d+)\s*\}"[\s\S]*?<a[^>]+href=['"][^'"]*/skills/view/forge"#
            ),
            Self::Meditation => lazy_regex!(
                r#"<li[^>]*x-data="\{\s*level\s*:\s*(\d+)\s*\}"[\s\S]*?<a[^>]+href=['"][^'"]*/skills/view/meditation"#
            ),
            _ => lazy_regex!(""),
        }
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
