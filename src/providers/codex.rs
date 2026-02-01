use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::auth::AuthManager;
use crate::error::{QuotaError, Result};
use crate::providers::{CodexData, Provider, ProviderData, WindowQuota};

pub struct CodexProvider {
    auth_manager: AuthManager,
}

impl CodexProvider {
    pub fn new() -> Self {
        Self {
            auth_manager: AuthManager::new(),
        }
    }
}

#[async_trait]
impl Provider for CodexProvider {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn is_configured(&self) -> bool {
        self.auth_manager
            .is_provider_configured("codex")
            .unwrap_or(false)
    }

    async fn fetch(&self, timeout: Duration) -> Result<ProviderData> {
        let auth = self
            .auth_manager
            .read_opencode_auth()?
            .ok_or_else(|| QuotaError::ProviderNotConfigured("codex".to_string()))?;

        let openai_auth = auth
            .openai
            .ok_or_else(|| QuotaError::ProviderNotConfigured("codex (no openai token)".to_string()))?;

        let client = Client::new();
        let mut request = client
            .get("https://chatgpt.com/backend-api/wham/usage")
            .header("Authorization", format!("Bearer {}", openai_auth.access))
            .timeout(timeout);

        // Add account ID header if available
        if let Some(account_id) = &openai_auth.account_id {
            request = request.header("ChatGPT-Account-Id", account_id);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(QuotaError::ApiError(format!(
                "Codex API error ({}): {}",
                status, error_text
            )));
        }

        let usage: CodexUsageResponse = response.json().await?;

        let data = CodexData {
            plan: usage.plan_type,
            primary_window: WindowQuota {
                used_percent: usage.rate_limit.primary_window.used_percent,
                resets_in_seconds: usage.rate_limit.primary_window.reset_after_seconds,
            },
            secondary_window: WindowQuota {
                used_percent: usage.rate_limit.secondary_window.used_percent,
                resets_in_seconds: usage.rate_limit.secondary_window.reset_after_seconds,
            },
        };

        Ok(ProviderData::Codex(data))
    }
}

impl Default for CodexProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct CodexUsageResponse {
    #[serde(rename = "plan_type")]
    plan_type: String,
    #[serde(rename = "rate_limit")]
    rate_limit: CodexRateLimit,
}

#[derive(Debug, Deserialize)]
struct CodexRateLimit {
    #[serde(rename = "primary_window")]
    primary_window: CodexWindow,
    #[serde(rename = "secondary_window")]
    secondary_window: CodexWindow,
}

#[derive(Debug, Deserialize)]
struct CodexWindow {
    #[serde(rename = "used_percent")]
    used_percent: i32,
    #[serde(rename = "reset_after_seconds")]
    reset_after_seconds: i64,
}
