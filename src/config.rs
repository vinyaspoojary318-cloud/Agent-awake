use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Battery percentage at which keep-awake is forced off (default 15)
    pub battery_cutoff: u8,

    /// For Low priority agents, only keep awake if battery is above this (default 30)
    pub low_priority_min_battery: u8,

    /// Automatically set lid-close action to "Do nothing" while agents run
    pub override_lid_close: bool,

    /// Restore original lid-close action when Agent-awake exits
    pub restore_lid_on_exit: bool,

    /// How often to scan for agents (seconds)
    pub monitor_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            battery_cutoff: 15,
            low_priority_min_battery: 30,
            override_lid_close: true,
            restore_lid_on_exit: true,
            monitor_interval_secs: 20,
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

    if path.exists() {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        let cfg: Config = toml::from_str(&content)
            .with_context(|| "Failed to parse config.toml")?;
        return Ok(cfg);
    }

    // Create default config file for the user
    let default = Config::default();
    if let Ok(serialized) = toml::to_string_pretty(&default) {
        let _ = fs::write(&path, serialized);
        log::info!("Created default config at {}", path.display());
    }

    // Also allow overrides from environment
    let mut cfg = default;
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
