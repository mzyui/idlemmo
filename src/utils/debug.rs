use reqwest::Response;
use serde;

use crate::error::{AppError, Result};

pub(crate) async fn debug_deserialize<T: serde::de::DeserializeOwned>(
    response: Response,
) -> Result<T> {
    let html = response.text().await?;
    if html.len() > 200 {
        println!("{}\n\n{} characters hidden", &html[..199], html.len() - 199);
    } else {
        println!("{}", &html);
    }
    serde_json::from_str::<T>(&html).map_err(AppError::SerdeJson)
}
