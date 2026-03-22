use clap::{Args, Subcommand};
use owo_colors::OwoColorize;

use chub_core::config;
use chub_core::identity::{detect_agent, detect_agent_version, detect_model};
use chub_core::team::tracking::{session_state, transcript};
use chub_core::team::{cost, hooks, session_journal, sessions};

use crate::output;

#[derive(Args)]
pub struct TrackArgs {
    #[command(subcommand)]
    command: TrackCommand,
}

#[derive(Subcommand)]
enum TrackCommand {
    /// Show active session and tracking status
    Status,
    /// Install agent hooks for automatic session tracking
    Enable(EnableArgs),
    /// Remove agent hooks
    Disable,
    /// Handle agent lifecycle hooks (called by agent hooks, not by user)
    Hook(HookArgs),
    /// Show session history
    Log(LogArgs),
    /// Show details for a specific session
    Show(ShowArgs),
    /// Aggregate usage report (costs, models, tools)
    Report(ReportArgs),
    /// Export session data as JSON
    Export(ExportArgs),
    /// Clear local session transcripts
    Clear,
    /// Launch local web dashboard for session tracking
    Dashboard(DashboardArgs),
}

#[derive(Args)]
struct EnableArgs {
    /// Agent to install hooks for (auto-detect if omitted)
    agent: Option<String>,

