use std::fs;
use std::io::IsTerminal;

use chub_core::config::chub_dir;
use owo_colors::OwoColorize;

const WELCOME_MARKER: &str = ".welcome_shown";

/// Show the first-run welcome notice if it hasn't been shown yet.
pub fn show_welcome_if_needed(json: bool) {
    if json {
        return;
    }

    // Don't show in non-interactive environments (piped output)
    if !std::io::stdout().is_terminal() || !std::io::stderr().is_terminal() {
        return;
    }

    let chub = chub_dir();
    let marker_path = chub.join(WELCOME_MARKER);
    let config_path = chub.join("config.yaml");

    if marker_path.exists() {
        return;
    }

    eprintln!(
        "\n{} Chub helps your AI coding agents make API calls correctly, by providing \
the latest documentation.\n\n\
By using chub, you agree to the Terms of Service at {}\n\n\
Chub asks agents to provide feedback on documentation, and this feedback is used to improve docs for the developer \
community. If you wish to disable this feedback, add {} to {}. See \
{} for details.\n",
        "Welcome to Context Hub (chub)!".bold(),
        "https://www.aichub.org/tos.html".underline(),
        "\"feedback: false\"".bold(),
        config_path.display().to_string().bold(),
        "https://github.com/nrl-ai/chub".underline(),
    );

    // Best-effort marker write
    let _ = fs::create_dir_all(&chub);
    let _ = fs::write(&marker_path, chub_core::util::now_iso8601());
}
