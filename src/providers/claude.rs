use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::auth::AuthManager;
use crate::error::{QuotaError, Result};
use crate::providers::{ClaudeData, Provider, ProviderData, WindowUsage};

pub struct ClaudeProvider {
    auth_manager: AuthManager,
}

impl ClaudeProvider {
    pub fn new() -> Self {
        Self {
            auth_manager: AuthManager::new(),
        }
    }
}

#[async_trait]
impl Provider for ClaudeProvider {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn is_configured(&self) -> bool {
        self.auth_manager
            .is_provider_configured("claude")
            .unwrap_or(false)
    }

    async fn fetch(&self, timeout: Duration, verbose: bool) -> Result<ProviderData> {
        let auth = self
            .auth_manager
            .read_opencode_auth()?
            .ok_or_else(|| QuotaError::ProviderNotConfigured("claude".to_string()))?;

        let anthropic_auth = auth
            .anthropic
            .ok_or_else(|| QuotaError::ProviderNotConfigured("claude (no token)".to_string()))?;

        let url = "https://api.anthropic.com/api/oauth/usage";
        if verbose {
            eprintln!("[claude] GET {}", url);
        }

        let client = Client::new();
        let response = client
            .get(url)
            .header("Authorization", format!("Bearer {}", anthropic_auth.access))
            .header("anthropic-beta", "oauth-2025-04-20")
            .timeout(timeout)
            .send()
            .await?;

        let status = response.status();
        if verbose {
            eprintln!("[claude] {} {}", status.as_u16(), status.canonical_reason().unwrap_or(""));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(QuotaError::ApiError(format!(
                "Claude API error ({}): {}",
                status, error_text
            )));
        }

        let usage: ClaudeUsageResponse = response.json().await?;

        let data = ClaudeData {
            five_hour: WindowUsage {
                utilization: usage.five_hour.utilization,
                resets_at: usage.five_hour.resets_at,
            },
            seven_day: WindowUsage {
                utilization: usage.seven_day.utilization,
                resets_at: usage.seven_day.resets_at,
            },
            seven_day_sonnet: usage.seven_day_sonnet.map(|w| WindowUsage {
                utilization: w.utilization,
                resets_at: w.resets_at,
            }),
            seven_day_opus: usage.seven_day_opus.map(|w| WindowUsage {
                utilization: w.utilization,
                resets_at: w.resets_at,
            }),
            extra_usage_enabled: usage.extra_usage.is_enabled,
        };

        Ok(ProviderData::Claude(data))
    }
}

impl Default for ClaudeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct ClaudeUsageResponse {
    #[serde(rename = "five_hour")]
    five_hour: ClaudeWindow,
    #[serde(rename = "seven_day")]
    seven_day: ClaudeWindow,
    #[serde(rename = "seven_day_sonnet")]
    seven_day_sonnet: Option<ClaudeWindow>,
    #[serde(rename = "seven_day_opus")]
    seven_day_opus: Option<ClaudeWindow>,
    #[serde(rename = "extra_usage")]
    extra_usage: ClaudeExtraUsage,
}

#[derive(Debug, Deserialize)]
struct ClaudeWindow {
    utilization: f64,
    #[serde(rename = "resets_at")]
    resets_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct ClaudeExtraUsage {
    #[serde(rename = "is_enabled")]
    is_enabled: bool,
}