    /// Overwrite existing hooks
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct HookArgs {
    /// Hook event name: session-start, stop, prompt, pre-tool, post-tool, commit-msg, post-commit
    event: String,

    /// Agent name (auto-detected if omitted)
    #[arg(long)]
    agent: Option<String>,

    /// Model name
    #[arg(long)]
    model: Option<String>,

    /// Tool name (for pre-tool/post-tool events)
    #[arg(long)]
    tool: Option<String>,

    /// Prompt text or input summary (read from stdin if not provided)
    #[arg(long)]
    input: Option<String>,

    /// Session ID (for stop event; auto-detected if omitted)
    #[arg(long)]
    session_id: Option<String>,

    /// Token counts: input,output,cache_read,cache_write[,reasoning]
    #[arg(long)]
    tokens: Option<String>,

    /// File path (for file-change events)
    #[arg(long)]
    file: Option<String>,
}

#[derive(Args)]
struct LogArgs {
    /// Number of days to show (default 30)
    #[arg(long, default_value = "30")]
    days: u64,
}

#[derive(Args)]
struct ShowArgs {
    /// Session ID
    id: String,
}

#[derive(Args)]
struct ReportArgs {
    /// Number of days (default 30)
    #[arg(long, default_value = "30")]
    days: u64,
}

#[derive(Args)]
struct ExportArgs {
    /// Number of days (default 30)
    #[arg(long, default_value = "30")]
    days: u64,
}

#[derive(Args)]
struct DashboardArgs {
    /// Port to listen on
    #[arg(short, long, default_value = "4243")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

pub async fn run(args: TrackArgs, json: bool) {
    match args.command {
        TrackCommand::Status => run_status(json),
        TrackCommand::Enable(enable_args) => run_enable(enable_args, json),
        TrackCommand::Disable => run_disable(json),
        TrackCommand::Hook(hook_args) => run_hook(hook_args, json),
        TrackCommand::Log(log_args) => run_log(log_args, json),
        TrackCommand::Show(show_args) => run_show(show_args, json),
        TrackCommand::Report(report_args) => run_report(report_args, json),
        TrackCommand::Export(export_args) => run_export(export_args, json),
        TrackCommand::Clear => run_clear(json),
        TrackCommand::Dashboard(dash_args) => run_dashboard(dash_args, json).await,
    }
}

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

fn run_status(json: bool) {
    let active = sessions::get_active_session();
    let journal_files = session_journal::list_journal_files();
    let entire_states = session_state::list_states();

    if json {
        println!(
            "{}",
            serde_json::json!({
                "active_session": active.as_ref().map(|s| serde_json::json!({
                    "session_id": s.session_id,
                    "agent": s.agent,
                    "model": s.model,
                    "started_at": s.started_at,
                    "turns": s.turns,
                    "tool_calls": s.tool_calls,
                })),
                "agent_detected": detect_agent(),
                "agent_version": detect_agent_version(),
                "model_detected": detect_model(),
                "local_journals": journal_files.len(),
                "entire_sessions": entire_states.len(),
            })
        );
    } else if let Some(ref session) = active {
        eprintln!("{}", "Active session:".bold());
        eprintln!("  ID:      {}", session.session_id);
        eprintln!("  Agent:   {}", session.agent);
        if let Some(ref model) = session.model {
            eprintln!("  Model:   {}", model);
        }
        eprintln!("  Started: {}", session.started_at);
        eprintln!("  Turns:   {}", session.turns);
        eprintln!("  Tools:   {} calls", session.tool_calls);
        if session.tokens.reasoning > 0 {
            eprintln!(
                "  Tokens:  {} in / {} out / {} reasoning",
                session.tokens.input, session.tokens.output, session.tokens.reasoning
            );
        } else {
            eprintln!(
                "  Tokens:  {} in / {} out",
                session.tokens.input, session.tokens.output
            );
        }

        // Show entire.io state info if available
        if let Some(state) = session_state::load_state(&session.session_id) {
            eprintln!("  Phase:   {:?}", state.phase);
            if !state.files_touched.is_empty() {
                eprintln!("  Files:   {} touched", state.files_touched.len());
            }
            if state.transcript_path.is_some() {
                eprintln!("  Transcript: linked");
            }
        }
    } else {
        eprintln!("{}", "No active session.".dimmed());
        eprintln!("  Agent detected: {}", detect_agent());
        if let Some(model) = detect_model() {
            eprintln!("  Model detected: {}", model);
        }
        if !journal_files.is_empty() {
            eprintln!("  Local journals: {} sessions", journal_files.len());
        }
        if !entire_states.is_empty() {
            eprintln!(
                "  Entire.io sessions: {} (in .git/entire-sessions/)",
                entire_states.len()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Enable / Disable hooks
// ---------------------------------------------------------------------------

fn run_enable(args: EnableArgs, json: bool) {
    match hooks::install_hooks(args.agent.as_deref(), args.force) {
        Ok(results) => {
            if json {
                let items: Vec<_> = results
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "agent": r.agent,
                            "config_file": r.config_file,
                            "action": format!("{:?}", r.action),
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&items).unwrap_or_default()
                );
            } else {
                eprintln!("{}\n", "Hook installation results:".bold());
                for r in &results {
                    let status = match &r.action {
                        hooks::HookAction::Installed => "installed".green().to_string(),
                        hooks::HookAction::Updated => "updated".yellow().to_string(),
                        hooks::HookAction::AlreadyInstalled => {
                            "already installed".dimmed().to_string()
                        }
                        hooks::HookAction::Removed => "removed".dimmed().to_string(),
                        hooks::HookAction::Error(e) => format!("error: {}", e).red().to_string(),
                    };
                    eprintln!(
                        "  {} {} → {}",
                        r.agent.cyan(),
                        r.config_file.dimmed(),
                        status
                    );
                }
                eprintln!(
                    "\n{}",
                    "Hooks installed. Sessions will be tracked automatically.".green()
                );
            }
        }
        Err(e) => output::error(&e.to_string(), json),
    }
}

fn run_disable(json: bool) {
    match hooks::uninstall_hooks() {
        Ok(results) => {
            if json {
                let items: Vec<_> = results
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "agent": r.agent,
                            "config_file": r.config_file,
                            "action": format!("{:?}", r.action),
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&items).unwrap_or_default()
                );
            } else {
                for r in &results {
                    eprintln!("  {} {} → removed", r.agent.cyan(), r.config_file.dimmed());
                }
                eprintln!("{}", "Hooks removed.".green());
            }
        }
        Err(e) => output::error(&e.to_string(), json),
    }
}

// ---------------------------------------------------------------------------
// Hook handler
// ---------------------------------------------------------------------------

fn run_hook(args: HookArgs, json: bool) {
    // Try to read stdin JSON from agent hooks (non-blocking)
    let stdin_data = hooks::parse_hook_stdin();

    match args.event.as_str() {
        "session-start" => {
            let agent = args
                .agent
                .or_else(|| {
                    stdin_data
                        .as_ref()
                        .and_then(|v| v.get("agent").and_then(|a| a.as_str()))
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| detect_agent().to_string());

            let model_from_stdin = stdin_data
                .as_ref()
                .and_then(|v| v.get("model").and_then(|m| m.as_str()))
                .map(|s| s.to_string());
            let model = args.model.or(model_from_stdin).or_else(detect_model);
            let model_ref = model.as_deref();

            match sessions::start_session(&agent, model_ref) {
                Some(session_id) => {
                    session_journal::record_session_start(&session_id, &agent, model_ref);

                    // Also create entire.io-compatible session state
                    let mut state = session_state::SessionState::new(&agent, model_ref);
                    // Override session_id to match chub's ID
                    state.session_id = session_id.clone();
                    // Link transcript if Claude Code
                    if agent.contains("claude") {
                        if let Some(repo_path) = chub_core::team::project::find_project_root(None) {
                            let repo_str = repo_path.to_string_lossy();
                            if let Some(t_path) =
                                transcript::find_transcript(&repo_str, &session_id)
                            {
                                state.transcript_path = Some(t_path.to_string_lossy().to_string());
                            }
                        }
                    }
                    session_state::save_state(&state);

                    if json {
                        println!(
                            "{}",
                            serde_json::json!({ "status": "started", "session_id": session_id })
                        );
                    } else {
                        eprintln!("Session started: {}", session_id);
                    }
                }
                None => {
                    output::error("Failed to start session (no .git directory?)", json);
                }
            }
        }

        "stop" | "session-end" => {
            if let Some(active) = sessions::get_active_session() {
                let session_id = active.session_id.clone();
                session_journal::record_session_end(&session_id, None, active.turns);

                // Finalize entire.io-compatible session state
                if let Some(mut state) = session_state::load_state(&session_id) {
                    state.apply_event(session_state::SessionEvent::SessionStop);

                    // Link transcript at stop time if not already linked
                    // (at session-start the transcript file may not exist yet)
                    if state.transcript_path.is_none() && active.agent.contains("claude") {
                        if let Some(repo_path) = chub_core::team::project::find_project_root(None) {
                            let repo_str = repo_path.to_string_lossy();
                            if let Some(t_path) =
                                transcript::find_transcript(&repo_str, &session_id)
                            {
                                state.transcript_path = Some(t_path.to_string_lossy().to_string());
                            }
                        }
                    }

                    // Parse transcript for final token counts and model
                    let mut transcript_model: Option<String> = None;
                    if let Some(ref t_path) = state.transcript_path {
                        let analysis = transcript::parse_transcript(std::path::Path::new(t_path));
                        state.token_usage = Some(analysis.token_usage);
                        state.step_count = analysis.turn_count;
                        transcript_model = analysis.model;

                        // Add any files from transcript we haven't seen
                        for f in analysis.modified_files {
                            state.touch_file(&f);
                        }
                    }

                    // Calculate cost on state — prefer active model, fall back to transcript
                    let model_for_cost = active.model.as_deref().or(transcript_model.as_deref());
                    if let Some(ref usage) = state.token_usage {
                        let chub_tokens = sessions::TokenUsage {
                            input: usage.input_tokens as u64,
                            output: usage.output_tokens as u64,
                            cache_read: usage.cache_read_tokens as u64,
                            cache_write: usage.cache_creation_tokens as u64,
                            reasoning: usage.reasoning_tokens as u64,
                        };
                        state.est_cost_usd = cost::estimate_cost(model_for_cost, &chub_tokens);
                    }

                    // Archive transcript to .git/chub/transcripts/ for LLM review
                    if let Some(ref t_path) = state.transcript_path {
                        transcript::archive_transcript_to_git(
                            std::path::Path::new(t_path),
                            &session_id,
                        );
                    }

                    session_state::save_state(&state);
                }

                if let Some(mut session) = sessions::end_session() {
                    // Link transcript for token/model enrichment on session data
                    if session.agent.contains("claude") {
                        if let Some(repo_path) = chub_core::team::project::find_project_root(None) {
                            let repo_str = repo_path.to_string_lossy();
                            if let Some(t_path) =
                                transcript::find_transcript(&repo_str, &session.session_id)
                            {
                                let analysis =
                                    transcript::parse_transcript(std::path::Path::new(&t_path));

                                // Set model from transcript if not already set
                                if session.model.is_none() {
                                    session.model = analysis.model;
                                }

                                // Enrich tokens from transcript
                                if session.tokens.total() == 0 {
                                    session.tokens = sessions::TokenUsage {
                                        input: analysis.token_usage.input_tokens as u64,
                                        output: analysis.token_usage.output_tokens as u64,
                                        cache_read: analysis.token_usage.cache_read_tokens as u64,
                                        cache_write: analysis.token_usage.cache_creation_tokens
                                            as u64,
                                        reasoning: analysis.token_usage.reasoning_tokens as u64,
                                    };
                                }

                                // Use transcript turn count (filters system messages)
                                if analysis.turn_count > 0 {
                                    session.turns = analysis.turn_count as u32;
                                }

                                for f in analysis.modified_files {
                                    session.files_changed.push(f);
                                }
                                session.files_changed.sort();
                                session.files_changed.dedup();

                                // Wire extended_thinking into session env
                                if analysis.has_extended_thinking {
                                    let env = session.env.get_or_insert_with(Default::default);
                                    env.extended_thinking = Some(true);
                                }
                            }
                        }
                    }

                    // Calculate cost
                    session.est_cost_usd =
                        cost::estimate_cost(session.model.as_deref(), &session.tokens);

                    // Re-write with cost
                    sessions::write_session_summary(&session);

                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&session).unwrap_or_default()
                        );
                    } else {
                        eprintln!("Session ended: {}", session.session_id);
                        if let Some(cost) = session.est_cost_usd {
                            eprintln!(
                                "  {} turns, {} tokens, ~${:.3}",
                                session.turns,
                                session.tokens.total(),
                                cost
                            );
                        }
                    }
                }
            } else if json {
                println!("{}", serde_json::json!({ "status": "no_active_session" }));
            } else {
                eprintln!("{}", "No active session to end.".dimmed());
            }
        }

