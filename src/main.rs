use anyhow::Result;
use log::{info, warn};
use teloxide::prelude::*;
use tokio::time::{sleep, Duration};

mod power;
mod battery;
mod agents;

use agents::Priority;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting Agent-awake...");

    let token = std::env::var("TELOXIDE_TOKEN")
        .expect("TELOXIDE_TOKEN must be set in .env file");

    let bot = Bot::new(token);

    // Background monitor
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
/status   - Current status + agent priorities\n\
/awake    - Force keep system awake\n\
/sleep    - Allow normal sleep again\n\
/battery  - Battery information\n\
/help     - Show this message\n\n\
Multi-agent prioritization is active.\n\
High priority agents (Claude, Codex, Antigravity, Cursor) keep the system awake preferentially.";
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
    let agents_text = agents::format_agents_for_status();
    let highest = agents::highest_priority();

    let priority_note = match highest {
        Some(Priority::Critical) | Some(Priority::High) => {
            "\n\n⚡ High priority agent active → keep-awake preferred"
        }
        Some(Priority::Normal) => "\n\n🟢 Normal priority agents running",
        Some(Priority::Low) => "\n\n⚪ Only low priority agents",
        None => "",
    };

    format!(
        "📊 Status\n\n\
Keep Awake: {}\n\n\
{}\n\n\
Running Agents:\n{}{}",
        if awake { "🟢 ON" } else { "⚪ OFF" },
        battery,
        agents_text,
        priority_note
    )
}

/// Background task with priority-aware keep-awake logic.
async fn background_monitor() {
    info!("Background agent monitor started (with prioritization)");

    loop {
        let agents = agents::detect_running_agents();
        let currently_awake = power::is_awake();
        let highest = agents.iter().map(|a| a.priority).max();

        // Decision logic
        let should_be_awake = match highest {
            Some(Priority::Critical) | Some(Priority::High) => true,
            Some(Priority::Normal) => true,  // still keep awake for normal agents
            Some(Priority::Low) => {
                // Only keep awake for low priority if battery is healthy
                match battery::get_battery_percent() {
                    Some(p) if p > 30 => true,
                    _ => false,
                }
            }
            None => false,
        };

        if should_be_awake && !currently_awake {
            match power::keep_awake(true) {
                Ok(_) => info!("Auto-enabled keep-awake (priority-aware)"),
                Err(e) => warn!("Failed to auto-enable keep-awake: {}", e),
            }
        }

        // Safety: force disable on very low battery
        if let Some(percent) = battery::get_battery_percent() {
            if percent <= 15 && power::is_awake() {
                let _ = power::keep_awake(false);
                warn!("Battery low ({}%). Keep-awake disabled for safety.", percent);
            }
        }

        sleep(Duration::from_secs(20)).await;
    }
}
