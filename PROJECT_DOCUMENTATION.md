# opencode-usage-companion Project Documentation

**Project:** opencode-usage-companion  
**Command:** `ocu` (short for "OpenCode Usage")  
**Repository:** `github.com/jsribeiro/opencode-usage-companion`  
**Author:** jsribeiro  
**Created:** 2026-02-01  
**Status:** Implementation Phase  

---

## 1. Project Overview

A fast, cross-platform Rust CLI tool that queries AI provider quotas and usage by reusing existing OpenCode authentication tokens. No additional authentication or configuration needed - works immediately if OpenCode is installed and authenticated.

### Key Features
- **Zero Configuration**: Automatically detects and uses OpenCode's existing auth tokens
- **Multi-Provider Support**: Gemini/Antigravity, Codex, Copilot, Claude
- **Cross-Platform**: Works on Windows (PowerShell), macOS, and Linux
- **Multiple Output Formats**: Table (default), JSON, Simple text
- **Colored Output**: Visual indicators for quota levels (with `--no-color` option)
- **Fast**: Concurrent provider querying support
- **Standalone Binary**: Single executable, no runtime dependencies

---

## 2. Naming Decisions

**Project Name:** `opencode-usage-companion`
- Long but descriptive
- Makes it clear it's not an official OpenCode tool
- "Companion" implies it works alongside OpenCode

**Binary/Command Name:** `ocu`
- Short (3 letters), memorable, easy to type
- Reads as a verb: "OpenCode, Usage!"
- User invokes: `ocu` rather than typing a long command

---

## 3. Supported Providers

### 3.1 Gemini / Antigravity (Google)

**Type:** Quota-based (shared buckets)  
**Auth File:** `~/.config/opencode/antigravity-accounts.json` (macOS/Linux), `%APPDATA%/opencode/antigravity-accounts.json` (Windows)  
**Multi-Account:** Yes, supports multiple Google accounts  
**API Endpoints:** `cloudcode-pa.googleapis.com` (production)  

**Implementation Flow:**
1. Read `antigravity-accounts.json` from multiple possible locations
2. For each account:
   - Extract `refreshToken`, `email`, `projectId`/`managedProjectId`
   - POST to `https://oauth2.googleapis.com/token` to refresh access token
   - POST to `https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist` (if projectId not available)
   - POST to `https://cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels` to get quota info
3. Parse response for shared quota buckets

**Required Headers:**
```
User-Agent: antigravity/1.15.8 {platform}
X-Goog-Api-Client: google-cloud-sdk vscode_cloudshelleditor/0.1
Content-Type: application/json
```

**API Response Structure (fetchAvailableModels):**
```json
{
  "models": {
    "MODEL_CLAUDE_4_5_SONNET": {
      "displayName": "Claude Sonnet 4.5 (no thinking)",
      "model": "MODEL_CLAUDE_4_5_SONNET",
      "quotaInfo": {
        "remainingFraction": 1.0,
        "resetTime": "2026-02-08T17:11:17Z"
      }
    }
  }
}
```

**OAuth Credentials (Public - from opencode-antigravity-auth):**
```
Client ID: 1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com
Client Secret: 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
```

**Display:**
- Shows email for each account
- Lists bucket remaining percentages
- Shows reset time for all models
- Filters out chat_, rev19, and gemini 2.5 models

**Known Models (Internal Grouping):**
- `Claude Models` - Includes Claude Sonnet, Opus, etc.
- `Gemini Flash` - Gemini 1.5/2.0 Flash models
- `Gemini 3 Pro` - Gemini 3 Pro models
- `Gemini 3 Pro Image` - Gemini 3 Pro Image models

---

### 3.2 Codex (OpenAI)

**Type:** Quota-based (rate limit windows)  
**Auth File:** `~/.local/share/opencode/auth.json`  
**Multi-Account:** No  

**Implementation Flow:**
1. Read `auth.json` → extract `.openai.access` and `.openai.accountId`
2. GET `https://chatgpt.com/backend-api/wham/usage`
3. Headers:
   - `Authorization: Bearer {access_token}`
   - `ChatGPT-Account-Id: {account_id}`

