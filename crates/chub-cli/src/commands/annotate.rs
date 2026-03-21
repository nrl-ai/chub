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
}

pub fn run(args: AnnotateArgs, json: bool) {
    if args.list {
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
        let data = write_annotation(&id, &note);
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        } else {
            eprintln!("Annotation saved for {}.", id.bold());
        }
        return;
    }

    // Show existing annotation
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
