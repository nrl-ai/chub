use clap::Args;
use owo_colors::OwoColorize;

use chub_core::registry::MergedRegistry;
use chub_core::team::detect;

use crate::output;

#[derive(Args)]
pub struct DetectArgs {
    /// Auto-pin all detected docs
    #[arg(long)]
    pin: bool,

    /// Show new deps since last detect
    #[arg(long)]
    diff: bool,
}

pub fn run(args: DetectArgs, json: bool, merged: &MergedRegistry) {
    let cwd = std::env::current_dir().unwrap_or_default();
    let deps = detect::detect_dependencies(&cwd);

    if deps.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({ "dependencies": [], "matches": [] })
            );
        } else {
            eprintln!(
                "{}",
                "No dependency files found (package.json, Cargo.toml, requirements.txt, etc.)."
                    .yellow()
            );
        }
        return;
    }

    // Build doc ID list from registry
    let doc_ids: Vec<(String, String)> = merged
        .docs
        .iter()
        .map(|e| (e.id().to_string(), e.name().to_string()))
        .collect();

    let matches = detect::match_deps_to_docs(&deps, &doc_ids);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "dependencies": deps.len(),
                "matches": matches.iter().map(|m| serde_json::json!({
                    "dependency": m.dep.name,
                    "language": m.dep.language,
                    "version": m.dep.version,
                    "doc_id": m.doc_id,
                    "doc_name": m.doc_name,
                    "confidence": m.confidence,
                })).collect::<Vec<_>>(),
                "unmatched": deps.iter()
                    .filter(|d| !matches.iter().any(|m| m.dep.name == d.name))
                    .map(|d| &d.name)
                    .collect::<Vec<_>>(),
            }))
            .unwrap_or_default()
        );
    } else {
        eprintln!(
            "Detected {} dependencies with {} available docs:\n",
            deps.len(),
            matches.len()
        );

        for m in &matches {
            let ver = m
                .dep
                .version
                .as_deref()
                .map(|v| format!(" ({})", v))
                .unwrap_or_default();
            let conf = if m.confidence >= 0.8 {
                "[pinnable]".green().to_string()
            } else {
                "[partial match]".yellow().to_string()
            };
            eprintln!(
                "  {} {}{}  →  {} {}",
                m.dep.name.bold(),
                m.dep.language.dimmed(),
                ver.dimmed(),
                m.doc_id.cyan(),
                conf,
            );
        }

        // Show unmatched
        let unmatched: Vec<&str> = deps
            .iter()
            .filter(|d| !matches.iter().any(|m| m.dep.name == d.name))
            .map(|d| d.name.as_str())
            .collect();
        if !unmatched.is_empty() {
            eprintln!();
            for name in &unmatched {
                eprintln!("  {} {}", "✗".red(), name.dimmed());
            }
        }

        if !matches.is_empty() {
            eprintln!("\nPin all? {}", "chub detect --pin".bold());
        }
    }

    // Auto-pin if requested
    if args.pin {
        let mut pinned = 0;
        for m in &matches {
            if m.confidence >= 0.5 {
                let lang = if m.dep.language == "javascript" || m.dep.language == "python" {
                    Some(m.dep.language.clone())
                } else {
                    None
                };
                if chub_core::team::pins::add_pin(&m.doc_id, lang, None, None, None).is_ok() {
                    pinned += 1;
                }
            }
        }
        if !json {
            output::success(&format!("Pinned {} docs.", pinned));
        }
    }
}
