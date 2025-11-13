use async_trait::async_trait;

use crate::error::Result;
use crate::models::Account;

#[allow(dead_code)]
#[async_trait]
pub trait AccountManagement {
    async fn load_account(&mut self, account: Account) -> Result<()>;
    async fn get_account(&self) -> Result<Vec<Account>>;
    async fn add_account(&mut self, email: &str, password: &str) -> Result<()>;
    async fn post_login(&mut self, email: &str, password: &str) -> Result<()>;
}