        "prompt" => {
            if let Some(mut active) = sessions::get_active_session() {
                active.turns += 1;

                // Extract prompt text from stdin or CLI flag
                let prompt_text = args.input.or_else(|| {
                    stdin_data
                        .as_ref()
                        .and_then(|v| v.get("prompt").and_then(|p| p.as_str()))
                        .map(|s| s.to_string())
                });

                // Check for model update from stdin
                if let Some(ref data) = stdin_data {
                    if let Some(model) = data.get("model").and_then(|m| m.as_str()) {
                        if active.model.as_deref() != Some(model) {
                            active.model = Some(model.to_string());
                        }
                    }
                }

                // Update entire.io-compatible session state
                if let Some(mut state) = session_state::load_state(&active.session_id) {
                    state.apply_event(session_state::SessionEvent::TurnStart);
                    // Set first_prompt only on the first prompt
                    if state.first_prompt.is_none() {
                        state.first_prompt = prompt_text.clone();
                    }
                    session_state::save_state(&state);
                }

                sessions::save_active_session(&active);
                session_journal::record_prompt(&active.session_id, prompt_text.as_deref());
            }
        }

        "pre-tool" => {
            if let Some(mut active) = sessions::get_active_session() {
                // Extract tool name from stdin or CLI flag
                let tool = args
                    .tool
                    .or_else(|| stdin_data.as_ref().and_then(hooks::extract_tool_name));
                let tool_name = tool.as_deref().unwrap_or("unknown");

                // Extract input summary
                let input_summary = args.input.or_else(|| {
                    stdin_data
                        .as_ref()
                        .and_then(|v| v.get("tool_input").map(|i| summarize_json(i, 120)))
                });

                active.tool_calls += 1;
                active.tools_used.insert(tool_name.to_string());

                // Update entire.io-compatible session state
                if let Some(mut state) = session_state::load_state(&active.session_id) {
                    state.step_count += 1;
                    state.tool_calls += 1;
                    state.tools_used.insert(tool_name.to_string());
                    session_state::save_state(&state);
                }

                sessions::save_active_session(&active);
                session_journal::record_tool_call(
                    &active.session_id,
                    tool_name,
                    input_summary.as_deref(),
                );
            }
        }

