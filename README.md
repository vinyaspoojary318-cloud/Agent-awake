# Agent-awake

**Keep your AI agents running even when you close the laptop lid.**  
Remote control via Telegram bot + **multi-agent prioritization**.

Built with **Rust**.

## Features

- ✅ Keep Windows awake while AI agents are working
- ✅ Works with laptop lid closed
- ✅ Telegram bot remote control
- ✅ Real battery percentage + low battery cutoff (15%)
- ✅ **Multi-agent prioritization** (Critical / High / Normal / Low)
- ✅ Auto keep-awake based on highest priority agent
- ✅ Supports **Claude Code, Codex, Antigravity, Cursor**, Ollama, and many more

### Priority System

| Priority   | Agents                                      | Behavior                                      |
|------------|---------------------------------------------|-----------------------------------------------|
| 🔴 Critical| (reserved for future manual override)       | Always keep awake                             |
| 🟠 High    | Claude Code, Codex, Antigravity, Cursor     | Strongly preferred, keep awake                |
| 🟢 Normal  | Ollama, Aider, Hermes, Goose, Cline, etc.   | Keep awake                                    |
| ⚪ Low     | Open WebUI, ComfyUI, SillyTavern, etc.      | Keep awake only if battery > 30%              |

## Telegram Commands

| Command     | Description                                      |
|-------------|--------------------------------------------------|
| `/status`   | Status + list of agents with their priorities    |
| `/awake`    | Force keep system awake                          |
| `/sleep`    | Allow normal sleep again                         |
| `/battery`  | Battery information                              |
| `/help`     | Show help                                        |

## Quick Start

```bash
git clone https://github.com/vinyaspoojary318-cloud/Agent-awake.git
cd Agent-awake
cp .env.example .env
# Put your Telegram bot token in .env
cargo run
```

Build release:
```bash
cargo build --release
```

## Supported Agents (detected)

**High Priority:** Claude Code, Codex, Antigravity (`antigravity` / `agy`), Cursor  
**Normal:** Ollama, LM Studio, Aider, Continue, Hermes, OpenCode, Goose, Cline, vLLM, llama.cpp  
**Low:** Open WebUI, ComfyUI, text-generation-webui, SillyTavern, etc.

## How prioritization works

- Background monitor checks every 20 seconds
- Highest priority running agent decides whether to keep the system awake
- Low priority agents only keep the machine awake when battery is healthy (>30%)
- Hard safety cutoff at 15% battery

## Roadmap

- [x] Core keep-awake + Telegram bot
- [x] Real battery reading
- [x] Agent detection
- [x] Multi-agent prioritization
- [x] Antigravity + Codex support
- [ ] Lid-close action override ("Do nothing")
- [ ] System tray icon
- [ ] Configurable priorities & battery thresholds
- [ ] Official Claude Code / Codex hooks

## License

MIT
