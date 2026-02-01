# opencode-usage-companion

**Command:** `ocu`  
**Purpose:** Check AI provider quotas using existing OpenCode authentication

## Overview

A fast, cross-platform Rust CLI tool that queries AI provider quotas and usage by reusing existing OpenCode authentication tokens. No additional authentication needed - works immediately if OpenCode is installed and authenticated.

## Features

- **Zero Configuration**: Automatically detects and uses OpenCode's existing auth tokens
- **Multi-Provider Support**: Gemini/Antigravity, Codex, Copilot, Claude
- **Cross-Platform**: Windows (PowerShell), macOS, Linux
- **Multiple Output Formats**: Table (default), JSON, Simple text
- **Colored Output**: Visual indicators for quota levels (with `--no-color` option)
- **Fast**: Concurrent provider querying support
- **Standalone Binary**: Single executable, no runtime dependencies

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
┌─────────────┬─────────────┬─────────────┬─────────────┬────────┐
│ Provider    │ Quota Info  │ Resets In   │ Status      │
├─────────────┼─────────────┼─────────────┼─────────────┼────────┤
│ Gemini      │ 2.0: 100%   │ 2h 15m      │ OK          │
│ user@email  │ 2.5 Pro:85% │             │             │
├─────────────┼─────────────┼─────────────┼─────────────┼────────┤
│ Claude      │ 5h: 23%     │ 2h 15m      │ OK          │
│             │ 7d: 4%      │             │             │
└─────────────┴─────────────┴─────────────┴─────────────┴────────┘
```

### JSON Format
```json
{
  "timestamp": "2026-02-01T14:30:00Z",
  "providers": [
    {
      "type": "gemini",
      "account": "user@example.com",
      "active": true,
      "models": [
        {"model": "gemini-2.0-flash", "remaining_percent": 100}
      ]
    }
  ]
}
```

## Development

Built with:
- Rust 1.70+
- Tokio (async runtime)
- Reqwest (HTTP client)
- Clap (CLI parsing)
- Serde (JSON serialization)

## License

MIT

## Author

jsribeiro