        "post-tool" => {
            if let Some(mut active) = sessions::get_active_session() {
                let tool = args
                    .tool
                    .or_else(|| stdin_data.as_ref().and_then(hooks::extract_tool_name));
                let tool_name = tool.as_deref().unwrap_or("unknown");

                // Track file changes from stdin tool_input (Write/Edit)
                let file_path = args.file.or_else(|| {
                    stdin_data
                        .as_ref()
                        .and_then(|v| v.get("tool_input"))
                        .and_then(hooks::extract_file_path)
                });
                if let Some(ref file) = file_path {
                    active.files_changed.insert(file.clone());
                    session_journal::record_file_change(&active.session_id, file, Some("edit"));
                }

                // Parse tokens if provided via CLI flag
                if let Some(ref token_str) = args.tokens {
                    if let Some(tokens) = parse_token_string(token_str) {
                        active.tokens.add(&tokens);
                        session_journal::record_response(&active.session_id, Some(tokens));
                    }
                }

                // Update entire.io-compatible session state
                if let Some(mut state) = session_state::load_state(&active.session_id) {
                    if let Some(ref file) = file_path {
                        state.touch_file(file);
                    }
                    session_state::save_state(&state);
                }

                // Estimate output size from tool_response in stdin
                let output_size = stdin_data
                    .as_ref()
                    .and_then(|v| v.get("tool_response"))
                    .map(|r| r.to_string().len() as u64);

                sessions::save_active_session(&active);
                session_journal::record_tool_result(&active.session_id, tool_name, output_size);
            }
        }

        "model-update" => {
            if let Some(mut active) = sessions::get_active_session() {
                let model = args.model.or_else(|| {
                    stdin_data
                        .as_ref()
                        .and_then(|v| v.get("model").and_then(|m| m.as_str()))
                        .map(|s| s.to_string())
                });
                if let Some(ref model) = model {
                    active.model = Some(model.clone());
                    sessions::save_active_session(&active);
                    session_journal::append_event(
                        &active.session_id,
                        &session_journal::SessionEvent::ModelUpdate {
                            ts: chub_core::util::now_iso8601(),
                            model: model.clone(),
                        },
                    );
                }
            }
        }