**API Response Structure:**
```json
{
  "plan_type": "pro",
  "rate_limit": {
    "primary_window": {
      "used_percent": 9,
      "reset_after_seconds": 7252
    },
    "secondary_window": {
      "used_percent": 3,
      "reset_after_seconds": 265266
    }
  },
  "credits": {
    "balance": "0",
    "unlimited": false
  }
}
```

**Display:**
- Plan type (pro/free)
- Primary window: used % and reset timer
- Secondary window: used % and reset timer

---

### 3.3 Copilot (GitHub)

**Type:** Quota-based with overage  
**Auth File:** `~/.local/share/opencode/auth.json`  
**Multi-Account:** No  

**Implementation Flow:**
1. Read `auth.json` → extract `.github-copilot.access`
2. GET `https://api.github.com/copilot_internal/user`
3. Headers:
   - `Authorization: token {access_token}`
   - `Accept: application/json`
   - `Editor-Version: vscode/1.96.2`
   - `X-Github-Api-Version: 2025-04-01`

**API Response Structure:**
```json
{
  "copilot_plan": "individual_pro",
  "quota_reset_date": "2026-02-01",
  "quota_snapshots": {
    "chat": {
      "entitlement": -1,
      "remaining": -1
    },
    "completions": {
      "entitlement": -1,
      "remaining": -1
    },
    "premium_interactions": {
      "entitlement": 1500,
      "remaining": -3821,
      "overage_permitted": true,
      "overage_count": 3821
    }
  }
}
```

**Key Points:**
- Negative `remaining` = overage used
- `overage_permitted` = can continue using (may cost extra)
- `overage_count` = number of premium overage requests
- `entitlement` = monthly quota limit
- Chat and completions show -1 (unlimited on most plans)

**Display:**
- Plan type
- Premium: used/entitlement (e.g., "5321/1500")
- Overage request count when available
- Reset date
- Warning symbol (⚠️) if over quota

---

### 3.4 Claude (Anthropic)

**Type:** Quota-based (rolling windows)  
**Auth File:** `~/.local/share/opencode/auth.json`  
**Multi-Account:** No  

**Implementation Flow:**
1. Read `auth.json` → extract `.anthropic.access`
2. GET `https://api.anthropic.com/api/oauth/usage`
3. Headers:
   - `Authorization: Bearer {access_token}`
   - `anthropic-beta: oauth-2025-04-20`

**API Response Structure:**
```json
{
  "five_hour": {
    "utilization": 23.0,
    "resets_at": "2026-01-29T20:00:00Z"
  },
  "seven_day": {
    "utilization": 4.0,
    "resets_at": "2026-02-05T15:00:00Z"
  },
  "seven_day_sonnet": {
    "utilization": 0.0,
    "resets_at": null
  },
  "seven_day_opus": null,
  "extra_usage": {
    "is_enabled": false
  }
}
```

**Display:**
- 5-hour window usage % and reset time
- 7-day window usage % and reset time
- Optional: Sonnet/Opus breakdown if available

---

## 4. Authentication & Token Files

### 4.1 File Locations (Cross-Platform)

All platforms use the same relative paths from home directory:

**OpenCode Auth:**
- Windows: `%USERPROFILE%\.local\share\opencode\auth.json`
- macOS: `~/.local/share/opencode/auth.json`
- Linux: `~/.local/share/opencode/auth.json`

**Antigravity Accounts:**
- Windows: `%APPDATA%\opencode\antigravity-accounts.json` (primary), `%USERPROFILE%\.config\opencode\antigravity-accounts.json` (fallback)
- macOS: `~/.config/opencode/antigravity-accounts.json`
- Linux: `~/.config/opencode/antigravity-accounts.json` or `~/.local/share/opencode/antigravity-accounts.json`

