use serde_json::Value;
use supabase_rs::SupabaseClient;
use tracing::{debug, info, warn};

use crate::{
    config::Config,
    error::{AppError, Result},
    models::Account,
};

#[derive(Clone, Debug)]
pub struct DbClient {
    client: SupabaseClient,
}

impl DbClient {
    #[tracing::instrument(skip(config))]
    pub fn new(config: &Config) -> Result<Self> {
        let client = SupabaseClient::new(config.supabase_url.clone(), config.supabase_key.clone())
            .map_err(|e| AppError::SupabaseBuilder(e.to_string()))?;
        info!("Supabase client initialized.");
        Ok(Self { client })
    }

    #[tracing::instrument(skip_all)]
    pub async fn remove_user(&self, user_id: u64) -> Result<()> {
        self.client
            .delete("users", &user_id.to_string())
            .await
            .map_err(|e| AppError::SupabaseRequest(e.to_string()))?;

        info!(%user_id, "User removed from database");
        Ok(())
    }

    #[tracing::instrument(skip(self, user))]
    pub async fn insert_user(&self, user: Value) -> Result<()> {
        let inserted_id = self
            .client
            .insert("users", user)
            .await
            .map_err(|e| AppError::SupabaseRequest(e.to_string()))?;

        info!(?inserted_id, "User inserted into database");
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn list_users(&self) -> Result<Vec<Account>> {
        info!("Fetching all users from Supabase 'users' table...");
        let raw_accounts_data = self
            .client
            .select("users")
            .execute()
            .await
            .map_err(|e| AppError::SupabaseRequest(e.to_string()))?;

        let raw_accounts_count = raw_accounts_data.len();
        debug!(raw_accounts_count, "Received raw values from Supabase.");

        let accounts: Vec<Account> = raw_accounts_data
            .into_iter()
            .filter_map(
                |raw_account_value| match serde_json::from_value::<Account>(raw_account_value.clone()) {
                    Ok(account) => Some(account),
                    Err(e) => {
                        warn!(
                            error = %e,
                            raw_json_value = ?raw_account_value.to_string(),
                            "Failed to deserialize user from raw value. Skipping this entry."
                        );
                        None
                    }
                },
            )
            .collect();

        let parsed_accounts_count = accounts.len();

        if parsed_accounts_count < raw_accounts_count {
            warn!(
                parsed_count = parsed_accounts_count,
                raw_count = raw_accounts_count,
                lost_count = raw_accounts_count - parsed_accounts_count,
                "Some user entries failed to parse and were skipped."
            );
        } else {
            info!(
                count = parsed_accounts_count,
                "Successfully fetched and parsed all users."
            );
        }

        Ok(accounts)
    }
}
