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

Download the latest release from the [Releases page](https://github.com/jsribeiro/opencode-usage-companion/releases) and copy the executable to a directory on your PATH.

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
- Shared quota buckets (Gemini Flash, Gemini 3 Pro, Claude Models, etc.)
- Auth: `~/.config/opencode/antigravity-accounts.json` (macOS/Linux) or `%APPDATA%\opencode\antigravity-accounts.json` (Windows)

### Codex (OpenAI)
- Primary/secondary rate limit windows
- Auth: `~/.local/share/opencode/auth.json`

### Copilot (GitHub)
- Premium requests with overage request count
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
╭─────────────────────┬──────────────────┬───────┬────────┬────────╮
│ Provider            │ Model            │ Usage │ Resets │ Status │
╞═════════════════════╪══════════════════╪═══════╪════════╪════════╡
│ Gemini              │ Claude Models    │ 0%    │ 6d     │ ✓ OK   │
│ user@example.com    │ Gemini Flash     │ 0%    │ 6d     │ ✓ OK   │
│                     │ Gemini 3 Pro     │ 0%    │ 6d     │ ✓ OK   │
├┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┼┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┼┄┄┄┄┄┄┄┼┄┄┄┄┄┄┄┄┼┄┄┄┄┄┄┄┄┤
│ Codex               │ Primary          │ 9%    │ 2h 1m  │ ✓ OK   │
│                     │ Secondary        │ 3%    │ 73h 41m│ ✓ OK   │
├┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┼┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┼┄┄┄┄┄┄┄┼┄┄┄┄┄┄┄┄┼┄┄┄┄┄┄┄┄┤
│ Claude              │ 5h Window        │ 23%   │ 4h 30m │ ✓ OK   │
│                     │ 7d Window        │ 4%    │ 5d     │ ✓ OK   │
╰─────────────────────┴──────────────────┴───────┴────────┴────────╯
```

- **Usage column**: Shows percentage of quota consumed (0% = all quota available, 100% = quota exhausted)
- **Status column**: Per-row status based on usage (each model/window has its own indicator)
- **Color coding**: Green (healthy), yellow (warning >50%), red (critical >80%)
- **Status icons**: `✓ OK`, `⚠️ WARNING`, `✗ ERROR`

### JSON Format

```json
{
  "timestamp": "2026-02-01T14:30:00Z",
  "providers": [
    {
      "type": "gemini",
      "accounts": [
        {
          "email": "user@example.com",
          "is_active": true,
          "models": [
            {"model": "Gemini Flash", "remaining_percent": 100.0, "reset_time": "2026-02-01T00:00:00Z"}
          ]
        }
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

### Simple Format

```
Gemini (user@example.com): Claude Models: 0%, Gemini Flash: 0% - resets in 6d
Codex: primary: 9%, secondary: 3% - primary resets in 2h 1m
Copilot: used 5321/1500 (overage permitted) - resets Feb 1
Claude: 5h: 23%, 7d: 4% - 5h resets in 4h 30m
```

## Development

Built with:
- Rust 1.70+
- Tokio (async runtime)
- Reqwest (HTTP client)
- Clap (CLI parsing)
- tabled (table formatting)
- Serde (JSON serialization)

## License

GPL-3.0-only

## Author

João Sena Ribeiro <sena@smux.net>
