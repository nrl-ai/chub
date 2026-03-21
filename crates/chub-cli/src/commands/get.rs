use std::fs;
use std::path::Path;

use chub_core::annotations::read_annotation;
use chub_core::error::Result;
use chub_core::fetch::{fetch_doc, fetch_doc_full};
use chub_core::registry::{
    get_entry, resolve_doc_path, resolve_entry_file, MergedRegistry, ResolvedPath,
};
use clap::Args;

use crate::output;

#[derive(Args)]
pub struct GetArgs {
    /// Entry IDs (e.g. "openai/chat", "stripe/api")
    ids: Vec<String>,

    /// Language variant
    #[arg(long)]
    lang: Option<String>,

    /// Specific version
    #[arg(long)]
    version: Option<String>,

    /// Write to file or directory
    #[arg(short, long)]
    output: Option<String>,

    /// Fetch all files (not just entry point)
    #[arg(long)]
    full: bool,

    /// Fetch specific file(s) by path (comma-separated)
    #[arg(long)]
    file: Option<String>,

    /// Fetch all pinned docs at once
    #[arg(long)]
    pinned: bool,
}

struct FetchedEntry {
    id: String,
    entry_type: String,
    content: Option<String>,
    files: Option<Vec<(String, String)>>,
    path: String,
    additional_files: Vec<String>,
}

