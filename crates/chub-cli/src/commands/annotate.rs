use clap::Args;
use owo_colors::OwoColorize;

use chub_core::annotations::{
    clear_annotation, list_annotations, read_annotation, write_annotation,
};

use crate::output;

#[derive(Args)]
pub struct AnnotateArgs {
    /// Entry ID
    id: Option<String>,

    /// Annotation text
    note: Option<String>,

    /// Remove annotation
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
                    for note in &a.notes {
                        eprintln!(
                            "  {} {} {}",
                            note.author.cyan(),
                            format!("({})", note.date).dimmed(),
                            note.note
                        );
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
                    eprintln!("{} {}", a.id.bold(), format!("({})", a.updated_at).dimmed());
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
        let removed = clear_annotation(&id);
        if json {
            println!("{}", serde_json::json!({ "id": id, "cleared": removed }));
        } else if removed {
            eprintln!("Annotation cleared for {}.", id.bold());
        } else {
            eprintln!("No annotation found for {}.", id.bold());
        }
        return;
    }

    if let Some(note) = args.note {
        if args.team {
            // Write team annotation
            let author = args.author.unwrap_or_else(|| {
                std::env::var("USER")
                    .or_else(|_| std::env::var("USERNAME"))
                    .unwrap_or_else(|_| "unknown".to_string())
            });
            match chub_core::team::team_annotations::write_team_annotation(&id, &note, &author) {
                Some(_ann) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({"status": "saved", "id": id, "type": "team", "author": author})
                        );
                    } else {
                        output::success(&format!(
                            "Team annotation saved for {} (by {})",
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
            let data = write_annotation(&id, &note);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&data).unwrap_or_default()
                );
            } else {
                eprintln!("Annotation saved for {}.", id.bold());
            }
        }
        return;
    }

    // Show existing annotation (merged if not --personal)
    if args.team {
        if let Some(ann) = chub_core::team::team_annotations::read_team_annotation(&id) {
            if json {
                println!("{}", serde_json::to_string_pretty(&ann).unwrap_or_default());
            } else {
                eprintln!("{}", id.bold());
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
                    "{} {}",
                    ann.id.bold(),
                    format!("({})", ann.updated_at).dimmed()
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
