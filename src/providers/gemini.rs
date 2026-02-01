use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::auth::{AntigravityAccount, AuthManager, GeminiTokenResponse};
use crate::error::{QuotaError, Result};
use crate::providers::{GeminiAccountData, GeminiData, GeminiModelQuota, Provider, ProviderData};

/// Public Google OAuth client credentials for CLI/installed apps
/// These are NOT secrets - see https://developers.google.com/identity/protocols/oauth2/native-app
const ANTIGRAVITY_CLIENT_ID: &str = "1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com";
const ANTIGRAVITY_CLIENT_SECRET: &str = "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";

/// Antigravity API endpoints (in fallback order)
const ANTIGRAVITY_ENDPOINT_PROD: &str = "https://cloudcode-pa.googleapis.com";
const _ANTIGRAVITY_ENDPOINT_DAILY: &str = "https://daily-cloudcode-pa.sandbox.googleapis.com";
const _ANTIGRAVITY_ENDPOINT_AUTOPUSH: &str = "https://autopush-cloudcode-pa.sandbox.googleapis.com";

/// Default headers for Antigravity API requests
const ANTIGRAVITY_VERSION: &str = "1.15.8";

/// Get platform string for User-Agent header
fn get_platform() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        #[cfg(target_arch = "x86_64")]
        return "windows/x64";
        #[cfg(target_arch = "aarch64")]
        return "windows/arm64";
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        return "windows/unknown";
    }
    #[cfg(target_os = "macos")]
    {
        #[cfg(target_arch = "x86_64")]
        return "darwin/x64";
        #[cfg(target_arch = "aarch64")]
        return "darwin/arm64";
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        return "darwin/unknown";
    }
    #[cfg(target_os = "linux")]
    {
        #[cfg(target_arch = "x86_64")]
        return "linux/x64";
        #[cfg(target_arch = "aarch64")]
        return "linux/arm64";
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        return "linux/unknown";
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return "unknown/unknown";
}

pub struct GeminiProvider {
    auth_manager: AuthManager,
}

impl GeminiProvider {
    pub fn new() -> Self {
        Self {
            auth_manager: AuthManager::new(),
        }
    }

    /// Refresh access token using refresh token
    async fn refresh_access_token(&self, refresh_token: &str) -> Result<String> {
        let client = Client::new();
        
        let params = [
            ("client_id", ANTIGRAVITY_CLIENT_ID),
            ("client_secret", ANTIGRAVITY_CLIENT_SECRET),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let response = client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(QuotaError::TokenRefreshError(format!(
                "Google OAuth refresh failed: {}",
                error_text
            )));
        }

        let token_response: GeminiTokenResponse = response.json().await?;
        Ok(token_response.access_token)
    }

    /// Load code assist to get project ID
    async fn load_code_assist(&self, access_token: &str, timeout: Duration) -> Result<LoadCodeAssistResponse> {
        let client = Client::new();
        
        let metadata = serde_json::json!({
            "ideType": "ANTIGRAVITY",
            "platform": "PLATFORM_UNSPECIFIED",
            "pluginType": "GEMINI",
        });

        let response = client
            .post(format!("{}/v1internal:loadCodeAssist", ANTIGRAVITY_ENDPOINT_PROD))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .header("User-Agent", format!("antigravity/{} {}", ANTIGRAVITY_VERSION, get_platform()))
            .header("X-Goog-Api-Client", "google-cloud-sdk vscode_cloudshelleditor/0.1")
            .json(&serde_json::json!({ "metadata": metadata }))
            .timeout(timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(QuotaError::ApiError(format!(
                "loadCodeAssist failed ({}): {}",
                status, error_text
            )));
        }

        let result: LoadCodeAssistResponse = response.json().await?;
        Ok(result)
    }

    /// Extract project ID from cloudaicompanionProject
    fn extract_project_id(&self, project: &Option<serde_json::Value>) -> Option<String> {
        if let Some(proj) = project {
            // Try to extract from string format "projects/PROJECT_ID/locations/LOCATION"
            if let Some(proj_str) = proj.as_str() {
                let parts: Vec<&str> = proj_str.split('/').collect();
                if parts.len() >= 2 && parts[0] == "projects" {
                    return Some(parts[1].to_string());
                }
            }
        }
        None
    }

    /// Fetch available models with quota info
    async fn fetch_available_models(
        &self,
        access_token: &str,
        project_id: Option<&str>,
        timeout: Duration,
    ) -> Result<FetchAvailableModelsResponse> {
        let client = Client::new();
        
        let payload = if let Some(pid) = project_id {
            serde_json::json!({ "project": format!("projects/{}", pid) })
        } else {
            serde_json::json!({})
        };

        let response = client
            .post(format!("{}/v1internal:fetchAvailableModels", ANTIGRAVITY_ENDPOINT_PROD))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .header("User-Agent", format!("antigravity/{} {}", ANTIGRAVITY_VERSION, get_platform()))
            .header("X-Goog-Api-Client", "google-cloud-sdk vscode_cloudshelleditor/0.1")
            .json(&payload)
            .timeout(timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(QuotaError::ApiError(format!(
                "fetchAvailableModels failed ({}): {}",
                status, error_text
            )));
        }

        let result: FetchAvailableModelsResponse = response.json().await?;
        Ok(result)
    }

