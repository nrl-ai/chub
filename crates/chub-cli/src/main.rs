mod commands;
mod mcp;
mod output;
mod welcome;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "chub",
    version,
    about = "chub-turbo — lightning-fast curated docs for AI coding agents"
)]
struct Cli {
    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build registry.json from a content directory
    Build(commands::build::BuildArgs),
    /// Search docs and skills (no query lists all)
    Search(commands::search::SearchArgs),
    /// Fetch docs or skills by ID (auto-detects type)
    Get(commands::get::GetArgs),
    /// Refresh the cached registry index
    Update(commands::update::UpdateArgs),
    /// Manage the local cache
    Cache(commands::cache::CacheArgs),
    /// Attach agent notes to a doc or skill
    Annotate(commands::annotate::AnnotateArgs),
    /// Rate a doc or skill (up/down)
    Feedback(commands::feedback::FeedbackArgs),
    /// Start MCP stdio server for AI coding agents
    Mcp,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // MCP server runs its own flow — no welcome, no registry preload from CLI
    if matches!(cli.command, Commands::Mcp) {
        if let Err(e) = mcp::server::run_mcp_server().await {
            eprintln!("[chub-mcp] Fatal: {}", e);
            std::process::exit(1);
        }
        return;
    }

    welcome::show_welcome_if_needed(cli.json);

    // Commands that don't need registry
    match cli.command {
        Commands::Build(args) => {
            if let Err(e) = commands::build::run(args, cli.json) {
                output::error(&e.to_string(), cli.json);
                std::process::exit(1);
            }
            return;
        }
        Commands::Update(args) => {
            if let Err(e) = commands::update::run(args, cli.json).await {
                output::error(&e.to_string(), cli.json);
                std::process::exit(1);
            }
            return;
        }
        Commands::Cache(args) => {
            commands::cache::run(args, cli.json);
            return;
        }
        Commands::Annotate(args) => {
            commands::annotate::run(args, cli.json);
            return;
        }
        _ => {}
    }

    // Commands that need registry — ensure it's available
    if let Err(e) = chub_core::fetch::ensure_registry().await {
        output::error(
            &format!(
                "Registry not available: {}. Run `chub update` to refresh.",
                e
            ),
            cli.json,
        );
        std::process::exit(1);
    }

    let merged = chub_core::registry::load_merged();

    match cli.command {
        Commands::Search(args) => {
            commands::search::run(args, cli.json, &merged);
        }
        Commands::Get(args) => {
            if let Err(e) = commands::get::run(args, cli.json, &merged).await {
                output::error(&e.to_string(), cli.json);
                std::process::exit(1);
            }
        }
        Commands::Feedback(args) => {
            commands::feedback::run(args, cli.json, Some(&merged)).await;
        }
        // Already handled above
        Commands::Build(_)
        | Commands::Update(_)
        | Commands::Cache(_)
        | Commands::Annotate(_)
        | Commands::Mcp => unreachable!(),
    }
}
