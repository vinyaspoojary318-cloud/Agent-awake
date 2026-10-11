use anyhow::{Context, Result};
use log::info;
use std::fs;
use std::path::PathBuf;

/// Directory where Agent-awake looks for live status from hooks.
pub fn status_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("agent-awake");
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Write a simple "agent is working" marker (used by Claude/Codex hooks).
pub fn mark_agent_active(agent: &str) {
    let path = status_dir().join(format!("{}.active", agent));
    let _ = fs::write(path, chrono::Local::now().to_rfc3339());
}

/// Clear the marker when agent finishes.
pub fn mark_agent_idle(agent: &str) {
    let path = status_dir().join(format!("{}.active", agent));
    let _ = fs::remove_file(path);
}

/// Returns list of agents that have an active hook marker (updated recently).
pub fn active_hook_agents() -> Vec<String> {
    let mut result = Vec::new();
    let dir = status_dir();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                if path.extension().and_then(|e| e.to_str()) == Some("active") {
                    // Consider active if file was modified in the last 10 minutes
                    if let Ok(meta) = fs::metadata(&path) {
                        if let Ok(modified) = meta.modified() {
                            if modified.elapsed().map(|d| d.as_secs() < 600).unwrap_or(false) {
                                result.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    result
}

/// Install recommended Claude Code hooks into ~/.claude/settings.json (best-effort merge).
pub fn install_claude_hooks() -> Result<()> {
    let home = directories::UserDirs::new()
        .map(|u| u.home_dir().to_path_buf())
        .context("Could not find home directory")?;

    let settings_path = home.join(".claude").join("settings.json");
    fs::create_dir_all(settings_path.parent().unwrap())?;

    // We provide a ready-to-copy snippet rather than risking a bad JSON merge.
    let snippet_path = status_dir().join("claude-hooks-snippet.json");
    let snippet = r#"{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "echo %TEMP%\\agent-awake\\claude.active > nul & echo. > %TEMP%\\agent-awake\\claude.active"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "del %TEMP%\\agent-awake\\claude.active 2>nul"
          }
        ]
      }
    ]
  }
}"#;

    fs::write(&snippet_path, snippet)?;
    info!("Claude Code hook snippet written to {}", snippet_path.display());
    info!("Manually merge it into {}", settings_path.display());

    Ok(())
}

/// Install recommended Codex hooks snippet.
pub fn install_codex_hooks() -> Result<()> {
    let home = directories::UserDirs::new()
        .map(|u| u.home_dir().to_path_buf())
        .context("Could not find home directory")?;

    let hooks_path = home.join(".codex").join("hooks.json");
    fs::create_dir_all(hooks_path.parent().unwrap())?;

    let snippet_path = status_dir().join("codex-hooks-snippet.json");
    let snippet = r#"{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "echo. > %TEMP%\\agent-awake\\codex.active"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "del %TEMP%\\agent-awake\\codex.active 2>nul"
          }
        ]
      }
    ]
  }
}"#;

    fs::write(&snippet_path, snippet)?;
    info!("Codex hook snippet written to {}", snippet_path.display());
    info!("Manually merge it into {}", hooks_path.display());

    Ok(())
}
