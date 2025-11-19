#![allow(unused, dead_code)]
mod client;
mod config;
mod db;
mod error;
mod models;
mod parser;

mod utils;

use requestty::{Answers, Question, question::Choice::DefaultSeparator};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{EnvFilter, fmt::Subscriber};

use crate::models::action_model::SkillType;
use crate::{
    client::{
        AccountManagement, CharacterApi, IdleMMOClient, LocationApi, actions::ActionSkillApi,
    },
    error::Result,
    models::SkillConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    let log_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    Subscriber::builder()
        .with_env_filter(log_filter)
        .with_target(false)
        .without_time()
        .compact()
        .init();

    run().await
}

#[allow(unreachable_code)]
async fn run() -> Result<()> {
    let mut client = IdleMMOClient::new()?;

    if true {
        let accounts = client.get_account().await?;
        let account = if accounts.is_empty() {
            warn!("No accounts found. Please add an account first.");
            return Ok(());
        } else {
            accounts[0].clone()
        };

        client.load_account(account).await?;
        dbg!(client.get_all_characters().await?);

        return Ok(());
        let chars = client.get_all_characters().await?;
        let char = fastrand::choice(chars).unwrap();
        client.switch_character(char).await?;

        let loc = fastrand::choice(client.get_locations(false).await?).unwrap();
        client
            .move_location(crate::models::location::TravelMode::Teleport, &loc)
            .await?;

        dbg!(client.get_skill_data(&SkillType::Fishing).await?);

        client
            .start_skill(SkillConfig {
                skill_type: SkillType::Mining,
                ..Default::default()
            })
            .await?;
        // &client.cache.character_info;

        return Ok(());
        //
    }

    eprintln!();
    loop {
        let prompts = make_questions();
        let answers = requestty::prompt(prompts)?;
        let choice = answers.get("choice").and_then(|a| a.as_list_item());

        eprintln!();
        if let Some(choice) = choice {
            if !handle_choice(&mut client, choice.index, &answers).await? {
                break;
            }
        } else {
            break;
        }
        eprintln!();
    }

    Ok(())
}

fn make_questions() -> Vec<Question<'static>> {
    vec![
        Question::select("choice")
            .message("What would you like to do?")
            .choices(vec![
                "Start IdleMMO bot".into(),
                "Add / log in IdleMMO account".into(),
                "Recheck accounts (remove inactive)".into(),
                DefaultSeparator,
                "Exit".into(),
            ])
            .build(),
        Question::input("email")
            .message("Email:")
            .when(|resps: &Answers| {
                resps
                    .get("choice")
                    .and_then(|answer| answer.as_list_item())
                    .is_some_and(|item| item.index == 1)
            })
            .validate_on_key(|v: &str, _: &Answers| v.contains('@'))
            .build(),
        Question::password("password")
            .message("Password:")
            .when(|resps: &Answers| {
                resps
                    .get("choice")
                    .and_then(|answer| answer.as_list_item())
                    .is_some_and(|item| item.index == 1)
            })
            .mask('•')
            .build(),
    ]
}

async fn handle_choice(
    client: &mut IdleMMOClient,
    choice_idx: usize,
    resps: &Answers,
) -> Result<bool> {
    match choice_idx {
        0 => {
            info!("Starting bot...");
            load_all_accounts(client).await?;
        }
        1 => {
            let user_email = resps
                .get("email")
                .and_then(|answer| answer.as_string())
                .unwrap_or_default();
            let password = resps
                .get("password")
                .and_then(|answer| answer.as_string())
                .unwrap_or_default();
            if user_email.is_empty() || password.is_empty() {
                warn!("Email and password cannot be empty.");
            } else {
                client.add_account(user_email, password).await?;
            }
        }
        2 => {
            info!("Rechecking accounts...");
            load_all_accounts(client).await?;
            info!("Accounts rechecked.");
        }
        _ => {
            info!("Exiting.");
            return Ok(false);
        }
    }
    Ok(true)
}

async fn load_all_accounts(client: &mut IdleMMOClient) -> Result<()> {
    for account in client.get_account().await? {
        client.load_account(account).await?;
    }
    Ok(())
}
