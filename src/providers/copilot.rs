use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::auth::AuthManager;
use crate::error::{QuotaError, Result};
use crate::providers::{CopilotData, Provider, ProviderData};

pub struct CopilotProvider {
    auth_manager: AuthManager,
}

impl CopilotProvider {
    pub fn new() -> Self {
        Self {
            auth_manager: AuthManager::new(),
        }
    }

}

#[async_trait]
impl Provider for CopilotProvider {
    fn name(&self) -> &'static str {
        "copilot"
    }

    fn is_configured(&self) -> bool {
        self.auth_manager
            .is_provider_configured("copilot")
            .unwrap_or(false)
    }

    async fn fetch(&self, timeout: Duration, verbose: bool) -> Result<ProviderData> {
        let auth = self
            .auth_manager
            .read_opencode_auth()?
            .ok_or_else(|| QuotaError::ProviderNotConfigured("copilot".to_string()))?;

        let copilot_auth = auth
            .github_copilot
            .ok_or_else(|| QuotaError::ProviderNotConfigured("copilot (no token)".to_string()))?;

        let client = Client::new();

        // Fetch quota data
        let url = "https://api.github.com/copilot_internal/user";
        if verbose {
            eprintln!("[copilot] GET {}", url);
        }

        let response = client
            .get(url)
            .header("Authorization", format!("token {}", copilot_auth.access))
            .header("Accept", "application/json")
            .header("User-Agent", "ocu/0.1.0")
            .header("Editor-Version", "vscode/1.96.2")
            .header("X-Github-Api-Version", "2025-04-01")
            .timeout(timeout)
            .send()
            .await?;

        let status = response.status();
        if verbose {
            eprintln!("[copilot] {} {}", status.as_u16(), status.canonical_reason().unwrap_or(""));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(QuotaError::ApiError(format!(
                "Copilot API error ({}): {}",
                status, error_text
            )));
        }

        let usage: CopilotUsageResponse = response.json().await?;
        let premium = &usage.quota_snapshots.premium_interactions;

        let data = CopilotData {
            plan: usage.copilot_plan,
            premium_entitlement: premium.entitlement,
            premium_remaining: premium.remaining,
            overage_permitted: premium.overage_permitted,
            overage_count: premium.overage_count,
            quota_reset_date: usage.quota_reset_date,
        };

        Ok(ProviderData::Copilot(data))
    }
}

impl Default for CopilotProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct CopilotUsageResponse {
    #[serde(rename = "copilot_plan")]
    copilot_plan: String,
    #[serde(rename = "quota_reset_date")]
    quota_reset_date: String,
    #[serde(rename = "quota_snapshots")]
    quota_snapshots: CopilotQuotaSnapshots,
}

#[derive(Debug, Deserialize)]
struct CopilotQuotaSnapshots {
    #[serde(rename = "premium_interactions")]
    premium_interactions: CopilotPremiumInteractions,
}

#[derive(Debug, Deserialize)]
struct CopilotPremiumInteractions {
    entitlement: i64,
    remaining: i64,
    #[serde(rename = "overage_permitted")]
    overage_permitted: bool,
    #[serde(default, rename = "overage_count", alias = "overageCount")]
    overage_count: i64,
}
