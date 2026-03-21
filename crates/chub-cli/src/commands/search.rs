use clap::Args;
use owo_colors::OwoColorize;

use chub_core::normalize::display_language;
use chub_core::registry::{
    get_entry, is_multi_source, list_entries, search_entries, MergedRegistry, SearchFilters,
    TaggedEntry,
};

#[derive(Args)]
pub struct SearchArgs {
    /// Search query (omit to list all)
    query: Option<String>,

    /// Filter by tags (comma-separated)
    #[arg(long)]
    tags: Option<String>,

    /// Filter by language
    #[arg(long)]
    lang: Option<String>,

    /// Max results
    #[arg(long, default_value = "20")]
    limit: usize,
}

fn format_entry_list(entries: &[TaggedEntry]) {
    let multi = is_multi_source();
    for entry in entries {
        let id = entry.id();
        let source = entry
            .source_quality()
            .map(|s| format!("[{}]", s).dimmed().to_string())
            .unwrap_or_default();
        let source_name = if multi {
            format!("({})", entry.source_name).cyan().to_string()
        } else {
            String::new()
        };
        let type_label = if entry.entry_type == "skill" {
            "[skill]".magenta().to_string()
        } else {
            "[doc]".blue().to_string()
        };
        let langs = entry
            .languages()
            .map(|ls| {
                ls.iter()
                    .map(|l| display_language(&l.language))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        let desc = entry.description();
        let desc_display = if desc.len() > 60 {
            format!("{}...", &desc[..57])
        } else {
            desc.to_string()
        };

        println!(
            "  {}  {}  {}  {} {}",
            id.bold(),
            type_label,
            langs.dimmed(),
            source,
            source_name
        );
        if !desc_display.is_empty() {
            println!("       {}", desc_display.dimmed());
        }
    }
}

fn format_entry_detail(entry: &TaggedEntry) {
    eprintln!("{}", entry.name().bold());
    if is_multi_source() {
        println!("  Source: {}", entry.source_name);
    }
    if let Some(q) = entry.source_quality() {
        println!("  Quality: {}", q);
    }
    if !entry.description().is_empty() {
        println!("  {}", entry.description().dimmed());
    }
    if !entry.tags().is_empty() {
        println!("  Tags: {}", entry.tags().join(", "));
    }
    eprintln!();

    if let Some(languages) = entry.languages() {
        for lang in languages {
            println!("  {}", display_language(&lang.language).bold());
            println!("    Recommended: {}", lang.recommended_version);
            for v in &lang.versions {
                let size = if v.size > 0 {
                    format!(" ({:.1} KB)", v.size as f64 / 1024.0)
                } else {
                    String::new()
                };
                println!("    {}{}  updated: {}", v.version, size, v.last_updated);
            }
        }
    } else if let Some(skill) = entry.as_skill() {
        let size = if skill.size > 0 {
            format!(" ({:.1} KB)", skill.size as f64 / 1024.0)
        } else {
            String::new()
        };
        println!("  Path: {}{}", skill.path, size);
        if !skill.last_updated.is_empty() {
            println!("  Updated: {}", skill.last_updated);
        }
        if !skill.files.is_empty() {
            println!("  Files: {}", skill.files.join(", "));
        }
    }
}

pub fn run(args: SearchArgs, json: bool, merged: &MergedRegistry) {
    let filters = SearchFilters {
        tags: args.tags,
        lang: args.lang,
    };

    // No query: list all
    if args.query.is_none() {
        let entries = list_entries(&filters, merged);
        let entries: Vec<_> = entries.into_iter().take(args.limit).collect();
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "results": entries.iter().map(simplify_entry).collect::<Vec<_>>(),
                    "total": entries.len(),
                })
            );
        } else {
            if entries.is_empty() {
                println!("{}", "No entries found.".yellow());
                return;
            }
            println!("{}", format!("{} entries:\n", entries.len()).bold());
            format_entry_list(&entries);
        }
        return;
    }

    let query = args.query.as_deref().unwrap();

    // Exact id match: show detail
    let result = get_entry(query, merged);
    if result.ambiguous {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "error": "ambiguous",
                    "alternatives": result.alternatives,
                })
            );
        } else {
            println!(
                "{}",
                format!("Multiple entries with id \"{}\". Be specific:", query).yellow()
            );
            for alt in &result.alternatives {
                println!("  {}", alt.bold());
            }
        }
        return;
    }

    if let Some(ref entry) = result.entry {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&simplify_entry(entry)).unwrap_or_default()
            );
        } else {
            format_entry_detail(entry);
        }
        return;
    }

    // Fuzzy search
    let results = search_entries(query, &filters, merged);
    let results: Vec<_> = results.into_iter().take(args.limit).collect();

    if json {
        println!(
            "{}",
            serde_json::json!({
                "results": results.iter().map(simplify_entry).collect::<Vec<_>>(),
                "total": results.len(),
                "query": query,
            })
        );
    } else {
        if results.is_empty() {
            println!("{}", format!("No results for \"{}\".", query).yellow());
            return;
        }
        println!(
            "{}",
            format!("{} results for \"{}\":\n", results.len(), query).bold()
        );
        format_entry_list(&results);
    }
}

fn simplify_entry(entry: &TaggedEntry) -> serde_json::Value {
    let mut val = serde_json::json!({
        "id": entry.id(),
        "name": entry.name(),
        "type": entry.entry_type,
        "description": entry.description(),
        "tags": entry.tags(),
    });
    if let Some(languages) = entry.languages() {
        val["languages"] =
            serde_json::json!(languages.iter().map(|l| &l.language).collect::<Vec<_>>());
    }
    val
}
