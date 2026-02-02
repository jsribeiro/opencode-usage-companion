/*
 * Copyright (C) 2026 João Sena Ribeiro <sena@smux.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::time::Duration;

use crate::error::Result;

pub mod claude;
pub mod codex;
pub mod copilot;
pub mod gemini;

/// Trait that all providers must implement
#[async_trait]
pub trait Provider: Send + Sync {
    /// Provider name
    fn name(&self) -> &'static str;

    /// Check if this provider is configured (has auth tokens)
    fn is_configured(&self) -> bool;

    /// Fetch quota/usage data from the provider
    async fn fetch(&self, timeout: Duration, verbose: bool) -> Result<ProviderData>;
}

/// Data returned by any provider
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderData {
    Gemini(GeminiData),
    Codex(CodexData),
    Copilot(CopilotData),
    Claude(ClaudeData),
    /// Provider API call failed - usage data is unknown
    Failed {
        provider: String,
        error: String,
    },
}

/// Gemini/Antigravity provider data (supports multiple accounts)
#[derive(Debug, Clone, Serialize)]
pub struct GeminiData {
    pub accounts: Vec<GeminiAccountData>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeminiAccountData {
    pub email: String,
    pub is_active: bool,
    pub models: Vec<GeminiModelQuota>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeminiModelQuota {
    pub model: String,
    pub remaining_percent: f64,
    pub reset_time: Option<DateTime<Utc>>,
}

/// Codex provider data
#[derive(Debug, Clone, Serialize)]
pub struct CodexData {
    pub plan: String,
    pub primary_window: WindowQuota,
    pub secondary_window: WindowQuota,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowQuota {
    pub used_percent: i32,
    pub resets_in_seconds: i64,
}

/// Copilot provider data
#[derive(Debug, Clone, Serialize)]
pub struct CopilotData {
    pub plan: String,
    pub premium_entitlement: i64,
    pub premium_remaining: i64,
    pub overage_permitted: bool,
    pub overage_count: i64,
    pub quota_reset_date: String,
}

/// Claude provider data
#[derive(Debug, Clone, Serialize)]
pub struct ClaudeData {
    pub five_hour: WindowUsage,
    pub seven_day: WindowUsage,
    pub seven_day_sonnet: Option<WindowUsage>,
    pub seven_day_opus: Option<WindowUsage>,
    pub extra_usage_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowUsage {
    pub utilization: f64,
    pub resets_at: Option<DateTime<Utc>>,
}

/// Provider status for display
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProviderStatus {
    Ok,
    Warning,
    Error,
}

impl ProviderData {
    /// Get the provider name
    pub fn provider_name(&self) -> &str {
        match self {
            ProviderData::Gemini(_) => "gemini",
            ProviderData::Codex(_) => "codex",
            ProviderData::Copilot(_) => "copilot",
            ProviderData::Claude(_) => "claude",
            ProviderData::Failed { provider, .. } => provider,
        }
    }

    /// Get display status based on quota levels
    pub fn status(&self) -> ProviderStatus {
        match self {
            ProviderData::Gemini(data) => {
                let min_remaining = data.accounts.iter()
                    .flat_map(|a| a.models.iter())
                    .map(|m| m.remaining_percent)
                    .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                match min_remaining {
                    Some(remaining) if remaining < 20.0 => ProviderStatus::Warning,
                    _ => ProviderStatus::Ok,
                }
            }
            ProviderData::Codex(data) => {
                if data.primary_window.used_percent > 80 || data.secondary_window.used_percent > 80 {
                    ProviderStatus::Warning
                } else {
                    ProviderStatus::Ok
                }
            }
            ProviderData::Copilot(data) => {
                if data.premium_remaining < 0 {
                    ProviderStatus::Warning
                } else if (data.premium_remaining as f64) < (data.premium_entitlement as f64 * 0.2) {
                    ProviderStatus::Warning
                } else {
                    ProviderStatus::Ok
                }
            }
            ProviderData::Claude(data) => {
                if data.five_hour.utilization > 80.0 || data.seven_day.utilization > 80.0 {
                    ProviderStatus::Warning
                } else {
                    ProviderStatus::Ok
                }
            }
            ProviderData::Failed { .. } => ProviderStatus::Error,
        }
    }
}
