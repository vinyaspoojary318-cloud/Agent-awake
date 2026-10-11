# Agent-awake

**Keep your AI agents running even when you close the laptop lid.**  
Telegram remote control • System tray • Multi-agent prioritization • Lid-close override

## Features (v0.2)

- ✅ Keep Windows awake while AI agents work
- ✅ **System tray icon** (right-click menu: Keep Awake / Allow Sleep / Quit)
- ✅ **Lid-close → Do nothing** while agents run
- ✅ Telegram bot remote control
- ✅ **Configurable battery cutoff** (default 15%)
- ✅ Multi-agent prioritization (High / Normal / Low)
- ✅ Claude Code + Codex + Antigravity detection
- ✅ Hook support (snippets for Claude Code & Codex)

## Quick Start

```bash
git clone https://github.com/vinyaspoojary318-cloud/Agent-awake.git
cd Agent-awake
cp .env.example .env
# Add your TELOXIDE_TOKEN from @BotFather
cargo run
```

Release build:
```bash
cargo build --release
# → target/release/agent-awake.exe
```

## System Tray

When the app runs you get a tray icon:

- **Green** = keep-awake is ON
- **Gray** = normal sleep behavior

Right-click menu:

- Keep Awake
- Allow Sleep
- Refresh Status
- Quit Agent-awake

## Configuration

Auto-created (usually `%APPDATA%\agent-awake\config.toml`):

```toml
battery_cutoff = 15
low_priority_min_battery = 30
override_lid_close = true
restore_lid_on_exit = true
monitor_interval_secs = 20
```

Env overrides: `BATTERY_CUTOFF`, `LOW_PRIORITY_MIN_BATTERY`

## Telegram Commands

| Command    | Description                     |
|------------|---------------------------------|
| `/status`  | Status + priorities + lid       |
| `/awake`   | Force keep awake + lid override |
| `/sleep`   | Restore normal sleep + lid      |
| `/battery` | Battery details                 |
| `/lid`     | Lid-close override status       |
| `/config`  | Show current configuration      |
| `/help`    | Help                            |

## Claude Code & Codex Hooks

On startup, snippets are written to `%TEMP%\agent-awake\`:

- `claude-hooks-snippet.json` → merge into `~/.claude/settings.json`
- `codex-hooks-snippet.json` → merge into `~/.codex/hooks.json`

Hooks create `.active` markers so detection is more accurate than process names alone.

## Priority System

| Priority | Agents                             | Behavior                         |
|----------|------------------------------------|----------------------------------|
| High     | Claude, Codex, Antigravity, Cursor | Prefer keep-awake                |
| Normal   | Ollama, Aider, Hermes, Goose…      | Keep awake                       |
| Low      | Open WebUI, ComfyUI, SillyTavern…  | Only if battery above threshold  |

## License

MIT
