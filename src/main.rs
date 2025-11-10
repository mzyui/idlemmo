#![allow(dead_code)]
mod client;
mod config;
mod db;
mod error;
mod models;
mod parser;
mod traits;
mod utils;

use tracing::info;
use tracing_subscriber::{EnvFilter, fmt::Subscriber};

use crate::{client::IdleMMOClient, error::Result, traits::AccountManagement};

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    Subscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(true)
        .with_level(false)
        .without_time()
        .init();

    run().await
}

async fn run() -> Result<()> {
    info!("1. Add or log in to your IdleMMO account.");
    info!("2. Recheck existing accounts and remove inactive ones.");
    info!("3. Set your main account; others will be used as helpers.");

    let choice = inquire::prompt_u64("Select")?;
    let mut idlemmo_client = IdleMMOClient::new()?;
    match choice {
        1 => {
            info!("Please enter your IdleMMO credentials:");
            let email = inquire::Text::new("Email:").prompt()?;
            let password = inquire::Password::new("Password:")
                .with_display_mode(inquire::PasswordDisplayMode::Full)
                .without_confirmation()
                .prompt()?;

            idlemmo_client.add_account(&email, &password).await?;
        }
        2 => {
            for user in idlemmo_client.get_users().await? {
                idlemmo_client.load_account(user).await?;
            }
        }
        _ => {}
    }
    // let users = idlemmo_client.get_users().await?;
    // idlemmo_client
    //     .load_account(
    //         users
    //             .iter()
    //             .find(|v| v.email.contains("jok"))
    //             .unwrap()
    //             .clone(),
    //     )
    //     .await?;
    //
    // let locations = idlemmo_client.get_locations().await?;
    // let loc = fastrand::choice(locations).unwrap();
    //
    // idlemmo_client
    //     .move_location(models::location::TravelMode::Walk, loc.clone())
    //     .await?;
    //
    // dbg!(idlemmo_client.get_active_action().await?);

    // let chars = idlemmo_client.get_all_characters().await?;
    // let char = chars.iter().find(|v| !v.is_current).cloned().unwrap();
    // dbg!(idlemmo_client.switch_character(char).await?);
    // dbg!(idlemmo_client.get_character_information().await?);

    Ok(())
}
