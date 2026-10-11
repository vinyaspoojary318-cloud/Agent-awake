# Agent-awake

**Keep your AI agents running even when you close the laptop lid.**  
Remote control via Telegram bot.

Built with **Rust** for performance, safety and low resource usage.

## Features (Planned / In Progress)

- Keep Windows awake while AI agents are working (Claude Code, Codex, Cursor, Ollama, etc.)
- Works with laptop lid closed
- Telegram bot for remote control
- Battery monitoring + automatic cutoff
- Heat awareness
- Auto-restore power settings when agents finish
- Lightweight system tray (future)

## Commands (Telegram)

| Command     | Description                              |
|-------------|------------------------------------------|
| `/status`   | Show battery, awake status, running agents |
| `/awake`    | Force keep system awake                  |
| `/sleep`    | Allow normal sleep again                 |
| `/battery`  | Detailed battery information             |
| `/help`     | Show available commands                  |

## Quick Start

### 1. Create Telegram Bot
1. Open Telegram and talk to [@BotFather](https://t.me/BotFather)
2. Create a new bot with `/newbot`
3. Copy the token

### 2. Setup

```bash
git clone https://github.com/vinyaspoojary318-cloud/Agent-awake.git
cd Agent-awake
cp .env.example .env
# Edit .env and put your TELOXIDE_TOKEN
```

### 3. Run

```bash
cargo run
```

### 4. Build Release

```bash
cargo build --release
```

The binary will be at `target/release/agent-awake.exe`

## Roadmap

- [x] Project structure
- [ ] Windows keep-awake (PowerSetRequest / SetThreadExecutionState)
- [ ] Telegram bot core
- [ ] Battery monitoring
- [ ] Agent process detection
- [ ] Lid-close friendly power profile
- [ ] System tray icon
- [ ] Auto start with Windows

## License

MIT
