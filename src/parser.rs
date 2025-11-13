use crate::lazy_regex;
use html_escape::decode_html_entities;
use once_cell::sync::OnceCell;
use regex::Regex;

use crate::{
    error::{AppError, Result},
    models::SkillType,
};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Parser {
    CsrfToken,
    ApiToken,
    CharacterId,
    TwoFactorUrl,
    SkillData,
    CharacterInformationApiEndpoint,
    CharactersAllApiEndpoint,
    LocationsAllApiEndpoint,
    LocationsTravelApiEndpoint,
    QuickViewLocationApiEndpoint,
    ActionActiveApiEndpoint,
    SkillsStartApiEndpoint,
    SkillsDataApiEndpoint,
}

impl Parser {
    pub fn to_regex(&self) -> &'static Regex {
        match self {
            Self::CsrfToken => lazy_regex!(r#"name="csrf-token"\s*content="([^"]+)"#),
            Self::ApiToken => lazy_regex!(r#"name="api-token"\s*content="([^"]+)""#),
            Self::CharacterId => lazy_regex!(r#"name="character-id"\s*content="([^"]+)"#),
            Self::TwoFactorUrl => lazy_regex!(r#"action="(https://web.idle-mmo.com/2fa/[^"]+)"#),
            Self::SkillData => lazy_regex!(r#"(?s)level: (\d+).+?skills/view/([^'\"]+)"#),
            Self::CharacterInformationApiEndpoint => {
                lazy_regex!(r#"(https?.+?/character\\?/information[^'"]+)""#)
            }
            Self::CharactersAllApiEndpoint => {
                lazy_regex!(r#"(https?.+?/characters\\?/all[^'"]+)""#)
            }
            Self::LocationsAllApiEndpoint => lazy_regex!(r#"(https?.*?/locations\\?/all[^'"]+)""#),
            Self::LocationsTravelApiEndpoint => {
                lazy_regex!(r#"travel.*?(https?.*?/locations\\?/travel[^'"]+)""#)
            }
            Self::QuickViewLocationApiEndpoint => {
                lazy_regex!(r#"(https?.*?/quick-view\\?/location[^'"]+)"#)
            }
            Self::ActionActiveApiEndpoint => lazy_regex!(r#"(https?.*?/action\\?/active[^'"]+)""#),
            Self::SkillsStartApiEndpoint => lazy_regex!(r#"(https?.*?/skills\\?/start[^'"]+)""#),
            Self::SkillsDataApiEndpoint => lazy_regex!(r#"(https?.*?/skills\\?/data[^'"]+)""#),
        }
    }

    pub fn get_value(&self, input_text: &str) -> Result<String> {
        let parser_regex = self.to_regex();
        let captured_value = parser_regex
            .captures(input_text)
            .and_then(|caps| caps.get(1))
            .map(|val| val.as_str())
            .ok_or_else(|| AppError::Parse(format!("Failed to find value for key: {self:?}")))?;
        let decoded_html = decode_html_entities(captured_value).to_string();
        let unescaped_string = decoded_html.replace('\\', "").replace("u0026", "&");
        Ok(unescaped_string)
    }
}
