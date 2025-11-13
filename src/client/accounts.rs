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
    async fn load_account(&mut self, account: Account) -> Result<()>;
    async fn get_account(&self) -> Result<Vec<Account>>;
    async fn add_account(&mut self, email: &str, password: &str) -> Result<()>;
    async fn post_login(&mut self, email: &str, password: &str) -> Result<()>;
}
#[async_trait]
impl AccountManagement for IdleMMOClient {
    #[tracing::instrument(skip(self, account_to_load))]
    async fn load_account(&mut self, account_to_load: Account) -> Result<()> {
        info!(user_id = account_to_load.id, user_email = %obfuscate_email(&account_to_load.email), "Loading account.");
        self.update_client(&account_to_load.api_token)?;

        info!("Attempting to load account with stored cookie...");
        let http_response = self
            .client
            .get(self.base_url.clone())
            .header(header::COOKIE, HeaderValue::from_str(&account_to_load.cookie_str)?)
            .send()
            .await?;

        let mut is_session_valid = false;
        if let Some(account_name) = http_response
            .url()
            .as_ref()
            .split('@')
            .filter(|v| !v.contains('/'))
            .next_back()
        {
            info!(%account_name, "Account loaded. Wellcome");
            is_session_valid =
                self.update_current_data().await.is_ok() && self.get_locations(false).await.is_ok();
        }

        if !is_session_valid {
            warn!("Session cookie appears invalid. Removing user from database.");
            self.db_client.remove_user(account_to_load.id).await?;
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
        let extracted_api_token = Parser::ApiToken.get_value(&self.cache.html)?;
        self.update_client(&extracted_api_token)?;
        let extracted_character_id = Parser::CharacterId.get_value(&self.cache.html)?;
        info!(token_prefix = %&extracted_api_token[..8], character_id = %extracted_character_id, "User data extracted");

        let session_cookies = self.jar.cookies(&self.base_url).unwrap();
        let session_cookie_string = session_cookies.to_str()?.to_string();
        let new_account_data = json!({
            "email": email,
            "api_token": extracted_api_token,
            "cookie_str": session_cookie_string
        });
        self.db_client.insert_user(new_account_data).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn post_login(&mut self, email: &str, password: &str) -> Result<()> {
        info!("Sending login credentials...");
        let login_params = json!({
            "remember": "true",
            "_token": self.cache.csrf_token,
            "email": email,
            "password": password
        });

        let http_response = self
            .client
            .post(format!("{}login", self.base_url))
            .form(&login_params)
            .send()
            .await?;
        let mut response_html = http_response.text().await?;
        let mut first_attempt = true;
        while let Ok(two_factor_auth_url) = Parser::TwoFactorUrl.get_value(&response_html) {
            if first_attempt {
                warn!("2FA Required: A code has been sent to your email. Please enter it below.");
            } else {
                warn!("Invalid 2FA code. Please try again.");
            }

            let two_factor_code = requestty::prompt([Question::int("2fa")
                .message("Two-Factor Code:")
                .build()])?["2fa"]
                .as_int()
                .unwrap();
            info!("Submitting 2FA code...");
            let http_response = self
                .client
                .post(&two_factor_auth_url)
                .form(&json!({
                    "_token": self.cache.csrf_token,
                    "code": two_factor_code
                }))
                .send()
                .await?;

            response_html = http_response.text().await?;
            debug!(html_len = response_html.len(), "2FA response HTML received");
            first_attempt = false;
        }

        info!("2FA check passed (or was not required).");

        self.update_current_data().await
    }
}
