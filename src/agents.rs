use sysinfo::{System, ProcessesToUpdate};

/// Very basic agent detection by process name.
/// Will be improved later with hooks and better heuristics.
pub fn detect_running_agents() -> Vec<String> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All);

    let mut found = Vec::new();

    let agent_names = [
        "claude",
        "codex",
        "cursor",
        "ollama",
        "node", // temporary - many agents run on node
    ];

    for (_, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_lowercase();

        for agent in agent_names {
            if name.contains(agent) {
                let display = format!("- {} ({})", process.name().to_string_lossy(), process.pid());
                if !found.contains(&display) {
                    found.push(display);
                }
            }
        }
    }

    found
}
