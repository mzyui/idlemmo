use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Account {
    pub id: i64,
    pub email: String,
    pub api_token: String,
    pub cookie_str: String,
}
