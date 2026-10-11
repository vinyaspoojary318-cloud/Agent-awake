use chrono::Local;
use log::info;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn audit_path() -> PathBuf {
    if let Some(proj) = directories::ProjectDirs::from("com", "AgentAwake", "agent-awake") {
        let dir = proj.data_local_dir();
        let _ = fs::create_dir_all(dir);
        return dir.join("audit.log");
    }
    PathBuf::from("audit.log")
}

/// Append a structured audit line explaining why keep-awake changed.
pub fn log_event(enabled: bool, reason: &str, details: &str) {
    let state = if enabled { "AWAKE_ON" } else { "AWAKE_OFF" };
    let line = format!(
        "{} | {} | {} | {}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        state,
        reason,
        details
    );

    info!("AUDIT {} — {} ({})", state, reason, details);

    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(audit_path())
    {
        let _ = f.write_all(line.as_bytes());
    }
}

pub fn path_display() -> String {
    audit_path().display().to_string()
}

/// Return last N lines of the audit log (for Telegram /status).
pub fn recent(n: usize) -> String {
    let path = audit_path();
    if !path.exists() {
        return "No audit entries yet.".into();
    }
    match fs::read_to_string(path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().rev().take(n).collect();
            if lines.is_empty() {
                "No audit entries yet.".into()
            } else {
                lines.into_iter().rev().collect::<Vec<_>>().join("\n")
            }
        }
        Err(_) => "Could not read audit log.".into(),
    }
}
