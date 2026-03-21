use clap::Args;
use owo_colors::OwoColorize;

use chub_core::team::context;

#[derive(Args)]
pub struct ContextArgs {
    /// Task description to find relevant context for
    query: Option<String>,

    /// List all project context docs
    #[arg(long)]
    list: bool,
}

pub fn run(args: ContextArgs, json: bool) {
    if args.list || args.query.is_none() {
        run_list(json);
        return;
    }

    // Task-scoped context: show relevant docs for the given task
    let query = args.query.as_deref().unwrap_or("");
    let docs = context::discover_context_docs();

    if docs.is_empty() {
        if json {
            println!("{}", serde_json::json!({"docs": [], "query": query}));
        } else {
            eprintln!(
                "{}",
                "No project context docs found in .chub/context/".dimmed()
            );
        }
        return;
    }

    // Simple keyword matching for relevance
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut scored: Vec<(&context::ContextDoc, f64)> = docs
        .iter()
        .map(|doc| {
            let mut score = 0.0f64;
            let name_lower = doc.name.to_lowercase();
            let desc_lower = doc.description.to_lowercase();

            for word in &query_words {
                if name_lower.contains(word) {
                    score += 10.0;
                }
                if desc_lower.contains(word) {
                    score += 5.0;
                }
                for tag in &doc.tags {
                    if tag.to_lowercase().contains(word) {
                        score += 8.0;
                    }
                }
            }
            (doc, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Show all docs, ranked by relevance
    if json {
        let items: Vec<serde_json::Value> = scored
            .iter()
            .map(|(doc, score)| {
                let stem = doc.file.strip_suffix(".md").unwrap_or(&doc.file);
                serde_json::json!({
                    "id": format!("project/{}", stem),
                    "name": doc.name,
                    "description": doc.description,
                    "relevance": score,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "query": query,
                "docs": items,
            }))
            .unwrap_or_default()
        );
    } else {
        eprintln!(
            "Context for \"{}\": {} doc(s)\n",
            query.bold(),
            scored.len()
        );
        for (doc, score) in &scored {
            let stem = doc.file.strip_suffix(".md").unwrap_or(&doc.file);
            let relevance = if *score > 0.0 {
                format!(" (relevance: {:.0})", score).dimmed().to_string()
            } else {
                String::new()
            };
            eprintln!(
                "  {}  {}{}",
                format!("project/{}", stem).cyan(),
                doc.description.dimmed(),
                relevance,
            );
        }
    }
}

fn run_list(json: bool) {
    let docs = context::list_context_docs();

    if json {
        let items: Vec<serde_json::Value> = docs
            .iter()
            .map(|doc| {
                let stem = doc.file.strip_suffix(".md").unwrap_or(&doc.file);
                serde_json::json!({
                    "id": format!("project/{}", stem),
                    "name": doc.name,
                    "description": doc.description,
                    "tags": doc.tags,
                    "file": doc.file,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "docs": items,
                "total": docs.len(),
            }))
            .unwrap_or_default()
        );
    } else {
        if docs.is_empty() {
            eprintln!(
                "{}",
                "No project context docs. Add .md files to .chub/context/".dimmed()
            );
            return;
        }
        eprintln!(
            "{}",
            format!("{} project context docs:\n", docs.len()).bold()
        );
        for doc in &docs {
            let stem = doc.file.strip_suffix(".md").unwrap_or(&doc.file);
            eprintln!("  {}", format!("project/{}", stem).cyan());
            if !doc.description.is_empty() {
                eprintln!("    {}", doc.description.dimmed());
            }
        }
    }
}
