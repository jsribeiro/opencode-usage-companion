use clap::Parser;
use colored::control;
use opencode_usage_companion::cli::{Args, ProviderArg};
use opencode_usage_companion::output::format_output;
use opencode_usage_companion::providers::{claude::ClaudeProvider, codex::CodexProvider, copilot::CopilotProvider, gemini::GeminiProvider, Provider};
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
    let mut errors = Vec::new();

    println!("Fetching quota information...\n");

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
                    eprintln!("Warning: {} failed: {}", name, e);
                    errors.push((name, e));
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
                    eprintln!("Warning: {} failed: {}", name, e);
                    errors.push((name, e));
                }
            }
        }
    }

    if results.is_empty() {
        eprintln!("\nError: All provider queries failed.");
        return ExitCode::from(1);
    }

    // Output results
    let output = format_output(&results, args.format, args.no_color);
    println!("{}", output);

    if !errors.is_empty() {
        eprintln!("\nNote: {} provider(s) failed to respond", errors.len());
    }

    ExitCode::from(0)
}
