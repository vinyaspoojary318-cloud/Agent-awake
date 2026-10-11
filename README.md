# Agent-awake

**Keep your AI agents running even with the laptop lid closed.**  
Telegram • System tray • Priorities • Bag mode • Thermal • Audit • Local model intelligence

## Features (v0.3)

| Area | What you get |
|------|----------------|
| Keep-awake | Windows sleep prevention + lid-close → Do nothing |
| **Security & audit** | `audit.log` of every AWAKE_ON/OFF decision + `/audit` |
| **Zero-config detection** | Broad process list (Claude, Codex, Antigravity, Cursor, Ollama, LM Studio, Aider, ComfyUI, …) |
| **Battery health** | Adaptive cutoff (higher floor on battery) + `/battery` guidance |
| **Bag mode** | Stricter rules for low-priority agents off AC; network-light profile flag |
| **Thermal (Windows)** | CPU-load proxy for heat; prefer high-priority agents when hot |
| **Local model intelligence** | Unload Ollama models on low battery / critical heat |
| Telegram + tray | Full remote + right-click menu |

## Quick Start

```bash
git clone https://github.com/vinyaspoojary318-cloud/Agent-awake.git
cd Agent-awake
cp .env.example .env
# set TELOXIDE_TOKEN
cargo run
```

## New commands

| Command | Description |
|---------|-------------|
| `/audit` | Last keep-awake decisions |
| `/thermal` | CPU usage + thermal state |
| `/bag` | Bag mode status |
| `/battery` | Level + longevity notes |
| `/config` | Full config including new flags |

## Config (`config.toml`)

```toml
battery_cutoff = 15
low_priority_min_battery = 30
override_lid_close = true
restore_lid_on_exit = true
monitor_interval_secs = 20
audit_log = true
bag_mode = true
heat_cpu_threshold = 85.0
heat_prefer_high_priority = true
local_model_intelligence = true
ollama_url = "http://127.0.0.1:11434"
adaptive_battery_cutoff = true
```

## How the new logic works

1. **Audit** — every auto/manual wake change is logged with reason (agents, hooks, battery, thermal).
2. **Bag mode** — when enabled, low-priority agents on battery do not keep the machine awake.
3. **Thermal** — high CPU (proxy for heat) blocks low-priority wake; critical heat can unload Ollama.
4. **Adaptive battery** — on battery, effective cutoff is raised (+5%, max 40%) to protect longevity.
5. **Local models** — calls Ollama `/api/generate` with `keep_alive: 0` to unload models under stress.

## License

MIT
