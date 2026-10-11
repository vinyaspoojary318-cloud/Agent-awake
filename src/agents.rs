use sysinfo::{System, ProcessesToUpdate};
use std::collections::HashSet;

/// Detect common AI coding agents and local model servers by process name.
pub fn detect_running_agents() -> Vec<String> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All);

    let mut found: HashSet<String> = HashSet::new();

    // Common process name fragments (lowercase)
    let agent_keywords = [
        "claude",
        "codex",
        "cursor",
        "ollama",
        "lm studio",
        "lmstudio",
        "textgen",
        "text-generation-webui",
        "comfyui",
        "invokeai",
        "aider",
        "continue",
        "open-webui",
        "sillytavern",
        "kobold",
        "tabbyapi",
        "vllm",
        "llama-server",
        "llama.cpp",
        "hermes",
        "opencode",
        "goose",
        "cline",
    ];

    for (_, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_lowercase();

        for keyword in agent_keywords {
            if name.contains(keyword) {
                // Clean display name
                let display_name = process.name().to_string_lossy().to_string();
                found.insert(display_name);
                break;
            }
        }
    }

    let mut list: Vec<String> = found.into_iter().collect();
    list.sort();
    list
}

/// Returns true if any known agent is currently running.
pub fn any_agent_running() -> bool {
    !detect_running_agents().is_empty()
}
