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

use crate::providers::ProviderData;
use chrono::Utc;
use serde::Serialize;

/// JSON output structure
#[derive(Serialize)]
struct JsonOutput<'a> {
    timestamp: String,
    providers: &'a [ProviderData],
}

/// Format data as JSON
pub fn format_json(data: &[ProviderData]) -> String {
    let output = JsonOutput {
        timestamp: Utc::now().to_rfc3339(),
        providers: data,
    };

    match serde_json::to_string_pretty(&output) {
        Ok(json) => json,
        Err(e) => format!("{{\"error\": \"Failed to serialize: {}\"}}", e),
    }
}
