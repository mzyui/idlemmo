use std::sync::Arc;

use async_trait::async_trait;
use fake::{Fake, faker::internet::en::UserAgent};
use reqwest::{
    Client, ClientBuilder, Url,
    cookie::{CookieStore, Jar},
    header::{self, HeaderMap, HeaderValue},
};
use serde_json::{Value, json};
use tracing::{debug, info, warn};

use crate::{
    config::Config,
    db::DbClient,
    error::Result,
    models::{
        Action, CachedData, Character, CharacterInfo, SkillType, User,
        location::{Location, LocationResponseData, TravelMode},
    },
    parser::Parser,
    traits::{AccountManagement, ActionSkillApi, CharacterApi, LocationApi},
    utils::{generate_obfuscated_data, obfuscate_email},
};

pub struct IdleMMOClient {
    pub(crate) jar: Arc<Jar>,
    pub(crate) client: Client,
    pub(crate) base_url: Url,
    pub(crate) cache: CachedData,
    pub(crate) db_client: DbClient,

    user_agent: String,
}

impl IdleMMOClient {
    #[tracing::instrument(skip_all)]
    pub fn new() -> Result<Self> {
        info!("Initializing IdleMMO client...");
        let user_agent = UserAgent().fake::<String>();
        let jar = Arc::new(Jar::default());
        let client = ClientBuilder::new()
            .cookie_provider(Arc::clone(&jar))
            .user_agent(user_agent.clone())
            .build()?;
        let config = Config::from_env()?;
        let db_client = DbClient::new(&config)?;

        info!("IdleMMO client initialized.");
        Ok(Self {
            jar,
            client,
            db_client,
            base_url: Url::parse("https://web.idle-mmo.com")?,
            cache: CachedData::default(),
            user_agent,
        })
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn update_current_data(&mut self) -> Result<()> {
        let response = self.client.get(self.base_url.as_ref()).send().await?;
        let html = response.text().await?;
        let csrf_token = Parser::CsrfToken.get_value(&html)?;

        self.cache.html = html;
        self.cache.csrf_token = csrf_token;
        match self.get_character_information().await {
            Ok(char_info) => self.cache.character_info = char_info,
            Err(e) => warn!(error = %e, "Failed to get character information during data update."),
        }

        info!(
            token_prefix = %&self.cache.csrf_token[..8],
            "Current data updated."
        );

        Ok(())
    }

    // Tetap skip api_token untuk keamanan
    #[tracing::instrument(skip(self, api_token))]
    fn update_client(&mut self, api_token: &str) -> Result<()> {
        let mut headers = HeaderMap::new();
        let auth_value = HeaderValue::from_str(&format!("Bearer {api_token}"))?;
        headers.insert(header::AUTHORIZATION, auth_value);

        headers.insert(
            header::REFERER,
            HeaderValue::from_str(self.base_url.as_ref())?,
        );

        self.client = ClientBuilder::new()
            .cookie_provider(Arc::clone(&self.jar))
            .default_headers(headers)
            .user_agent(self.user_agent.clone())
            .build()?;

        info!("Reqwest client successfully rebuilt with updated default headers.");
        Ok(())
    }
}

#[async_trait]
impl CharacterApi for IdleMMOClient {
    #[tracing::instrument(skip(self))]
    async fn get_character_information(&mut self) -> Result<CharacterInfo> {
        let api_url = Parser::CharacterInformationApiEndpoint.get_value(&self.cache.html)?;
        debug!(url = %api_url, "Calling API: Get Character Information");

        let api_response = self.client.post(&api_url).json(&json!({})).send().await?;
        let mut char_info = api_response.json::<CharacterInfo>().await?;

        for skill_type in enum_iterator::all::<SkillType>() {
            if let Ok(value) = Parser::SkillData(skill_type.clone()).get_value(&self.cache.html) {
                char_info.update_skill(skill_type, &value)?;
            }
        }

        info!(
            name = %char_info.name,
            id = char_info.id,
            "Character information fetched."
        );
        Ok(char_info)
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_characters(&self) -> Result<Vec<Character>> {
        let api_url = Parser::CharactersAllApiEndpoint.get_value(&self.cache.html)?;
        debug!(url = %api_url, "Calling API: Get All Characters");

        let api_response = self.client.post(&api_url).json(&json!({})).send().await?;
        let raw_value = api_response.json::<Value>().await?;

        let mut chars = vec![];
        if let Some(characters) = raw_value.get("characters").and_then(|v| v.as_array()) {
            for char in characters.clone() {
                chars.push(serde_json::from_value::<Character>(char)?);
            }
        }
        info!(count = chars.len(), "All characters fetched.");

        Ok(chars)
    }

    #[tracing::instrument(skip(self, char))]
    async fn switch_character(&mut self, char: Character) -> Result<()> {
        if char.is_current {
            info!("Target character is already currently active. Skipping switch.");
            return Ok(());
        }

        self.client
            .post(format!(
                "{}user/character/switch/{}",
                self.base_url, char.id
            ))
            .form(&json!({
                "_token": self.cache.csrf_token,
                "return_to_current_page": false
            }))
            .send()
            .await?;

        info!(
            name = %char.name,
            id = char.id,
            "Character switched.");
        self.update_current_data().await?;
        Ok(())
    }
}

#[async_trait]
impl LocationApi for IdleMMOClient {
    #[tracing::instrument(skip(self))]
    async fn get_locations(&mut self) -> Result<Vec<Location>> {
        let all_locs_url = Parser::LocationsAllApiEndpoint.get_value(&self.cache.html)?;
        debug!(url = %all_locs_url, "Calling API: Get All Locations");

        let response = self.client.post(all_locs_url).send().await?;
        let json: Value = response.json().await?;

        let location_ids: Vec<i64> = json
            .as_object()
            .map(|obj| {
                obj.values()
                    .filter_map(|v| v.get("id").and_then(serde_json::Value::as_i64))
                    .collect()
            })
            .unwrap_or_default();

        let total_locs = location_ids.len();
        debug!(count = total_locs, "Found initial locations.");
        let mut locations = Vec::with_capacity(total_locs);
        if total_locs == 0 {
            warn!("No locations found in the initial fetch.");
        } else {
            let quick_view_url =
                Parser::QuickViewLocationApiEndpoint.get_value(&self.cache.html)?;
            for location_id in location_ids {
                let response = self
                    .client
                    .post(&quick_view_url)
                    .json(&json!({ "location_id": location_id }))
                    .send()
                    .await?;

                let mut location = response.json::<Location>().await?;
                location
                    .enemies
                    .retain(|enemy| enemy.level <= self.cache.character_info.combat_level);
                location.skill_items.retain(|skill| {
                    skill.level_required
                        <= *self
                            .cache
                            .character_info
                            .skill_level
                            .entry(skill.r#type.clone())
                            .or_default()
                });
                if !location.enemies.is_empty() || !location.skill_items.is_empty() {
                    locations.push(location);
                }
            }
            self.cache.locations.clone_from(&locations);
        }
        debug!(count = locations.len(), "Finished filtering locations.");
        Ok(locations)
    }

    #[tracing::instrument(skip(self, location, travel_mode))]
    async fn move_location(&mut self, travel_mode: TravelMode, location: Location) -> Result<()> {
        info!(location = %location.name, ?travel_mode, "Attempting to move to new location.");
        let message = match travel_mode {
            TravelMode::Teleport => {
                let current_gold = self.cache.character_info.gold;
                if current_gold < location.teleport_cost {
                    warn!(
                        current_gold,
                        cost = location.teleport_cost,
                        "Teleport failed: Not enough gold."
                    );
                    return Ok(());
                }

                self.client
                    .post(format!(
                        "{}locations/teleport/{}",
                        self.base_url, location.key
                    ))
                    .form(&json!({
                        "_token": self.cache.csrf_token,
                    }))
                    .send()
                    .await?;

                self.update_current_data().await?;

                if current_gold > self.cache.character_info.gold {
                    "Teleport successful".to_string()
                } else {
                    "You already at location".to_string()
                }
            }
            TravelMode::Walk => {
                let api_url = Parser::LocationsTravelApiEndpoint.get_value(&self.cache.html)?;
                let response = self
                    .client
                    .post(api_url)
                    .json(&json!({
                        "location_id": location.id,
                        "ts2mic5ytx": generate_obfuscated_data(None),
                        "qty6bx4peh": generate_obfuscated_data(None),
                        "v": "1.0.0.1"
                    }))
                    .send()
                    .await?;
                let msg_data = response.json::<LocationResponseData>().await?;
                msg_data.message
            }
        };

        info!(
            location = %location.name,
            "{}", message
        );
        Ok(())
    }
}

#[async_trait]
impl ActionSkillApi for IdleMMOClient {
    #[tracing::instrument(skip_all)]
    async fn start_skill(&self) -> Result<()> {
        let _api_url = Parser::SkillsStartApiEndpoint.get_value(&self.cache.html)?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_active_action(&self) -> Result<Option<Action>> {
        let api_url = Parser::ActionActiveApiEndpoint.get_value(&self.cache.html)?;
        debug!(url = %api_url, "Calling API: Get Active Action");
        let api_response = self
            .client
            .post(&api_url)
            .json(&json!({
                "character_id": self.cache.character_info.id,
                "v": "1.0.0.1"
            }))
            .send()
            .await?;

        let json = api_response.json::<Value>().await?;
        if json.is_array() {
            info!("No active action found for current character.");
            Ok(None)
        } else {
            let action = serde_json::from_value::<Action>(json)?;
            info!(skill_type = ?action.skill_type,
                item_name = ?action.item_name , "Active action found.");
            Ok(Some(action))
        }
    }
}

#[async_trait]
impl AccountManagement for IdleMMOClient {
    #[tracing::instrument(skip(self, user))]
    async fn load_account(&mut self, user: User) -> Result<()> {
        info!(user_id = user.id, user_email = %obfuscate_email(&user.email), "Loading account.");
        self.update_client(&user.api_token)?;

        info!("Attempting to load account with stored cookie...");
        let response = self
            .client
            .get(self.base_url.clone())
            .header(header::COOKIE, HeaderValue::from_str(&user.cookie_str)?)
            .send()
            .await?;

        if let Some(name) = response
            .url()
            .as_ref()
            .split('@')
            .filter(|v| !v.contains('/'))
            .next_back()
        {
            info!(%name, "Account loaded. Wellcome");
            self.update_current_data().await?;
            self.get_locations().await?;
        } else {
            warn!("Session cookie appears invalid. Removing user from database.");
            self.db_client.remove_user(user.id).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_users(&self) -> Result<Vec<User>> {
        self.db_client.get_users().await
    }

    #[tracing::instrument(skip(self, email, password))]
    async fn add_account(&mut self, email: &str, password: &str) -> Result<()> {
        self.update_current_data().await?;
        self.post_login(email, password).await?;

        info!("Extracting API token and user metadata...");
        let api_token = Parser::ApiToken.get_value(&self.cache.html)?;
        self.update_client(&api_token)?;
        let character_id = Parser::CharacterId.get_value(&self.cache.html)?;
        info!(token_prefix = %&api_token[..8], character_id = %character_id, "User data extracted");

        let cookie_str = self
            .jar
            .cookies(&self.base_url)
            .unwrap()
            .to_str()?
            .to_string();
        let user = json!({
            "email": email,
            "api_token": api_token,
            "cookie_str": cookie_str
        });
        self.db_client.insert_user(user).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn post_login(&mut self, email: &str, password: &str) -> Result<()> {
        info!("Sending login credentials...");
        let params = json!({
            "remember": "true",
            "_token": self.cache.csrf_token,
            "email": email,
            "password": password
        });

        let response = self
            .client
            .post(format!("{}login", self.base_url))
            .form(&params)
            .send()
            .await?;
        let mut html = response.text().await?;
        let mut first_attempt = true;
        while let Ok(twofactor_url) = Parser::TwoFactorUrl.get_value(&html) {
            if first_attempt {
                warn!("2FA Required: A code has been sent to your email. Please enter it below.");
            } else {
                warn!("Invalid 2FA code. Please try again.");
            }

            let code = inquire::Text::new("Two-Factor Code:")
                .with_validator(|v: &str| {
                    Ok(v.parse::<usize>()
                        .map(|_| inquire::validator::Validation::Valid)
                        .unwrap_or(inquire::validator::Validation::Invalid(
                            "Input must be a number.".into(),
                        )))
                })
                .prompt()?;

            info!("Submitting 2FA code...");
            let response = self
                .client
                .post(&twofactor_url)
                .form(&json!({
                    "_token": self.cache.csrf_token,
                    "code": code
                }))
                .send()
                .await?;

            html = response.text().await?;
            debug!(html_len = html.len(), "2FA response HTML received");
            first_attempt = false;
        }

        info!("2FA check passed (or was not required).");

        self.update_current_data().await
    }
}

