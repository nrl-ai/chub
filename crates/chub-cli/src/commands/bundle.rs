use std::path::PathBuf;

use clap::{Args, Subcommand};
use owo_colors::OwoColorize;

use chub_core::team::bundles;

use crate::output;

#[derive(Args)]
pub struct BundleArgs {
    #[command(subcommand)]
    command: BundleCommand,
}

#[derive(Subcommand)]
pub enum BundleCommand {
    /// Create a new bundle
    Create(BundleCreateArgs),
    /// Install a bundle (pin all its entries)
    Install(BundleInstallArgs),
    /// List available bundles
    List,
}

#[derive(Args)]
pub struct BundleCreateArgs {
    /// Bundle name
    name: String,
    /// Description
    #[arg(long)]
    description: Option<String>,
    /// Author name
    #[arg(long)]
    author: Option<String>,
    /// Entry IDs (comma-separated)
    #[arg(long)]
    entries: String,
    /// Notes
    #[arg(long)]
    notes: Option<String>,
}

#[derive(Args)]
pub struct BundleInstallArgs {
    /// Bundle name or path to bundle YAML
    name: String,
}

pub fn run(args: BundleArgs, json: bool) {
    match args.command {
        BundleCommand::Create(create_args) => run_create(create_args, json),
        BundleCommand::Install(install_args) => run_install(install_args, json),
        BundleCommand::List => run_list(json),
    }
}

fn run_create(args: BundleCreateArgs, json: bool) {
    let entries: Vec<String> = args
        .entries
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    match bundles::create_bundle(
        &args.name,
        args.description.as_deref(),
        args.author.as_deref(),
        entries,
        args.notes.as_deref(),
    ) {
        Ok(path) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "created",
                        "path": path.display().to_string(),
                    })
                );
            } else {
                output::success(&format!("Bundle created: {}", path.display()));
            }
        }
        Err(e) => {
            output::error(&e.to_string(), json);
            std::process::exit(1);
        }
    }
}

fn run_install(args: BundleInstallArgs, json: bool) {
    // Try as path first, then as name
    let bundle = if PathBuf::from(&args.name).exists() {
        bundles::load_bundle(&PathBuf::from(&args.name))
    } else {
        bundles::load_bundle_by_name(&args.name)
    };

    match bundle {
        Ok(b) => {
            let name = b.name.clone();
            match bundles::install_bundle(&b) {
                Ok(pinned) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({
                                "status": "installed",
                                "bundle": name,
                                "pinned": pinned,
                            })
                        );
                    } else {
                        output::success(&format!(
                            "Installed bundle \"{}\": pinned {} entries.",
                            name.bold(),
                            pinned.len()
                        ));
                        for id in &pinned {
                            eprintln!("  + {}", id);
                        }
                    }
                }
                Err(e) => {
                    output::error(&e.to_string(), json);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            output::error(&e.to_string(), json);
            std::process::exit(1);
        }
    }
}

fn run_list(json: bool) {
    let bundles_list = bundles::list_bundles();

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "bundles": bundles_list,
                "total": bundles_list.len(),
            }))
            .unwrap_or_default()
        );
    } else {
        if bundles_list.is_empty() {
            eprintln!("{}", "No bundles found in .chub/bundles/".dimmed());
            return;
        }
        eprintln!("{}", format!("{} bundles:\n", bundles_list.len()).bold());
        for b in &bundles_list {
            eprintln!("  {}", b.name.bold());
            if let Some(ref desc) = b.description {
                eprintln!("    {}", desc.dimmed());
            }
            eprintln!("    Entries: {}", b.entries.join(", "));
        }
    }
}