**Note:** On Windows, the `opencode-antigravity-auth` plugin stores the accounts file in `%APPDATA%\opencode\` (e.g., `C:\Users\<user>\AppData\Roaming\opencode\`), not in `.config/opencode/`.

### 4.2 OpenCode Auth Structure

```json
{
  "anthropic": {
    "type": "oauth",
    "access": "sk-ant-oat01-...",
    "refresh": "sk-ant-ort01-...",
    "expires": 1769729563641
  },
  "openai": {
    "type": "oauth",
    "access": "eyJ...",
    "refresh": "rt_...",
    "expires": 1770563557150,
    "accountId": "uuid"
  },
  "github-copilot": {
    "type": "oauth",
    "access": "gho_...",
    "refresh": "gho_...",
    "expires": 0
  }
}
```

### 4.3 Antigravity Accounts Structure

```json
{
  "version": 3,
  "accounts": [
    {
      "email": "user@example.com",
      "refreshToken": "1//03dATCZ...",
      "projectId": "sage-brace-7bc5s",
      "managedProjectId": "sage-brace-7bc5s",
      "addedAt": 1769185240016,
      "lastUsed": 1769962739706,
      "rateLimitResetTimes": {
        "claude": 1769599586092.3398,
        "gemini-antigravity:antigravity-gemini-3-pro": 1769203099686,
        "gemini-cli:gemini-3-flash-preview": 1769700023092
      },
      "fingerprint": {
        "deviceId": "fc912446-c1e0-4ab9-931d-d8f30ab9fc71",
        "sessionToken": "76fa62d108d3b48ee652e03976b2d351",
        "userAgent": "antigravity/1.15.8 win32/arm64",
        "apiClient": "google-cloud-sdk android-studio/2024.1",
        "clientMetadata": {
          "ideType": "IDE_UNSPECIFIED",
          "platform": "WINDOWS",
          "pluginType": "GEMINI"
        }
      }
    }
  ],
  "activeIndex": 0,
  "activeIndexByFamily": {
    "claude": 0,
    "gemini": 0
  }
}
```

**Important Note on Data Types:**
The `rateLimitResetTimes` field contains timestamps that may be floating-point numbers (e.g., `1769599586092.3398`), not integers. This is because the timestamps include millisecond precision. The parser must handle `f64` values, not `i64`.

---

## 5. CLI Interface

### 5.1 Command Syntax

```bash
ocu [OPTIONS]
```

### 5.2 Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--provider` | `-p` | Provider(s) to check [gemini, codex, copilot, claude, all] | all |
| `--format` | `-f` | Output format [table, json, simple] | table |
| `--timeout` | `-t` | Timeout per provider in seconds | 10 |
| `--concurrent` | `-c` | Query providers concurrently | false (sequential) |
| `--no-color` | | Disable colored output | false (colors enabled) |
| `--help` | `-h` | Print help | |
| `--version` | `-V` | Print version | |

### 5.3 Usage Examples

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
ocu -f table --no-color

# Faster execution with concurrent queries
ocu -c

# Custom timeout
ocu -t 5

