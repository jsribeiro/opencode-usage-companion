use crate::providers::{ClaudeData, CodexData, CopilotData, GeminiData, ProviderData};
use chrono::Utc;

/// Format data as simple text (one line per provider)
pub fn format_simple(data: &[ProviderData]) -> String {
    if data.is_empty() {
        return "No provider data available.".to_string();
    }

    data.iter()
        .map(format_provider_simple)
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_provider_simple(data: &ProviderData) -> String {
    match data {
        ProviderData::Gemini(gemini) => format_gemini_simple(gemini),
        ProviderData::Codex(codex) => format_codex_simple(codex),
        ProviderData::Copilot(copilot) => format_copilot_simple(copilot),
        ProviderData::Claude(claude) => format_claude_simple(claude),
    }
}

fn format_gemini_simple(data: &GeminiData) -> String {
    data.accounts
        .iter()
        .map(|account| {
            let active_marker = if account.is_active { "" } else { " [inactive]" };

            let models = account
                .models
                .iter()
                .map(|m| format!("{}:{:.0}%", m.model, m.remaining_percent))
                .collect::<Vec<_>>()
                .join(", ");

            let reset = account
                .models
                .first()
                .and_then(|m| m.reset_time)
                .map(|t| {
                    let now = Utc::now();
                    let duration = t.signed_duration_since(now);
                    if duration.num_hours() > 24 {
                        format!("{} days", duration.num_days())
                    } else if duration.num_hours() > 0 {
                        format!("{}h {}m", duration.num_hours(), duration.num_minutes() % 60)
                    } else {
                        format!("{}m", duration.num_minutes())
                    }
                })
                .unwrap_or_else(|| "-".to_string());

            format!(
                "Gemini ({}){}: {} - resets in {}",
                account.email, active_marker, models, reset
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_codex_simple(data: &CodexData) -> String {
    let primary_reset = if data.primary_window.resets_in_seconds > 3600 {
        format!("{}h", data.primary_window.resets_in_seconds / 3600)
    } else {
        format!("{}m", data.primary_window.resets_in_seconds / 60)
    };

    format!(
        "Codex: primary:{}%, secondary:{}% - primary resets in {}",
        data.primary_window.used_percent, data.secondary_window.used_percent, primary_reset
    )
}

fn format_copilot_simple(data: &CopilotData) -> String {
    let used = data.premium_entitlement - data.premium_remaining;

    if data.premium_remaining < 0 || data.overage_count > 0 {
        format!(
            "Copilot: used {}/{} ({} overage reqs, permitted: {}) - resets {}",
            used,
            data.premium_entitlement,
            data.overage_count,
            data.overage_permitted,
            data.quota_reset_date
        )
    } else {
        format!(
            "Copilot: used {}/{} - resets {}",
            used, data.premium_entitlement, data.quota_reset_date
        )
    }
}

fn format_claude_simple(data: &ClaudeData) -> String {
    let five_h_reset = data
        .five_hour
        .resets_at
        .map(|t| {
            let now = Utc::now();
            let duration = t.signed_duration_since(now);
            if duration.num_hours() > 24 {
                format!("{} days", duration.num_days())
            } else if duration.num_hours() > 0 {
                format!("{}h {}m", duration.num_hours(), duration.num_minutes() % 60)
            } else {
                format!("{}m", duration.num_minutes())
            }
        })
        .unwrap_or_else(|| "-".to_string());

    format!(
        "Claude: 5h:{:.0}%, 7d:{:.0}% - 5h resets in {}",
        data.five_hour.utilization, data.seven_day.utilization, five_h_reset
    )
}
