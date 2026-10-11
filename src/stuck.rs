//! Detect agents that are running but likely waiting for human input
//! (low CPU for a sustained period while keep-awake is on).

use log::info;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use sysinfo::System;

use crate::agents::{detect_running_agents, DetectedAgent};

lazy_static::lazy_static! {
    static ref TRACKER: Mutex<StuckTracker> = Mutex::new(StuckTracker::default());
}

// Avoid adding lazy_static dep if not present — use OnceLock instead.
use std::sync::OnceLock;

fn tracker() -> &'static Mutex<StuckTracker> {
    static T: OnceLock<Mutex<StuckTracker>> = OnceLock::new();
    T.get_or_init(|| Mutex::new(StuckTracker::default()))
}

#[derive(Default)]
struct StuckTracker {
    /// pid -> first time we saw low CPU while this agent was present
    low_cpu_since: HashMap<u32, Instant>,
    /// last time we fired a notification (cooldown)
    last_notify: Option<Instant>,
}

/// Returns agents that look stuck (process alive, very low CPU for `idle_secs`).
pub fn check_stuck_agents(idle_secs: u64, notify_cooldown_secs: u64) -> Vec<DetectedAgent> {
    let agents = detect_running_agents();
    if agents.is_empty() {
        if let Ok(mut t) = tracker().lock() {
            t.low_cpu_since.clear();
        }
        return Vec::new();
    }

    let mut sys = System::new();
    sys.refresh_processes();
    // Second sample helps CPU% stabilize on some systems
    std::thread::sleep(Duration::from_millis(200));
    sys.refresh_processes();

    let now = Instant::now();
    let idle = Duration::from_secs(idle_secs.max(60));
    let mut stuck = Vec::new();

    if let Ok(mut t) = tracker().lock() {
        let live_pids: std::collections::HashSet<u32> =
            agents.iter().map(|a| a.pid).collect();

        // Drop pids that exited
        t.low_cpu_since.retain(|pid, _| live_pids.contains(pid));

        for agent in &agents {
            let cpu = sys
                .process(sysinfo::Pid::from_u32(agent.pid))
                .map(|p| p.cpu_usage())
                .unwrap_or(0.0);

            // "Waiting for input" heuristic: process exists but almost idle
            if cpu < 2.0 {
                let since = t.low_cpu_since.entry(agent.pid).or_insert(now);
                if now.duration_since(*since) >= idle {
                    stuck.push(agent.clone());
                }
            } else {
                t.low_cpu_since.remove(&agent.pid);
            }
        }

        // Cooldown so we don't spam Telegram
        if !stuck.is_empty() {
            let cooldown = Duration::from_secs(notify_cooldown_secs.max(120));
            if let Some(last) = t.last_notify {
                if now.duration_since(last) < cooldown {
                    return Vec::new();
                }
            }
            t.last_notify = Some(now);
            info!(
                "Stuck heuristic: {} agent(s) low-CPU for >= {}s",
                stuck.len(),
                idle_secs
            );
        }
    }

    stuck
}

pub fn format_stuck_message(agents: &[DetectedAgent]) -> String {
    let list = agents
        .iter()
        .map(|a| format!("• {} (pid {})", a.name, a.pid))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "⚠️ Possible wait for input\n\n\
These agents are running but almost idle (may need Submit / Approve):\n{}\n\n\
Open the lid or remote desktop, click once, then close again.\n\n\
Tip: for terminal agents use auto-approve modes so this is rarer:\n\
• Codex: codex --full-auto\n\
• Claude: claude --permission-mode auto\n\
• Claude YOLO: claude --dangerously-skip-permissions\n\n\
/status for details",
        list
    )
}
