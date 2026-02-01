use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{QuotaError, Result};

/// OpenCode Auth structure for ~/.local/share/opencode/auth.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenCodeAuth {
    #[serde(rename = "google")]
    pub google: Option<OAuthToken>,
    #[serde(rename = "anthropic")]
    pub anthropic: Option<OAuthToken>,
    #[serde(rename = "openai")]
    pub openai: Option<OAuthToken>,
    #[serde(rename = "github-copilot")]
    pub github_copilot: Option<OAuthToken>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuthToken {
    #[serde(rename = "type")]
    pub token_type: String,
    pub access: String,
    #[serde(default)]
    pub refresh: Option<String>,
    #[serde(default)]
    pub expires: Option<i64>,
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
}

/// Antigravity Accounts structure for antigravity-accounts.json
/// On Windows: %APPDATA%/opencode/antigravity-accounts.json
/// On macOS/Linux: ~/.config/opencode/antigravity-accounts.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AntigravityAccounts {
    pub version: i32,
    pub accounts: Vec<AntigravityAccount>,
    #[serde(rename = "activeIndex")]
    pub active_index: usize,
    #[serde(rename = "activeIndexByFamily")]
    pub active_index_by_family: Option<std::collections::HashMap<String, usize>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AntigravityAccount {
    pub email: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
    #[serde(rename = "managedProjectId")]
    pub managed_project_id: Option<String>,
    #[serde(rename = "rateLimitResetTimes")]
    pub rate_limit_reset_times: Option<std::collections::HashMap<String, f64>>,
    #[serde(rename = "addedAt")]
    pub added_at: Option<i64>,
    #[serde(rename = "lastUsed")]
    pub last_used: Option<i64>,
    pub fingerprint: Option<serde_json::Value>,
}

/// Gemini OAuth token response
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiTokenResponse {
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "expires_in")]
    pub expires_in: i32,
    #[serde(rename = "token_type")]
    pub token_type: Option<String>,
}

pub struct AuthManager;

impl AuthManager {
    pub fn new() -> Self {
        Self
    }

    /// Get path to OpenCode auth file
    fn get_opencode_auth_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| QuotaError::AuthFileNotFound("Could not find home directory".to_string()))?;
        Ok(home.join(".local").join("share").join("opencode").join("auth.json"))
    }

    /// Get possible paths to Antigravity accounts file
    /// Tries multiple locations for cross-platform support
    fn get_antigravity_accounts_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        if let Some(home) = dirs::home_dir() {
            // Windows: %APPDATA%/opencode/antigravity-accounts.json
            if let Some(app_data) = dirs::data_dir() {
                paths.push(app_data.join("opencode").join("antigravity-accounts.json"));
            }
            
            // Windows/Linux: ~/.config/opencode/antigravity-accounts.json
            paths.push(home.join(".config").join("opencode").join("antigravity-accounts.json"));
            
            // Linux: ~/.local/share/opencode/antigravity-accounts.json
            paths.push(home.join(".local").join("share").join("opencode").join("antigravity-accounts.json"));
        }
        
        paths
    }

    /// Read OpenCode auth file
    pub fn read_opencode_auth(&self) -> Result<Option<OpenCodeAuth>> {
        let path = Self::get_opencode_auth_path()?;
        
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)?;
        let auth: OpenCodeAuth = serde_json::from_str(&content)?;
        Ok(Some(auth))
    }

    /// Read Antigravity accounts file
    /// Tries multiple locations and returns the first one found
    pub fn read_antigravity_accounts(&self) -> Result<Option<AntigravityAccounts>> {
        let paths = Self::get_antigravity_accounts_paths();
        
        for path in &paths {
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                let accounts: AntigravityAccounts = serde_json::from_str(&content)?;
                return Ok(Some(accounts));
            }
        }

        Ok(None)
    }

    /// Check if a specific provider is configured
    /// Each auth method is checked independently so that if one fails, we still check the others
    pub fn is_provider_configured(&self, provider: &str) -> Result<bool> {
        // Check each auth source independently - don't fail early if one errors
        let opencode_auth = self.read_opencode_auth().ok().flatten();
        let antigravity_accounts = self.read_antigravity_accounts().ok().flatten();

        match provider {
            "gemini" => {
                // Only check for antigravity-accounts.json - Google OAuth alone is not sufficient
                // since the Gemini provider only supports Antigravity accounts
                Ok(antigravity_accounts.is_some())
            }
            "claude" => Ok(opencode_auth.as_ref().map(|a| a.anthropic.is_some()).unwrap_or(false)),
            "codex" => Ok(opencode_auth.as_ref().map(|a| a.openai.is_some()).unwrap_or(false)),
            "copilot" => Ok(opencode_auth.as_ref().map(|a| a.github_copilot.is_some()).unwrap_or(false)),
            _ => Ok(false),
        }
    }

    /// Get list of configured providers
    pub fn get_configured_providers(&self) -> Result<Vec<String>> {
        let mut providers = Vec::new();

        if self.is_provider_configured("gemini")? {
            providers.push("gemini".to_string());
        }
        if self.is_provider_configured("claude")? {
            providers.push("claude".to_string());
        }
        if self.is_provider_configured("codex")? {
            providers.push("codex".to_string());
        }
        if self.is_provider_configured("copilot")? {
            providers.push("copilot".to_string());
        }

        Ok(providers)
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}