# Combined options
ocu -p gemini -p codex -f json -c
```

---

## 6. Output Formats

### 6.1 Table Format (Default)

Uses `tabled` crate (0.20.0) with proper colorization and cell spanning.

**Features:**
- **Rounded corners** for modern appearance
- **Double horizontal line** after header for visual separation
- **Cell spanning**: Provider names and Status span multiple rows for providers with multiple entries
- **Native tabled colors**: No ANSI escape codes in content (prevents width misalignment)
- **Blue header row** with white text
- **Status icons**: ✓ OK, ⚠️ WARNING, ✗ ERROR

**Example:**
```
╭─────────────────────┬──────────────────┬───────┬────────┬────────╮
│ Provider            │ Model            │ Usage │ Resets │ Status │
╞═════════════════════╪══════════════════╪═══════╪════════╪════════╡
│ Gemini              │ Claude Models    │ 0%    │ 6d     │ ✓ OK   │
│ jsribeiro@gmail.com │ Gemini Flash     │ 0%    │ 6d     │ ✓ OK   │
│                     │ Gemini 3 Pro     │ 0%    │ 6d     │ ✓ OK   │
├─────────────────────┼──────────────────┼───────┼────────┼────────┤
│ Codex               │ Primary          │ 9%    │ 2h 1m  │ ✓ OK   │
│                     │ Secondary        │ 3%    │ 73h 41m│ ✓ OK   │
├─────────────────────┼──────────────────┼───────┼────────┼────────┤
│ Claude              │ 5h Window        │ 23%   │ 4h 30m │ ✓ OK   │
│                     │ 7d Window        │ 4%    │ 5d     │ ✓ OK   │
╰─────────────────────┴──────────────────┴───────┴────────┴────────╯
```

**Color Coding:**
- **Green** (< 50% usage): Healthy quota
- **Yellow** (50-80% usage): Getting full
- **Red** (> 80% usage): Nearly exhausted
- **Red + Warning Icon** (overage): Over quota
- **Blue background**: Header row
- **White text**: Header row text

**Important Note on Gemini/Antigravity:**
The API reports percentage as "remaining" (e.g., 100% = full quota available), but the tool displays it as "used" percentage (0% = all quota available, 100% = quota exhausted). This is inverted to be consistent with other providers that report usage/utilization percentages.

### 6.2 JSON Format

Machine-readable, useful for scripting and CI/CD.

**Structure:**
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
            {
              "model": "Gemini Flash",
              "remaining_percent": 100.0,
              "reset_time": "2026-02-01T17:05:02Z"
            },
            {
              "model": "Gemini 3 Pro",
              "remaining_percent": 85.0,
              "reset_time": "2026-02-01T17:05:02Z"
            }
          ]
        }
      ]
    },
    {
      "type": "codex",
      "plan": "pro",
      "primary_window": {
        "used_percent": 9,
        "resets_in_seconds": 7252
      },
      "secondary_window": {
        "used_percent": 3,
        "resets_in_seconds": 265266
      }
    },
    {
      "type": "copilot",
      "plan": "individual_pro",
      "premium_remaining": -3821,
      "premium_entitlement": 1500,
      "overage_permitted": true,
      "overage_count": 3821,
      "quota_reset_date": "2026-02-01"
    },
    {
      "type": "claude",
      "five_hour": {
        "utilization": 23.0,
        "resets_at": "2026-01-29T20:00:00Z"
      },
      "seven_day": {
        "utilization": 4.0,
        "resets_at": "2026-02-05T15:00:00Z"
      },
      "seven_day_sonnet": null,
      "seven_day_opus": null,
      "extra_usage_enabled": false
    }
  ]
}
```

### 6.3 Simple Format

Minimal text output, one line per provider.

**Example:**
```
Gemini (user@example.com): Claude Models:0%, Gemini Flash:0% - resets in 6d
Gemini (user2@example.com) [inactive]: Gemini 3 Pro:90%, Gemini Flash:50% - resets in 1h 30m
Codex: primary:9%, secondary:3% - primary resets in 1h 52m
Copilot: used 5321/1500 (overage permitted) - resets Feb 1
Claude: 5h:23%, 7d:4% - 5h resets in 2h 15m
```

---

## 7. Exit Codes

| Code | Meaning | When Used |
|------|---------|-----------|
| `0` | Success | All requested providers queried successfully, data displayed |
| `1` | Error | Network failure, API error, parse error, unexpected exception |
| `2` | No Providers | Auth files exist but no relevant tokens found for requested providers |

**Design Decision:** Exit code 0 even if quotas are high/overage. Use JSON output for scripting/quota checking logic. Non-zero only for actual errors.

---

## 8. Architecture

### 8.1 Project Structure

```
opencode-usage-companion/
├── Cargo.toml                  # Package config, binary name = "ocu"
├── build.rs                    # Build script (optional)
├── src/
│   ├── main.rs                 # Entry point
│   ├── cli.rs                  # CLI arguments (clap)
│   ├── error.rs                # Error types and handling
│   ├── auth.rs                 # Token loading from OpenCode files
│   ├── lib.rs                  # Public library exports
│   ├── providers/              # Provider implementations
│   │   ├── mod.rs              # Provider trait definition
│   │   ├── gemini.rs           # Google Gemini/Antigravity
│   │   ├── codex.rs            # OpenAI Codex
│   │   ├── copilot.rs          # GitHub Copilot
│   │   └── claude.rs           # Anthropic Claude
│   └── output/                 # Output formatters
│       ├── mod.rs              # Output trait
│       ├── table.rs            # Table output (tabled 0.20)
│       ├── json.rs             # JSON output
│       └── simple.rs           # Simple text output
└── .github/
    └── workflows/
        └── release.yml         # Automated multi-platform builds
```

### 8.2 Key Modules

