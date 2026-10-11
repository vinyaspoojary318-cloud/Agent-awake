use anyhow::Result;
use log::{info, warn};
use teloxide::prelude::*;
use tokio::time::{sleep, Duration};

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

    let token = std::env::var("TELOXIDE_TOKEN")
        .expect("TELOXIDE_TOKEN must be set in .env file");

    let bot = Bot::new(token);

    // Start background monitor (auto keep-awake when agents are running)
    tokio::spawn(async {
        background_monitor().await;
    });

    info!("Bot is running. Send /help to your bot.");

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        if let Some(text) = msg.text() {
            let chat_id = msg.chat.id;

            match text.trim() {
                "/start" | "/help" => {
                    let help = "Agent-awake is running.\n\n\
Available commands:\n\
/status  - Current status\n\
/awake   - Force keep system awake\n\
/sleep   - Allow normal sleep again\n\
/battery - Battery information\n\
/help    - Show this message\n\n\
The system will automatically stay awake while AI agents are detected.";
                    bot.send_message(chat_id, help).await?;
                }

                "/status" => {
                    let status = get_status();
                    bot.send_message(chat_id, status).await?;
                }

                "/awake" => {
                    match power::keep_awake(true) {
                        Ok(_) => bot.send_message(chat_id, "✅ System will stay awake.").await?,
                        Err(e) => bot.send_message(chat_id, format!("❌ Failed: {}", e)).await?,
                    };
                }

                "/sleep" => {
                    match power::keep_awake(false) {
                        Ok(_) => bot.send_message(chat_id, "😴 Normal sleep behavior restored.").await?,
                        Err(e) => bot.send_message(chat_id, format!("❌ Failed: {}", e)).await?,
                    };
                }

                "/battery" => {
                    let info = battery::get_battery_info();
                    bot.send_message(chat_id, info).await?;
                }

                _ => {}
            }
        }
        Ok(())
    })
    .await;

    Ok(())
}

fn get_status() -> String {
    let battery = battery::get_battery_info();
    let awake = power::is_awake();
    let agents = agents::detect_running_agents();

    let agents_text = if agents.is_empty() {
        "None detected".to_string()
    } else {
        agents.iter().map(|a| format!("• {}", a)).collect::<Vec<_>>().join("\n")
    };

    format!(
        "📊 Status\n\n\
Keep Awake: {}\n\n\
{}\n\n\
Running Agents:\n{}",
        if awake { "🟢 ON" } else { "⚪ OFF" },
        battery,
        agents_text
    )
}

/// Background task that automatically enables keep-awake when agents are running.
async fn background_monitor() {
    info!("Background agent monitor started");

    loop {
        let agents_running = agents::any_agent_running();
        let currently_awake = power::is_awake();

        if agents_running && !currently_awake {
            match power::keep_awake(true) {
                Ok(_) => info!("Auto-enabled keep-awake (agents detected)"),
                Err(e) => warn!("Failed to auto-enable keep-awake: {}", e),
            }
        } else if !agents_running && currently_awake {
            // Optional: auto disable when no agents are running
            // Currently we leave it ON so manual /awake stays active.
            // Uncomment the next lines if you want strict auto mode:
            //
            // let _ = power::keep_awake(false);
            // info!("Auto-disabled keep-awake (no agents)");
        }

        // Also check battery cutoff (example: stop below 15%)
        if let Some(percent) = battery::get_battery_percent() {
            if percent <= 15 && power::is_awake() {
                let _ = power::keep_awake(false);
                warn!("Battery low ({}%). Keep-awake disabled for safety.", percent);
            }
        }

        sleep(Duration::from_secs(20)).await;
    }
}
