use clap::Args;
use owo_colors::OwoColorize;

use crate::output;

#[derive(Args)]
pub struct InitArgs {
    /// Auto-detect dependencies from package.json/Cargo.toml/etc.
    #[arg(long)]
    from_deps: bool,

    /// Scaffold for monorepo (root + per-package .chub/ dirs)
    #[arg(long)]
    monorepo: bool,
}

pub fn run(args: InitArgs, json: bool) {
    match chub_core::team::project::init_project(args.from_deps, args.monorepo) {
        Ok(chub_dir) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "created",
                        "path": chub_dir.display().to_string(),
                    })
                );
            } else {
                output::success(&format!("Created .chub/ at {}", chub_dir.display()));
                eprintln!("  {}", "config.yaml".dimmed());
                eprintln!("  {}", "pins.yaml".dimmed());
                eprintln!("  {}", "profiles/base.yaml".dimmed());
                eprintln!("  {}", "context/architecture.md".dimmed());
                eprintln!("  {}", "annotations/".dimmed());
                eprintln!();
                eprintln!("Next steps:");
                eprintln!(
                    "  {} — pin docs for your project",
                    "chub pin openai/chat --lang python".bold()
                );
                eprintln!(
                    "  {} — auto-detect from dependencies",
                    "chub detect --pin".bold()
                );
                eprintln!("  {} — create context profiles", "chub profile list".bold());
            }

            // If --from-deps, run detect
            if args.from_deps {
                eprintln!();
                run_detect_after_init(json);
            }
        }
        Err(e) => {
            output::error(&e.to_string(), json);
            std::process::exit(1);
        }
    }
}

fn run_detect_after_init(json: bool) {
    let cwd = std::env::current_dir().unwrap_or_default();
    let deps = chub_core::team::detect::detect_dependencies(&cwd);

    if deps.is_empty() {
        if !json {
            eprintln!("{}", "No dependency files found.".dimmed());
        }
        return;
    }

    if !json {
        eprintln!("Detected {} dependencies from project files:", deps.len());
        for dep in &deps {
            let ver = dep
                .version
                .as_deref()
                .map(|v| format!(" ({})", v))
                .unwrap_or_default();
            eprintln!(
                "  {} {}{}",
                dep.name.bold(),
                dep.language.dimmed(),
                ver.dimmed()
            );
        }
        eprintln!();
        eprintln!(
            "Run {} to auto-pin detected docs.",
            "chub detect --pin".bold()
        );
    }

    // Clean up the marker file
    if let Some(chub_dir) = chub_core::team::project::project_chub_dir() {
        let _ = std::fs::remove_file(chub_dir.join(".init_from_deps"));
    }
}