**`cli.rs`** - CLI argument parsing
```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "ocu")]
#[command(about = "OpenCode Usage Companion - Check AI provider quotas")]
pub struct Args {
    #[arg(short, long, value_enum)]
    pub provider: Vec<Provider>,
    
    #[arg(short, long, value_enum, default_value = "table")]
    pub format: OutputFormat,
    
    #[arg(short, long, default_value = "10")]
    pub timeout: u64,
    
    #[arg(short, long)]
    pub concurrent: bool,
    
    #[arg(long)]
    pub no_color: bool,
}
```

**`providers/mod.rs`** - Provider trait
```rust
#[async_trait::async_trait]
pub trait Provider {
    fn name(&self) -> &'static str;
    async fn fetch(&self, timeout: Duration) -> Result<ProviderData, QuotaError>;
    fn is_configured(&self) -> bool;
}

pub enum ProviderData {
    Gemini(GeminiData),
    Codex(CodexData),
    Copilot(CopilotData),
    Claude(ClaudeData),
}
```

**`auth.rs`** - Token loading
```rust
pub struct AuthManager;

impl AuthManager {
    pub fn read_opencode_auth() -> Result<OpenCodeAuth, AuthError>;
    pub fn read_antigravity_accounts() -> Result<AntigravityAccounts, AuthError>;
    pub fn get_provider_tokens(&self) -> Vec<ProviderToken>;
}
```

