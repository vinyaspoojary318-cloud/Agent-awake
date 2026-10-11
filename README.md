<div align="center">

# Agent-awake

**Close the lid. Let your agents cook.**

Keep Windows AI coding agents running after you shut the laptop — with priorities, bag mode, thermal gates, adaptive battery cutoff, Telegram remote control, and a system tray.

[![CI](https://github.com/vinyaspoojary318-cloud/Agent-awake/actions/workflows/ci.yml/badge.svg)](https://github.com/vinyaspoojary318-cloud/Agent-awake/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%2F11-0078D6.svg)](https://github.com/vinyaspoojary318-cloud/Agent-awake)

**Repo:** [github.com/vinyaspoojary318-cloud/Agent-awake](https://github.com/vinyaspoojary318-cloud/Agent-awake)  
**CI builds:** [Actions → agent-awake-windows artifact](https://github.com/vinyaspoojary318-cloud/Agent-awake/actions)  
**Setup (Codex · Antigravity · Ollama · Claude):** [Supported agents & setup](https://github.com/vinyaspoojary318-cloud/Agent-awake#supported-agents--setup)  
**Telegram:** set `TELOXIDE_TOKEN` from [@BotFather](https://t.me/BotFather)

</div>

---

## Why this exists

You start Claude Code / Codex / Cursor on a long refactor.  
You close the lid to catch a train.  
Windows sleeps. The agent dies mid-sentence.

Generic “keep awake” tools stay on forever and kill your battery.  
**Agent-awake** stays awake **only while agents need the machine**, then restores normal sleep, lid policy, and (optionally) unloads local models.

Inspired by the Mac “lid closed, agents still working” wave — built natively for **Windows**, with extra control most Mac tools don’t ship.

---

## Feature matrix (v0.3)

| Area | What you get |
|------|----------------|
| **Keep-awake** | `SetThreadExecutionState` (system required) so idle sleep doesn’t kill agents |
| **Lid-close override** | Temporarily sets power plan lid action to **Do nothing** via `powercfg`; restores on exit / sleep |
| **Multi-agent priority** | Critical → High → Normal → Low; only meaningful work keeps the box awake |
| **Zero-config detection** | Process-name matching: Claude, Codex, Antigravity, Cursor, Windsurf, Ollama, LM Studio, Aider, Copilot, Gemini CLI, OpenCode, Jan, LocalAI, … |
| **Hooks** | Optional Claude Code / Codex “active” marker files for more precise sessions |
| **System tray** | Color-coded icon + menu: Keep Awake · Allow Sleep · Toggle Bag Mode · Log Status · Quit |
| **Bag mode** | Stricter rules when you’re likely in a bag (low-priority work off AC is refused) |
| **Battery longevity** | Configurable cutoff + **adaptive** raise when drain is rapid |
| **Thermal gate** | CPU-load heat proxy; prefer high-priority agents when hot; can unload Ollama |
| **Local model intelligence** | Talks to Ollama HTTP API (`keep_alive: 0`) to unload models under stress |
| **Telegram bot** | Full remote control from your phone: status, awake/sleep, bag, audit, thermal |
| **Audit trail** | Disk log + in-memory ring; `/audit` and tray “Log Status” |
| **Config** | `%APPDATA%\agent-awake\config.toml` + env overrides |
| **CI** | GitHub Actions `windows-latest` → uploads `agent-awake.exe` |

---

## Supported agents & setup

> **No API keys. No OAuth. No “link account” step.**  
> Start Agent-awake, run your agent as usual. It watches process names (and optional hook files).

### Codex (OpenAI)

1. Install and run [Codex CLI](https://github.com/openai/codex) the normal way.  
2. Start **Agent-awake** (tray or `agent-awake.exe`).  
3. When a process name contains `codex`, it is treated as **High** priority → machine can stay awake with lid closed.  
4. Optional: install hook markers (Telegram `/hooks` or auto on start) so “session active” is clearer even if the process name is odd.

**Check:** Telegram `/status` or tray tooltip should list Codex while it is running.

### Antigravity

1. Run Antigravity as you already do (desktop app / CLI).  
2. Agent-awake matches process names containing `antigravity` or `agy` → **High** priority.  
3. No extra config. Close the lid when a long job is running; keep-awake + lid override apply while it is detected.

**Check:** `/status` should show Antigravity under Agents.

### Ollama

Two layers:

| Layer | What happens |
|-------|----------------|
| **Detection** | Process name contains `ollama` → **Normal** priority (can keep machine awake) |
| **Local model intelligence** | On low battery or critical heat, Agent-awake can call Ollama’s HTTP API to unload models (`keep_alive: 0`) |

**Setup:**

1. Run [Ollama](https://ollama.com/) on the same PC (default `http://127.0.0.1:11434`).  
2. In `%APPDATA%\agent-awake\config.toml`:

```toml
local_model_intelligence = true
ollama_url = "http://127.0.0.1:11434"
```

3. If Ollama listens elsewhere, change `ollama_url` or set env `AGENT_AWAKE_OLLAMA_URL`.

No token required. Unload is best-effort (ignored if Ollama is not running).

### Claude Code

1. Run Claude Code as usual.  
2. Process name containing `claude` → **High** priority.  
3. Optional hooks: Agent-awake can install marker files so “active session” is detected more reliably (same idea as Codex).

### Cursor / Windsurf / Aider / Copilot / others

Zero-config process matching. Common fragments:

| Priority | Matched names (examples) |
|----------|---------------------------|
| **High** | `claude`, `codex`, `antigravity`, `agy`, `cursor`, `windsurf`, `aide` |
| **Normal** | `ollama`, `lmstudio`, `aider`, `opencode`, `copilot`, `gemini`, `localai`, `jan`, `vllm`, … |
| **Low** | `gpt`, `comfyui`, `open-webui`, lightweight UIs |

If your tool’s process name is missing, open an issue with the exact name from Task Manager → we can add it.

### Quick verify checklist

1. Start Agent-awake.  
2. Start Codex / Antigravity / Ollama / Claude.  
3. `/status` (Telegram) or tray → agents listed.  
4. Close lid (with `override_lid_close = true`) → work continues.  
5. When agents exit (or battery hits cutoff) → sleep policy restored.

---

## How it works

```text
┌─────────────────────────────────────────────────────────────┐
│                     Agent-awake (tray + bot)                │
├──────────────┬──────────────┬──────────────┬────────────────┤
│  agents.rs   │  power.rs    │  lid.rs      │  battery*.rs   │
│  detect      │  keep awake  │  powercfg    │  % + adaptive  │
├──────────────┼──────────────┼──────────────┼────────────────┤
│  thermal.rs  │  bagmode.rs  │  local_models│  audit.rs      │
│  CPU proxy   │  bag policy  │  Ollama API  │  decisions     │
└──────────────┴──────────────┴──────────────┴────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
   background_monitor   Telegram commands     tray menu
```

1. **Monitor loop** (default every 20s) scans processes + optional hook markers.  
2. If priority / hooks say “stay up” and battery / heat / bag rules allow it → enable keep-awake + optional lid override.  
3. When work ends, battery hits cutoff, or heat is critical → release keep-awake, restore lid policy, optionally unload Ollama.  
4. **Telegram** and **tray** can force awake/sleep or toggle bag mode anytime.

---

## Quick start

### 1. Prerequisites

- Windows 10 / 11  
- [Rust](https://rustup.rs/) (stable)  
- Optional: [Telegram bot token](https://t.me/BotFather) for remote control  

### 2. Clone & configure

```powershell
git clone https://github.com/vinyaspoojary318-cloud/Agent-awake.git
cd Agent-awake
copy .env.example .env
# Edit .env → set TELOXIDE_TOKEN=123456:ABC-DEF...
```

### 3. Run (dev)

```powershell
cargo run
```

### 4. Release binary

```powershell
cargo build --release
.\target\release\agent-awake.exe
```

Or download the latest **CI artifact** from the [Actions](https://github.com/vinyaspoojary318-cloud/Agent-awake/actions) tab → **agent-awake-windows** → `agent-awake.exe`.

First run creates:

```text
%APPDATA%\agent-awake\config.toml
%APPDATA%\agent-awake\audit.log   (if audit_log = true)
```

> **Note:** Lid override uses `powercfg` and may need a normal user session (not a restricted sandbox). Keep-awake itself does not need admin.

---

## System tray

| Icon color | Meaning |
|------------|---------|
| **Green** | Keep-awake **ON** |
| **Blue** | Keep-awake **ON** + **bag mode** |
| **Gray** | Normal sleep allowed |

**Right-click menu**

- Keep Awake  
- Allow Sleep  
- Toggle Bag Mode  
- Log Status  
- Quit  

Tooltip shows awake state, bag mode, and a short CPU/thermal hint when available.

---

## Telegram commands

| Command | Action |
|---------|--------|
| `/start` · `/help` | Help + config summary |
| `/status` | Awake, lid, bag, thermal, battery, agents, cutoff |
| `/awake` | Force keep-awake (+ lid override / bag if configured) |
| `/sleep` | Restore normal sleep + lid policy |
| `/battery` | Battery health summary |
| `/lid` | Current lid-close policy |
| `/config` | Full config dump + paths |
| `/audit` | Recent keep-awake decisions |
| `/thermal` | CPU usage + heat proxy state |
| `/bag` | Bag mode status |

Create a bot with [@BotFather](https://t.me/BotFather), put the token in `.env` as `TELOXIDE_TOKEN`, start Agent-awake, then message the bot.

---

## Configuration

**File:** `%APPDATA%\agent-awake\config.toml`  
(Created on first run with sensible defaults.)

```toml
# Stop keeping the machine awake at or below this battery %
battery_cutoff = 15

# Low-priority agents need at least this % to justify awake
low_priority_min_battery = 30

# Raise cutoff automatically when battery is draining fast
adaptive_battery_cutoff = true

# Temporarily set lid-close → Do nothing while agents need the machine
override_lid_close = true

# Restore original lid action on exit / sleep
restore_lid_on_exit = true

# How often the monitor loop runs (seconds)
monitor_interval_secs = 20

# Write decisions to audit.log
audit_log = true

# Stricter policy when bag mode is on (especially off AC)
bag_mode = true

# CPU % treated as “hot” for thermal gate
heat_cpu_threshold = 85.0

# When hot, prefer high/critical agents over low priority
heat_prefer_high_priority = true

# Talk to Ollama to unload models under battery/heat stress
local_model_intelligence = true
ollama_url = "http://127.0.0.1:11434"
```

Environment overrides (optional):

| Variable | Maps to |
|----------|---------|
| `TELOXIDE_TOKEN` | Telegram bot token (required for bot) |
| `AGENT_AWAKE_BATTERY_CUTOFF` | `battery_cutoff` |
| `AGENT_AWAKE_BAG_MODE` | `bag_mode` |
| `AGENT_AWAKE_OLLAMA_URL` | `ollama_url` |

---

## Safety model

Agent-awake is designed so “agents can finish” does **not** mean “drain to 0% in a backpack”:

1. **Hard battery cutoff** — releases keep-awake at `battery_cutoff` (adaptive can raise it).  
2. **Priority gates** — Low priority needs higher battery; bag mode can refuse Low off AC.  
3. **Thermal** — Critical heat can unload local models and drop low-priority awake.  
4. **Restore on exit** — lid policy and keep-awake flags cleared on Ctrl+C / tray Quit.  
5. **Audit** — every auto decision can be logged for later review.

---

## Project layout

```text
Agent-awake/
├── Cargo.toml
├── LICENSE
├── README.md
├── .env.example
├── .github/workflows/ci.yml      # windows-latest build + artifact
└── src/
    ├── main.rs                   # entry, Telegram, background_monitor
    ├── config.rs                 # TOML + env
    ├── agents.rs                 # detection + Priority
    ├── power.rs                  # SetThreadExecutionState (FFI)
    ├── lid.rs                    # powercfg lid action
    ├── battery.rs                # GetSystemPowerStatus
    ├── battery_health.rs         # adaptive cutoff + summary
    ├── thermal.rs                # CPU heat proxy
    ├── bagmode.rs                # bag flag + policy helpers
    ├── audit.rs                  # audit.log + recent()
    ├── local_models.rs           # Ollama unload
    ├── hooks.rs                  # Claude / Codex markers
    └── tray.rs                   # tray-icon + menu
```

---

## CI & downloads

On every push to `main`:

1. `cargo check`  
2. `cargo build --release`  
3. Upload **agent-awake-windows** artifact (`agent-awake.exe`)

Open: [Actions](https://github.com/vinyaspoojary318-cloud/Agent-awake/actions) → latest green run → Artifacts.

---

## Comparison (high level)

| Capability | Agent-awake | Typical Mac lid tools | Generic caffeine apps |
|------------|-------------|------------------------|------------------------|
| Windows-native | ✅ | ❌ / limited | ✅ |
| Agent-aware (not always-on) | ✅ | ✅ | ❌ |
| Multi-priority agents | ✅ | rare | ❌ |
| Lid-close “Do nothing” | ✅ | macOS pmset | depends |
| Bag mode | ✅ | rare | ❌ |
| Adaptive battery cutoff | ✅ | rare | ❌ |
| Thermal gate | ✅ (CPU proxy) | some | ❌ |
| Local model unload | ✅ Ollama | rare | ❌ |
| Telegram remote | ✅ | rare | ❌ |
| System tray | ✅ | menu bar | often |
| Audit log | ✅ | rare | ❌ |

---

## Roadmap ideas

- [ ] Real WMI battery wear (design capacity vs full charge)  
- [ ] Optional bag-mode network / firewall profile  
- [ ] Richer MCP / stdio agent hooks  
- [ ] Task Scheduler / Startup folder helper  
- [ ] Optional display-off while system stays awake  

PRs and issues welcome.

---

## License

[MIT](LICENSE) — use it, fork it, ship it with your agents.

---

<div align="center">

**Agent-awake** · Close the lid · Keep the agents · Save the battery  

[Repository](https://github.com/vinyaspoojary318-cloud/Agent-awake) · [Setup guide](https://github.com/vinyaspoojary318-cloud/Agent-awake#supported-agents--setup) · [CI / Downloads](https://github.com/vinyaspoojary318-cloud/Agent-awake/actions) · [Issues](https://github.com/vinyaspoojary318-cloud/Agent-awake/issues)

</div>
