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
        let generated_user_agent = UserAgent().fake::<String>();
        let jar = Arc::new(Jar::default());
        let client_user_agent = ClientBuilder::new()
            .cookie_provider(Arc::clone(&jar))
            .user_agent(generated_user_agent.clone())
            .build()?;
        let app_config = Config::from_env()?;
        let db_client = DbClient::new(&app_config)?;

        info!("IdleMMO client initialized.");
        Ok(Self {
            jar,
            client: client_user_agent,
            db_client,
            base_url: Url::parse("https://web.idle-mmo.com")?,
            cache: CachedData::default(),
            user_agent: generated_user_agent,
        })
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn update_current_data(&mut self) -> Result<()> {
        let http_response = self.client.get(self.base_url.as_ref()).send().await?;
        let response_html = http_response.text().await?;
        let extracted_csrf_token = Parser::CsrfToken.get_value(&response_html)?;

        self.cache.html = response_html;
        self.cache.csrf_token = extracted_csrf_token;
        match self.get_character_information().await {
            Ok(character_information) => self.cache.character_info = character_information,
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
        let mut default_headers = HeaderMap::new();
        let authorization_header_value = HeaderValue::from_str(&format!("Bearer {api_token}"))?;
        default_headers.insert(header::AUTHORIZATION, authorization_header_value);

        default_headers.insert(
            header::REFERER,
            HeaderValue::from_str(self.base_url.as_ref())?,
        );

        self.client = ClientBuilder::new()
            .cookie_provider(Arc::clone(&self.jar))
            .default_headers(default_headers)
            .user_agent(self.user_agent.clone())
            .build()?;

        info!("Reqwest client successfully rebuilt with updated default headers.");
        Ok(())
    }
}