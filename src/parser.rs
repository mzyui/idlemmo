use html_escape::decode_html_entities;
use once_cell_regex::regex;
use regex::Regex;

use crate::{error::{AppError, Result}, models::SkillType};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Parser {
    CsrfToken,
    ApiToken,
    CharacterId,
    TwoFactorUrl,
    SkillData(SkillType),
    CharacterInformationApiEndpoint,
    CharactersAllApiEndpoint,
    LocationsAllApiEndpoint,
    LocationsTravelApiEndpoint,
    QuickViewLocationApiEndpoint,
    ActionActiveApiEndpoint,

    SkillsStartApiEndpoint,
}

impl Parser {
    pub fn to_regex(&self) -> &'static Regex {
        match self.clone() {
            Self::CsrfToken => regex!(r#"name="csrf-token"\s*content="([^"]+)"#),
            Self::ApiToken => regex!(r#"name="api-token"\s*content="([^"]+)"#),
            Self::CharacterId => regex!(r#"name="character-id"\s*content="([^"]+)"#),
            Self::TwoFactorUrl => regex!(r#"action="(https://web.idle-mmo.com/2fa/[^"]+)"#),
            Self::SkillData(ref skill_type) => skill_type.to_regex(),
            Self::CharacterInformationApiEndpoint => {
                regex!(r#"(https?.+?/character\\?/information[^'"]+)"#)
            }
            Self::CharactersAllApiEndpoint => regex!(r#"(https?.+?/characters\\?/all[^'"]+)"#),
            Self::LocationsAllApiEndpoint => regex!(r#"(https?.*?/locations\\?/all[^'"]+)"#),
            Self::LocationsTravelApiEndpoint => {
                regex!(r#"travel.*?(https?.*?/locations\\?/travel[^'"]+)"#)
            }
            Self::QuickViewLocationApiEndpoint => {
                regex!(r#"(https?.*?/quick-view\\?/location[^'"]+)"#)
            }
            Self::ActionActiveApiEndpoint => regex!(r#"(https?.*?/action\\?/active[^'"]+)"#),
            Self::SkillsStartApiEndpoint => regex!(r#"(https?.*?/skills\\?/start[^'"]+)"#),
        }
    }

    pub fn get_value(&self, haystack: &str) -> Result<String> {
        let regex = self.to_regex();
        let value = regex
            .captures(haystack)
            .and_then(|caps| caps.get(1))
            .map(|val| val.as_str())
            .ok_or_else(|| AppError::Parse(format!("Failed to find value for key: {self:?}")))?;
        let decoded = decode_html_entities(value).to_string();
        let unescaped = decoded.replace('\\', "").replace("u0026", "&");
        Ok(unescaped)
    }
}
