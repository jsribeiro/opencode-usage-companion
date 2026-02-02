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

use crate::providers::{ClaudeData, CodexData, CopilotData, GeminiData, ProviderData};
use chrono::Utc;
use colored::Colorize;

/// Format data as simple text (one line per provider)
pub fn format_simple(data: &[ProviderData], no_color: bool) -> String {
    if data.is_empty() {
        return "No provider data available.".to_string();
    }

    data.iter()
        .map(|d| format_provider_simple(d, no_color))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_provider_simple(data: &ProviderData, no_color: bool) -> String {
    match data {
        ProviderData::Gemini(gemini) => format_gemini_simple(gemini, no_color),
        ProviderData::Codex(codex) => format_codex_simple(codex, no_color),
        ProviderData::Copilot(copilot) => format_copilot_simple(copilot, no_color),
        ProviderData::Claude(claude) => format_claude_simple(claude, no_color),
    }
}

fn colorize_usage(percent: i32, no_color: bool) -> String {
    let s = format!("{}%", percent);
    if no_color {
        return s;
    }
    if percent < 50 {
        s.green().to_string()
    } else if percent < 80 {
        s.yellow().to_string()
    } else {
        s.red().to_string()
    }
}

fn format_gemini_simple(data: &GeminiData, no_color: bool) -> String {
    data.accounts
        .iter()
        .map(|account| {
            let active_marker = if account.is_active { "" } else { " [inactive]" };

            let models = account
                .models
                .iter()
                .map(|m| {
                    // Invert usage: 100% remaining -> 0% used
                    let used_percent = (100.0 - m.remaining_percent).round() as i32;
                    let usage_str = colorize_usage(used_percent, no_color);
                    format!("{}: {}", m.model, usage_str)
                })
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

fn format_codex_simple(data: &CodexData, no_color: bool) -> String {
    let primary_reset = if data.primary_window.resets_in_seconds > 3600 {
        format!("{}h", data.primary_window.resets_in_seconds / 3600)
    } else {
        format!("{}m", data.primary_window.resets_in_seconds / 60)
    };

    let primary_usage = colorize_usage(data.primary_window.used_percent, no_color);
    let secondary_usage = colorize_usage(data.secondary_window.used_percent, no_color);

    format!(
        "Codex: primary: {}, secondary: {} - primary resets in {}",
        primary_usage, secondary_usage, primary_reset
    )
}

fn format_copilot_simple(data: &CopilotData, no_color: bool) -> String {
    let used = data.premium_entitlement - data.premium_remaining;

    // Calculate usage percentage for coloring
    let used_percent = if data.premium_entitlement > 0 {
        let remaining_fraction = data.premium_remaining as f64 / data.premium_entitlement as f64;
        ((1.0 - remaining_fraction) * 100.0).clamp(0.0, 100.0) as i32
    } else {
        0
    };

    let usage_display = if no_color {
        format!("{}/{}", used, data.premium_entitlement)
    } else {
        let s = format!("{}/{}", used, data.premium_entitlement);
        if used_percent < 50 {
            s.green().to_string()
        } else if used_percent < 80 {
            s.yellow().to_string()
        } else {
            s.red().to_string()
        }
    };

    if data.premium_remaining < 0 || data.overage_count > 0 {
        format!(
            "Copilot: used {} ({} overage reqs, permitted: {}) - resets {}",
            usage_display, data.overage_count, data.overage_permitted, data.quota_reset_date
        )
    } else {
        format!(
            "Copilot: used {} - resets {}",
            usage_display, data.quota_reset_date
        )
    }
}

fn format_claude_simple(data: &ClaudeData, no_color: bool) -> String {
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

    let five_h_usage = colorize_usage(data.five_hour.utilization as i32, no_color);
    let seven_d_usage = colorize_usage(data.seven_day.utilization as i32, no_color);

    format!(
        "Claude: 5h: {}, 7d: {} - 5h resets in {}",
        five_h_usage, seven_d_usage, five_h_reset
    )
}
