use clap::{Args, Subcommand};
use owo_colors::OwoColorize;

use chub_core::team::profiles;

use crate::output;

#[derive(Args)]
pub struct ProfileArgs {
    #[command(subcommand)]
    command: ProfileCommand,
}

#[derive(Subcommand)]
pub enum ProfileCommand {
    /// Activate a profile for this session
    Use(ProfileUseArgs),
    /// List available profiles
    List,
    /// Show the currently active profile
    Current,
}

#[derive(Args)]
pub struct ProfileUseArgs {
    /// Profile name (or "none" to clear)
    name: String,
}

pub fn run(args: ProfileArgs, json: bool) {
    match args.command {
        ProfileCommand::Use(use_args) => run_use(use_args, json),
        ProfileCommand::List => run_list(json),
        ProfileCommand::Current => run_current(json),
    }
}

fn run_use(args: ProfileUseArgs, json: bool) {
    let name = if args.name == "none" || args.name == "off" || args.name == "clear" {
        None
    } else {
        Some(args.name.as_str())
    };

    match profiles::set_active_profile(name) {
        Ok(()) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "ok",
                        "profile": name,
                    })
                );
            } else if let Some(n) = name {
                output::success(&format!("Profile set to: {}", n.bold()));
                // Show resolved profile info
                if let Ok(resolved) = profiles::resolve_profile(n) {
                    if !resolved.pins.is_empty() {
                        eprintln!("  Pins: {}", resolved.pins.join(", ").dimmed());
                    }
                    if !resolved.context.is_empty() {
                        eprintln!("  Context: {}", resolved.context.join(", ").dimmed());
                    }
                    if !resolved.rules.is_empty() {
                        eprintln!("  Rules: {} rule(s)", resolved.rules.len());
                    }
                }
            } else {
                output::success("Profile cleared.");
            }
        }
        Err(e) => {
            output::error(&e.to_string(), json);
            std::process::exit(1);
        }
    }
}

fn run_list(json: bool) {
    let profiles = profiles::list_profiles();
    let active = profiles::get_active_profile();

    if json {
        let items: Vec<serde_json::Value> = profiles
            .iter()
            .map(|(name, desc)| {
                serde_json::json!({
                    "name": name,
                    "description": desc,
                    "active": active.as_deref() == Some(name.as_str()),
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "profiles": items,
                "active": active,
            }))
            .unwrap_or_default()
        );
    } else {
        if profiles.is_empty() {
            eprintln!(
                "{}",
                "No profiles found. Create profiles in .chub/profiles/".dimmed()
            );
            return;
        }
        eprintln!("{}", "Available profiles:\n".bold());
        for (name, desc) in &profiles {
            let marker = if active.as_deref() == Some(name.as_str()) {
                " *".green().to_string()
            } else {
                String::new()
            };
            eprintln!("  {}{}", name.bold(), marker);
            if let Some(d) = desc {
                eprintln!("    {}", d.dimmed());
            }
        }
    }
}

fn run_current(json: bool) {
    let active = profiles::get_active_profile();

    if json {
        println!("{}", serde_json::json!({ "profile": active }));
    } else {
        match active {
            Some(name) => {
                eprintln!("Active profile: {}", name.bold());
                if let Ok(resolved) = profiles::resolve_profile(&name) {
                    if let Some(desc) = &resolved.description {
                        eprintln!("  {}", desc.dimmed());
                    }
                    if !resolved.pins.is_empty() {
                        eprintln!("  Pins: {}", resolved.pins.join(", "));
                    }
                    if !resolved.context.is_empty() {
                        eprintln!("  Context: {}", resolved.context.join(", "));
                    }
                    eprintln!("  Rules: {}", resolved.rules.len());
                }
            }
            None => eprintln!("{}", "No active profile.".dimmed()),
        }
    }
}
