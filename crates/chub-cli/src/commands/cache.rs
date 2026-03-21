use clap::{Args, Subcommand};
use owo_colors::OwoColorize;

use chub_core::cache::{clear_cache, get_cache_stats, SourceStat};

#[derive(Args)]
pub struct CacheArgs {
    #[command(subcommand)]
    command: CacheCommand,
}

#[derive(Subcommand)]
enum CacheCommand {
    /// Show cache information
    Status,
    /// Clear cached data
    Clear,
}

pub fn run(args: CacheArgs, json: bool) {
    match args.command {
        CacheCommand::Status => run_status(json),
        CacheCommand::Clear => run_clear(json),
    }
}

fn run_status(json: bool) {
    let stats = get_cache_stats();

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&stats).unwrap_or_default()
        );
        return;
    }

    if !stats.exists || stats.sources.is_empty() {
        eprintln!(
            "{}",
            "No cache found. Run `chub update` to initialize.".yellow()
        );
        return;
    }

    eprintln!("{}", "Cache Status\n".bold());
    for src in &stats.sources {
        match src {
            SourceStat::Local { name, path } => {
                eprintln!("  {} {}", name.bold(), "(local)".dimmed());
                eprintln!("    Path: {}", path);
            }
            SourceStat::Remote {
                name,
                has_registry,
                last_updated,
                full_bundle,
                file_count,
                data_size,
            } => {
                eprintln!("  {} {}", name.bold(), "(remote)".dimmed());
                let reg = if *has_registry {
                    "yes".green().to_string()
                } else {
                    "no".red().to_string()
                };
                eprintln!("    Registry: {}", reg);
                eprintln!(
                    "    Last updated: {}",
                    last_updated.as_deref().unwrap_or("never")
                );
                eprintln!(
                    "    Full bundle: {}",
                    if *full_bundle { "yes" } else { "no" }
                );
                eprintln!("    Cached files: {}", file_count);
                eprintln!("    Size: {:.1} KB", *data_size as f64 / 1024.0);
            }
        }
    }
}

fn run_clear(json: bool) {
    clear_cache();

    if json {
        println!("{}", serde_json::json!({ "status": "cleared" }));
    } else {
        eprintln!("{}", "Cache cleared.".green());
    }
}
