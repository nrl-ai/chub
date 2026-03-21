use clap::Args;
use owo_colors::OwoColorize;

use chub_core::annotations::{
    clear_annotation, list_annotations, read_annotation, write_annotation, AnnotationKind,
};

use crate::output;

#[derive(Args)]
pub struct AnnotateArgs {
    /// Entry ID
    id: Option<String>,

    /// Annotation text
    note: Option<String>,

    /// Remove annotation (respects --team flag: removes team annotation when --team is set)
    #[arg(long)]
    clear: bool,

    /// List all annotations
    #[arg(long)]
    list: bool,

    /// Save as team annotation (git-tracked in .chub/annotations/)
    #[arg(long)]
    team: bool,

    /// Save as personal annotation only (default)
    #[arg(long)]
    personal: bool,

    /// Author name for team annotations
    #[arg(long)]
    author: Option<String>,

    /// Annotation kind: note (default), issue, fix, practice
    #[arg(long, value_name = "KIND")]
    kind: Option<String>,

    /// Severity for issue annotations: high, medium, low
    #[arg(long, value_name = "LEVEL")]
    severity: Option<String>,
}

fn parse_kind(s: Option<&str>) -> AnnotationKind {
    s.and_then(AnnotationKind::parse)
        .unwrap_or(AnnotationKind::Note)
}

