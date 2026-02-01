pub mod json;
pub mod simple;
pub mod table;

use crate::cli::OutputFormat;
use crate::providers::ProviderData;

/// Format provider data according to the specified format
pub fn format_output(data: &[ProviderData], format: OutputFormat, no_color: bool) -> String {
    match format {
        OutputFormat::Table => table::format_table(data, no_color),
        OutputFormat::Json => json::format_json(data),
        OutputFormat::Simple => simple::format_simple(data),
    }
}
