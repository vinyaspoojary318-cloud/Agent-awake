# Agent-awake

**Keep your AI agents running even when you close the laptop lid.**  
Telegram remote control • Multi-agent prioritization • Lid-close override • Configurable safety

## Features (v0.2)

- ✅ Keep Windows awake while AI agents work
- ✅ **Lid-close → Do nothing** (automatic while agents run)
- ✅ Telegram bot remote control
- ✅ **Configurable battery cutoff** (default 15%)
- ✅ Multi-agent prioritization (High / Normal / Low)
- ✅ Claude Code + Codex + Antigravity detection
- ✅ Hook support (snippets generated for Claude Code & Codex)
- ⏳ System tray icon (foundation added, polish coming)

## Quick Start

```bash
git clone https://github.com/vinyaspoojary318-cloud/Agent-awake.git
cd Agent-awake
cp .env.example .env
# Add your TELOXIDE_TOKEN
cargo run
```

## Configuration

A `config.toml` is created automatically (usually under `%APPDATA%\agent-awake\config.toml`).

```toml
battery_cutoff = 15
low_priority_min_battery = 30
override_lid_close = true
restore_lid_on_exit = true
monitor_interval_secs = 20
```

You can also set `BATTERY_CUTOFF` and `LOW_PRIORITY_MIN_BATTERY` in `.env`.

## Telegram Commands

| Command    | Description                        |
|------------|------------------------------------|
| `/status`  | Full status + priorities + lid     |
| `/awake`   | Force keep awake + lid override    |
| `/sleep`   | Restore normal sleep + lid         |
| `/battery` | Battery details                    |
| `/lid`     | Lid-close override status          |
| `/config`  | Show current configuration         |
| `/help`    | Help                               |

## Claude Code & Codex Hooks

On startup Agent-awake writes ready-to-use hook snippets to:

- `%TEMP%\agent-awake\claude-hooks-snippet.json`
- `%TEMP%\agent-awake\codex-hooks-snippet.json`

Merge them into:

- Claude Code → `~/.claude/settings.json`
- Codex → `~/.codex/hooks.json`

These hooks create `.active` marker files so Agent-awake knows real work is happening (more accurate than process name alone).

## Priority System

| Priority | Agents                                      | Behavior                          |
|----------|---------------------------------------------|-----------------------------------|
| High     | Claude, Codex, Antigravity, Cursor          | Always prefer keep-awake          |
| Normal   | Ollama, Aider, Hermes, Goose, Cline…        | Keep awake                        |
| Low      | Open WebUI, ComfyUI, SillyTavern…           | Only if battery > configured min  |

## License

MIT
