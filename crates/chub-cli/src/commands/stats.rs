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

        if stats.most_fetched.is_empty() {
            eprintln!("{}", "No fetch events recorded.".dimmed());
            return;
        }

        eprintln!("Most fetched docs:");
        for (i, (id, count)) in stats.most_fetched.iter().take(10).enumerate() {
            eprintln!("  {}. {}  — {} fetches", i + 1, id.bold(), count,);
        }

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

        eprintln!("\n  Total fetches: {}", stats.total_fetches);
    }
}