        "commit-msg" => {
            // Called by prepare-commit-msg git hook
            // Adds Chub-Session and Chub-Checkpoint trailers to the commit message
            // (user can remove the trailer before committing to skip linking)
            if let Some(active) = sessions::get_active_session() {
                let msg_file = args.input.as_deref();
                if let Some(path) = msg_file {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        // Skip during rebase
                        if is_rebase_in_progress() {
                            return;
                        }
                        let mut trailers = String::new();
                        if !content.contains("Chub-Session:") {
                            trailers.push_str(&format!("\nChub-Session: {}", active.session_id));
                        }
                        if !content.contains("Chub-Checkpoint:") {
                            // Generate checkpoint ID and stash it for post-commit
                            let checkpoint_id =
                                chub_core::team::tracking::types::CheckpointID::generate();
                            trailers.push_str(&format!("\nChub-Checkpoint: {}", checkpoint_id.0));
                        }
                        if !trailers.is_empty() {
                            let new_content = format!("{}{}\n", content.trim_end(), trailers);
                            let _ = std::fs::write(path, new_content);
                        }
                    }
                }
            }
        }

        "post-commit" => {
            // Called by post-commit git hook
            // Records the commit hash, creates a checkpoint on the orphan branch
            if is_rebase_in_progress() {
                return;
            }

            if let Some(mut active) = sessions::get_active_session() {
                // Get the latest commit hash
                if let Ok(output) = std::process::Command::new("git")
                    .args(["rev-parse", "--short", "HEAD"])
                    .output()
                {
                    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !hash.is_empty() {
                        active.commits.push(hash.clone());
                        sessions::save_active_session(&active);

                        // Update entire.io-compatible session state
                        if let Some(mut state) = session_state::load_state(&active.session_id) {
                            state.apply_event(session_state::SessionEvent::GitCommit);
                            state.commits.push(hash);

                            // Create checkpoint on orphan branch (condense)
                            use chub_core::team::tracking::checkpoint;
                            let t_path =
                                state.transcript_path.as_ref().map(std::path::PathBuf::from);
                            let attribution = state.base_commit.as_str();
                            let attr = if !attribution.is_empty() {
                                transcript::calculate_attribution(attribution)
                            } else {
                                None
                            };
                            checkpoint::create_checkpoint(&state, t_path.as_deref(), attr);

                            session_state::save_state(&state);
                        }
                    }
                }
            }
        }

        "pre-push" => {
            // Push chub/sessions/v1 and entire/checkpoints/v1 alongside user's push
            let remote = args.input.as_deref().unwrap_or("origin");
            let sessions_pushed = sessions::push_sessions(remote);
            // Also push checkpoint branch if it exists
            use chub_core::team::tracking::branch_store;
            let checkpoints_pushed = branch_store::push_branch("entire/checkpoints/v1", remote);
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "remote": remote,
                        "sessions": if sessions_pushed { "pushed" } else { "skipped" },
                        "checkpoints": if checkpoints_pushed { "pushed" } else { "skipped" },
                    })
                );
            }
        }

        other => {
            output::error(&format!("Unknown hook event: \"{}\"", other), json);
        }
    }
}

/// Check if a rebase is in progress (skip checkpoint operations during rebase).
fn is_rebase_in_progress() -> bool {
    // Check for .git/rebase-merge/ or .git/rebase-apply/
    if let Some(root) = chub_core::team::project::find_project_root(None) {
        let git_dir = root.join(".git");
        return git_dir.join("rebase-merge").is_dir() || git_dir.join("rebase-apply").is_dir();
    }
    false
}

/// Summarize a JSON value to at most `max_len` characters.
fn summarize_json(value: &serde_json::Value, max_len: usize) -> String {
    let s = value.to_string();
    if s.len() <= max_len {
        s
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn parse_token_string(s: &str) -> Option<sessions::TokenUsage> {
    let parts: Vec<u64> = s.split(',').filter_map(|p| p.trim().parse().ok()).collect();
    if parts.is_empty() {
        return None;
    }
    Some(sessions::TokenUsage {
        input: *parts.first().unwrap_or(&0),
        output: *parts.get(1).unwrap_or(&0),
        cache_read: *parts.get(2).unwrap_or(&0),
        cache_write: *parts.get(3).unwrap_or(&0),
        reasoning: *parts.get(4).unwrap_or(&0),
    })
}

// ---------------------------------------------------------------------------
// Log
// ---------------------------------------------------------------------------

fn run_log(args: LogArgs, json: bool) {
    let session_list = sessions::list_sessions(args.days);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&session_list).unwrap_or_default()
        );
        return;
    }

    if session_list.is_empty() {
        eprintln!(
            "{}",
            format!("No sessions in the last {} days.", args.days).dimmed()
        );
        return;
    }

    eprintln!(
        "{}\n",
        format!("{} sessions (last {} days):", session_list.len(), args.days).bold()
    );

    for s in &session_list {
        let cost_str = s
            .est_cost_usd
            .map(|c| format!("${:.3}", c))
            .unwrap_or_else(|| "-".to_string());
        let model_str = s.model.as_deref().unwrap_or("-");
        let duration_str = s
            .duration_s
            .map(format_duration)
            .unwrap_or_else(|| "active".yellow().to_string());

        eprintln!(
            "  {} {} {} {} {} {}",
            s.session_id.bold(),
            s.agent.cyan(),
            model_str.dimmed(),
            duration_str,
            format!("{} turns", s.turns).dimmed(),
            cost_str.green(),
        );
    }
}