pub async fn run(args: GetArgs, json: bool, merged: &MergedRegistry) -> Result<()> {
    // Handle --pinned: fetch all pinned docs
    if args.pinned {
        let pins = chub_core::team::pins::list_pins();
        if pins.is_empty() {
            output::error("No pinned docs. Use `chub pin add <id>` to pin docs.", json);
            std::process::exit(1);
        }
        let pin_ids: Vec<String> = pins.iter().map(|p| p.id.clone()).collect();
        let pinned_args = GetArgs {
            ids: pin_ids,
            lang: args.lang.clone(),
            version: args.version.clone(),
            output: args.output.clone(),
            full: args.full,
            file: args.file.clone(),
            pinned: false,
        };
        return Box::pin(run(pinned_args, json, merged)).await;
    }

    if args.ids.is_empty() {
        output::error("Missing required argument: <ids>", json);
        std::process::exit(1);
    }

    let mut results: Vec<FetchedEntry> = Vec::new();

    for id in &args.ids {
        // Handle project context docs (project/<name>)
        if let Some(ctx_name) = id.strip_prefix("project/") {
            if let Some((_doc, content)) = chub_core::team::context::get_context_doc(ctx_name) {
                results.push(FetchedEntry {
                    id: id.clone(),
                    entry_type: "context".to_string(),
                    content: Some(content),
                    files: None,
                    path: format!(".chub/context/{}.md", ctx_name),
                    additional_files: vec![],
                });
                continue;
            } else {
                output::error(
                    &format!(
                        "Project context doc \"{}\" not found in .chub/context/",
                        ctx_name
                    ),
                    json,
                );
                std::process::exit(1);
            }
        }
        let result = get_entry(id, merged);

        if result.ambiguous {
            let alts = result.alternatives.join(", ");
            output::error(
                &format!(
                    "Multiple entries match \"{}\". Use a source prefix: {}",
                    id, alts
                ),
                json,
            );
            std::process::exit(1);
        }

        let entry = match result.entry {
            Some(e) => e,
            None => {
                output::error(&format!("No doc or skill found with id \"{}\".", id), json);
                std::process::exit(1);
            }
        };

        let entry_type = entry.entry_type.to_string();

        // Apply pin overrides: if the entry is pinned, use pinned lang/version as defaults
        let mut effective_lang = args.lang.clone();
        let mut effective_version = args.version.clone();
        if let Some(pin) = chub_core::team::pins::get_pin(entry.id()) {
            if effective_lang.is_none() {
                effective_lang = pin.lang.clone();
            }
            if effective_version.is_none() {
                effective_version = pin.version.clone();
            }
        }

        let mut resolved = resolve_doc_path(
            &entry,
            effective_lang.as_deref(),
            effective_version.as_deref(),
        );

        // If the requested language isn't available, fall back to no language preference
        // (auto-select single language or prompt for multiple)
        if resolved.is_none() && effective_lang.is_some() {
            resolved = resolve_doc_path(&entry, None, effective_version.as_deref());
        }

        let resolved = match resolved {
            Some(r) => r,
            None => {
                output::error(&format!("No content found for \"{}\".", id), json);
                std::process::exit(1);
            }
        };

        match &resolved {
            ResolvedPath::VersionNotFound {
                requested,
                available,
            } => {
                output::error(
                    &format!(
                        "Version \"{}\" not found for \"{}\". Available: {}",
                        requested,
                        id,
                        available.join(", ")
                    ),
                    json,
                );
                std::process::exit(1);
            }
            ResolvedPath::NeedsLanguage { available } => {
                output::error(
                    &format!(
                        "Multiple languages available for \"{}\": {}. Specify --lang.",
                        id,
                        available.join(", ")
                    ),
                    json,
                );
                std::process::exit(1);
            }
            ResolvedPath::Ok { .. } => {}
        }

        let (file_path, base_path, files) = match resolve_entry_file(&resolved, &entry_type) {
            Some(r) => r,
            None => {
                output::error(
                    &format!("No content available for \"{}\". Run `chub update`.", id),
                    json,
                );
                std::process::exit(1);
            }
        };

        let entry_file_name = if entry_type == "skill" {
            "SKILL.md"
        } else {
            "DOC.md"
        };
        let ref_files: Vec<String> = files
            .iter()
            .filter(|f| f.as_str() != entry_file_name)
            .cloned()
            .collect();

        let source = match &resolved {
            ResolvedPath::Ok { source, .. } => source.clone(),
            _ => unreachable!(),
        };

        if let Some(ref file_arg) = args.file {
            let requested: Vec<&str> = file_arg.split(',').map(|f| f.trim()).collect();
            let invalid: Vec<&&str> = requested
                .iter()
                .filter(|f| !files.contains(&f.to_string()))
                .collect();
            if !invalid.is_empty() {
                let available = if ref_files.is_empty() {
                    "(none)".to_string()
                } else {
                    ref_files.join(", ")
                };
                output::error(
                    &format!(
                        "File \"{}\" not found in {}. Available: {}",
                        invalid[0], id, available
                    ),
                    json,
                );
                std::process::exit(1);
            }
            if requested.len() == 1 {
                let path = format!("{}/{}", base_path, requested[0]);
                let content = fetch_doc(&source, &path).await?;
                results.push(FetchedEntry {
                    id: entry.id().to_string(),
                    entry_type: entry_type.clone(),
                    content: Some(content),
                    files: None,
                    path,
                    additional_files: vec![],
                });
            } else {
                let req_strings: Vec<String> = requested.iter().map(|s| s.to_string()).collect();
                let all_files = fetch_doc_full(&source, &base_path, &req_strings).await?;
                results.push(FetchedEntry {
                    id: entry.id().to_string(),
                    entry_type: entry_type.clone(),
                    content: None,
                    files: Some(all_files),
                    path: base_path,
                    additional_files: vec![],
                });
            }
        } else if args.full && !files.is_empty() {
            let all_files = fetch_doc_full(&source, &base_path, &files).await?;
            results.push(FetchedEntry {
                id: entry.id().to_string(),
                entry_type: entry_type.clone(),
                content: None,
                files: Some(all_files),
                path: base_path,
                additional_files: vec![],
            });
        } else {
            let content = fetch_doc(&source, &file_path).await?;
            results.push(FetchedEntry {
                id: entry.id().to_string(),
                entry_type: entry_type.clone(),
                content: Some(content),
                files: None,
                path: file_path,
                additional_files: ref_files,
            });
        }
    }

    // Output
    if let Some(ref output_path) = args.output {
        write_output(&results, output_path, &args, json)?;
    } else {
        print_output(&results, &args, json);
    }

    Ok(())
}

