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