    /// Fetch quota for a specific account
    async fn fetch_account_quota(
        &self,
        account: &AntigravityAccount,
        is_active: bool,
        timeout: Duration,
    ) -> Result<GeminiAccountData> {
        let access_token = self.refresh_access_token(&account.refresh_token).await?;

        // Get project ID - either from account or from loadCodeAssist
        let project_id = account.project_id.clone()
            .or_else(|| account.managed_project_id.clone());

        // If no project ID, try to get it from loadCodeAssist
        let project_id = if project_id.is_none() {
            match self.load_code_assist(&access_token, timeout).await {
                Ok(assist) => self.extract_project_id(&assist.cloudaicompanion_project),
                Err(_) => None,
            }
        } else {
            project_id
        };

        let models_response = self.fetch_available_models(&access_token, project_id.as_deref(), timeout).await?;

        let now = Utc::now();
        let mut models: Vec<GeminiModelQuota> = Vec::new();

        if let Some(models_map) = models_response.models {
            for (model_key, info) in models_map {
                if let Some(quota_info) = info.quota_info {
                    // Use display_name for user-friendly output, fall back to model key
                    let display_name = info.display_name.unwrap_or_else(|| model_key.clone());
                    let lower_name = display_name.to_lowercase();

                    // Filter out internal/test models only
                    if lower_name.starts_with("chat_") || lower_name.starts_with("rev19") {
                        continue;
                    }

                    let remaining_fraction = quota_info.remaining_fraction.unwrap_or(0.0)
                        .clamp(0.0, 1.0);

                    let reset_time = quota_info.reset_time
                        .and_then(|t| t.parse::<DateTime<Utc>>().ok())
                        .or_else(|| Some(now + chrono::Duration::days(1)));

                    models.push(GeminiModelQuota {
                        model: display_name,
                        remaining_percent: remaining_fraction * 100.0,
                        reset_time,
                    });
                }
            }
        }

        // Sort models by display name
        models.sort_by(|a, b| a.model.cmp(&b.model));

        Ok(GeminiAccountData {
            email: account.email.clone(),
            is_active,
            models,
        })
    }
}

#[async_trait]
impl Provider for GeminiProvider {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn is_configured(&self) -> bool {
        self.auth_manager
            .is_provider_configured("gemini")
            .unwrap_or(false)
    }

    async fn fetch(&self, timeout: Duration) -> Result<ProviderData> {
        // Read antigravity accounts
        let antigravity = self
            .auth_manager
            .read_antigravity_accounts()?
            .ok_or_else(|| QuotaError::ProviderNotConfigured("gemini (no antigravity accounts found)".to_string()))?;

        if antigravity.accounts.is_empty() {
            return Err(QuotaError::ProviderNotConfigured(
                "gemini (no accounts in antigravity file)".to_string(),
            ));
        }

        // Fetch quota for all accounts
        let mut account_data: Vec<GeminiAccountData> = Vec::new();

        for (idx, account) in antigravity.accounts.iter().enumerate() {
            let is_active = idx == antigravity.active_index;
            match self.fetch_account_quota(account, is_active, timeout).await {
                Ok(data) => account_data.push(data),
                Err(e) => {
                    // Log error but continue with other accounts
                    eprintln!("Warning: Failed to fetch quota for {}: {}", account.email, e);
                }
            }
        }

        if account_data.is_empty() {
            return Err(QuotaError::ApiError(
                "Failed to fetch quota for any Gemini account".to_string(),
            ));
        }

        Ok(ProviderData::Gemini(GeminiData { accounts: account_data }))
    }
}

impl Default for GeminiProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Response from loadCodeAssist
#[derive(Debug, Deserialize)]
struct LoadCodeAssistResponse {
    #[serde(rename = "currentTier")]
    _current_tier: Option<serde_json::Value>,
    #[serde(rename = "paidTier")]
    _paid_tier: Option<serde_json::Value>,
    #[serde(rename = "cloudaicompanionProject")]
    cloudaicompanion_project: Option<serde_json::Value>,
}

/// Response from fetchAvailableModels
#[derive(Debug, Deserialize)]
struct FetchAvailableModelsResponse {
    models: Option<std::collections::HashMap<String, CloudCodeModelInfo>>,
}

#[derive(Debug, Deserialize)]
struct CloudCodeModelInfo {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "quotaInfo")]
    quota_info: Option<CloudCodeQuotaInfo>,
    #[serde(rename = "supportsImages")]
    _supports_images: Option<bool>,
    #[serde(rename = "supportsVideo")]
    _supports_video: Option<bool>,
    #[serde(rename = "supportsThinking")]
    _supports_thinking: Option<bool>,
    _recommended: Option<bool>,
    #[serde(rename = "tagTitle")]
    _tag_title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CloudCodeQuotaInfo {
    #[serde(rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
}
