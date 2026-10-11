use log::{info, warn};
use serde_json::json;

/// Ask Ollama to unload models / stop heavy work when battery or heat is critical.
/// Best-effort: ignores errors if Ollama is not running.
pub async fn throttle_ollama(base_url: &str) {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    // List running models
    let ps_url = format!("{}/api/ps", base_url.trim_end_matches('/'));
    let running = match client.get(&ps_url).send().await {
        Ok(r) if r.status().is_success() => r.json::<serde_json::Value>().await.ok(),
        _ => None,
    };

    let Some(body) = running else {
        return;
    };

    let models = body
        .get("models")
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default();

    if models.is_empty() {
        return;
    }

    info!("Local model intelligence: {} Ollama model(s) loaded — requesting unload", models.len());

    for m in models {
        if let Some(name) = m.get("name").and_then(|n| n.as_str()) {
            let generate_url = format!("{}/api/generate", base_url.trim_end_matches('/'));
            let payload = json!({
                "model": name,
                "keep_alive": 0
            });
            match client.post(&generate_url).json(&payload).send().await {
                Ok(_) => info!("Requested unload for Ollama model {}", name),
                Err(e) => warn!("Failed to unload {}: {}", name, e),
            }
        }
    }
}

pub fn ollama_running_process() -> bool {
    use sysinfo::{ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All);
    sys.processes().values().any(|p| {
        p.name().to_string_lossy().to_lowercase().contains("ollama")
    })
}
