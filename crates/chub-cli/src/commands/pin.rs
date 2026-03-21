use clap::{Args, Subcommand};
use owo_colors::OwoColorize;

use chub_core::team::pins;

use crate::output;

#[derive(Args)]
pub struct PinArgs {
    #[command(subcommand)]
    command: PinCommand,
}

#[derive(Subcommand)]
pub enum PinCommand {
    /// Pin a doc to the project
    Add(PinAddArgs),
    /// Remove a pinned doc
    Remove(PinRemoveArgs),
    /// List all pinned docs
    List,
    /// Fetch all pinned docs at once
    Get(PinGetArgs),
}

#[derive(Args)]
pub struct PinAddArgs {
    /// Entry ID (e.g. "openai/chat")
    id: String,
    /// Language variant
    #[arg(long)]
    lang: Option<String>,
    /// Specific version to lock
    #[arg(long)]
    version: Option<String>,
    /// Reason for pinning
    #[arg(long)]
    reason: Option<String>,
    /// Source name for private registry
    #[arg(long)]
    source: Option<String>,
}

#[derive(Args)]
pub struct PinRemoveArgs {
    /// Entry ID to unpin
    id: String,
}

#[derive(Args)]
pub struct PinGetArgs {
    /// Language variant (overrides pin lang)
    #[arg(long)]
    lang: Option<String>,
}

pub fn run(args: PinArgs, json: bool) {
    match args.command {
        PinCommand::Add(add_args) => run_add(add_args, json),
        PinCommand::Remove(rm_args) => run_remove(rm_args, json),
        PinCommand::List => run_list(json),
        PinCommand::Get(_) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({"info": "Use `chub get --pinned` to fetch all pinned docs."})
                );
            } else {
                eprintln!(
                    "Use {} to fetch all pinned docs.",
                    "chub get --pinned".bold()
                );
            }
        }
    }
}

fn run_add(args: PinAddArgs, json: bool) {
    match pins::add_pin(
        &args.id,
        args.lang.clone(),
        args.version.clone(),
        args.reason.clone(),
        args.source.clone(),
    ) {
        Ok(()) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "pinned",
                        "id": args.id,
                        "lang": args.lang,
                        "version": args.version,
                    })
                );
            } else {
                let mut msg = format!("Pinned {}", args.id.bold());
                if let Some(ref lang) = args.lang {
                    msg.push_str(&format!(" ({})", lang));
                }
                if let Some(ref ver) = args.version {
                    msg.push_str(&format!(" v{}", ver));
                }
                output::success(&msg);
            }
        }
        Err(e) => {
            output::error(&e.to_string(), json);
            std::process::exit(1);
        }
    }
}

fn run_remove(args: PinRemoveArgs, json: bool) {
    match pins::remove_pin(&args.id) {
        Ok(true) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({"status": "unpinned", "id": args.id})
                );
            } else {
                output::success(&format!("Unpinned {}", args.id.bold()));
            }
        }
        Ok(false) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({"status": "not_found", "id": args.id})
                );
            } else {
                eprintln!("No pin found for {}.", args.id.bold());
            }
        }
        Err(e) => {
            output::error(&e.to_string(), json);
            std::process::exit(1);
        }
    }
}

fn run_list(json: bool) {
    let pins = pins::list_pins();

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "pins": pins,
                "total": pins.len(),
            }))
            .unwrap_or_default()
        );
    } else {
        if pins.is_empty() {
            eprintln!(
                "{}",
                "No pins configured. Use `chub pin add <id>` to pin a doc.".dimmed()
            );
            return;
        }
        eprintln!("{}", format!("{} pinned docs:\n", pins.len()).bold());
        for pin in &pins {
            let mut line = format!("  {}", pin.id.bold());
            if let Some(ref lang) = pin.lang {
                line.push_str(&format!("  {}", lang.cyan()));
            }
            if let Some(ref ver) = pin.version {
                line.push_str(&format!("  v{}", ver));
            }
            if let Some(ref src) = pin.source {
                line.push_str(&format!("  [{}]", src.dimmed()));
            }
            eprintln!("{}", line);
            if let Some(ref reason) = pin.reason {
                eprintln!("       {}", reason.dimmed());
            }
        }
    }
}
