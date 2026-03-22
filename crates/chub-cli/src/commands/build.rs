use chub_core::build::builder::{build_registry, write_build_output_with_opts, BuildOptions};
use chub_core::error::Result;
use chub_core::team::analytics;
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use crate::output;

#[derive(Args)]
pub struct BuildArgs {
    /// Content directory to build from
    content_dir: PathBuf,

    /// Output directory (default: <content-dir>/dist)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Base URL for CDN deployment
    #[arg(long)]
    base_url: Option<String>,

    /// Validate without writing output
    #[arg(long)]
    validate_only: bool,

    /// Disable incremental builds (copy all files regardless of changes)
    #[arg(long)]
    no_incremental: bool,
}

pub fn run(args: BuildArgs, json: bool) -> Result<()> {
    let content_dir = &args.content_dir;
    let output_dir = args.output.unwrap_or_else(|| content_dir.join("dist"));

    let opts = BuildOptions {
        base_url: args.base_url,
        validate_only: args.validate_only,
        incremental: !args.no_incremental,
    };

    // Show spinner during discovery + index build
    let pb = if !json {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏-"),
        );
        pb.set_message("Scanning content directory...");
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        Some(pb)
    } else {
        None
    };

    let build_start = std::time::Instant::now();
    let result = build_registry(content_dir, &opts)?;
    let build_duration = build_start.elapsed().as_millis() as u64;

    analytics::record_build(
        result.docs_count + result.skills_count,
        build_duration,
        result.warnings.len(),
        args.validate_only,
    );

    // Print warnings
    for w in &result.warnings {
        output::warn(w);
    }

    if args.validate_only {
        if let Some(pb) = pb {
            pb.finish_and_clear();
        }
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "docs": result.docs_count,
                    "skills": result.skills_count,
                    "warnings": result.warnings.len(),
                })
            );
        } else {
            output::success(&format!(
                "Valid: {} docs, {} skills, {} warnings",
                result.docs_count,
                result.skills_count,
                result.warnings.len()
            ));
        }
        return Ok(());
    }

    if let Some(ref pb) = pb {
        pb.set_message("Writing output...");
    }

    // Write output
    write_build_output_with_opts(content_dir, &output_dir, &result, &opts)?;

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "docs": result.docs_count,
                "skills": result.skills_count,
                "warnings": result.warnings.len(),
                "output": output_dir.display().to_string(),
            })
        );
    } else {
        output::success(&format!(
            "Built: {} docs, {} skills → {}",
            result.docs_count,
            result.skills_count,
            output_dir.display()
        ));
    }

    Ok(())
}
