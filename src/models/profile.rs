use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, fmt};

use crate::models::action::SkillType;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Character {
    pub id: u64,
    pub name: String,
    pub change_url: String,
    pub class_name: String,
    pub level: u64,
    pub is_current: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Profile {
    pub id: u64,
    pub name: String,
    pub combat_level: u64,
    #[serde(default)]
    pub skill_level: BTreeMap<SkillType, u64>,
    pub gold: u64,
    pub health: u64,
    pub health_percentage: u64,
    pub location_id: u64,
    pub max_health: u64,
    pub party: Option<Party>,
    pub shards: u64,
    pub tokens: u64,
    pub total_level: u64,
    pub unread_mail_count: u64,
    pub unread_notification_count: u64,
}

impl Profile {
    pub fn update_skill(
        &mut self,
        skill_type: SkillType,
        value: &str,
    ) -> std::result::Result<(), crate::error::AppError> {
        let value_int = value.parse::<u64>()?;
        self.skill_level.insert(skill_type, value_int);
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Party {
    pub id: u64,
    pub is_in_active_league: bool,
    pub is_member: bool,
    pub leader: Leader,
    pub members: Vec<Member>,
    pub members_count: u64,
    pub name: String,
    pub pending_invites: Vec<PendingInvite>,
    pub permissions: Permissions,
    #[serde(deserialize_with = "de_datetime_from_rfc3339")]
    pub created_at: DateTime<FixedOffset>,
    #[serde(deserialize_with = "de_datetime_from_rfc3339")]
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingInvite {
    id: u64,
    #[serde(rename = "character", deserialize_with = "extract_name")]
    name: String,
    #[serde(deserialize_with = "de_datetime_from_rfc3339")]
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Leader {
    pub id: u64,
    pub location_id: u64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    pub id: u64,
    #[serde(deserialize_with = "de_datetime_from_rfc3339")]
    pub joined_at: DateTime<FixedOffset>,
    pub location_id: u64,
    pub name: String,
    pub rank: Rank,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Role {
    Leader,
    Member,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rank {
    pub name: String,
    pub position: u64,
    #[serde(rename = "value")]
    pub role: Role,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Permissions {
    pub close: bool,
    pub invite_create: bool,
    pub invite_delete: bool,
    pub leave: bool,
}

fn de_datetime_from_rfc3339<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error as DeError;
    let s = String::deserialize(deserializer)?;
    DateTime::parse_from_rfc3339(&s)
        .map_err(|e| DeError::custom(format!("invalid datetime: {}", e)))
}

fn extract_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error as DeError;
    let value = Value::deserialize(deserializer)?;
    if let Some(s) = value
        .as_object()
        .and_then(|obj| obj.get("name").and_then(|v| v.as_str()))
    {
        return Ok(s.to_string());
    }
    Err(D::Error::custom("Cannot extract name"))
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Profile(id={}, name={}, level={}, gold={})",
            self.id, self.name, self.combat_level, self.gold
        )
    }
}
