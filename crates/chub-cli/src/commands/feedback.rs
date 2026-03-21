use clap::Args;
use owo_colors::OwoColorize;

use chub_core::identity::get_or_create_client_id;
use chub_core::registry::{get_entry, MergedRegistry};
use chub_core::telemetry::{
    get_telemetry_url, is_feedback_enabled, is_telemetry_enabled, send_feedback, FeedbackOpts,
    VALID_LABELS,
};

use crate::output;

#[derive(Args)]
pub struct FeedbackArgs {
    /// Entry ID
    id: Option<String>,

    /// Rating: up or down
    rating: Option<String>,

    /// Optional comment
    comment: Option<String>,

    /// Explicit type: doc or skill
    #[arg(long, name = "type")]
    entry_type: Option<String>,

    /// Language variant of the doc
    #[arg(long)]
    lang: Option<String>,

    /// Version of the doc
    #[arg(long)]
    doc_version: Option<String>,

    /// Specific file within the entry
    #[arg(long)]
    file: Option<String>,

    /// Feedback label (repeatable)
    #[arg(long = "label")]
    labels: Vec<String>,

    /// AI coding tool name
    #[arg(long)]
    agent: Option<String>,

    /// LLM model name
    #[arg(long)]
    model: Option<String>,

    /// Show feedback and telemetry status
    #[arg(long)]
    status: bool,
}

pub async fn run(args: FeedbackArgs, json: bool, merged: Option<&MergedRegistry>) {
    if args.status {
        run_status(json).await;
        return;
    }

    let id = match &args.id {
        Some(id) => id.clone(),
        None => {
            output::error(
                "Missing required arguments: <id> and <rating>. Run: chub feedback <id> <up|down> [comment]",
                json,
            );
            std::process::exit(1);
        }
    };

    let rating = match &args.rating {
        Some(r) => r.clone(),
        None => {
            output::error(
                "Missing required arguments: <id> and <rating>. Run: chub feedback <id> <up|down> [comment]",
                json,
            );
            std::process::exit(1);
        }
    };

    if rating != "up" && rating != "down" {
        output::error("Rating must be \"up\" or \"down\".", json);
        std::process::exit(1);
    }

    if !is_feedback_enabled() {
        if json {
            println!(
                "{}",
                serde_json::json!({ "status": "skipped", "reason": "feedback_disabled" })
            );
        } else {
            eprintln!(
                "{}",
                "Feedback is disabled. Enable with: feedback: true in ~/.chub/config.yaml".yellow()
            );
        }
        return;
    }

    // Auto-detect entry type
    let mut entry_type = args.entry_type.clone();
    let mut doc_lang = args.lang.clone();
    let mut doc_version = args.doc_version.clone();
    let mut source = None;

    if let Some(merged) = merged {
        let result = get_entry(&id, merged);
        if let Some(ref entry) = result.entry {
            if entry_type.is_none() {
                entry_type = Some(entry.entry_type.to_string());
            }
            source = Some(entry.source_name.clone());

            if let Some(languages) = entry.languages() {
                if doc_lang.is_none() && languages.len() == 1 {
                    doc_lang = Some(languages[0].language.clone());
                }
                if doc_version.is_none() {
                    let lang_obj = languages
                        .iter()
                        .find(|l| Some(&l.language) == doc_lang.as_ref())
                        .or(languages.first());
                    if let Some(l) = lang_obj {
                        doc_version = Some(l.recommended_version.clone());
                    }
                }
            }
        }
    }
    let entry_type = entry_type.unwrap_or_else(|| "doc".to_string());

    // Parse labels
    let labels: Option<Vec<String>> = if args.labels.is_empty() {
        None
    } else {
        let valid: Vec<String> = args
            .labels
            .iter()
            .map(|l| l.trim().to_lowercase())
            .filter(|l| VALID_LABELS.contains(&l.as_str()))
            .collect();
        if valid.is_empty() {
            None
        } else {
            Some(valid)
        }
    };

    let result = send_feedback(
        &id,
        &entry_type,
        &rating,
        FeedbackOpts {
            comment: args.comment,
            doc_lang: doc_lang.clone(),
            doc_version: doc_version.clone(),
            target_file: args.file.clone(),
            labels,
            agent: args.agent,
            model: args.model,
            cli_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            source,
        },
    )
    .await;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_default()
        );
    } else if result.status == "sent" {
        let mut parts = vec![format!("Feedback recorded for {}", id).green().to_string()];
        if let Some(ref lang) = doc_lang {
            parts.push(format!("lang={}", lang).dimmed().to_string());
        }
        if let Some(ref ver) = doc_version {
            parts.push(format!("version={}", ver).dimmed().to_string());
        }
        if let Some(ref file) = args.file {
            parts.push(format!("file={}", file).dimmed().to_string());
        }
        eprintln!("{}", parts.join(" "));
    } else if result.status == "error" {
        let reason = result
            .reason
            .as_deref()
            .or(result.code.map(|_| "HTTP error").as_ref().map(|s| *s))
            .unwrap_or("unknown");
        eprintln!("{}", format!("Failed to send feedback: {}", reason).red());
    }
}

async fn run_status(json: bool) {
    let feedback_enabled = is_feedback_enabled();
    let telemetry_enabled = is_telemetry_enabled();

    if json {
        let client_id = get_or_create_client_id();
        println!(
            "{}",
            serde_json::json!({
                "feedback": feedback_enabled,
                "telemetry": telemetry_enabled,
                "client_id_prefix": client_id.as_deref().map(|s| &s[..s.len().min(8)]),
                "endpoint": get_telemetry_url(),
                "valid_labels": VALID_LABELS,
            })
        );
    } else {
        let fb = if feedback_enabled {
            "enabled".green().to_string()
        } else {
            "disabled".red().to_string()
        };
        let tl = if telemetry_enabled {
            "enabled".green().to_string()
        } else {
            "disabled".red().to_string()
        };
        eprintln!("Feedback:  {}", fb);
        eprintln!("Telemetry: {}", tl);
        if let Some(cid) = get_or_create_client_id() {
            let prefix = &cid[..cid.len().min(8)];
            eprintln!("Client ID: {}...", prefix);
        }
        eprintln!("Endpoint:  {}", get_telemetry_url());
        eprintln!("Labels:    {}", VALID_LABELS.join(", "));
    }
}
