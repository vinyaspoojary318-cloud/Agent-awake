use sysinfo::{System, ProcessesToUpdate};
use std::collections::HashMap;

/// Priority levels for agents.
/// Higher priority agents are preferred when resources (battery/heat) are limited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical = 3, // Must keep awake (e.g. long important runs)
    High = 2,     // Important coding agents
    Normal = 1,   // Regular agents / local models
    Low = 0,      // Background / less important
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Critical => "Critical",
            Priority::High => "High",
            Priority::Normal => "Normal",
            Priority::Low => "Low",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Priority::Critical => "🔴",
            Priority::High => "🟠",
            Priority::Normal => "🟢",
            Priority::Low => "⚪",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectedAgent {
    pub name: String,
    pub priority: Priority,
    pub pid: u32,
}

/// Known agents with their default priority.
fn agent_definitions() -> Vec<(&'static str, Priority)> {
    vec![
        // High priority - main coding agents
        ("claude", Priority::High),
        ("codex", Priority::High),
        ("antigravity", Priority::High),
        ("agy", Priority::High),           // Antigravity CLI short name
        ("cursor", Priority::High),

        // Normal priority
        ("ollama", Priority::Normal),
        ("lm studio", Priority::Normal),
        ("lmstudio", Priority::Normal),
        ("aider", Priority::Normal),
        ("continue", Priority::Normal),
        ("hermes", Priority::Normal),
        ("opencode", Priority::Normal),
        ("goose", Priority::Normal),
        ("cline", Priority::Normal),

        // Lower priority / local model UIs
        ("open-webui", Priority::Low),
        ("textgen", Priority::Low),
        ("text-generation-webui", Priority::Low),
        ("comfyui", Priority::Low),
        ("invokeai", Priority::Low),
        ("sillytavern", Priority::Low),
        ("kobold", Priority::Low),
        ("tabbyapi", Priority::Low),
        ("vllm", Priority::Normal),
        ("llama-server", Priority::Normal),
        ("llama.cpp", Priority::Normal),
    ]
}

/// Detect running agents and assign priorities.
pub fn detect_running_agents() -> Vec<DetectedAgent> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All);

    let definitions = agent_definitions();
    let mut found: HashMap<String, DetectedAgent> = HashMap::new();

    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        let display_name = process.name().to_string_lossy().to_string();

        for (keyword, priority) in &definitions {
            if name.contains(keyword) {
                // Keep the highest priority if the same process matches multiple keywords
                let entry = found.entry(display_name.clone()).or_insert(DetectedAgent {
                    name: display_name.clone(),
                    priority: *priority,
                    pid: pid.as_u32(),
                });

                if *priority > entry.priority {
                    entry.priority = *priority;
                }
                break;
            }
        }
    }

    let mut list: Vec<DetectedAgent> = found.into_values().collect();

    // Sort by priority (highest first), then by name
    list.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.name.cmp(&b.name))
    });

    list
}

/// Returns true if any known agent is currently running.
pub fn any_agent_running() -> bool {
    !detect_running_agents().is_empty()
}

/// Returns the highest priority among currently running agents.
pub fn highest_priority() -> Option<Priority> {
    detect_running_agents()
        .into_iter()
        .map(|a| a.priority)
        .max()
}

/// Nice formatted list for Telegram status.
pub fn format_agents_for_status() -> String {
    let agents = detect_running_agents();

    if agents.is_empty() {
        return "None detected".to_string();
    }

    agents
        .iter()
        .map(|a| {
            format!
                "{} {} ({})",
                a.priority.emoji(),
                a.name,
                a.priority.as_str()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
