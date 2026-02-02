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
        OutputFormat::Simple => simple::format_simple(data, no_color),
    }
}