// ---------------------------------------------------------------------------
// Show
// ---------------------------------------------------------------------------

fn run_show(args: ShowArgs, json: bool) {
    // Try summary first
    if let Some(session) = sessions::get_session(&args.id) {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&session).unwrap_or_default()
            );
        } else {
            print_session_detail(&session);
        }
        return;
    }

    // Check active session
    if let Some(active) = sessions::get_active_session() {
        if active.session_id == args.id {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&active).unwrap_or_default()
                );
            } else {
                eprintln!("{} (active)\n", active.session_id.bold());
                eprintln!("  Agent:   {}", active.agent);
                if let Some(ref model) = active.model {
                    eprintln!("  Model:   {}", model);
                }
                eprintln!("  Started: {}", active.started_at);
                eprintln!("  Turns:   {}", active.turns);
                eprintln!("  Tools:   {} calls", active.tool_calls);
                if active.tokens.reasoning > 0 {
                    eprintln!(
                        "  Tokens:  {} total ({} in / {} out / {} reasoning)",
                        active.tokens.total(),
                        active.tokens.input,
                        active.tokens.output,
                        active.tokens.reasoning
                    );
                } else {
                    eprintln!(
                        "  Tokens:  {} total ({} in / {} out)",
                        active.tokens.total(),
                        active.tokens.input,
                        active.tokens.output
                    );
                }
            }
            return;
        }
    }

    output::error(&format!("Session \"{}\" not found.", args.id), json);
}

