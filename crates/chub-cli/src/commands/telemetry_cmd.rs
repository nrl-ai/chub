use clap::{Args, Subcommand};
use owo_colors::OwoColorize;

use chub_core::identity::get_or_create_client_id;
use chub_core::team::analytics;
use chub_core::telemetry::{get_telemetry_url, is_feedback_enabled, is_telemetry_enabled};

#[derive(Args)]
pub struct TelemetryArgs {
    #[command(subcommand)]
    command: TelemetryCommand,
}

#[derive(Subcommand)]
enum TelemetryCommand {
    /// Show telemetry configuration and status
    Status,
    /// Export the raw event journal (JSONL)
    Export,
    /// Clear the local event journal
    Clear,
}

pub fn run(args: TelemetryArgs, json: bool) {
    match args.command {
        TelemetryCommand::Status => run_status(json),
        TelemetryCommand::Export => run_export(),
        TelemetryCommand::Clear => run_clear(json),
    }
}

fn run_status(json: bool) {
    let telemetry_enabled = is_telemetry_enabled();
    let feedback_enabled = is_feedback_enabled();
    let remote_url = get_telemetry_url();
    let journal_size = analytics::journal_size_bytes();
    let events = analytics::load_events();

    if json {
        let client_id = get_or_create_client_id();
        println!(
            "{}",
            serde_json::json!({
                "telemetry": telemetry_enabled,
                "feedback": feedback_enabled,
                "mode": if remote_url.is_some() { "local+remote" } else { "local" },
                "remote_url": remote_url,
                "client_id_prefix": client_id.as_deref().map(|s| &s[..s.len().min(8)]),
                "journal_events": events.len(),
                "journal_size_bytes": journal_size,
            })
        );
    } else {
        let tl = if telemetry_enabled {
            "enabled".green().to_string()
        } else {
            "disabled".red().to_string()
        };
        let fb = if feedback_enabled {
            "enabled".green().to_string()
        } else {
            "disabled".red().to_string()
        };
        let mode = if remote_url.is_some() {
            "local + remote".cyan().to_string()
        } else {
            "local only".yellow().to_string()
        };

        eprintln!("{}", "Telemetry Status".bold());
        eprintln!("  Telemetry: {}", tl);
        eprintln!("  Feedback:  {}", fb);
        eprintln!("  Mode:      {}", mode);
        if let Some(ref url) = remote_url {
            eprintln!("  Remote:    {}", url);
        } else {
            eprintln!(
                "  Remote:    {}",
                "(not configured — set telemetry_url in config to enable)".dimmed()
            );
        }
        if let Some(cid) = get_or_create_client_id() {
            let prefix = &cid[..cid.len().min(8)];
            eprintln!("  Client ID: {}...", prefix);
        }
        eprintln!(
            "  Journal:   {} events, {:.1} KB",
            events.len(),
            journal_size as f64 / 1024.0
        );

        eprintln!("\n{}", "Configuration:".bold());
        eprintln!("  Disable:  CHUB_TELEMETRY=0 or telemetry: false in config");
        eprintln!("  Remote:   Set telemetry_url in ~/.chub/config.yaml");
        eprintln!("  View:     chub stats --days 30");
        eprintln!("  Export:   chub telemetry export > events.jsonl");
        eprintln!("  Clear:    chub telemetry clear");
    }
}

fn run_export() {
    print!("{}", analytics::export_raw());
}

fn run_clear(json: bool) {
    let events = analytics::load_events();
    let count = events.len();

    if analytics::clear_journal() {
        if json {
            println!(
                "{}",
                serde_json::json!({ "status": "cleared", "events_removed": count })
            );
        } else {
            eprintln!("{} ({} events removed)", "Journal cleared.".green(), count);
        }
    } else if json {
        println!(
            "{}",
            serde_json::json!({ "status": "error", "reason": "could_not_delete" })
        );
    } else {
        eprintln!("{}", "Failed to clear journal.".red());
    }
}
