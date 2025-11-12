use std::sync::Arc;

use fake::{Fake, faker::internet::en::UserAgent};
use reqwest::{
    Client, ClientBuilder, Url,
    cookie::Jar,
    header::{self, HeaderMap, HeaderValue},
};
use tracing::{info, warn};

use crate::{
    config::Config,
    db::DbClient,
    error::Result,
    models::CachedData,
    parser::Parser,
};

pub mod accounts;
pub mod actions;
pub mod character;
pub mod location;

pub use accounts::AccountManagement;
pub use actions::ActionSkillApi;
pub use character::CharacterApi;
pub use location::LocationApi;

#[derive(Debug)]
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