fn print_session_detail(s: &sessions::Session) {
    eprintln!("{}\n", s.session_id.bold());
    eprintln!("  Agent:    {}", s.agent);
    if let Some(ref model) = s.model {
        eprintln!("  Model:    {}", model);
    }
    eprintln!("  Started:  {}", s.started_at);
    if let Some(ref ended) = s.ended_at {
        eprintln!("  Ended:    {}", ended);
    }
    if let Some(d) = s.duration_s {
        eprintln!("  Duration: {}", format_duration(d));
    }
    eprintln!("  Turns:    {}", s.turns);
    if s.tokens.reasoning > 0 {
        eprintln!(
            "  Tokens:   {} total ({} in / {} out / {} reasoning / {} cache-r / {} cache-w)",
            s.tokens.total(),
            s.tokens.input,
            s.tokens.output,
            s.tokens.reasoning,
            s.tokens.cache_read,
            s.tokens.cache_write
        );
    } else {
        eprintln!(
            "  Tokens:   {} total ({} in / {} out / {} cache-r / {} cache-w)",
            s.tokens.total(),
            s.tokens.input,
            s.tokens.output,
            s.tokens.cache_read,
            s.tokens.cache_write
        );
    }
    eprintln!("  Tools:    {} calls", s.tool_calls);
    if !s.tools_used.is_empty() {
        eprintln!("            {}", s.tools_used.join(", "));
    }
    if let Some(cost) = s.est_cost_usd {
        eprintln!("  Cost:     ${:.3}", cost);
    }
    if !s.files_changed.is_empty() {
        eprintln!("  Files:    {}", s.files_changed.len());
        for f in &s.files_changed {
            eprintln!("            {}", f.dimmed());
        }
    }
    if !s.commits.is_empty() {
        eprintln!("  Commits:  {}", s.commits.join(", "));
    }
    if let Some(ref env) = s.env {
        let mut parts = Vec::new();
        if let Some(ref os) = env.os {
            parts.push(format!("os={}", os));
        }
        if let Some(ref arch) = env.arch {
            parts.push(format!("arch={}", arch));
        }
        if let Some(ref branch) = env.branch {
            parts.push(format!("branch={}", branch));
        }
        if let Some(ref repo) = env.repo {
            parts.push(format!("repo={}", repo));
        }
        if let Some(ref user) = env.git_user {
            parts.push(format!("user={}", user));
        }
        if env.extended_thinking == Some(true) {
            parts.push("thinking=on".to_string());
        }
        if !parts.is_empty() {
            eprintln!("  Env:      {}", parts.join(", ").dimmed());
        }
    }

    // Show journal size if available
    let jsize = session_journal::journal_size(&s.session_id);
    if jsize > 0 {
        eprintln!("  Journal:  {:.1} KB", jsize as f64 / 1024.0);
    }
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

fn run_report(args: ReportArgs, json: bool) {
    let report = sessions::generate_report(args.days);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
        return;
    }

    eprintln!(
        "{}\n",
        format!("AI Usage Report (last {} days)", report.period_days).bold()
    );

    eprintln!("  Sessions:  {}", report.session_count);
    eprintln!("  Duration:  {}", format_duration(report.total_duration_s));
    if report.total_tokens.reasoning > 0 {
        eprintln!(
            "  Tokens:    {} total ({} in / {} out / {} reasoning)",
            report.total_tokens.total(),
            report.total_tokens.input,
            report.total_tokens.output,
            report.total_tokens.reasoning
        );
    } else {
        eprintln!(
            "  Tokens:    {} total ({} in / {} out)",
            report.total_tokens.total(),
            report.total_tokens.input,
            report.total_tokens.output
        );
    }
    eprintln!("  Tool calls: {}", report.total_tool_calls);
    eprintln!(
        "  Est. cost: {}",
        format!("${:.2}", report.total_est_cost_usd).green()
    );

    if !report.by_agent.is_empty() {
        eprintln!("\n{}", "By agent:".bold());
        for (agent, count, cost_val) in &report.by_agent {
            eprintln!("  {}  {} sessions, ${:.2}", agent.cyan(), count, cost_val);
        }
    }

    if !report.by_model.is_empty() {
        eprintln!("\n{}", "By model:".bold());
        for (model, count, tokens) in &report.by_model {
            eprintln!("  {}  {} sessions, {} tokens", model, count, tokens);
        }
    }

    if !report.top_tools.is_empty() {
        eprintln!("\n{}", "Top tools:".bold());
        for (tool, count) in report.top_tools.iter().take(10) {
            eprintln!("  {}  {} calls", tool, count);
        }
    }

    // Budget alert
    let cfg = config::load_config();
    let budget = cfg.tracking.budget_alert_usd;
    if budget > 0.0 {
        let pct = (report.total_est_cost_usd / budget) * 100.0;
        if pct >= 100.0 {
            eprintln!(
                "\n{}",
                format!(
                    "Budget exceeded: ${:.2} / ${:.2} ({:.0}%)",
                    report.total_est_cost_usd, budget, pct
                )
                .red()
                .bold()
            );
        } else if pct >= 80.0 {
            eprintln!(
                "\n{}",
                format!(
                    "Budget warning: ${:.2} / ${:.2} ({:.0}%)",
                    report.total_est_cost_usd, budget, pct
                )
                .yellow()
            );
        } else {
            eprintln!(
                "\n  Budget: ${:.2} / ${:.2} ({:.0}%)",
                report.total_est_cost_usd, budget, pct
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

fn run_export(args: ExportArgs, json: bool) {
    let session_list = sessions::list_sessions(args.days);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&session_list).unwrap_or_default()
        );
    } else {
        // Export as JSONL for piping
        for s in &session_list {
            println!("{}", serde_json::to_string(s).unwrap_or_default());
        }
    }
}

// ---------------------------------------------------------------------------
// Clear
// ---------------------------------------------------------------------------

fn run_clear(json: bool) {
    let count = session_journal::clear_journals();
    if json {
        println!(
            "{}",
            serde_json::json!({ "status": "cleared", "journals_removed": count })
        );
    } else {
        eprintln!(
            "{} ({} journal files removed)",
            "Local transcripts cleared.".green(),
            count
        );
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

async fn run_dashboard(args: DashboardArgs, _json: bool) {
    use axum::{extract::Query, routing::get, Json, Router};
    use tower_http::cors::CorsLayer;
    use tower_http::services::{ServeDir, ServeFile};

    #[derive(serde::Deserialize)]
    struct DaysQuery {
        #[serde(default = "default_days")]
        days: u64,
    }
    fn default_days() -> u64 {
        30
    }

    #[derive(serde::Deserialize)]
    struct SessionQuery {
        id: String,
    }

    // API routes
    let mut app = Router::new()
        .route("/api/status", get(|| async {
            let active = sessions::get_active_session();
            let entire_states = session_state::list_states();
            Json(serde_json::json!({
                "active_session": active.as_ref().map(|s| serde_json::json!({
                    "session_id": s.session_id,
                    "agent": s.agent,
                    "model": s.model,
                    "started_at": s.started_at,
                    "turns": s.turns,
                    "tool_calls": s.tool_calls,
                    "tokens": { "input": s.tokens.input, "output": s.tokens.output, "reasoning": s.tokens.reasoning, "total": s.tokens.total() },
                    "env": s.env,
                })),
                "agent_detected": detect_agent(),
                "model_detected": detect_model(),
                "entire_sessions": entire_states.len(),
            }))
        }))
        .route("/api/sessions", get(|Query(q): Query<DaysQuery>| async move {
            let session_list = sessions::list_sessions(q.days);
            Json(session_list)
        }))
        .route("/api/report", get(|Query(q): Query<DaysQuery>| async move {
            let report = sessions::generate_report(q.days);
            Json(report)
        }))
        .route("/api/session", get(|Query(q): Query<SessionQuery>| async move {
            let session = sessions::get_session(&q.id);
            Json(session)
        }))
        .route("/api/transcript", get(|Query(q): Query<SessionQuery>| async move {
            // Find transcript path from session state
            if let Some(state) = session_state::load_state(&q.id) {
                if let Some(ref t_path) = state.transcript_path {
                    let messages = transcript::parse_conversation(std::path::Path::new(t_path));
                    return Json(serde_json::json!({
                        "session_id": q.id,
                        "messages": messages,
                    }));
                }
            }
            // Try to find transcript by scanning
            if let Some(repo_path) = chub_core::team::project::find_project_root(None) {
                let repo_str = repo_path.to_string_lossy();
                if let Some(t_path) = transcript::find_transcript(&repo_str, &q.id) {
                    let messages = transcript::parse_conversation(&t_path);
                    return Json(serde_json::json!({
                        "session_id": q.id,
                        "messages": messages,
                    }));
                }
            }
            Json(serde_json::json!({
                "session_id": q.id,
                "messages": [],
                "error": "No transcript found",
            }))
        }))
        .route("/api/entire-states", get(|| async {
            let states = session_state::list_states();
            let summaries: Vec<_> = states.iter().map(|s| serde_json::json!({
                "sessionID": s.session_id,
                "phase": format!("{:?}", s.phase),
                "agentType": s.agent_type,
                "startedAt": s.started_at,
                "endedAt": s.ended_at,
                "stepCount": s.step_count,
                "filesTouched": &s.files_touched,
                "tool_calls": s.tool_calls,
                "commits": s.commits,
                "est_cost_usd": s.est_cost_usd,
                "transcriptPath": s.transcript_path,
            })).collect();
            Json(summaries)
        }))
        .layer(CorsLayer::permissive());

    // Serve React SPA from website/dashboard/dist if it exists, otherwise fallback HTML
    let dashboard_dir = find_dashboard_dir();
    if let Some(ref dir) = dashboard_dir {
        let index = dir.join("index.html");
        app = app.fallback_service(ServeDir::new(dir).not_found_service(ServeFile::new(index)));
        eprintln!("  Dashboard: React SPA from {}", dir.display());
    } else {
        app = app.route("/", get(dashboard_fallback_html));
        eprintln!("  Dashboard: built-in fallback (run `npm run build` in website/dashboard/ for full UI)");
    }

    let host: std::net::IpAddr = args.host.parse().unwrap_or_else(|_| {
        eprintln!("Invalid host, using 127.0.0.1");
        "127.0.0.1".parse().unwrap()
    });
    let addr = std::net::SocketAddr::from((host, args.port));

    eprintln!("{}\n", "Chub Tracking Dashboard".bold());
    eprintln!(
        "  {}",
        format!("http://localhost:{}", args.port).bold().underline()
    );
    eprintln!("\nPress Ctrl+C to stop.\n");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            output::error(
                &format!("Failed to bind to port {}: {}", args.port, e),
                false,
            );
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        output::error(&format!("Server error: {}", e), false);
    }
}

/// Find the dashboard SPA dist directory. Checks several locations:
/// 1. Next to the binary: <exe_dir>/dashboard/
/// 2. Project workspace: website/dashboard/dist/
/// 3. Relative to CWD: website/dashboard/dist/
fn find_dashboard_dir() -> Option<std::path::PathBuf> {
    // Next to binary
    if let Ok(exe) = std::env::current_exe() {
        let beside_exe = exe.parent().unwrap_or(exe.as_path()).join("dashboard");
        if beside_exe.join("index.html").exists() {
            return Some(beside_exe);
        }
    }

    // Project root (find .chub/ directory)
    if let Some(root) = chub_core::team::project::find_project_root(None) {
        let dist = root.join("website/dashboard/dist");
        if dist.join("index.html").exists() {
            return Some(dist);
        }
    }

    // CWD
    let cwd_dist = std::path::PathBuf::from("website/dashboard/dist");
    if cwd_dist.join("index.html").exists() {
        return Some(cwd_dist);
    }

    None
}

async fn dashboard_fallback_html() -> axum::response::Html<&'static str> {
    axum::response::Html(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Chub Dashboard</title>
<style>body{font-family:system-ui;background:#0d1117;color:#c9d1d9;display:flex;align-items:center;justify-content:center;min-height:100vh;margin:0;}
.c{text-align:center;max-width:500px;padding:32px;}h1{color:#58a6ff;margin-bottom:12px;}
code{background:#161b22;padding:4px 8px;border-radius:4px;font-size:14px;}
a{color:#58a6ff;}</style></head>
<body><div class="c">
<h1>Chub Dashboard</h1>
<p>The React dashboard is not built yet. Build it with:</p>
<p style="margin:16px 0"><code>cd website/dashboard && npm install && npm run build</code></p>
<p>Then restart <code>chub track dashboard</code>.</p>
<p style="margin-top:24px;color:#8b949e;">API is available at <a href="/api/status">/api/status</a>, <a href="/api/sessions">/api/sessions</a>, <a href="/api/report">/api/report</a></p>
</div></body></html>"#,
    )
}
