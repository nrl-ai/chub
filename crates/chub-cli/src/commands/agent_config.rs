use clap::{Args, Subcommand};
use owo_colors::OwoColorize;

use chub_core::team::agent_config;

use crate::output;

#[derive(Args)]
pub struct AgentConfigArgs {
    #[command(subcommand)]
    command: AgentConfigCommand,
}

#[derive(Subcommand)]
pub enum AgentConfigCommand {
    /// Generate all target files (CLAUDE.md, .cursorrules, GEMINI.md, etc.)
    Generate,
    /// Update targets only if source changed (idempotent)
    Sync,
    /// Show what would change without writing
    Diff,
}

pub fn run(args: AgentConfigArgs, json: bool) {
    match args.command {
        AgentConfigCommand::Generate | AgentConfigCommand::Sync => run_sync(json),
        AgentConfigCommand::Diff => run_diff(json),
    }
}

fn run_sync(json: bool) {
    match agent_config::sync_configs() {
        Ok(results) => {
            if json {
                let items: Vec<serde_json::Value> = results
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "target": r.target,
                            "filename": r.filename,
                            "action": format!("{:?}", r.action).to_lowercase(),
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "results": items,
                    }))
                    .unwrap_or_default()
                );
            } else {
                if results.is_empty() {
                    eprintln!(
                        "{}",
                        "No targets configured in agent_rules.targets.".yellow()
                    );
                    return;
                }
                for r in &results {
                    let action = match r.action {
                        agent_config::SyncAction::Created => "created".green().to_string(),
                        agent_config::SyncAction::Updated => "updated".yellow().to_string(),
                        agent_config::SyncAction::Unchanged => "unchanged".dimmed().to_string(),
                        agent_config::SyncAction::Unknown => {
                            format!(
                                "{}  (known: {})",
                                "unknown target".red(),
                                agent_config::Target::all_target_names().join(", ")
                            )
                        }
                    };
                    eprintln!("  {} {}", r.filename.bold(), action);
                }
            }
        }
        Err(e) => {
            output::error(&e.to_string(), json);
            std::process::exit(1);
        }
    }
}

fn run_diff(json: bool) {
    match agent_config::diff_configs() {
        Ok(diffs) => {
            if json {
                let items: Vec<serde_json::Value> = diffs
                    .iter()
                    .map(|(filename, new_content, existing)| {
                        serde_json::json!({
                            "filename": filename,
                            "exists": existing.is_some(),
                            "would_change": existing.as_ref().map(|e| e != new_content).unwrap_or(true),
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "diffs": items }))
                        .unwrap_or_default()
                );
            } else {
                if diffs.is_empty() {
                    eprintln!("{}", "No targets configured.".yellow());
                    return;
                }
                for (filename, new_content, existing) in &diffs {
                    match existing {
                        None => {
                            eprintln!("  {} {}", filename.bold(), "(new file)".green());
                            for line in new_content.lines().take(10) {
                                eprintln!("    + {}", line.green());
                            }
                            if new_content.lines().count() > 10 {
                                eprintln!("    {} more lines...", "...".dimmed());
                            }
                        }
                        Some(old) if old != new_content => {
                            eprintln!("  {} {}", filename.bold(), "(changed)".yellow());
                        }
                        Some(_) => {
                            eprintln!("  {} {}", filename.bold(), "(unchanged)".dimmed());
                        }
                    }
                }
            }
        }
        Err(e) => {
            output::error(&e.to_string(), json);
            std::process::exit(1);
        }
    }
}
