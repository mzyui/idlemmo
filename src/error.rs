use thiserror::Error;
#[allow(unused_imports)]
use url::ParseError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("HTTP request failed: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Failed to parse value: {0}")]
    Parse(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database request error: {0}")]
    SupabaseRequest(String),

    #[error("Database builder error: {0}")]
    SupabaseBuilder(String),

    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("An unknown error occurred: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Failed to convert cookie to string: {0}")]
    ToStr(#[from] reqwest::header::ToStrError),

    #[error("Failed to parse integer: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("User input error: {0}")]
    UserInputError(#[from] requestty::ErrorKind),
}

pub type Result<T> = std::result::Result<T, AppError>;
