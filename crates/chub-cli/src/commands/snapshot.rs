use clap::{Args, Subcommand};
use owo_colors::OwoColorize;

use chub_core::team::snapshots;

use crate::output;

#[derive(Args)]
pub struct SnapshotArgs {
    #[command(subcommand)]
    command: SnapshotCommand,
}

#[derive(Subcommand)]
pub enum SnapshotCommand {
    /// Capture all pinned doc versions as a snapshot
    Create(SnapshotCreateArgs),
    /// Restore exact doc versions from a snapshot
    Restore(SnapshotRestoreArgs),
    /// Show what changed between two snapshots
    Diff(SnapshotDiffArgs),
    /// List all snapshots
    List,
}

#[derive(Args)]
pub struct SnapshotCreateArgs {
    /// Snapshot name (e.g. "v2.1.0")
    name: String,
}

#[derive(Args)]
pub struct SnapshotRestoreArgs {
    /// Snapshot name to restore
    name: String,
}

#[derive(Args)]
pub struct SnapshotDiffArgs {
    /// First snapshot name
    from: String,
    /// Second snapshot name
    to: String,
}

pub fn run(args: SnapshotArgs, json: bool) {
    match args.command {
        SnapshotCommand::Create(a) => run_create(a, json),
        SnapshotCommand::Restore(a) => run_restore(a, json),
        SnapshotCommand::Diff(a) => run_diff(a, json),
        SnapshotCommand::List => run_list(json),
    }
}

fn run_create(args: SnapshotCreateArgs, json: bool) {
    match snapshots::create_snapshot(&args.name) {
        Ok(snap) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "created",
                        "name": snap.name,
                        "pins": snap.pins.len(),
                        "created_at": snap.created_at,
                    })
                );
            } else {
                output::success(&format!(
                    "Snapshot \"{}\" created with {} pins.",
                    snap.name.bold(),
                    snap.pins.len()
                ));
            }
        }
        Err(e) => {
            output::error(&e.to_string(), json);
            std::process::exit(1);
        }
    }
}

fn run_restore(args: SnapshotRestoreArgs, json: bool) {
    match snapshots::restore_snapshot(&args.name) {
        Ok(snap) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "restored",
                        "name": snap.name,
                        "pins": snap.pins.len(),
                    })
                );
            } else {
                output::success(&format!(
                    "Restored snapshot \"{}\": {} pins.",
                    snap.name.bold(),
                    snap.pins.len()
                ));
            }
        }
        Err(e) => {
            output::error(&e.to_string(), json);
            std::process::exit(1);
        }
    }
}

fn run_diff(args: SnapshotDiffArgs, json: bool) {
    match snapshots::diff_snapshots(&args.from, &args.to) {
        Ok(diffs) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "from": args.from,
                        "to": args.to,
                        "changes": diffs,
                    }))
                    .unwrap_or_default()
                );
            } else {
                if diffs.is_empty() {
                    eprintln!("{}", "No changes between snapshots.".dimmed());
                    return;
                }
                eprintln!(
                    "{}\n",
                    format!("{} → {} ({} changes):", args.from, args.to, diffs.len()).bold()
                );
                for diff in &diffs {
                    match &diff.change {
                        snapshots::DiffChange::Added { version } => {
                            let v = version.as_deref().unwrap_or("latest");
                            eprintln!("  {} {} ({})", "+".green(), diff.id.bold(), v);
                        }
                        snapshots::DiffChange::Removed { version } => {
                            let v = version.as_deref().unwrap_or("latest");
                            eprintln!("  {} {} ({})", "-".red(), diff.id.bold(), v);
                        }
                        snapshots::DiffChange::Changed {
                            from_version,
                            to_version,
                        } => {
                            let from = from_version.as_deref().unwrap_or("?");
                            let to = to_version.as_deref().unwrap_or("?");
                            eprintln!("  {} {} ({} → {})", "~".yellow(), diff.id.bold(), from, to);
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

fn run_list(json: bool) {
    let snapshots_list = snapshots::list_snapshots();

    if json {
        let items: Vec<serde_json::Value> = snapshots_list
            .iter()
            .map(|(name, created_at)| {
                serde_json::json!({
                    "name": name,
                    "created_at": created_at,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "snapshots": items,
                "total": snapshots_list.len(),
            }))
            .unwrap_or_default()
        );
    } else {
        if snapshots_list.is_empty() {
            eprintln!(
                "{}",
                "No snapshots. Use `chub snapshot create <name>` to create one.".dimmed()
            );
            return;
        }
        eprintln!(
            "{}",
            format!("{} snapshots:\n", snapshots_list.len()).bold()
        );
        for (name, created_at) in &snapshots_list {
            eprintln!("  {}  {}", name.bold(), created_at.dimmed());
        }
    }
}
