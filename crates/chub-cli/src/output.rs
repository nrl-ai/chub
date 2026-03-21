#![allow(dead_code)] // Functions here are used across multiple command phases.

use owo_colors::OwoColorize;

/// Print an info message to stderr.
pub fn info(msg: &str) {
    eprintln!("{}", msg);
}

/// Print a warning to stderr.
pub fn warn(msg: &str) {
    eprintln!("{}", format!("Warning: {}", msg).yellow());
}

/// Print an error to stderr (or JSON to stdout).
pub fn error(msg: &str, json: bool) {
    if json {
        println!("{}", serde_json::json!({ "error": msg }));
    } else {
        eprintln!("{}", format!("Error: {}", msg).red());
    }
}

/// Print success message to stdout.
pub fn success(msg: &str) {
    println!("{}", msg.green());
}
