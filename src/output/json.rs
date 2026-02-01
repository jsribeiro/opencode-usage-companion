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
