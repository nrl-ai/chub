//! `chub scan` — secret scanning, gitleaks/betterleaks-compatible.
//!
//! Subcommands:
//!   secrets git [repo]    — scan git history for secrets
//!   secrets dir [path]    — scan directory for secrets
//!   secrets stdin         — scan stdin for secrets

use clap::{Args, Subcommand};
use owo_colors::OwoColorize;
use std::path::PathBuf;

#[derive(Args)]
pub struct ScanArgs {
    #[command(subcommand)]
    command: ScanCommand,
}

#[derive(Subcommand)]
enum ScanCommand {
    /// Scan for secrets (drop-in replacement for gitleaks/betterleaks)
    Secrets(SecretsArgs),
}

#[derive(Args)]
struct SecretsArgs {
    #[command(subcommand)]
    command: SecretsCommand,

    /// Config file path (.gitleaks.toml / .betterleaks.toml / .chub-scan.toml)
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Baseline report path (filter out known findings)
    #[arg(short, long, global = true)]
    baseline_path: Option<String>,

    /// Report format: json, sarif, csv
    #[arg(short = 'f', long, global = true, default_value = "json")]
    report_format: String,

    /// Report output path (use - for stdout)
    #[arg(short = 'r', long, global = true)]
    report_path: Option<String>,

    /// Redact secrets in output (0-100%)
    #[arg(long, global = true)]
    redact: Option<Option<u8>>,

    /// Exit code when secrets are found (default: 1)
    #[arg(long, global = true, default_value = "1")]
    exit_code: i32,

    /// Max file size in megabytes to scan
    #[arg(long, global = true, default_value = "10")]
    max_target_megabytes: u64,

    /// Only enable specific rule IDs
    #[arg(long, global = true)]
    enable_rule: Vec<String>,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress banner
    #[arg(long, global = true)]
    no_banner: bool,

    /// Suppress colored output
    #[arg(long, global = true)]
    no_color: bool,
}

#[derive(Subcommand)]
enum SecretsCommand {
    /// Scan a git repository (history or staged changes)
    Git(GitArgs),
    /// Scan a directory or file
    #[command(alias = "file", alias = "directory")]
    Dir(DirArgs),
    /// Scan from standard input
    Stdin(StdinArgs),
}

#[derive(Args)]
struct GitArgs {
    /// Repository path (default: current directory)
    path: Option<String>,

    /// Scan only staged files (pre-commit mode)
    #[arg(long)]
    staged: bool,

    /// Alias for --staged
    #[arg(long)]
    pre_commit: bool,

    /// Custom git log options (e.g. "--all HEAD~10..HEAD")
    #[arg(long)]
    log_opts: Option<String>,

    /// Follow symlinks
    #[arg(long)]
    follow_symlinks: bool,

    /// Scan full blob content of every file in history (thorough but slow).
    /// Default: scan only added/modified files per commit, like gitleaks.
    #[arg(long)]
    thorough: bool,
}

#[derive(Args)]
struct DirArgs {
    /// Directory or file path to scan
    path: Option<String>,

    /// Follow symlinks
    #[arg(long)]
    follow_symlinks: bool,
}

#[derive(Args)]
struct StdinArgs {
    /// Label for the input source
    #[arg(long, default_value = "stdin")]
    label: String,
}

pub async fn run(args: ScanArgs, json: bool) -> Result<(), String> {
    match args.command {
        ScanCommand::Secrets(secrets_args) => run_secrets(secrets_args, json),
    }
}

fn run_secrets(args: SecretsArgs, cli_json: bool) -> Result<(), String> {
    use chub_core::scan::report::{write_report, ReportFormat};
    use chub_core::scan::scanner::{ScanOptions, Scanner};

    // Determine redaction percentage
    let redact_percent = match args.redact {
        Some(Some(p)) => p.min(100),
        Some(None) => 100, // --redact without value = 100%
        None => 0,
    };

    let thorough = matches!(&args.command, SecretsCommand::Git(g) if g.thorough);

    let options = ScanOptions {
        config_path: args.config.clone(),
        baseline_path: args.baseline_path.clone(),
        max_target_bytes: args.max_target_megabytes * 1024 * 1024,
        redact_percent,
        enable_rules: args.enable_rule.clone(),
        follow_symlinks: false,
        ignore_paths: Vec::new(),
        diff_only: !thorough,
    };

    let scanner = Scanner::new(options);

    if !args.no_banner && !cli_json {
        eprintln!(
            "    {} {} secret scanner ({} rules loaded)",
            "chub".bold(),
            "scan secrets".cyan(),
            scanner.rule_count()
        );
        eprintln!();
    }

    // Run the appropriate scan
    let findings = match args.command {
        SecretsCommand::Git(ref git_args) => {
            let repo_path = git_args
                .path
                .as_deref()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

            let staged = git_args.staged || git_args.pre_commit;

            if args.verbose && !cli_json {
                if staged {
                    eprintln!("  Scanning staged changes in {}", repo_path.display());
                } else {
                    eprintln!("  Scanning git history in {}", repo_path.display());
                }
            }

            scanner.scan_git(&repo_path, git_args.log_opts.as_deref(), staged)
        }
        SecretsCommand::Dir(ref dir_args) => {
            let dir_path = dir_args
                .path
                .as_deref()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

            if args.verbose && !cli_json {
                eprintln!("  Scanning directory {}", dir_path.display());
            }

            scanner.scan_dir(&dir_path)
        }
        SecretsCommand::Stdin(ref stdin_args) => {
            if args.verbose && !cli_json {
                eprintln!("  Scanning stdin (label: {})", stdin_args.label);
            }

            scanner.scan_reader(std::io::stdin(), &stdin_args.label)
        }
    };

    let count = findings.len();

    // Determine report format
    let format = if cli_json {
        ReportFormat::Json
    } else {
        // Try from explicit flag, then from file extension, fallback to json
        ReportFormat::parse(&args.report_format)
            .or_else(|| {
                args.report_path
                    .as_deref()
                    .and_then(ReportFormat::from_extension)
            })
            .unwrap_or(ReportFormat::Json)
    };

    // Write report
    if let Some(ref path) = args.report_path {
        if path == "-" {
            // Stdout
            let mut stdout = std::io::stdout();
            write_report(&findings, &mut stdout, format)
                .map_err(|e| format!("Failed to write report: {}", e))?;
        } else {
            let mut file = std::fs::File::create(path)
                .map_err(|e| format!("Cannot create {}: {}", path, e))?;
            write_report(&findings, &mut file, format)
                .map_err(|e| format!("Failed to write report: {}", e))?;
            if !cli_json {
                eprintln!("  Report written to {}", path);
            }
        }
    } else {
        // Default: print to stdout
        let mut stdout = std::io::stdout();
        write_report(&findings, &mut stdout, format)
            .map_err(|e| format!("Failed to write report: {}", e))?;
    }

    // Summary
    if !cli_json && !args.no_banner {
        eprintln!();
        if count > 0 {
            eprintln!("  {} found", format!("{} secret(s)", count).red().bold(),);
        } else {
            eprintln!("  {} No secrets found", "ok".green().bold());
        }
    }

    // Exit code
    if count > 0 {
        std::process::exit(args.exit_code);
    }

    Ok(())
}
