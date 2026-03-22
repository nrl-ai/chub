use clap::Args;
use owo_colors::OwoColorize;

use chub_core::team::freshness;

use crate::output;

#[derive(Args)]
pub struct CheckArgs {
    /// Auto-update outdated pins to current installed version
    #[arg(long)]
    fix: bool,
}

pub fn run(args: CheckArgs, json: bool) {
    let cwd = std::env::current_dir().unwrap_or_default();
    let results = freshness::check_freshness(&cwd);

    if results.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({ "status": "no_pins", "results": [] })
            );
        } else {
            eprintln!(
                "{}",
                "No pins to check. Use `chub pin add <id>` first.".dimmed()
            );
        }
        return;
    }

    let outdated_count = results
        .iter()
        .filter(|r| r.status == freshness::FreshnessStatus::Outdated)
        .count();

    if json {
        let items: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.pin_id,
                    "status": r.status,
                    "pinned_version": r.pinned_version,
                    "installed_version": r.installed_version,
                    "suggestion": r.suggestion,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "results": items,
                "outdated": outdated_count,
            }))
            .unwrap_or_default()
        );
    } else {
        for r in &results {
            match r.status {
                freshness::FreshnessStatus::Current => {
                    eprintln!("  {} {} docs are current", "✓".green(), r.pin_id.bold());
                }
                freshness::FreshnessStatus::Outdated => {
                    eprintln!(
                        "  {} {} pinned to {} docs, but {} is installed",
                        "⚠".yellow(),
                        r.pin_id.bold(),
                        r.pinned_version.as_deref().unwrap_or("?"),
                        r.installed_version.as_deref().unwrap_or("?"),
                    );
                    if let Some(ref suggestion) = r.suggestion {
                        eprintln!("    → {}", suggestion.dimmed());
                    }
                }
                freshness::FreshnessStatus::Unknown => {
                    eprintln!(
                        "  {} {} (no matching dependency found)",
                        "?".dimmed(),
                        r.pin_id.bold(),
                    );
                }
            }
        }

        if outdated_count > 0 {
            eprintln!(
                "\n{} outdated pin(s) found. Run {} to update.",
                outdated_count,
                "chub check --fix".bold()
            );
        } else {
            eprintln!("\n{}", "All pins are current.".green());
        }
    }

    if args.fix && outdated_count > 0 {
        let fixed = freshness::auto_fix_freshness(&results);
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "fixed": fixed.len(),
                    "status": "updated",
                }))
                .unwrap_or_default()
            );
        } else {
            output::success(&format!("Updated {} pin(s).", fixed.len()));
        }
    }
}
