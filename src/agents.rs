use sysinfo::{System, ProcessesToUpdate};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical = 3,
    High = 2,
    Normal = 1,
    Low = 0,
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

/// Broader zero-config keyword list (process name fragments).
fn agent_definitions() -> Vec<(&'static str, Priority)> {
    vec![
        // High — main coding agents
        ("claude", Priority::High),
        ("codex", Priority::High),
        ("antigravity", Priority::High),
        ("agy", Priority::High),
        ("cursor", Priority::High),
        ("windsurf", Priority::High),
        ("aide", Priority::High),
        ("zed", Priority::Normal), // editor; may host agents

        // Normal — CLIs & local inference
        ("ollama", Priority::Normal),
        ("lm studio", Priority::Normal),
        ("lmstudio", Priority::Normal),
        ("aider", Priority::Normal),
        ("continue", Priority::Normal),
        ("hermes", Priority::Normal),
        ("opencode", Priority::Normal),
        ("goose", Priority::Normal),
        ("cline", Priority::Normal),
        ("roo", Priority::Normal),
        ("copilot", Priority::Normal),
        ("gemini", Priority::Normal),
        ("gpt", Priority::Low),
        ("vllm", Priority::Normal),
        ("llama-server", Priority::Normal),
        ("llama.cpp", Priority::Normal),
        ("text-generation", Priority::Low),
        ("tabbyapi", Priority::Normal),
        ("kobold", Priority::Low),
        ("comfyui", Priority::Low),
        ("invokeai", Priority::Low),
        ("open-webui", Priority::Low),
        ("sillytavern", Priority::Low),
        ("oobabooga", Priority::Low),
        ("faraday", Priority::Normal),
        ("jan.exe", Priority::Normal),
        ("nomic", Priority::Low),
        ("anythingllm", Priority::Low),
        ("privategpt", Priority::Low),
        ("localai", Priority::Normal),
        ("lm-studio", Priority::Normal),
    ]
}

pub fn detect_running_agents() -> Vec<DetectedAgent> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All);

    let definitions = agent_definitions();
    let mut found: HashMap<String, DetectedAgent> = HashMap::new();

    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        let display_name = process.name().to_string_lossy().to_string();

        // Skip obvious non-agents
        if name.contains("agent-awake") {
            continue;
        }

        for (keyword, priority) in &definitions {
            if name.contains(keyword) {
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
    list.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.name.cmp(&b.name)));
    list
}

pub fn any_agent_running() -> bool {
    !detect_running_agents().is_empty()
}

pub fn highest_priority() -> Option<Priority> {
    detect_running_agents().into_iter().map(|a| a.priority).max()
}

pub fn format_agents_for_status() -> String {
    let agents = detect_running_agents();
    if agents.is_empty() {
        return "None detected".to_string();
    }
    agents
        .iter()
        .map(|a| format!("{} {} ({})", a.priority.emoji(), a.name, a.priority.as_str()))
        .collect::<Vec<_>>()
        .join("\n")
}
