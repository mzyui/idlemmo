#![allow(dead_code, unused)]
mod client;
mod config;
mod db;
mod error;
mod models;
mod parser;

mod utils;

use requestty::{Answers, Question, question::Choice::DefaultSeparator};
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, fmt::Subscriber};

use crate::{
    client::{AccountManagement, ActionSkillApi, CharacterApi, IdleMMOClient},
    error::Result,
    models::SkillConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    Subscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .compact()
        .init();

    run().await
}

#[allow(unreachable_code)]
async fn run() -> Result<()> {
    let mut client = IdleMMOClient::new()?;

    let users = client.get_account().await?;
    let user = fastrand::choice(users).unwrap();

    client.load_account(user).await?;
    dbg!(client.get_all_characters().await?);

    // client
    //     .start_skill(SkillConfig {
    //         skill_type: models::SkillType::Mining,
    //         ..Default::default()
    //     })
    //     .await?;

    return Ok(());

    eprintln!();
    loop {
        let questions = make_questions();
        let answers = requestty::prompt(questions)?;
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
            .when(|ans: &Answers| {
                ans.get("choice")
                    .and_then(|a| a.as_list_item())
                    .is_some_and(|i| i.index == 1)
            })
            .validate_on_key(|v: &str, _: &Answers| v.contains('@'))
            .build(),
        Question::password("password")
            .message("Password:")
            .when(|ans: &Answers| {
                ans.get("choice")
                    .and_then(|a| a.as_list_item())
                    .is_some_and(|i| i.index == 1)
            })
            .mask('•')
            .build(),
    ]
}

async fn handle_choice(
    client: &mut IdleMMOClient,
    choice_index: usize,
    answers: &Answers,
) -> Result<bool> {
    match choice_index {
        0 => {
            info!("Starting bot...");

            for user in client.get_account().await? {
                client.load_account(user).await?;
            }
        }
        1 => {
            let email = answers
                .get("email")
                .and_then(|a| a.as_string())
                .unwrap_or_default();
            let password = answers
                .get("password")
                .and_then(|a| a.as_string())
                .unwrap_or_default();
            if email.is_empty() || password.is_empty() {
                warn!("Email and password cannot be empty.");
            } else {
                client.add_account(email, password).await?;
            }
        }
        2 => {
            info!("Rechecking accounts...");
            for user in client.get_account().await? {
                client.load_account(user).await?;
            }
            info!("Accounts rechecked.");
        }
        _ => {
            info!("Exiting.\n");
            return Ok(false);
        }
    }
    Ok(true)
}
