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
mod audit;
mod thermal;
mod battery_health;
mod bagmode;
mod local_models;

use agents::Priority;
use config::Config;
use thermal::ThermalState;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting Agent-awake v0.3...");

    let cfg = Arc::new(config::load()?);
    info!("Config loaded (cutoff={}%, file: {})", cfg.battery_cutoff, config::path_display());

    let _ = hooks::install_claude_hooks();
    let _ = hooks::install_codex_hooks();

    tray::start(cfg.override_lid_close, cfg.restore_lid_on_exit);

    let token = std::env::var("TELOXIDE_TOKEN")
        .expect("TELOXIDE_TOKEN must be set in .env file");

    let bot = Bot::new(token);
    let cfg_monitor = cfg.clone();

    tokio::spawn(async move {
        background_monitor(cfg_monitor).await;
    });

    let restore = cfg.restore_lid_on_exit;
    ctrlc::set_handler(move || {
        if restore {
            let _ = lid::restore_default();
        }
        let _ = power::keep_awake(false);
        bagmode::set_active(false);
        std::process::exit(0);
    }).ok();

    info!("Bot + tray running. Send /help");

    let cfg_bot = cfg.clone();
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let cfg = cfg_bot.clone();
        async move {
            if let Some(text) = msg.text() {
                let chat_id = msg.chat.id;
                match text.trim() {
                    "/start" | "/help" => {
                        let help = format!(
                            "Agent-awake v0.3\n\n\
/status /awake /sleep /battery /lid /config\n\
/audit — recent keep-awake decisions\n\
/thermal — CPU / heat proxy\n\
/bag — bag mode status\n\n\
Cutoff: {}% | Bag mode: {} | Audit: {}\n\
Config: {}",
                            cfg.battery_cutoff,
                            cfg.bag_mode,
                            cfg.audit_log,
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
                                if cfg.bag_mode {
                                    bagmode::set_active(true);
                                }
                                if cfg.audit_log {
                                    audit::log_event(true, "manual", "/awake command");
                                }
                                bot.send_message(chat_id, "✅ System will stay awake.").await?;
                            }
                            Err(e) => {
                                bot.send_message(chat_id, format!("❌ {}", e)).await?;
                            }
                        }
                    }
                    "/sleep" => {
                        let _ = power::keep_awake(false);
                        if cfg.restore_lid_on_exit {
                            let _ = lid::restore_default();
                        }
                        bagmode::set_active(false);
                        if cfg.audit_log {
                            audit::log_event(false, "manual", "/sleep command");
                        }
                        bot.send_message(chat_id, "😴 Normal sleep restored.").await?;
                    }
                    "/battery" => {
                        bot.send_message(chat_id, battery_health::summary()).await?;
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
                            "Config:\n\
• Battery cutoff: {}%\n\
• Low-priority min: {}%\n\
• Adaptive cutoff: {}\n\
• Lid override: {}\n\
• Bag mode: {}\n\
• Audit log: {}\n\
• Heat CPU threshold: {}%\n\
• Local model intelligence: {}\n\
• Ollama URL: {}\n\n\
File: {}\nAudit: {}",
                            cfg.battery_cutoff,
                            cfg.low_priority_min_battery,
                            cfg.adaptive_battery_cutoff,
                            cfg.override_lid_close,
                            cfg.bag_mode,
                            cfg.audit_log,
                            cfg.heat_cpu_threshold,
                            cfg.local_model_intelligence,
                            cfg.ollama_url,
                            config::path_display(),
                            audit::path_display()
                        );
                        bot.send_message(chat_id, text).await?;
                    }
                    "/audit" => {
                        bot.send_message(chat_id, audit::recent(12)).await?;
                    }
                    "/thermal" => {
                        let state = thermal::current_state(cfg.heat_cpu_threshold);
                        let cpu = thermal::cpu_usage();
                        bot.send_message(
                            chat_id,
                            format!(
                                "Thermal (proxy)\nCPU: {:.0}%\nState: {}\nThreshold: {:.0}%",
                                cpu,
                                state.as_str(),
                                cfg.heat_cpu_threshold
                            ),
                        )
                        .await?;
                    }
                    "/bag" => {
                        bot.send_message(chat_id, bagmode::status_line()).await?;
                    }
                    _ => {}
                }
            }
            Ok(())
        }
    })
    .await;

    if cfg.restore_lid_on_exit {
        let _ = lid::restore_default();
    }
    let _ = power::keep_awake(false);
    bagmode::set_active(false);

    Ok(())
}

