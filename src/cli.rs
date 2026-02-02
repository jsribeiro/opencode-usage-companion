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

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "ocu")]
#[command(about = "OpenCode Usage Companion - Check AI provider quotas")]
#[command(version)]
pub struct Args {
    /// Provider(s) to check
    #[arg(short, long, value_enum)]
    pub provider: Vec<ProviderArg>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "table")]
    pub format: OutputFormat,

    /// Timeout per provider in seconds
    #[arg(short, long, default_value = "10")]
    pub timeout: u64,

    /// Query providers concurrently
    #[arg(short, long)]
    pub concurrent: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Show verbose output (API requests and responses)
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum ProviderArg {
    /// Google Gemini / Antigravity
    Gemini,
    /// OpenAI Codex
    Codex,
    /// GitHub Copilot
    Copilot,
    /// Anthropic Claude
    Claude,
    /// All configured providers
    All,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// Pretty table format with colors
    Table,
    /// JSON output for scripting
    Json,
    /// Simple text format
    Simple,
}
