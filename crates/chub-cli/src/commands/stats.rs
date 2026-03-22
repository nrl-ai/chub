use clap::Args;
use owo_colors::OwoColorize;

use chub_core::team::analytics;

#[derive(Args)]
pub struct StatsArgs {
    /// Number of days to show stats for (default 30)
    #[arg(long, default_value = "30")]
    days: u64,
}

pub fn run(args: StatsArgs, json: bool) {
    let stats = analytics::get_stats(args.days);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&stats).unwrap_or_default()
        );
    } else {
        eprintln!(
            "{}\n",
            format!("Usage stats (last {} days):", stats.period_days).bold()
        );

        eprintln!(
            "  Events: {}  (fetches: {}, searches: {}, builds: {}, MCP calls: {}, annotations: {}, feedback: {})",
            stats.total_events,
            stats.total_fetches,
            stats.total_searches,
            stats.total_builds,
            stats.total_mcp_calls,
            stats.total_annotations,
            stats.total_feedback,
        );

        if stats.total_events == 0 {
            eprintln!("\n{}", "No events recorded.".dimmed());
            return;
        }

        // Most fetched docs
        if !stats.most_fetched.is_empty() {
            eprintln!("\n{}", "Most fetched docs:".bold());
            for (i, (id, count)) in stats.most_fetched.iter().take(10).enumerate() {
                eprintln!("  {}. {}  — {} fetches", i + 1, id.bold(), count);
            }
        }

        // Top search queries
        if !stats.top_queries.is_empty() {
            eprintln!("\n{}", "Top search queries:".bold());
            for (i, (query, count)) in stats.top_queries.iter().take(10).enumerate() {
                eprintln!("  {}. \"{}\"  — {} times", i + 1, query, count);
            }
            if stats.total_searches > 0 {
                eprintln!("  Avg results per search: {:.1}", stats.avg_search_results);
            }
        }

        // Top MCP tools
        if !stats.top_mcp_tools.is_empty() {
            eprintln!("\n{}", "Top MCP tools:".bold());
            for (tool, count) in stats.top_mcp_tools.iter().take(10) {
                eprintln!("  {}  — {} calls", tool, count);
            }
        }

        // Agents
        if !stats.agents.is_empty() {
            eprintln!("\n{}", "Agents:".bold());
            for (agent, count) in &stats.agents {
                eprintln!("  {}  — {} events", agent, count);
            }
        }

        // Never-fetched pins
        if !stats.never_fetched_pins.is_empty() {
            eprintln!("\n{}", "Never fetched (pinned but unused):".yellow());
            for id in &stats.never_fetched_pins {
                eprintln!("  - {}", id.dimmed());
            }
            eprintln!(
                "\n{}",
                "Suggestion: unpin unused docs to reduce noise.".dimmed()
            );
        }

        // Journal info
        let size = analytics::journal_size_bytes();
        if size > 0 {
            eprintln!("\n  Journal: {:.1} KB", size as f64 / 1024.0);
        }
    }
}
