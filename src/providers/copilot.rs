use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::auth::AuthManager;
use crate::error::{QuotaError, Result};
use crate::providers::{CopilotData, CopilotOverageCharges, Provider, ProviderData};

pub struct CopilotProvider {
    auth_manager: AuthManager,
}

impl CopilotProvider {
    pub fn new() -> Self {
        Self {
            auth_manager: AuthManager::new(),
        }
    }

    /// Fetch billing/overage data from GitHub API
    /// Returns None if the API call fails, with warnings logged for specific errors
    async fn fetch_billing_data(
        &self,
        client: &Client,
        token: &str,
        timeout: Duration,
        verbose: bool,
    ) -> Option<CopilotOverageCharges> {
        // First, get the authenticated user's login
        let user_url = "https://api.github.com/user";
        if verbose {
            eprintln!("[copilot] GET {}", user_url);
        }

        let user_response = match client
            .get(user_url)
            .header("Authorization", format!("token {}", token))
            .header("Accept", "application/json")
            .header("User-Agent", "ocu/0.1.0")
            .header("X-Github-Api-Version", "2022-11-28")
            .timeout(timeout)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Warning: Failed to fetch GitHub user info: {}", e);
                return None;
            }
        };

        let status = user_response.status();
        if verbose {
            eprintln!("[copilot] {} {}", status.as_u16(), status.canonical_reason().unwrap_or(""));
        }

        if !status.is_success() {
            eprintln!(
                "Warning: GitHub user API returned status {}",
                status
            );
            return None;
        }

        let user: serde_json::Value = match user_response.json().await {
            Ok(u) => u,
            Err(e) => {
                eprintln!("Warning: Failed to parse GitHub user response: {}", e);
                return None;
            }
        };

        let username = match user.get("login").and_then(|l| l.as_str()) {
            Some(u) => u,
            None => {
                eprintln!("Warning: GitHub user response missing 'login' field");
                return None;
            }
        };

        // Fetch billing premium request usage
        let billing_url = format!(
            "https://api.github.com/users/{}/settings/billing/premium_request/usage",
            username
        );

        if verbose {
            eprintln!("[copilot] GET {}", billing_url);
        }

        let billing_response = match client
            .get(&billing_url)
            .header("Authorization", format!("token {}", token))
            .header("Accept", "application/json")
            .header("User-Agent", "ocu/0.1.0")
            .header("X-Github-Api-Version", "2022-11-28")
            .timeout(timeout)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Warning: Failed to fetch Copilot billing data: {}", e);
                return None;
            }
        };

        let status = billing_response.status();
        if verbose {
            eprintln!("[copilot] {} {}", status.as_u16(), status.canonical_reason().unwrap_or(""));
        }

        if !status.is_success() {
            if status.as_u16() == 403 {
                eprintln!(
                    "Warning: Copilot billing API returned 403 Forbidden - token may lack billing scope"
                );
            } else if status.as_u16() == 404 {
                // 404 is expected for users without billing access - don't warn
            } else {
                eprintln!(
                    "Warning: Copilot billing API returned status {}",
                    status
                );
            }
            return None;
        }

        let billing: BillingUsageResponse = match billing_response.json().await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Warning: Failed to parse Copilot billing response: {}", e);
                return None;
            }
        };

        // Sum up all Copilot-related charges
        let (total_quantity, total_amount) = billing
            .usage_items
            .iter()
            .filter(|item| item.product.to_lowercase().contains("copilot"))
            .fold((0.0, 0.0), |(q, a), item| {
                (q + item.net_quantity, a + item.net_amount)
            });

        if total_quantity > 0.0 || total_amount > 0.0 {
            Some(CopilotOverageCharges {
                quantity: total_quantity as i64,
                amount: total_amount,
            })
        } else {
            // Return zero charges to indicate the API worked but no overages
            Some(CopilotOverageCharges {
                quantity: 0,
                amount: 0.0,
            })
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

        // Try to fetch billing/overage data (may fail if token doesn't have permission)
        let overage_charges = self.fetch_billing_data(&client, &copilot_auth.access, timeout, verbose).await;

        let data = CopilotData {
            plan: usage.copilot_plan,
            premium_entitlement: premium.entitlement,
            premium_remaining: premium.remaining,
            overage_permitted: premium.overage_permitted,
            quota_reset_date: usage.quota_reset_date,
            overage_charges,
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
}

/// Billing premium request usage response
#[derive(Debug, Deserialize)]
struct BillingUsageResponse {
    #[serde(rename = "usageItems", default)]
    usage_items: Vec<BillingUsageItem>,
}

#[derive(Debug, Deserialize)]
struct BillingUsageItem {
    #[serde(default)]
    product: String,
    #[serde(rename = "netQuantity", default)]
    net_quantity: f64,
    #[serde(rename = "netAmount", default)]
    net_amount: f64,
}
