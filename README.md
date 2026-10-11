# Agent-awake

**Keep your AI agents running even with the laptop lid closed.**  
Telegram • System tray • Priorities • Bag mode • Thermal • Audit • Local model intelligence

![CI](https://github.com/vinyaspoojary318-cloud/Agent-awake/actions/workflows/ci.yml/badge.svg)

## Features (v0.3)

| Area | What you get |
|------|----------------|
| Keep-awake | Windows sleep prevention + lid-close → Do nothing |
| System tray | Green/blue/gray icon + Keep Awake / Sleep / **Bag Mode** / Quit |
| Security & audit | `audit.log` + `/audit` |
| Zero-config detection | Claude, Codex, Antigravity, Cursor, Ollama, LM Studio, Aider, … |
| Battery longevity | Adaptive cutoff on battery + `/battery` |
| Bag mode | Stricter low-priority rules off AC; tray toggle |
| Thermal (Windows) | CPU-load heat proxy; prefer high-priority when hot |
| Local models | Unload Ollama on low battery / critical heat |
| Telegram | Full remote control |

## Quick Start

```bash
git clone https://github.com/vinyaspoojary318-cloud/Agent-awake.git
cd Agent-awake
cp .env.example .env
# set TELOXIDE_TOKEN from @BotFather
cargo run
```

Release:
```bash
cargo build --release
# target/release/agent-awake.exe
```

## Tray icon colors

| Color | Meaning |
|-------|---------|
| Green | Keep-awake ON |
| Blue | Keep-awake ON + bag mode |
| Gray | Normal sleep allowed |

Right-click: **Keep Awake** · **Allow Sleep** · **Toggle Bag Mode** · **Log Status** · **Quit**

## Telegram commands

`/status` `/awake` `/sleep` `/battery` `/lid` `/config` `/audit` `/thermal` `/bag` `/help`

## Config

Usually `%APPDATA%\agent-awake\config.toml`:

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

## License

MIT