**`error.rs`** - Error types
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuotaError {
    #[error("Authentication file not found: {0}")]
    AuthFileNotFound(String),
    
    #[error("Provider not configured: {0}")]
    ProviderNotConfigured(String),
    
    #[error("API request failed: {0}")]
    ApiError(String),
    
    #[error("Token refresh failed: {0}")]
    TokenRefreshError(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),
}
```

---

## 9. Dependencies

```toml
[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# HTTP client
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# CLI parsing
clap = { version = "4.5", features = ["derive"] }

# Table formatting
tabled = "0.20"

# Terminal colors (for simple/json output, not table cells)
colored = "2.1"

# Date/time handling
chrono = { version = "0.4", features = ["serde", "clock"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Cross-platform paths
dirs = "5.0"

# Async trait support
async-trait = "0.1"
```

---

## 10. Error Handling Strategy

### 10.1 Graceful Degradation

The tool should never fail completely. If one provider errors, continue with others:

```rust
// Pseudocode
let mut results = Vec::new();
let mut errors = Vec::new();

for provider in providers {
    match provider.fetch(timeout).await {
        Ok(data) => results.push(data),
        Err(e) => {
            eprintln!("Warning: {} failed: {}", provider.name(), e);
            errors.push((provider.name(), e));
        }
    }
}

if results.is_empty() {
    // All failed - exit with error
    exit(1);
} else if !errors.is_empty() {
    // Some failed - show warning but exit 0
    display_results(results);
    eprintln!("\nNote: {} provider(s) failed to respond", errors.len());
    exit(0);
} else {
    // All succeeded
    display_results(results);
    exit(0);
}
```

### 10.2 Common Error Scenarios

| Scenario | User Message | Exit Code |
|----------|--------------|-----------|
| Auth file not found | "Warning: OpenCode auth file not found at {path}. Skipping {provider}." | 0 (if other providers work) |
| Token expired/invalid | "Warning: {provider} authentication failed. Token may be expired." | 0 |
| Network timeout | "Warning: {provider} request timed out after {timeout}s." | 0 |
| No providers configured | "Error: No AI providers configured. Please authenticate with OpenCode first." | 2 |
| All providers failed | "Error: All provider queries failed." | 1 |

---

## 11. Development Phases

### Phase 1: Foundation (Week 1)
- [x] Initialize Cargo project
- [x] Set up CLI argument parsing (clap)
- [x] Create error types and handling
- [x] Implement auth file reading (with Windows path support)
- [x] Define provider trait

### Phase 2: Providers (Week 2)
- [x] Implement Gemini provider with OAuth refresh (Antigravity API)
- [x] Implement Codex provider
- [x] Implement Copilot provider
- [x] Implement Claude provider
- [x] Add concurrent fetching support
- [x] Fix auth detection to check multiple sources independently

### Phase 3: Output (Week 3)
- [x] Implement table output with comfy-table
- [x] Implement JSON output
- [x] Implement simple output
- [x] Add colored output support
- [x] Add --no-color flag

### Phase 4: Testing & Polish (Week 4) - COMPLETED
- [x] Test on Windows (PowerShell) - **Claude and Antigravity working**
- [ ] Test on macOS
- [ ] Test on Linux
- [x] Error handling edge cases (graceful degradation, independent auth checks)
- [x] Fix Antigravity API integration (correct endpoints, headers, JSON parsing)
- [x] Optimize concurrent performance

### Phase 5: Release (Week 5)
- [x] Write comprehensive README
- [ ] Create GitHub Actions workflow
- [ ] Set up multi-platform builds
- [ ] Create GitHub release
- [x] Write installation instructions

---

## 12. Future Enhancements (Not in MVP)

These are ideas for future versions, NOT part of initial implementation:

1. **Watch Mode**: `ocu --watch 30s` to auto-refresh every 30 seconds
2. **TUI Mode**: Interactive terminal UI using `ratatui` crate
3. **Caching**: Cache results to avoid API rate limits on frequent runs
4. **Additional Providers**:
   - OpenRouter (pay-as-you-go)
   - Kimi (quota-based)
   - OpenCode Zen (local CLI stats)
5. **Configuration File**: `~/.config/ocu/config.toml` for default settings
6. **Notifications**: Desktop notifications when quota reaches threshold
7. **History Tracking**: Store historical usage data locally
8. **Predictions**: Estimate when quotas will run out based on usage patterns

---

## 13. Reference Implementations

The following reference implementations were used to understand provider APIs:

1. **opencode-bar** (macOS menubar app)
   - Location: `C:\dev\projects\opencode-bar`
   - Language: Swift
   - Shows: API endpoints, auth patterns, token refresh logic

2. **Shell Scripts** (in opencode-bar)
   - `query-gemini-cli.sh` - Google OAuth refresh flow
   - `query-codex.sh` - OpenAI API with account ID header
   - `query-copilot.sh` - GitHub Copilot internal API
   - `query-claude.sh` - Anthropic usage endpoint
   - `query-antigravity-local.sh` - Language server detection (NOT used - we use cloud API instead)

3. **opencode-antigravity-quota** (OpenCode Plugin)
   - Location: `C:\dev\projects\opencode-antigravity-quota`
   - Language: TypeScript
   - **Critical Reference**: This is the official implementation used by OpenCode
   - Shows:
     - Correct API endpoints: `v1internal:loadCodeAssist` and `v1internal:fetchAvailableModels`
     - Proper headers: User-Agent, X-Goog-Api-Client, Client-Metadata
     - Windows auth file location: `%APPDATA%/opencode/antigravity-accounts.json`
     - OAuth credentials and token refresh flow
     - Response parsing and model filtering logic

4. **opencode-antigravity-auth** (npm package)
   - URL: https://unpkg.com/opencode-antigravity-auth@latest/dist/src/constants.js
   - **Critical Reference**: Contains OAuth credentials and API constants
   - Shows:
     - Client ID: `1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com`
     - Client Secret: `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`
     - API endpoints and version strings
     - Header configurations

5. **antigravity-usage** (GitHub)
   - Repository: https://github.com/skainguyen1412/antigravity-usage
   - Shows: Alternative implementation approach (separate auth)
   - Note: We use opencode-antigravity plugin auth instead

---

## 14. Build & Release

### 14.1 Local Build

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run
cargo run -- --help
./target/release/ocu
```

### 14.2 Cross-Compilation Targets

```bash
# Windows x64
rustup target add x86_64-pc-windows-msvc

# macOS Intel
rustup target add x86_64-apple-darwin

# macOS Apple Silicon  
rustup target add aarch64-apple-darwin

# Linux x64
rustup target add x86_64-unknown-linux-gnu
```

### 14.3 GitHub Actions Workflow

Create `.github/workflows/release.yml`:
- Trigger on version tags (e.g., `v1.0.0`)
- Build for all 4 targets
- Create GitHub release
- Attach binaries with checksums

---

## 15. Implementation Notes

### 15.1 Design Decisions Made

1. **Language**: Rust (learning opportunity, fast binaries, cross-platform)
2. **Async Runtime**: Tokio (industry standard)
3. **HTTP Client**: reqwest with rustls-tls (no OpenSSL dependency)
4. **CLI Parser**: clap with derive macro (ergonomic, typed)
5. **Table Output**: comfy-table (simple, flexible)
6. **Colors**: colored crate (cross-platform, easy to disable)
7. **Errors**: anyhow + thiserror (ergonomic error handling)

### 15.2 Code Style

- Use `snake_case` for functions and variables
- Use `PascalCase` for types and traits
- Use `SCREAMING_SNAKE_CASE` for constants
- Async/await for all I/O operations
- Result types for error handling (no unwrap/expect in production code)
- Comprehensive error messages for users

### 15.3 Testing Strategy

- Unit tests for token parsing
- Unit tests for API response parsing
- Mock HTTP responses for testing
- Integration tests with real auth (manual)

### 15.4 Technical Challenges & Solutions

**Challenge 1: Table Cell Colorization**
- **Problem**: Using ANSI escape codes (from `colored` crate) in table cell content caused misaligned columns
- **Root Cause**: tabled counts ANSI escape sequences as visible characters when calculating column widths
- **Solution**: Use tabled's native `Color` settings via `table.modify((row, col), Color::FG_GREEN)` instead
- **Implementation**: Track colors in a separate vector while building rows, then apply them after table construction
- **Benefit**: Perfect alignment with colored cells

**Challenge 2: Antigravity API Endpoints**
- **Problem**: Initial implementation used `v1internal:retrieveUserQuota` endpoint, which doesn't work with the `opencode-antigravity-auth` plugin
- **Solution**: Studied the `opencode-antigravity-quota` plugin source code and discovered the correct flow:
  1. Call `v1internal:loadCodeAssist` to get project ID (if not in accounts file)
  2. Call `v1internal:fetchAvailableModels` to get quota information
- **Key Learning**: The plugin uses different endpoints than standalone Gemini CLI

**Challenge 2: Windows Auth File Paths**
- **Problem**: Antigravity accounts file on Windows is NOT in `~/.config/opencode/`
- **Solution**: The plugin stores it in `%APPDATA%/opencode/` (e.g., `C:\Users\<user>\AppData\Roaming\opencode\`)
- **Implementation**: Added multiple path checks using `dirs::data_dir()` for Windows

**Challenge 3: Auth Detection Logic**
- **Problem**: `is_provider_configured()` was failing early if `read_opencode_auth()` errored, before checking antigravity accounts
- **Root Cause**: Using `?` operator meant any error in one auth source prevented checking others
- **Solution**: Changed to `.ok().flatten()` to handle each auth source independently
- **Impact**: This was why Antigravity wasn't being detected even though the file existed

**Challenge 4: JSON Float Parsing**
- **Problem**: `rateLimitResetTimes` contains timestamps like `1769599586092.3398` (with decimal)
- **Error**: Serde couldn't parse `f64` into `i64` field
- **Solution**: Changed field type from `HashMap<String, i64>` to `HashMap<String, f64>`

**Challenge 5: API Headers**
- **Problem**: Requests were being rejected without proper headers
- **Solution**: Added required headers from `opencode-antigravity-auth`:
  - `User-Agent: antigravity/1.15.8 {platform}`
  - `X-Goog-Api-Client: google-cloud-sdk vscode_cloudshelleditor/0.1`

---

## 16. Changelog

### 2026-02-01 - Table Improvements & Colorization Fix
- **Fixed Table Width Issues:**
  - Switched from ANSI escape codes to tabled's native `Color` settings
  - Implemented color tracking during row building
  - Applied colors via `table.modify()` after table construction
  - All columns now properly aligned with colored content
- **Enhanced Table Styling:**
  - Changed to `Style::rounded()` for modern rounded corners
  - Added double horizontal line (═) after header row
  - Implemented cell spanning for Provider and Status columns
  - Blue header row with white text background
- **Added Status Icons:**
  - ✓ OK (green)
  - ⚠️ WARNING (yellow)
  - ✗ ERROR (red)
- **Fixed Gemini Usage Display:**
  - Inverted percentage to show % used instead of % remaining
  - Now consistent with other providers (0% = all free, 100% = exhausted)
  - Uses `get_usage_color()` (green for low usage, red for high)
- **Updated Dependencies:**
  - Switched from `comfy-table` to `tabled` (0.20.0)
  - tabled provides better cell spanning and native color support

### 2026-02-01 - Final Implementation & Fixes
- **Fixed Antigravity Provider:**
  - Implemented correct API flow: `loadCodeAssist` → `fetchAvailableModels`
  - Added proper headers (User-Agent, X-Goog-Api-Client) matching TypeScript plugin
  - Fixed Windows auth file path detection: `%APPDATA%/opencode/antigravity-accounts.json`
  - Fixed JSON parsing for `f64` values in `rateLimitResetTimes`
  - Added OAuth credentials from `opencode-antigravity-auth` npm package
- **Fixed Auth Detection:**
  - Made auth checks independent so one failure doesn't block others
  - Changed from `?` operator to `.ok().flatten()` for graceful handling
  - This fixed the "No AI providers configured" error when Antigravity was working
- **Verified Working Providers:**
  - ✅ **Claude**: 5h/7d windows working correctly
  - ✅ **Antigravity**: All models showing quota (Claude 4.5, Gemini 3, GPT-OSS 120B)
- **Both providers now fully functional on Windows**

### 2026-02-01 - Project Inception
- Initial planning session with user
- Decided on project name: `opencode-usage-companion`
- Decided on command name: `ocu`
- Selected 4 initial providers: Gemini, Codex, Copilot, Claude
- Finalized CLI interface and output formats
- Created this comprehensive documentation

---

## Appendix A: Example Auth Files

### OpenCode Auth (`~/.local/share/opencode/auth.json`)

```json
{
  "google": {
    "type": "oauth",
    "refresh": "1//03alCl_poc9PdCgYIARAAGAMSNwF...",
    "access": "ya29.a0AUMWg_Iz0rLDtHQfXQZEpj-kIjNO2LeXx1-Ao6ln...",
    "expires": 1769188838030
  },
  "anthropic": {
    "type": "oauth",
    "refresh": "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",
    "access": "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",
    "expires": 1769247466529
  },
  "openai": {
    "type": "oauth",
    "access": "eyJ...",
    "refresh": "rt_...",
    "expires": 1770563557150,
    "accountId": "uuid"
  },
  "github-copilot": {
    "type": "oauth",
    "access": "gho_...",
    "refresh": "gho_...",
    "expires": 0
  }
}
```

### Antigravity Accounts

**macOS/Linux:** `~/.config/opencode/antigravity-accounts.json`

**Windows:** `%APPDATA%/opencode/antigravity-accounts.json`
(e.g., `C:\Users\jsribeiro\AppData\Roaming\opencode\antigravity-accounts.json`)

```json
{
  "version": 3,
  "accounts": [
    {
      "email": "jsribeiro@gmail.com",
      "refreshToken": "1//03alCl_poc9PdCgYIARAAGAMSNwF...",
      "projectId": "sage-brace-7bc5s",
      "managedProjectId": "sage-brace-7bc5s",
      "addedAt": 1769185240016,
      "lastUsed": 1769962739706,
      "rateLimitResetTimes": {
        "claude": 1769599586092.3398,
        "gemini-antigravity:antigravity-gemini-3-pro": 1769203099686
      },
      "fingerprint": {
        "deviceId": "fc912446-c1e0-4ab9-931d-d8f30ab9fc71",
        "sessionToken": "76fa62d108d3b48ee652e03976b2d351",
        "userAgent": "antigravity/1.15.8 win32/arm64",
        "apiClient": "google-cloud-sdk android-studio/2024.1",
        "clientMetadata": {
          "ideType": "IDE_UNSPECIFIED",
          "platform": "WINDOWS",
          "pluginType": "GEMINI"
        }
      }
    }
  ],
  "activeIndex": 0,
  "activeIndexByFamily": {
    "claude": 0,
    "gemini": 0
  }
}
```

---

**End of Documentation**

*This document serves as the authoritative source for project specifications. All implementation should align with these specifications.*
