use anyhow::Result;
use log::{info, warn};
use teloxide::prelude::*;
use tokio::time::{sleep, Duration};
use std::sync::Arc;

mod power;
mod battery;
mod agents;
mod config;
mod lid;
mod hooks;
mod tray;

use agents::Priority;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting Agent-awake v0.2...");

    let cfg = Arc::new(config::load()?);
    info!("Config loaded (cutoff={}%, config file: {})", cfg.battery_cutoff, config::path_display());

    // Generate hook snippets for the user
    let _ = hooks::install_claude_hooks();
    let _ = hooks::install_codex_hooks();

    // System tray (runs on its own thread)
    tray::start(cfg.override_lid_close, cfg.restore_lid_on_exit);

    let token = std::env::var("TELOXIDE_TOKEN")
        .expect("TELOXIDE_TOKEN must be set in .env file");

    let bot = Bot::new(token);
    let cfg_monitor = cfg.clone();

    // Background monitor
    tokio::spawn(async move {
        background_monitor(cfg_monitor).await;
    });

    // Restore lid action on Ctrl+C
    let restore = cfg.restore_lid_on_exit;
    ctrlc::set_handler(move || {
        if restore {
            let _ = lid::restore_default();
        }
        let _ = power::keep_awake(false);
        std::process::exit(0);
    }).ok();

    info!("Bot is running. System tray is active. Send /help in Telegram.");

    let cfg_bot = cfg.clone();
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let cfg = cfg_bot.clone();
        async move {
            if let Some(text) = msg.text() {
                let chat_id = msg.chat.id;

                match text.trim() {
                    "/start" | "/help" => {
                        let help = format!(
                            "Agent-awake v0.2 is running.\n\n\
Commands:\n\
/status   - Status + priorities\n\
/awake    - Force keep awake\n\
/sleep    - Allow sleep\n\
/battery  - Battery info\n\
/lid      - Lid-close override status\n\
/config   - Show current config\n\
/help     - This message\n\n\
Also use the system tray icon (right-click).\n\n\
Battery cutoff: {}%\n\
Lid override: {}\n\
Config file: {}",
                            cfg.battery_cutoff,
                            if cfg.override_lid_close { "enabled" } else { "disabled" },
                            config::path_display()
                        );
                        bot.send_message(chat_id, help).await?;
                    }

                    "/status" => {
                        bot.send_message(chat_id, get_status(&cfg)).await?;
                    }

                    "/awake" => {
                        match power::keep_awake(true) {
                            Ok(_) => {
                                if cfg.override_lid_close {
                                    let _ = lid::set_do_nothing();
                                }
                                bot.send_message(chat_id, "✅ System will stay awake.").await?;
                            }
                            Err(e) => bot.send_message(chat_id, format!("❌ {}", e)).await?,
                        };
                    }

                    "/sleep" => {
                        let _ = power::keep_awake(false);
                        if cfg.restore_lid_on_exit {
                            let _ = lid::restore_default();
                        }
                        bot.send_message(chat_id, "😴 Normal sleep restored.").await?;
                    }

                    "/battery" => {
                        bot.send_message(chat_id, battery::get_battery_info()).await?;
                    }

                    "/lid" => {
                        let status = if lid::is_overridden() {
                            "Lid-close is currently set to Do nothing"
                        } else {
                            "Lid-close is using the normal system setting"
                        };
                        bot.send_message(chat_id, status).await?;
                    }

                    "/config" => {
                        let text = format!(
                            "Current config:\n\
• Battery cutoff: {}%\n\
• Low-priority min battery: {}%\n\
• Override lid-close: {}\n\
• Restore lid on exit: {}\n\
• Monitor interval: {}s\n\n\
Edit: {}",
                            cfg.battery_cutoff,
                            cfg.low_priority_min_battery,
                            cfg.override_lid_close,
                            cfg.restore_lid_on_exit,
                            cfg.monitor_interval_secs,
                            config::path_display()
                        );
                        bot.send_message(chat_id, text).await?;
                    }

                    _ => {}
                }
            }
            Ok(())
        }
    })
    .await;

    // Cleanup
    if cfg.restore_lid_on_exit {
        let _ = lid::restore_default();
    }
    let _ = power::keep_awake(false);

    Ok(())
}

fn get_status(cfg: &Config) -> String {
    let battery = battery::get_battery_info();
    let awake = power::is_awake();
    let agents_text = agents::format_agents_for_status();
    let hook_agents = hooks::active_hook_agents();
    let highest = agents::highest_priority();

    let priority_note = match highest {
        Some(Priority::Critical) | Some(Priority::High) => {
            "\n\n⚡ High priority agent active"
        }
        Some(Priority::Normal) => "\n\n🟢 Normal priority agents",
        Some(Priority::Low) => "\n\n⚪ Only low priority agents",
        None => "",
    };

    let hooks_text = if hook_agents.is_empty() {
        String::new()
    } else {
        format!("\n\nHook-active: {}", hook_agents.join(", "))
    };

    format!(
        "📊 Status\n\n\
Keep Awake: {}\n\
Lid Override: {}\n\n\
{}\n\n\
Running Agents:\n{}{}{}\n\n\
Cutoff: {}%",
        if awake { "🟢 ON" } else { "⚪ OFF" },
        if lid::is_overridden() { "Do nothing" } else { "Normal" },
        battery,
        agents_text,
        priority_note,
        hooks_text,
        cfg.battery_cutoff
    )
}

async fn background_monitor(cfg: Arc<Config>) {
    info!("Background monitor started");

    loop {
        let agents = agents::detect_running_agents();
        let hook_agents = hooks::active_hook_agents();
        let currently_awake = power::is_awake();

        let highest = agents.iter().map(|a| a.priority).max();
        let has_hooks = !hook_agents.is_empty();

        let should_be_awake = if has_hooks {
            true
        } else {
            match highest {
                Some(Priority::Critical) | Some(Priority::High) => true,
                Some(Priority::Normal) => true,
                Some(Priority::Low) => {
                    match battery::get_battery_percent() {
                        Some(p) if p > cfg.low_priority_min_battery => true,
                        _ => false,
                    }
                }
                None => false,
            }
        };

        if should_be_awake && !currently_awake {
            if power::keep_awake(true).is_ok() {
                info!("Auto-enabled keep-awake");
                if cfg.override_lid_close {
                    let _ = lid::set_do_nothing();
                }
            }
        }

        if let Some(percent) = battery::get_battery_percent() {
            if percent <= cfg.battery_cutoff && power::is_awake() {
                let _ = power::keep_awake(false);
                if cfg.restore_lid_on_exit {
                    let _ = lid::restore_default();
                }
                warn!("Battery {}% ≤ cutoff {}%. Keep-awake disabled.", percent, cfg.battery_cutoff);
            }
        }

        sleep(Duration::from_secs(cfg.monitor_interval_secs)).await;
    }
}
