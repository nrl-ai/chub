use std::path::PathBuf;

use clap::Args;
use owo_colors::OwoColorize;

use chub_core::build::builder::{build_registry, write_build_output_with_opts, BuildOptions};
use chub_core::error::Result;

use crate::output;

#[derive(Args)]
pub struct ServeArgs {
    /// Content directory to serve from
    content_dir: PathBuf,

    /// Port to listen on
    #[arg(short, long, default_value = "4242")]
    port: u16,

    /// Output directory for built content (default: temp dir)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

pub async fn run(args: ServeArgs, json: bool) -> Result<()> {
    let content_dir = &args.content_dir;
    let output_dir = args.output.unwrap_or_else(|| content_dir.join("dist"));

    // Build the content first
    if !json {
        eprintln!("Building content from {}...", content_dir.display());
    }

    let opts = BuildOptions {
        base_url: Some(format!("http://localhost:{}", args.port)),
        validate_only: false,
        incremental: true,
    };

    let result = build_registry(content_dir, &opts)?;

    for w in &result.warnings {
        output::warn(w);
    }

    write_build_output_with_opts(content_dir, &output_dir, &result, &opts)?;

    if !json {
        output::success(&format!(
            "Built: {} docs, {} skills → {}",
            result.docs_count,
            result.skills_count,
            output_dir.display()
        ));
    }

    // Serve the output directory
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], args.port));

    if !json {
        eprintln!(
            "\nServing registry at {}\n",
            format!("http://localhost:{}", args.port).bold().underline()
        );
        eprintln!(
            "  {}",
            format!("http://localhost:{}/registry.json", args.port).dimmed()
        );
        eprintln!(
            "  {}",
            format!("http://localhost:{}/search-index.json", args.port).dimmed()
        );
        eprintln!();
        eprintln!("Press Ctrl+C to stop.");
    }

    use axum::Router;
    use tower_http::cors::CorsLayer;
    use tower_http::services::ServeDir;

    let app = Router::new()
        .fallback_service(ServeDir::new(&output_dir))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        chub_core::error::Error::Config(format!("Failed to bind to port {}: {}", args.port, e))
    })?;

    axum::serve(listener, app)
        .await
        .map_err(|e| chub_core::error::Error::Config(format!("Server error: {}", e)))?;

    Ok(())
}
