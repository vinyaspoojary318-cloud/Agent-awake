use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub battery_cutoff: u8,
    pub low_priority_min_battery: u8,
    pub override_lid_close: bool,
    pub restore_lid_on_exit: bool,
    pub monitor_interval_secs: u64,

    #[serde(default = "default_true")]
    pub audit_log: bool,

    #[serde(default = "default_true")]
    pub bag_mode: bool,

    #[serde(default = "default_heat_cpu")]
    pub heat_cpu_threshold: f32,

    #[serde(default = "default_true")]
    pub heat_prefer_high_priority: bool,

    #[serde(default = "default_true")]
    pub local_model_intelligence: bool,

    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,

    #[serde(default = "default_true")]
    pub adaptive_battery_cutoff: bool,

    /// Telegram alert when an agent looks stuck / waiting for Submit
    #[serde(default = "default_true")]
    pub stuck_notify: bool,

    /// Seconds of near-zero CPU before we treat an agent as stuck
    #[serde(default = "default_stuck_idle")]
    pub stuck_idle_secs: u64,

    /// Minimum seconds between stuck alerts
    #[serde(default = "default_stuck_cooldown")]
    pub stuck_notify_cooldown_secs: u64,
}

fn default_true() -> bool { true }
fn default_heat_cpu() -> f32 { 85.0 }
fn default_ollama_url() -> String { "http://127.0.0.1:11434".into() }
fn default_stuck_idle() -> u64 { 180 }
fn default_stuck_cooldown() -> u64 { 300 }

impl Default for Config {
    fn default() -> Self {
        Self {
            battery_cutoff: 15,
            low_priority_min_battery: 30,
            override_lid_close: true,
            restore_lid_on_exit: true,
            monitor_interval_secs: 20,
            audit_log: true,
            bag_mode: true,
            heat_cpu_threshold: 85.0,
            heat_prefer_high_priority: true,
            local_model_intelligence: true,
            ollama_url: default_ollama_url(),
            adaptive_battery_cutoff: true,
            stuck_notify: true,
            stuck_idle_secs: 180,
            stuck_notify_cooldown_secs: 300,
        }
    }
}

fn config_path() -> PathBuf {
    if let Some(proj) = directories::ProjectDirs::from("com", "AgentAwake", "agent-awake") {
        let dir = proj.config_dir();
        let _ = fs::create_dir_all(dir);
        return dir.join("config.toml");
    }
    PathBuf::from("config.toml")
}

pub fn load() -> Result<Config> {
    let path = config_path();

    let mut cfg = if path.exists() {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        toml::from_str(&content).with_context(|| "Failed to parse config.toml")?
    } else {
        let default = Config::default();
        if let Ok(serialized) = toml::to_string_pretty(&default) {
            let _ = fs::write(&path, serialized);
            log::info!("Created default config at {}", path.display());
        }
        default
    };

    if let Ok(v) = std::env::var("BATTERY_CUTOFF") {
        if let Ok(n) = v.parse::<u8>() {
            cfg.battery_cutoff = n;
        }
    }
    if let Ok(v) = std::env::var("LOW_PRIORITY_MIN_BATTERY") {
        if let Ok(n) = v.parse::<u8>() {
            cfg.low_priority_min_battery = n;
        }
    }

    Ok(cfg)
}

pub fn path_display() -> String {
    config_path().display().to_string()
}
