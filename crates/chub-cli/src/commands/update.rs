use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

use chub_core::cache::evict_lru_cache;
use chub_core::config::load_config;
use chub_core::error::Result;
use chub_core::fetch::{fetch_all_registries, fetch_full_bundle};

use crate::output;

#[derive(Args)]
pub struct UpdateArgs {
    /// Force re-download even if cache is fresh
    #[arg(long)]
    force: bool,

    /// Download the full bundle for offline use
    #[arg(long)]
    full: bool,
}

pub async fn run(args: UpdateArgs, json: bool) -> Result<()> {
    let config = load_config();

    if args.full {
        let remote_count = config.sources.iter().filter(|s| s.path.is_none()).count();
        let pb = if !json && remote_count > 0 {
            let pb = ProgressBar::new(remote_count as u64);
            pb.set_style(
                ProgressStyle::with_template("{spinner:.cyan} [{bar:20}] {pos}/{len} {msg}")
                    .unwrap()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏-"),
            );
            Some(pb)
        } else {
            None
        };

        for source in &config.sources {
            if source.path.is_some() {
                if !json {
                    output::info(&format!("Skipping local source: {}", source.name));
                }
                continue;
            }
            if let Some(ref pb) = pb {
                pb.set_message(format!("Downloading {}...", source.name));
            }
            fetch_full_bundle(&source.name).await?;
            if let Some(ref pb) = pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_and_clear();
        }

        // Run LRU eviction after downloading bundles
        let freed = evict_lru_cache(None);

        if json {
            println!("{}", serde_json::json!({ "status": "ok", "mode": "full" }));
        } else {
            output::success("Full bundle(s) downloaded and extracted.");
            if freed > 0 {
                output::info(&format!("Cache eviction freed {} KB.", freed / 1024));
            }
        }
    } else {
        let pb = if !json {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::with_template("{spinner:.cyan} {msg}")
                    .unwrap()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏-"),
            );
            pb.set_message("Updating registries...");
            pb.enable_steady_tick(std::time::Duration::from_millis(80));
            Some(pb)
        } else {
            None
        };

        let errors = fetch_all_registries(true).await;

        if let Some(pb) = pb {
            pb.finish_and_clear();
        }

        for e in &errors {
            eprintln!("{}", format!("Warning: {}: {}", e.source, e.error).yellow());
        }

        let updated = config.sources.iter().filter(|s| s.path.is_none()).count() - errors.len();

        if json {
            println!(
                "{}",
                serde_json::json!({
                    "status": "ok",
                    "mode": "registry",
                    "updated": updated,
                    "errors": errors,
                })
            );
        } else {
            output::success(&format!("Registry updated ({} remote source(s)).", updated));
        }
    }

    Ok(())
}