fn get_status(cfg: &Config) -> String {
    let battery = battery_health::summary();
    let awake = power::is_awake();
    let agents_text = agents::format_agents_for_status();
    let hook_agents = hooks::active_hook_agents();
    let thermal = thermal::current_state(cfg.heat_cpu_threshold);
    let cutoff = battery_health::effective_cutoff(cfg.battery_cutoff, cfg.adaptive_battery_cutoff);

    let hooks_text = if hook_agents.is_empty() {
        String::new()
    } else {
        format!("\nHook-active: {}", hook_agents.join(", "))
    };

    format!(
        "📊 Status v0.3\n\n\
Keep Awake: {}\n\
Lid: {}\n\
{}\n\
Thermal: {}\n\n\
{}\n\n\
Agents:\n{}{}\n\n\
Effective cutoff: {}%",
        if awake { "🟢 ON" } else { "⚪ OFF" },
        if lid::is_overridden() { "Do nothing" } else { "Normal" },
        bagmode::status_line(),
        thermal.as_str(),
        battery,
        agents_text,
        hooks_text,
        cutoff
    )
}

async fn background_monitor(cfg: Arc<Config>) {
    info!("Background monitor v0.3 started");

    loop {
        let agents = agents::detect_running_agents();
        let hook_agents = hooks::active_hook_agents();
        let currently_awake = power::is_awake();
        let highest = agents.iter().map(|a| a.priority).max();
        let has_hooks = !hook_agents.is_empty();
        let thermal = thermal::current_state(cfg.heat_cpu_threshold);
        let effective_cutoff =
            battery_health::effective_cutoff(cfg.battery_cutoff, cfg.adaptive_battery_cutoff);

        let heat_blocks_low = cfg.heat_prefer_high_priority
            && matches!(thermal, ThermalState::Elevated | ThermalState::Critical);

        let mut should_be_awake = if has_hooks {
            true
        } else {
            match highest {
                Some(Priority::Critical) | Some(Priority::High) => true,
                Some(Priority::Normal) => !matches!(thermal, ThermalState::Critical),
                Some(Priority::Low) => {
                    if heat_blocks_low {
                        false
                    } else {
                        match battery::get_battery_percent() {
                            Some(p) if p > cfg.low_priority_min_battery => true,
                            _ => false,
                        }
                    }
                }
                None => false,
            }
        };

        if cfg.bag_mode && should_be_awake {
            if matches!(highest, Some(Priority::Low)) && !battery_health::assess().on_ac {
                should_be_awake = false;
            }
        }

        if should_be_awake && !currently_awake {
            if power::keep_awake(true).is_ok() {
                info!("Auto-enabled keep-awake");
                if cfg.override_lid_close {
                    let _ = lid::set_do_nothing();
                }
                if cfg.bag_mode {
                    bagmode::set_active(true);
                }
                if cfg.audit_log {
                    let reason = if has_hooks { "hooks" } else { "agents" };
                    let details = format!(
                        "priority={:?}, thermal={}, agents={}",
                        highest.map(|p| p.as_str()),
                        thermal.as_str(),
                        agents.len()
                    );
                    audit::log_event(true, reason, &details);
                }
            }
        }

        if let Some(percent) = battery::get_battery_percent() {
            if percent <= effective_cutoff && power::is_awake() {
                let _ = power::keep_awake(false);
                if cfg.restore_lid_on_exit {
                    let _ = lid::restore_default();
                }
                bagmode::set_active(false);
                if cfg.audit_log {
                    audit::log_event(
                        false,
                        "battery_cutoff",
                        &format!("{}% <= effective {}", percent, effective_cutoff),
                    );
                }
                warn!("Battery {}% ≤ cutoff {}%", percent, effective_cutoff);

                if cfg.local_model_intelligence && local_models::ollama_running_process() {
                    local_models::throttle_ollama(&cfg.ollama_url).await;
                }
            }
        }

        if matches!(thermal, ThermalState::Critical)
            && cfg.local_model_intelligence
            && local_models::ollama_running_process()
        {
            local_models::throttle_ollama(&cfg.ollama_url).await;
            if power::is_awake() && matches!(highest, Some(Priority::Low) | None) {
                let _ = power::keep_awake(false);
                if cfg.audit_log {
                    audit::log_event(false, "thermal", "critical CPU proxy");
                }
            }
        }

        sleep(Duration::from_secs(cfg.monitor_interval_secs)).await;
    }
}