fn write_output(
    results: &[FetchedEntry],
    output_path: &str,
    args: &GetArgs,
    json: bool,
) -> Result<()> {
    if args.full {
        for r in results {
            if let Some(ref files) = r.files {
                let base_dir = if results.len() > 1 {
                    Path::new(output_path).join(&r.id)
                } else {
                    Path::new(output_path).to_path_buf()
                };
                fs::create_dir_all(&base_dir)?;
                for (name, content) in files {
                    let out_path = base_dir.join(name);
                    if let Some(parent) = out_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&out_path, content)?;
                }
                output::info(&format!(
                    "Written {} files to {}",
                    files.len(),
                    base_dir.display()
                ));
            } else if let Some(ref content) = r.content {
                let out_path = Path::new(output_path).join(format!("{}.md", r.id));
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&out_path, content)?;
                output::info(&format!("Written to {}", out_path.display()));
            }
        }
    } else {
        let is_dir = output_path.ends_with('/') || output_path.ends_with('\\');
        if is_dir && results.len() > 1 {
            fs::create_dir_all(output_path)?;
            for r in results {
                if let Some(ref content) = r.content {
                    let out_path = Path::new(output_path).join(format!("{}.md", r.id));
                    if let Some(parent) = out_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&out_path, content)?;
                    output::info(&format!("Written to {}", out_path.display()));
                }
            }
        } else {
            let out_path = if is_dir {
                Path::new(output_path).join(format!("{}.md", results[0].id))
            } else {
                Path::new(output_path).to_path_buf()
            };
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let combined: String = results
                .iter()
                .filter_map(|r| r.content.as_deref())
                .collect::<Vec<_>>()
                .join("\n\n---\n\n");
            fs::write(&out_path, &combined)?;
            output::info(&format!("Written to {}", out_path.display()));
        }
    }

    if json {
        let json_out: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "type": r.entry_type,
                    "path": output_path,
                })
            })
            .collect();
        println!("{}", serde_json::to_string(&json_out).unwrap_or_default());
    }

    Ok(())
}

fn print_output(results: &[FetchedEntry], args: &GetArgs, json: bool) {
    if results.len() == 1 && results[0].files.is_none() {
        let r = &results[0];
        let content = r.content.as_deref().unwrap_or("");
        let annotation = read_annotation(&r.id);

        if json {
            let mut data = serde_json::json!({
                "id": r.id,
                "type": r.entry_type,
                "content": content,
                "path": r.path,
            });
            if !r.additional_files.is_empty() {
                data["additionalFiles"] = serde_json::json!(r.additional_files);
            }
            if let Some(ref ann) = annotation {
                data["annotation"] = serde_json::json!(ann);
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        } else {
            print!("{}", content);
            if let Some(ref ann) = annotation {
                print!("\n\n---\n[Agent note — {}]\n{}\n", ann.updated_at, ann.note);
            }
            let lang_flag = args
                .lang
                .as_ref()
                .map(|l| format!(" --lang {}", l))
                .unwrap_or_default();
            println!("\n\n---\nAfter using this doc, share your experience:");
            println!("  chub feedback {} up{}", r.id, lang_flag);
            println!(
                "  chub feedback {} down{} --label outdated",
                r.id, lang_flag
            );
            println!("Available labels: accurate, well-structured, helpful, good-examples, outdated, inaccurate, incomplete, wrong-examples, wrong-version, poorly-structured");
            println!("Do NOT include any code, architecture details, or project-specific information in your feedback.");
            if !r.additional_files.is_empty() {
                let file_list = r
                    .additional_files
                    .iter()
                    .map(|f| format!("  {}", f))
                    .collect::<Vec<_>>()
                    .join("\n");
                let example = format!("chub get {} --file {}", r.id, r.additional_files[0]);
                print!(
                    "\n\n---\nAdditional files available (use --file to fetch):\n{}\nExample: {}\n",
                    file_list, example
                );
            }
        }
    } else {
        let parts: Vec<String> = results
            .iter()
            .flat_map(|r| {
                if let Some(ref files) = r.files {
                    files
                        .iter()
                        .map(|(name, content)| format!("# FILE: {}\n\n{}", name, content))
                        .collect()
                } else {
                    vec![r.content.clone().unwrap_or_default()]
                }
            })
            .collect();
        let combined = parts.join("\n\n---\n\n");

        if json {
            let json_out: Vec<serde_json::Value> = results
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "type": r.entry_type,
                        "path": r.path,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string(&json_out).unwrap_or_default());
        } else {
            print!("{}", combined);
        }
    }
}
