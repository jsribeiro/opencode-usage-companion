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

use clap::Parser;
use colored::{control, Colorize};
use opencode_usage_companion::cli::{Args, ProviderArg};
use opencode_usage_companion::output::format_output;
use opencode_usage_companion::providers::{claude::ClaudeProvider, codex::CodexProvider, copilot::CopilotProvider, gemini::GeminiProvider, Provider, ProviderData};
use std::process::ExitCode;
use std::time::Duration;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    // Disable colors if requested
    if args.no_color {
        control::set_override(false);
    }

    // Determine which providers to query
    let provider_names = if args.provider.is_empty() || args.provider.contains(&ProviderArg::All) {
        vec!["gemini", "codex", "copilot", "claude"]
    } else {
        args.provider
            .iter()
            .map(|p| match p {
                ProviderArg::Gemini => "gemini",
                ProviderArg::Codex => "codex",
                ProviderArg::Copilot => "copilot",
                ProviderArg::Claude => "claude",
                ProviderArg::All => unreachable!(),
            })
            .collect()
    };

    // Build provider instances
    let mut providers: Vec<Box<dyn Provider>> = Vec::new();
    let mut configured_count = 0;

    for name in &provider_names {
        let provider: Box<dyn Provider> = match *name {
            "gemini" => {
                let p = GeminiProvider::new();
                if p.is_configured() {
                    configured_count += 1;
                }
                Box::new(p)
            }
            "codex" => {
                let p = CodexProvider::new();
                if p.is_configured() {
                    configured_count += 1;
                }
                Box::new(p)
            }
            "copilot" => {
                let p = CopilotProvider::new();
                if p.is_configured() {
                    configured_count += 1;
                }
                Box::new(p)
            }
            "claude" => {
                let p = ClaudeProvider::new();
                if p.is_configured() {
                    configured_count += 1;
                }
                Box::new(p)
            }
            _ => continue,
        };
        providers.push(provider);
    }

    if providers.is_empty() {
        eprintln!("Error: No providers specified.");
        return ExitCode::from(2);
    }

    if configured_count == 0 {
        eprintln!("Error: No AI providers configured.");
        eprintln!("Please authenticate with OpenCode first:");
        eprintln!("  - gemini: opencode auth login gemini");
        eprintln!("  - codex: opencode auth login openai");
        eprintln!("  - copilot: opencode auth login github-copilot");
        eprintln!("  - claude: opencode auth login anthropic");
        return ExitCode::from(2);
    }

    let timeout = Duration::from_secs(args.timeout);
    let mut results = Vec::new();
    let mut has_errors = false;
    let mut first_warning = true;
    let no_color = args.no_color;

    println!("Fetching quota information...");

    let verbose = args.verbose;

    if args.concurrent {
        // Concurrent fetching - only fetch configured providers
        let futures = providers.iter()
            .filter(|p| p.is_configured())
            .map(|provider| {
                let timeout = timeout;
                async move {
                    let name = provider.name();
                    match provider.fetch(timeout, verbose).await {
                        Ok(data) => Ok(data),
                        Err(e) => Err((name, e)),
                    }
                }
            });

        let outcomes = futures::future::join_all(futures).await;

        for outcome in outcomes {
            match outcome {
                Ok(data) => results.push(data),
                Err((name, e)) => {
                    if first_warning {
                        eprintln!();
                        first_warning = false;
                    }
                    print_warning(name, &e.to_string(), no_color);
                    results.push(ProviderData::Failed {
                        provider: name.to_string(),
                        error: e.to_string(),
                    });
                    has_errors = true;
                }
            }
        }
    } else {
        // Sequential fetching
        for provider in &providers {
            let name = provider.name();
            if !provider.is_configured() {
                continue;
            }

            match provider.fetch(timeout, verbose).await {
                Ok(data) => results.push(data),
                Err(e) => {
                    if first_warning {
                        eprintln!();
                        first_warning = false;
                    }
                    print_warning(name, &e.to_string(), no_color);
                    results.push(ProviderData::Failed {
                        provider: name.to_string(),
                        error: e.to_string(),
                    });
                    has_errors = true;
                }
            }
        }
    }

    if results.is_empty() {
        eprintln!("\nError: All provider queries failed.");
        return ExitCode::from(1);
    }

    // Check if all results are failures
    let all_failed = results.iter().all(|r| matches!(r, ProviderData::Failed { .. }));
    if all_failed {
        eprintln!("\nError: All provider queries failed.");
        return ExitCode::from(1);
    }

    // Output results (with blank line before for separation)
    println!();
    let output = format_output(&results, args.format, no_color);
    println!("{}", output);

    if has_errors {
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}

/// Print a formatted warning message for a failed provider
fn print_warning(provider: &str, error: &str, no_color: bool) {
    // Split error message: if it contains a JSON body, put that on a new line
    let (summary, detail) = if let Some(json_start) = error.find("\n{") {
        // JSON on its own line already
        let (s, d) = error.split_at(json_start);
        (s.trim(), Some(d.trim()))
    } else if let Some(json_start) = error.find('{') {
        // JSON inline - split it out
        let (s, d) = error.split_at(json_start);
        (s.trim(), Some(d.trim()))
    } else {
        (error, None)
    };

    if no_color {
        eprintln!("Warning: {} query failed: {}", provider, summary);
        if let Some(d) = detail {
            eprintln!("    {}", d);
        }
    } else {
        eprintln!(
            "{} {} query failed: {}",
            "Warning:".yellow().bold(),
            provider.bright_blue(),
            summary,
        );
        if let Some(d) = detail {
            eprintln!("    {}", d);
        }
    }
}