pub fn run(args: AnnotateArgs, json: bool) {
    if args.list {
        if args.team {
            // List team annotations
            let annotations = chub_core::team::team_annotations::list_team_annotations();
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&annotations).unwrap_or_default()
                );
            } else {
                if annotations.is_empty() {
                    eprintln!("No team annotations.");
                    return;
                }
                for a in &annotations {
                    eprintln!("{}", a.id.bold());
                    if !a.issues.is_empty() {
                        eprintln!("  {}", "Issues:".yellow());
                        for note in &a.issues {
                            let sev = note
                                .severity
                                .as_deref()
                                .map(|s| format!(" [{}]", s))
                                .unwrap_or_default();
                            eprintln!(
                                "    {} {}{} {}",
                                note.author.cyan(),
                                format!("({})", note.date).dimmed(),
                                sev.yellow(),
                                note.note
                            );
                        }
                    }
                    if !a.fixes.is_empty() {
                        eprintln!("  {}", "Fixes:".green());
                        for note in &a.fixes {
                            eprintln!(
                                "    {} {} {}",
                                note.author.cyan(),
                                format!("({})", note.date).dimmed(),
                                note.note
                            );
                        }
                    }
                    if !a.practices.is_empty() {
                        eprintln!("  {}", "Practices:".blue());
                        for note in &a.practices {
                            eprintln!(
                                "    {} {} {}",
                                note.author.cyan(),
                                format!("({})", note.date).dimmed(),
                                note.note
                            );
                        }
                    }
                    if !a.notes.is_empty() {
                        eprintln!("  {}", "Notes:".dimmed());
                        for note in &a.notes {
                            eprintln!(
                                "    {} {} {}",
                                note.author.cyan(),
                                format!("({})", note.date).dimmed(),
                                note.note
                            );
                        }
                    }
                    eprintln!();
                }
            }
        } else {
            let annotations = list_annotations();
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&annotations).unwrap_or_default()
                );
            } else {
                if annotations.is_empty() {
                    eprintln!("No annotations.");
                    return;
                }
                for a in &annotations {
                    eprintln!(
                        "{} {} [{}]",
                        a.id.bold(),
                        format!("({})", a.updated_at).dimmed(),
                        a.kind.as_str().cyan()
                    );
                    eprintln!("  {}", a.note);
                    eprintln!();
                }
            }
        }
        return;
    }

    let id = match args.id {
        Some(id) => id,
        None => {
            output::error(
                "Missing required argument: <id>. Run: chub annotate <id> <note> | chub annotate <id> --clear | chub annotate --list",
                json,
            );
            std::process::exit(1);
        }
    };

    if args.clear {
        // --clear respects --team: removes team annotation when --team is set
        let removed = if args.team {
            chub_core::team::team_annotations::clear_team_annotation(&id)
        } else {
            clear_annotation(&id)
        };
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "id": id,
                    "cleared": removed,
                    "scope": if args.team { "team" } else { "personal" },
                })
            );
        } else if removed {
            eprintln!(
                "{} annotation cleared for {}.",
                if args.team { "Team" } else { "Personal" },
                id.bold()
            );
        } else {
            eprintln!(
                "No {} annotation found for {}.",
                if args.team { "team" } else { "personal" },
                id.bold()
            );
        }
        return;
    }

    if let Some(note) = args.note {
        let kind = parse_kind(args.kind.as_deref());

        if args.team {
            // Write team annotation (append semantics — adds to the appropriate section)
            let author = args.author.unwrap_or_else(|| {
                std::env::var("USER")
                    .or_else(|_| std::env::var("USERNAME"))
                    .unwrap_or_else(|_| "unknown".to_string())
            });
            match chub_core::team::team_annotations::write_team_annotation(
                &id,
                &note,
                &author,
                kind.clone(),
                args.severity.clone(),
            ) {
                Some(_ann) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({
                                "status": "saved",
                                "id": id,
                                "scope": "team",
                                "kind": kind.as_str(),
                                "author": author,
                            })
                        );
                    } else {
                        output::success(&format!(
                            "Team {} saved for {} (by {})",
                            kind.as_str(),
                            id.bold(),
                            author
                        ));
                    }
                }
                None => {
                    output::error(
                        "Failed to save team annotation. Is .chub/ initialized?",
                        json,
                    );
                    std::process::exit(1);
                }
            }
        } else {
            // Write personal annotation (overwrite semantics — replaces previous note for this entry)
            let data = write_annotation(&id, &note, kind.clone(), args.severity.clone());
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&data).unwrap_or_default()
                );
            } else {
                eprintln!("{} saved for {}.", kind.as_str().cyan(), id.bold());
            }
        }
        return;
    }

    // Read mode: show existing annotation
    if args.team {
        if let Some(ann) = chub_core::team::team_annotations::read_team_annotation(&id) {
            if json {
                println!("{}", serde_json::to_string_pretty(&ann).unwrap_or_default());
            } else {
                eprintln!("{}", id.bold());
                if !ann.issues.is_empty() {
                    eprintln!("  {}", "Issues:".yellow());
                    for note in &ann.issues {
                        let sev = note
                            .severity
                            .as_deref()
                            .map(|s| format!(" [{}]", s))
                            .unwrap_or_default();
                        eprintln!(
                            "    {} {}{} {}",
                            note.author.cyan(),
                            format!("({})", note.date).dimmed(),
                            sev.yellow(),
                            note.note
                        );
                    }
                }
                if !ann.fixes.is_empty() {
                    eprintln!("  {}", "Fixes:".green());
                    for note in &ann.fixes {
                        eprintln!(
                            "    {} {} {}",
                            note.author.cyan(),
                            format!("({})", note.date).dimmed(),
                            note.note
                        );
                    }
                }
                if !ann.practices.is_empty() {
                    eprintln!("  {}", "Practices:".blue());
                    for note in &ann.practices {
                        eprintln!(
                            "    {} {} {}",
                            note.author.cyan(),
                            format!("({})", note.date).dimmed(),
                            note.note
                        );
                    }
                }
                for note in &ann.notes {
                    eprintln!(
                        "  {} {} {}",
                        note.author.cyan(),
                        format!("({})", note.date).dimmed(),
                        note.note
                    );
                }
            }
        } else if json {
            println!("{}", serde_json::json!({ "id": id, "notes": [] }));
        } else {
            eprintln!("No team annotation for {}.", id.bold());
        }
    } else {
        let existing = read_annotation(&id);
        if let Some(ann) = existing {
            if json {
                println!("{}", serde_json::to_string_pretty(&ann).unwrap_or_default());
            } else {
                eprintln!(
                    "{} {} [{}]",
                    ann.id.bold(),
                    format!("({})", ann.updated_at).dimmed(),
                    ann.kind.as_str().cyan()
                );
                eprintln!("{}", ann.note);
            }
        } else if json {
            println!("{}", serde_json::json!({ "id": id, "note": null }));
        } else {
            eprintln!("No annotation for {}.", id.bold());
        }
    }
}
