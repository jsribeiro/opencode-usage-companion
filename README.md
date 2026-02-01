# opencode-usage-companion

**Command:** `ocu`
**Purpose:** Check AI provider quotas using existing OpenCode authentication

## Overview

A fast, cross-platform Rust CLI tool that queries AI provider quotas and usage by reusing existing OpenCode authentication tokens. No additional authentication needed - works immediately if OpenCode is installed and authenticated.

## Features

- **Zero Configuration**: Automatically detects and uses OpenCode's existing auth tokens
- **Multi-Provider Support**: Gemini/Antigravity, Codex, Copilot, Claude
- **Cross-Platform**: Windows, macOS, Linux
- **Multiple Output Formats**: Table (default), JSON, Simple text
- **Colored Output**: Visual indicators for quota levels (with `--no-color` option)
- **Concurrent Querying**: Optional parallel provider queries for faster results

## Installation

### Download from GitHub Releases

```bash
# Windows
curl -L -o ocu.exe https://github.com/jsribeiro/opencode-usage-companion/releases/latest/download/ocu-x86_64-pc-windows-msvc.exe

# macOS Intel
curl -L -o ocu https://github.com/jsribeiro/opencode-usage-companion/releases/latest/download/ocu-x86_64-apple-darwin
chmod +x ocu

# macOS Apple Silicon
curl -L -o ocu https://github.com/jsribeiro/opencode-usage-companion/releases/latest/download/ocu-aarch64-apple-darwin
chmod +x ocu

# Linux
curl -L -o ocu https://github.com/jsribeiro/opencode-usage-companion/releases/latest/download/ocu-x86_64-unknown-linux-gnu
chmod +x ocu
```

### Build from Source

```bash
git clone https://github.com/jsribeiro/opencode-usage-companion.git
cd opencode-usage-companion
cargo build --release
# Binary will be at: target/release/ocu
```

## Usage

```bash
# Check all configured providers (table format with colors)
ocu

# Check specific providers
ocu -p gemini
ocu -p gemini -p codex

# JSON output (useful for scripting)
ocu -f json

# Simple/minimal output
ocu -f simple

# No colors
ocu --no-color

# Concurrent queries (faster)
ocu -c

# Custom timeout
ocu -t 5
```

## CLI Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--provider` | `-p` | Provider(s) to check | all |
| `--format` | `-f` | Output format (table, json, simple) | table |
| `--timeout` | `-t` | Timeout per provider in seconds | 10 |
| `--concurrent` | `-c` | Query providers concurrently | false |
| `--no-color` | | Disable colored output | false |
| `--help` | `-h` | Print help | |
| `--version` | `-V` | Print version | |

## Supported Providers

### Gemini / Antigravity (Google)
- Multi-account support
- Per-model quotas (gemini-2.0-flash, gemini-2.5-pro, etc.)
- Auth: `~/.config/opencode/antigravity-accounts.json`

### Codex (OpenAI)
- Primary/secondary rate limit windows
- Auth: `~/.local/share/opencode/auth.json`

### Copilot (GitHub)
- Premium requests with overage tracking
- Auth: `~/.local/share/opencode/auth.json`

### Claude (Anthropic)
- 5-hour and 7-day rolling windows
- Auth: `~/.local/share/opencode/auth.json`

## Requirements

- OpenCode must be installed and authenticated with at least one provider
- No additional configuration needed

## Exit Codes

- `0`: Success (data displayed)
- `1`: Error (network, API, or parse failure)
- `2`: No providers configured

## Example Output

### Table Format

```
╭─────────────────────┬──────────────────────────────────┬───────┬────────┬────────╮
│ Provider            │ Model                            │ Usage │ Resets │ Status │
╞═════════════════════╪══════════════════════════════════╪═══════╪════════╪════════╡
│ Gemini              │ MODEL_CLAUDE_4_5_SONNET          │ 0%    │ 6d     │ ✓ OK   │
│ user@example.com    │ MODEL_CLAUDE_4_5_SONNET_THINKING │ 0%    │ 6d     │        │
│                     │ MODEL_GPT_4O                     │ 0%    │ 6d     │        │
├─────────────────────┼──────────────────────────────────┼───────┼────────┼────────┤
│ Codex               │ Primary                          │ 9%    │ 2h 1m  │ ✓ OK   │
│                     │ Secondary                        │ 3%    │ 73h 41m│        │
├─────────────────────┼──────────────────────────────────┼───────┼────────┼────────┤
│ Claude              │ 5h Window                        │ 23%   │ 4h 30m │ ✓ OK   │
│                     │ 7d Window                        │ 4%    │ 5d     │        │
╰─────────────────────┴──────────────────────────────────┴───────┴────────┴────────╯
```

- **Usage column**: Shows percentage of quota consumed (0% = all quota available, 100% = quota exhausted)
- **Color coding**: Green (healthy), yellow (warning >50%), red (critical >80%)
- **Status icons**: `✓ OK`, `⚠️ WARNING`, `✗ ERROR`

### JSON Format

```json
{
  "timestamp": "2025-01-15T14:30:00Z",
  "providers": [
    {
      "type": "gemini",
      "account_email": "user@example.com",
      "is_active": true,
      "models": [
        {"model": "gemini-2.0-flash", "remaining_percent": 100.0, "reset_time": "2025-01-22T00:00:00Z"}
      ]
    },
    {
      "type": "codex",
      "plan": "plus",
      "primary_window": {"used_percent": 9, "resets_in_seconds": 7260},
      "secondary_window": {"used_percent": 3, "resets_in_seconds": 265260}
    }
  ]
}
```

Note: JSON output uses raw API values (`remaining_percent` for Gemini, `used_percent` for others).

## Development

Built with:
- Rust 1.70+
- Tokio (async runtime)
- Reqwest (HTTP client)
- Clap (CLI parsing)
- tabled (table formatting)
- Serde (JSON serialization)

## License

MIT

## Author

jsribeiro
