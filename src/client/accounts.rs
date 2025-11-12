use async_trait::async_trait;
use requestty::Question;
use reqwest::{cookie::CookieStore, header::{self, HeaderValue}};
use serde_json::json;
use tracing::{debug, info, warn};

use crate::{
    client::{IdleMMOClient, LocationApi},
    error::Result,
    models::Account,
    parser::Parser,
    utils::obfuscate_email,
};

#[allow(dead_code)]
#[async_trait]
pub trait AccountManagement {
    async fn load_account(&mut self, user: Account) -> Result<()>;
    async fn get_account(&self) -> Result<Vec<Account>>;
    async fn add_account(&mut self, email: &str, password: &str) -> Result<()>;
    async fn post_login(&mut self, email: &str, password: &str) -> Result<()>;
}
#[async_trait]
impl AccountManagement for IdleMMOClient {
    #[tracing::instrument(skip(self, user))]
    async fn load_account(&mut self, user: Account) -> Result<()> {
        info!(user_id = user.id, user_email = %obfuscate_email(&user.email), "Loading account.");
        self.update_client(&user.api_token)?;

        info!("Attempting to load account with stored cookie...");
        let response = self
            .client
            .get(self.base_url.clone())
            .header(header::COOKIE, HeaderValue::from_str(&user.cookie_str)?)
            .send()
            .await?;

        let mut is_valid = false;
        if let Some(name) = response
            .url()
            .as_ref()
            .split('@')
            .filter(|v| !v.contains('/'))
            .next_back()
        {
            info!(%name, "Account loaded. Wellcome");
            is_valid =
                self.update_current_data().await.is_ok() && self.get_locations(false).await.is_ok();
        }

        if !is_valid {
            warn!("Session cookie appears invalid. Removing user from database.");
            self.db_client.remove_user(user.id).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_account(&self) -> Result<Vec<Account>> {
        self.db_client.list_users().await
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

        let cookies = self.jar.cookies(&self.base_url).unwrap();
        let cookie_str = cookies.to_str()?.to_string();
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

            let code = requestty::prompt([Question::int("2fa")
                .message("Two-Factor Code:")
                .build()])?["2fa"]
                .as_int()
                .unwrap();
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
