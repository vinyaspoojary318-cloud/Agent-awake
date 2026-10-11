# Agent-awake

**Keep your AI agents running even when you close the laptop lid.**  
Remote control via Telegram bot.

Built with **Rust** for performance, safety and low resource usage.

## Features

- ✅ Keep Windows awake while AI agents are working
- ✅ Works with laptop lid closed (screen turns off, system keeps running)
- ✅ Telegram bot for remote control
- ✅ Real battery percentage reading
- ✅ Automatic keep-awake when agents are detected
- ✅ Low battery safety cutoff (15%)
- ✅ Lightweight

### Supported / Detected Agents
Claude Code, Codex, Cursor, Ollama, LM Studio, Aider, Continue, Open WebUI, ComfyUI, vLLM, llama.cpp, Hermes, Goose, Cline, and more.

## Telegram Commands

| Command     | Description                              |
|-------------|------------------------------------------|
| `/status`   | Show battery, awake status, running agents |
| `/awake`    | Force keep system awake                  |
| `/sleep`    | Allow normal sleep again                 |
| `/battery`  | Battery information                      |
| `/help`     | Show available commands                  |

## Quick Start

### 1. Create Telegram Bot
1. Open Telegram → talk to [@BotFather](https://t.me/BotFather)
2. Send `/newbot` and follow instructions
3. Copy the token

### 2. Setup the project

```bash
git clone https://github.com/vinyaspoojary318-cloud/Agent-awake.git
cd Agent-awake
cp .env.example .env
```

Edit `.env` and paste your token:

```
TELOXIDE_TOKEN=your_bot_token_here
```

### 3. Run

```bash
cargo run
```

### 4. Build optimized release

```bash
cargo build --release
```

Binary location: `target/release/agent-awake.exe`

## How it works

- Uses Windows `SetThreadExecutionState` to prevent system sleep
- Display is allowed to turn off (saves battery)
- Background monitor checks every 20 seconds for running agents
- Automatically enables keep-awake when agents are detected
- Disables keep-awake if battery drops to 15% or below

## Roadmap

- [x] Project structure
- [x] Windows keep-awake
- [x] Telegram bot core
- [x] Real battery reading
- [x] Agent process detection
- [x] Auto keep-awake when agents run
- [ ] Change lid-close action to "Do nothing" (advanced)
- [ ] System tray icon
- [ ] Better agent hooks (Claude Code / Codex official hooks)
- [ ] Configurable battery cutoff
- [ ] Auto-start with Windows

## License

MIT
