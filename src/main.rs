use anyhow::Result;
use log::{info, error};
use teloxide::prelude::*;
use teloxide::types::ParseMode;

mod power;
mod battery;
mod agents;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting Agent-awake...");

    // Check if token exists
    let token = std::env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN must be set in .env");
    
    let bot = Bot::new(token);

    info!("Bot started. Waiting for commands...");

    // Simple command handler
    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        if let Some(text) = msg.text() {
            let chat_id = msg.chat.id;

            match text {
                "/start" | "/help" => {
                    let help = r#"
*Agent-awake* is running.

Available commands:
/status  - Current status
/awake   - Keep system awake
/sleep   - Allow normal sleep
/battery - Battery information
/help    - Show this message
"#;
                    bot.send_message(chat_id, help)
                        .parse_mode(ParseMode::MarkdownV2)
                        .await?;
                }

                "/status" => {
                    let status = get_status().await;
                    bot.send_message(chat_id, status).await?;
                }

                "/awake" => {
                    match power::keep_awake(true) {
                        Ok(_) => bot.send_message(chat_id, "System will stay awake.").await?,
                        Err(e) => bot.send_message(chat_id, format!("Failed: {}", e)).await?,
                    };
                }

                "/sleep" => {
                    match power::keep_awake(false) {
                        Ok(_) => bot.send_message(chat_id, "Normal sleep behavior restored.").await?,
                        Err(e) => bot.send_message(chat_id, format!("Failed: {}", e)).await?,
                    };
                }

                "/battery" => {
                    let info = battery::get_battery_info();
                    bot.send_message(chat_id, info).await?;
                }

                _ => {
                    // Ignore unknown messages for now
                }
            }
        }
        Ok(())
    })
    .await;

    Ok(())
}

async fn get_status() -> String {
    let battery = battery::get_battery_info();
    let awake = power::is_awake();
    let agents = agents::detect_running_agents();

    format!(
        "Status\n\nKeep Awake: {}\n\n{}\n\nRunning Agents:\n{}",
        if awake { "ON" } else { "OFF" },
        battery,
        if agents.is_empty() {
            "None detected".to_string()
        } else {
            agents.join("\n")
        }
    )
}
