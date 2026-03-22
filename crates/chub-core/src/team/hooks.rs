//! Agent hook installation and git hook generation.
//!
//! Generates hook configurations for supported AI coding agents
//! and git hooks for commit-level session linking.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::team::project::find_project_root;

// ---------------------------------------------------------------------------
// Supported agents
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum AgentKind {
    ClaudeCode,
    Cursor,
    Copilot,
    GeminiCli,
    CodexCli,
}

impl AgentKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude-code" | "claude" | "claudecode" => Some(Self::ClaudeCode),
            "cursor" => Some(Self::Cursor),
            "copilot" | "copilot-cli" | "github-copilot" => Some(Self::Copilot),
            "gemini" | "gemini-cli" => Some(Self::GeminiCli),
            "codex" | "codex-cli" | "openai-codex" => Some(Self::CodexCli),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Cursor => "cursor",
            Self::Copilot => "copilot",
            Self::GeminiCli => "gemini-cli",
            Self::CodexCli => "codex",
        }
    }

    pub fn all() -> &'static [AgentKind] {
        &[
            AgentKind::ClaudeCode,
            AgentKind::Cursor,
            AgentKind::Copilot,
            AgentKind::GeminiCli,
            AgentKind::CodexCli,
        ]
    }
}

/// Auto-detect which agents are present in the project.
pub fn detect_agents(project_root: &Path) -> Vec<AgentKind> {
    let mut found = Vec::new();
    if project_root.join(".claude").is_dir() || project_root.join(".claude/settings.json").exists()
    {
        found.push(AgentKind::ClaudeCode);
    }
    if project_root.join(".cursor").is_dir() {
        found.push(AgentKind::Cursor);
    }
    if project_root.join(".github").is_dir() {
        found.push(AgentKind::Copilot);
    }
    if project_root.join(".gemini").is_dir() {
        found.push(AgentKind::GeminiCli);
    }
    if project_root.join(".codex").is_dir() {
        found.push(AgentKind::CodexCli);
    }
    found
}

// ---------------------------------------------------------------------------
// Hook installation results
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HookInstallResult {
    pub agent: String,
    pub config_file: String,
    pub action: HookAction,
}

#[derive(Debug, Clone)]
pub enum HookAction {
    Installed,
    AlreadyInstalled,
    Updated,
    Removed,
    Error(String),
}

// ---------------------------------------------------------------------------
// Claude Code hooks (.claude/settings.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Default)]
struct ClaudeSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hooks: Option<ClaudeHooks>,
    #[serde(flatten)]
    other: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "PascalCase")]
struct ClaudeHooks {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    session_start: Option<Vec<ClaudeHookEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<ClaudeHookEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    user_prompt_submit: Option<Vec<ClaudeHookEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pre_tool_use: Option<Vec<ClaudeHookEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    post_tool_use: Option<Vec<ClaudeHookEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    session_end: Option<Vec<ClaudeHookEntry>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ClaudeHookEntry {
    #[serde(default)]
    matcher: String,
    hooks: Vec<ClaudeHookCmd>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ClaudeHookCmd {
    #[serde(rename = "type")]
    cmd_type: String,
    command: String,
}

const CHUB_HOOK_MARKER: &str = "track hook";

/// Resolve the chub binary name. Uses the bare command name so hook configs
/// are portable across machines — chub must be on PATH.
fn resolve_chub_binary() -> String {
    "chub".to_string()
}

fn claude_hook_entry(event: &str, matcher: &str, chub_bin: &str) -> ClaudeHookEntry {
    ClaudeHookEntry {
        matcher: matcher.to_string(),
        hooks: vec![ClaudeHookCmd {
            cmd_type: "command".to_string(),
            // 2>/dev/null || true — never block the IDE if chub is missing or fails
            command: format!("{} track hook {} 2>/dev/null || true", chub_bin, event),
        }],
    }
}

fn is_chub_hook(entry: &ClaudeHookEntry) -> bool {
    entry
        .hooks
        .iter()
        .any(|h| h.command.contains(CHUB_HOOK_MARKER))
}

pub fn install_claude_code_hooks(project_root: &Path, force: bool) -> HookInstallResult {
    let config_dir = project_root.join(".claude");
    let _ = fs::create_dir_all(&config_dir);
    let config_path = config_dir.join("settings.json");

    let mut settings: ClaudeSettings = if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => ClaudeSettings::default(),
        }
    } else {
        ClaudeSettings::default()
    };

    let mut hooks = settings.hooks.unwrap_or_default();

    // Check if already installed
    let already_installed = hooks
        .session_start
        .as_ref()
        .map(|entries| entries.iter().any(is_chub_hook))
        .unwrap_or(false);

    if already_installed && !force {
        return HookInstallResult {
            agent: "claude-code".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::AlreadyInstalled,
        };
    }

    // Remove existing chub hooks if force
    if force {
        remove_chub_entries(&mut hooks.session_start);
        remove_chub_entries(&mut hooks.stop);
        remove_chub_entries(&mut hooks.user_prompt_submit);
        remove_chub_entries(&mut hooks.pre_tool_use);
        remove_chub_entries(&mut hooks.post_tool_use);
        remove_chub_entries(&mut hooks.session_end);
    }

    // Add chub hooks (resolve binary path so it works even if chub isn't on PATH)
    let chub_bin = resolve_chub_binary();
    append_hook_entry(
        &mut hooks.session_start,
        claude_hook_entry("session-start", "", &chub_bin),
    );
    append_hook_entry(
        &mut hooks.session_end,
        claude_hook_entry("stop", "", &chub_bin),
    );
    append_hook_entry(&mut hooks.stop, claude_hook_entry("stop", "", &chub_bin));
    append_hook_entry(
        &mut hooks.user_prompt_submit,
        claude_hook_entry("prompt", "", &chub_bin),
    );
    // Track all tool uses
    append_hook_entry(
        &mut hooks.pre_tool_use,
        claude_hook_entry("pre-tool", "", &chub_bin),
    );
    append_hook_entry(
        &mut hooks.post_tool_use,
        claude_hook_entry("post-tool", "", &chub_bin),
    );

    settings.hooks = Some(hooks);

    let json = match serde_json::to_string_pretty(&settings) {
        Ok(j) => j,
        Err(e) => {
            return HookInstallResult {
                agent: "claude-code".to_string(),
                config_file: config_path.display().to_string(),
                action: HookAction::Error(e.to_string()),
            }
        }
    };

    match crate::util::atomic_write(&config_path, json.as_bytes()) {
        Ok(_) => HookInstallResult {
            agent: "claude-code".to_string(),
            config_file: config_path.display().to_string(),
            action: if already_installed {
                HookAction::Updated
            } else {
                HookAction::Installed
            },
        },
        Err(e) => HookInstallResult {
            agent: "claude-code".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::Error(e.to_string()),
        },
    }
}

fn remove_chub_entries(entries: &mut Option<Vec<ClaudeHookEntry>>) {
    if let Some(ref mut v) = entries {
        v.retain(|e| !is_chub_hook(e));
        if v.is_empty() {
            *entries = None;
        }
    }
}

fn append_hook_entry(entries: &mut Option<Vec<ClaudeHookEntry>>, entry: ClaudeHookEntry) {
    let v = entries.get_or_insert_with(Vec::new);
    // Don't duplicate
    if !v.iter().any(is_chub_hook) {
        v.push(entry);
    }
}

pub fn uninstall_claude_code_hooks(project_root: &Path) -> HookInstallResult {
    let config_path = project_root.join(".claude/settings.json");
    if !config_path.exists() {
        return HookInstallResult {
            agent: "claude-code".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::Removed,
        };
    }

    let mut settings: ClaudeSettings = match fs::read_to_string(&config_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => {
            return HookInstallResult {
                agent: "claude-code".to_string(),
                config_file: config_path.display().to_string(),
                action: HookAction::Removed,
            }
        }
    };

    if let Some(ref mut hooks) = settings.hooks {
        remove_chub_entries(&mut hooks.session_start);
        remove_chub_entries(&mut hooks.stop);
        remove_chub_entries(&mut hooks.user_prompt_submit);
        remove_chub_entries(&mut hooks.pre_tool_use);
        remove_chub_entries(&mut hooks.post_tool_use);
        remove_chub_entries(&mut hooks.session_end);
    }

    let json = serde_json::to_string_pretty(&settings).unwrap_or_default();
    let _ = crate::util::atomic_write(&config_path, json.as_bytes());

    HookInstallResult {
        agent: "claude-code".to_string(),
        config_file: config_path.display().to_string(),
        action: HookAction::Removed,
    }
}

// ---------------------------------------------------------------------------
// Cursor hooks (.cursor/hooks.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Default)]
struct CursorHooksFile {
    #[serde(default)]
    version: u32,
    #[serde(default)]
    hooks: CursorHooks,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CursorHooks {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    session_start: Option<Vec<CursorHookCmd>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    session_end: Option<Vec<CursorHookCmd>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    before_submit_prompt: Option<Vec<CursorHookCmd>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<CursorHookCmd>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CursorHookCmd {
    command: String,
}

pub fn install_cursor_hooks(project_root: &Path, force: bool) -> HookInstallResult {
    let config_dir = project_root.join(".cursor");
    let _ = fs::create_dir_all(&config_dir);
    let config_path = config_dir.join("hooks.json");

    let mut file: CursorHooksFile = if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => CursorHooksFile::default(),
        }
    } else {
        CursorHooksFile::default()
    };
    file.version = 1;

    let already_installed = file
        .hooks
        .session_start
        .as_ref()
        .map(|cmds| cmds.iter().any(|c| c.command.contains(CHUB_HOOK_MARKER)))
        .unwrap_or(false);

    if already_installed && !force {
        return HookInstallResult {
            agent: "cursor".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::AlreadyInstalled,
        };
    }

    let chub_bin = resolve_chub_binary();
    let chub_cmd = |event: &str| CursorHookCmd {
        command: format!("{} track hook {} --agent cursor", chub_bin, event),
    };

    let append_cursor = |cmds: &mut Option<Vec<CursorHookCmd>>, cmd: CursorHookCmd| {
        let v = cmds.get_or_insert_with(Vec::new);
        if !v.iter().any(|c| c.command.contains(CHUB_HOOK_MARKER)) {
            v.push(cmd);
        }
    };

    if force {
        for v in [
            &mut file.hooks.session_start,
            &mut file.hooks.session_end,
            &mut file.hooks.before_submit_prompt,
            &mut file.hooks.stop,
        ]
        .into_iter()
        .flatten()
        {
            v.retain(|c| !c.command.contains(CHUB_HOOK_MARKER));
        }
    }

    append_cursor(&mut file.hooks.session_start, chub_cmd("session-start"));
    append_cursor(&mut file.hooks.session_end, chub_cmd("stop"));
    append_cursor(&mut file.hooks.before_submit_prompt, chub_cmd("prompt"));
    append_cursor(&mut file.hooks.stop, chub_cmd("stop"));

    let json = serde_json::to_string_pretty(&file).unwrap_or_default();
    match crate::util::atomic_write(&config_path, json.as_bytes()) {
        Ok(_) => HookInstallResult {
            agent: "cursor".to_string(),
            config_file: config_path.display().to_string(),
            action: if already_installed {
                HookAction::Updated
            } else {
                HookAction::Installed
            },
        },
        Err(e) => HookInstallResult {
            agent: "cursor".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::Error(e.to_string()),
        },
    }
}

pub fn uninstall_cursor_hooks(project_root: &Path) -> HookInstallResult {
    let config_path = project_root.join(".cursor/hooks.json");
    if !config_path.exists() {
        return HookInstallResult {
            agent: "cursor".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::Removed,
        };
    }

    let mut file: CursorHooksFile = match fs::read_to_string(&config_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => {
            return HookInstallResult {
                agent: "cursor".to_string(),
                config_file: config_path.display().to_string(),
                action: HookAction::Removed,
            }
        }
    };

    for v in [
        &mut file.hooks.session_start,
        &mut file.hooks.session_end,
        &mut file.hooks.before_submit_prompt,
        &mut file.hooks.stop,
    ]
    .into_iter()
    .flatten()
    {
        v.retain(|c| !c.command.contains(CHUB_HOOK_MARKER));
    }

    let json = serde_json::to_string_pretty(&file).unwrap_or_default();
    let _ = crate::util::atomic_write(&config_path, json.as_bytes());

    HookInstallResult {
        agent: "cursor".to_string(),
        config_file: config_path.display().to_string(),
        action: HookAction::Removed,
    }
}

// ---------------------------------------------------------------------------
// Gemini CLI hooks (.gemini/settings.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Default)]
struct GeminiSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hooks: Option<GeminiHooks>,
    #[serde(flatten)]
    other: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "PascalCase")]
struct GeminiHooks {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    session_start: Option<Vec<GeminiHookEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    session_end: Option<Vec<GeminiHookEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    before_tool: Option<Vec<GeminiHookEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    after_tool: Option<Vec<GeminiHookEntry>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GeminiHookEntry {
    command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    matcher: Option<String>,
}

fn gemini_hook_entry(event: &str, chub_bin: &str) -> GeminiHookEntry {
    GeminiHookEntry {
        command: format!(
            "{} track hook {} --agent gemini-cli 2>/dev/null || true",
            chub_bin, event
        ),
        matcher: None,
    }
}

fn is_chub_gemini_hook(entry: &GeminiHookEntry) -> bool {
    entry.command.contains(CHUB_HOOK_MARKER)
}

pub fn install_gemini_hooks(project_root: &Path, force: bool) -> HookInstallResult {
    let config_dir = project_root.join(".gemini");
    let _ = fs::create_dir_all(&config_dir);
    let config_path = config_dir.join("settings.json");

    let mut settings: GeminiSettings = if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => GeminiSettings::default(),
        }
    } else {
        GeminiSettings::default()
    };

    let mut hooks = settings.hooks.unwrap_or_default();

    let already_installed = hooks
        .session_start
        .as_ref()
        .map(|entries| entries.iter().any(is_chub_gemini_hook))
        .unwrap_or(false);

    if already_installed && !force {
        return HookInstallResult {
            agent: "gemini-cli".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::AlreadyInstalled,
        };
    }

    let chub_bin = resolve_chub_binary();

    let append_gemini = |entries: &mut Option<Vec<GeminiHookEntry>>, entry: GeminiHookEntry| {
        let v = entries.get_or_insert_with(Vec::new);
        if !v.iter().any(is_chub_gemini_hook) {
            v.push(entry);
        }
    };

    if force {
        for v in [
            &mut hooks.session_start,
            &mut hooks.session_end,
            &mut hooks.before_tool,
            &mut hooks.after_tool,
        ]
        .into_iter()
        .flatten()
        {
            v.retain(|e| !is_chub_gemini_hook(e));
        }
    }

    append_gemini(
        &mut hooks.session_start,
        gemini_hook_entry("session-start", &chub_bin),
    );
    append_gemini(&mut hooks.session_end, gemini_hook_entry("stop", &chub_bin));
    append_gemini(
        &mut hooks.before_tool,
        gemini_hook_entry("pre-tool", &chub_bin),
    );
    append_gemini(
        &mut hooks.after_tool,
        gemini_hook_entry("post-tool", &chub_bin),
    );

    settings.hooks = Some(hooks);

    let json = serde_json::to_string_pretty(&settings).unwrap_or_default();
    match crate::util::atomic_write(&config_path, json.as_bytes()) {
        Ok(_) => HookInstallResult {
            agent: "gemini-cli".to_string(),
            config_file: config_path.display().to_string(),
            action: if already_installed {
                HookAction::Updated
            } else {
                HookAction::Installed
            },
        },
        Err(e) => HookInstallResult {
            agent: "gemini-cli".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::Error(e.to_string()),
        },
    }
}

pub fn uninstall_gemini_hooks(project_root: &Path) -> HookInstallResult {
    let config_path = project_root.join(".gemini/settings.json");
    if !config_path.exists() {
        return HookInstallResult {
            agent: "gemini-cli".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::Removed,
        };
    }

    let mut settings: GeminiSettings = match fs::read_to_string(&config_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => {
            return HookInstallResult {
                agent: "gemini-cli".to_string(),
                config_file: config_path.display().to_string(),
                action: HookAction::Removed,
            }
        }
    };

    if let Some(ref mut hooks) = settings.hooks {
        for v in [
            &mut hooks.session_start,
            &mut hooks.session_end,
            &mut hooks.before_tool,
            &mut hooks.after_tool,
        ]
        .into_iter()
        .flatten()
        {
            v.retain(|e| !is_chub_gemini_hook(e));
        }
    }

    let json = serde_json::to_string_pretty(&settings).unwrap_or_default();
    let _ = crate::util::atomic_write(&config_path, json.as_bytes());

    HookInstallResult {
        agent: "gemini-cli".to_string(),
        config_file: config_path.display().to_string(),
        action: HookAction::Removed,
    }
}

// ---------------------------------------------------------------------------
// Copilot hooks (.github/hooks/chub-tracking.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct CopilotHooksFile {
    version: u32,
    hooks: CopilotHooks,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CopilotHooks {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    session_start: Option<Vec<CopilotHookDef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    session_end: Option<Vec<CopilotHookDef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    user_prompt_submitted: Option<Vec<CopilotHookDef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pre_tool_use: Option<Vec<CopilotHookDef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    post_tool_use: Option<Vec<CopilotHookDef>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CopilotHookDef {
    #[serde(rename = "type")]
    hook_type: String,
    bash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    timeout_sec: Option<u32>,
}

fn copilot_hook_def(event: &str, chub_bin: &str) -> CopilotHookDef {
    CopilotHookDef {
        hook_type: "command".to_string(),
        bash: format!(
            "{} track hook {} --agent copilot 2>/dev/null || true",
            chub_bin, event
        ),
        timeout_sec: Some(10),
    }
}

pub fn install_copilot_hooks(project_root: &Path, force: bool) -> HookInstallResult {
    let hooks_dir = project_root.join(".github/hooks");
    let _ = fs::create_dir_all(&hooks_dir);
    let config_path = hooks_dir.join("chub-tracking.json");

    if config_path.exists() && !force {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if content.contains(CHUB_HOOK_MARKER) {
                return HookInstallResult {
                    agent: "copilot".to_string(),
                    config_file: config_path.display().to_string(),
                    action: HookAction::AlreadyInstalled,
                };
            }
        }
    }

    let chub_bin = resolve_chub_binary();
    let file = CopilotHooksFile {
        version: 1,
        hooks: CopilotHooks {
            session_start: Some(vec![copilot_hook_def("session-start", &chub_bin)]),
            session_end: Some(vec![copilot_hook_def("stop", &chub_bin)]),
            user_prompt_submitted: Some(vec![copilot_hook_def("prompt", &chub_bin)]),
            pre_tool_use: Some(vec![copilot_hook_def("pre-tool", &chub_bin)]),
            post_tool_use: Some(vec![copilot_hook_def("post-tool", &chub_bin)]),
        },
    };

    let json = serde_json::to_string_pretty(&file).unwrap_or_default();
    match crate::util::atomic_write(&config_path, json.as_bytes()) {
        Ok(_) => HookInstallResult {
            agent: "copilot".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::Installed,
        },
        Err(e) => HookInstallResult {
            agent: "copilot".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::Error(e.to_string()),
        },
    }
}

pub fn uninstall_copilot_hooks(project_root: &Path) -> HookInstallResult {
    let config_path = project_root.join(".github/hooks/chub-tracking.json");
    if config_path.exists() {
        let _ = fs::remove_file(&config_path);
    }
    HookInstallResult {
        agent: "copilot".to_string(),
        config_file: config_path.display().to_string(),
        action: HookAction::Removed,
    }
}

// ---------------------------------------------------------------------------
// Codex CLI hooks (appended to config.toml)
// ---------------------------------------------------------------------------

pub fn install_codex_hooks(project_root: &Path, force: bool) -> HookInstallResult {
    // Codex uses ~/.codex/config.toml or project-level config.
    // We write a project-level .codex/config.toml with [[hooks]] entries.
    let config_dir = project_root.join(".codex");
    let _ = fs::create_dir_all(&config_dir);
    let config_path = config_dir.join("config.toml");

    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if content.contains(CHUB_HOOK_MARKER) && !force {
                return HookInstallResult {
                    agent: "codex".to_string(),
                    config_file: config_path.display().to_string(),
                    action: HookAction::AlreadyInstalled,
                };
            }
        }
    }

    let chub_bin = resolve_chub_binary();
    let hooks_block = format!(
        r#"
{marker}
[[hooks]]
event = "SessionStart"
command = "{chub_bin} track hook session-start --agent codex 2>/dev/null || true"

[[hooks]]
event = "Stop"
command = "{chub_bin} track hook stop --agent codex 2>/dev/null || true"

[[hooks]]
event = "UserPromptSubmit"
command = "{chub_bin} track hook prompt --agent codex 2>/dev/null || true"

[[hooks]]
event = "AfterToolUse"
command = "{chub_bin} track hook post-tool --agent codex 2>/dev/null || true"
"#,
        marker = GIT_HOOK_MARKER,
        chub_bin = chub_bin,
    );

    // If file exists and doesn't have our marker, append; otherwise write fresh
    let content = if config_path.exists() {
        let existing = fs::read_to_string(&config_path).unwrap_or_default();
        if force {
            // Remove old chub section and re-add
            let cleaned: String = existing
                .split('\n')
                .scan(false, |in_chub, line| {
                    if line.contains(GIT_HOOK_MARKER) {
                        *in_chub = true;
                        return Some(String::new());
                    }
                    if *in_chub {
                        // Skip until next non-hook section
                        if line.starts_with('[') && !line.starts_with("[[hooks]]") {
                            *in_chub = false;
                            return Some(line.to_string());
                        }
                        return Some(String::new());
                    }
                    Some(line.to_string())
                })
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            format!("{}\n{}", cleaned.trim_end(), hooks_block)
        } else {
            format!("{}\n{}", existing.trim_end(), hooks_block)
        }
    } else {
        hooks_block.trim_start().to_string()
    };

    match crate::util::atomic_write(&config_path, content.as_bytes()) {
        Ok(_) => HookInstallResult {
            agent: "codex".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::Installed,
        },
        Err(e) => HookInstallResult {
            agent: "codex".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::Error(e.to_string()),
        },
    }
}

pub fn uninstall_codex_hooks(project_root: &Path) -> HookInstallResult {
    let config_path = project_root.join(".codex/config.toml");
    if !config_path.exists() {
        return HookInstallResult {
            agent: "codex".to_string(),
            config_file: config_path.display().to_string(),
            action: HookAction::Removed,
        };
    }

    if let Ok(content) = fs::read_to_string(&config_path) {
        // Remove chub hooks section
        let cleaned: String = content
            .split('\n')
            .scan(false, |in_chub, line| {
                if line.contains(GIT_HOOK_MARKER) {
                    *in_chub = true;
                    return Some(String::new());
                }
                if *in_chub {
                    if line.starts_with('[') && !line.starts_with("[[hooks]]") {
                        *in_chub = false;
                        return Some(line.to_string());
                    }
                    return Some(String::new());
                }
                Some(line.to_string())
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        let _ = crate::util::atomic_write(&config_path, cleaned.trim().as_bytes());
    }

    HookInstallResult {
        agent: "codex".to_string(),
        config_file: config_path.display().to_string(),
        action: HookAction::Removed,
    }
}

// ---------------------------------------------------------------------------
// Git hooks (prepare-commit-msg, post-commit)
// ---------------------------------------------------------------------------

const GIT_HOOK_MARKER: &str = "# chub track hooks";

pub fn install_git_hooks(project_root: &Path) -> Result<Vec<HookInstallResult>> {
    let git_dir = project_root.join(".git");
    if !git_dir.is_dir() {
        return Err(Error::Config("Not a git repository.".to_string()));
    }
    let hooks_dir = git_dir.join("hooks");
    let _ = fs::create_dir_all(&hooks_dir);
    let chub_bin = resolve_chub_binary();

    let mut results = Vec::new();

    // prepare-commit-msg — add Chub-Session and Chub-Checkpoint trailers
    results.push(install_one_git_hook(
        &hooks_dir,
        "prepare-commit-msg",
        &format!(
            r#"#!/bin/sh
{marker}
"{chub_bin}" track hook commit-msg --input "$1" 2>/dev/null || true
"#,
            marker = GIT_HOOK_MARKER,
            chub_bin = chub_bin,
        ),
    ));

    // post-commit — snapshot session summary
    results.push(install_one_git_hook(
        &hooks_dir,
        "post-commit",
        &format!(
            r#"#!/bin/sh
{marker}
"{chub_bin}" track hook post-commit 2>/dev/null || true
"#,
            marker = GIT_HOOK_MARKER,
            chub_bin = chub_bin,
        ),
    ));

    // pre-push — sync session data branch to remote
    results.push(install_one_git_hook(
        &hooks_dir,
        "pre-push",
        &format!(
            r#"#!/bin/sh
{marker}
"{chub_bin}" track hook pre-push --input "$1" 2>/dev/null || true
"#,
            marker = GIT_HOOK_MARKER,
            chub_bin = chub_bin,
        ),
    ));

    Ok(results)
}

fn install_one_git_hook(hooks_dir: &Path, name: &str, content: &str) -> HookInstallResult {
    let hook_path = hooks_dir.join(name);

    // Check if our hook is already installed
    if hook_path.exists() {
        if let Ok(existing) = fs::read_to_string(&hook_path) {
            if existing.contains(GIT_HOOK_MARKER) {
                return HookInstallResult {
                    agent: "git".to_string(),
                    config_file: hook_path.display().to_string(),
                    action: HookAction::AlreadyInstalled,
                };
            }
            // Existing hook that's not ours — back it up and chain
            let backup = hooks_dir.join(format!("{}.pre-chub", name));
            let _ = fs::rename(&hook_path, &backup);

            let chained = format!(
                r#"{content}
# Chain: run pre-existing hook
_chub_hook_dir="$(dirname "$0")"
if [ -x "$_chub_hook_dir/{name}.pre-chub" ]; then
    "$_chub_hook_dir/{name}.pre-chub" "$@"
fi
"#
            );
            return match fs::write(&hook_path, chained) {
                Ok(_) => {
                    set_executable(&hook_path);
                    HookInstallResult {
                        agent: "git".to_string(),
                        config_file: hook_path.display().to_string(),
                        action: HookAction::Installed,
                    }
                }
                Err(e) => HookInstallResult {
                    agent: "git".to_string(),
                    config_file: hook_path.display().to_string(),
                    action: HookAction::Error(e.to_string()),
                },
            };
        }
    }

    match fs::write(&hook_path, content) {
        Ok(_) => {
            set_executable(&hook_path);
            HookInstallResult {
                agent: "git".to_string(),
                config_file: hook_path.display().to_string(),
                action: HookAction::Installed,
            }
        }
        Err(e) => HookInstallResult {
            agent: "git".to_string(),
            config_file: hook_path.display().to_string(),
            action: HookAction::Error(e.to_string()),
        },
    }
}

pub fn uninstall_git_hooks(project_root: &Path) -> Vec<HookInstallResult> {
    let hooks_dir = project_root.join(".git/hooks");
    let mut results = Vec::new();

    for name in &["prepare-commit-msg", "post-commit", "pre-push"] {
        let hook_path = hooks_dir.join(name);
        let backup = hooks_dir.join(format!("{}.pre-chub", name));

        if hook_path.exists() {
            if let Ok(content) = fs::read_to_string(&hook_path) {
                if content.contains(GIT_HOOK_MARKER) {
                    let _ = fs::remove_file(&hook_path);
                    // Restore backup if it exists
                    if backup.exists() {
                        let _ = fs::rename(&backup, &hook_path);
                    }
                }
            }
        }

        results.push(HookInstallResult {
            agent: "git".to_string(),
            config_file: hook_path.display().to_string(),
            action: HookAction::Removed,
        });
    }

    results
}

#[cfg(unix)]
fn set_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_mode(0o755);
        let _ = fs::set_permissions(path, perms);
    }
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) {
    // On Windows, files are executable by default
}

// ---------------------------------------------------------------------------
// Combined install/uninstall
// ---------------------------------------------------------------------------

/// Install hooks for a specific agent (or all detected agents).
pub fn install_hooks(agent: Option<&str>, force: bool) -> Result<Vec<HookInstallResult>> {
    let project_root = find_project_root(None).ok_or_else(|| {
        Error::Config("No .chub/ directory found. Run `chub init` first.".to_string())
    })?;

    let mut results = Vec::new();

    let agents = if let Some(name) = agent {
        let kind = AgentKind::parse(name).ok_or_else(|| {
            Error::Config(format!(
                "Unknown agent: \"{}\". Supported: claude-code, cursor, copilot, gemini-cli, codex",
                name
            ))
        })?;
        vec![kind]
    } else {
        let detected = detect_agents(&project_root);
        if detected.is_empty() {
            // Default to Claude Code
            vec![AgentKind::ClaudeCode]
        } else {
            detected
        }
    };

    for kind in &agents {
        let result = match kind {
            AgentKind::ClaudeCode => install_claude_code_hooks(&project_root, force),
            AgentKind::Cursor => install_cursor_hooks(&project_root, force),
            AgentKind::Copilot => install_copilot_hooks(&project_root, force),
            AgentKind::GeminiCli => install_gemini_hooks(&project_root, force),
            AgentKind::CodexCli => install_codex_hooks(&project_root, force),
        };
        results.push(result);
    }

    // Always install git hooks
    match install_git_hooks(&project_root) {
        Ok(git_results) => results.extend(git_results),
        Err(e) => results.push(HookInstallResult {
            agent: "git".to_string(),
            config_file: ".git/hooks/".to_string(),
            action: HookAction::Error(e.to_string()),
        }),
    }

    Ok(results)
}

/// Remove all chub hooks.
pub fn uninstall_hooks() -> Result<Vec<HookInstallResult>> {
    let project_root = find_project_root(None)
        .ok_or_else(|| Error::Config("No .chub/ directory found.".to_string()))?;

    let mut results = vec![
        uninstall_claude_code_hooks(&project_root),
        uninstall_cursor_hooks(&project_root),
        uninstall_copilot_hooks(&project_root),
        uninstall_gemini_hooks(&project_root),
        uninstall_codex_hooks(&project_root),
    ];
    results.extend(uninstall_git_hooks(&project_root));
    Ok(results)
}

// ---------------------------------------------------------------------------
// Hook stdin parsing — reads data from agent hooks
// ---------------------------------------------------------------------------

/// Data passed by Claude Code hooks via stdin.
#[derive(Debug, Deserialize, Default)]
pub struct ClaudeCodeHookInput {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub tool_use_id: Option<String>,
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    #[serde(default)]
    pub tool_response: Option<serde_json::Value>,
}

/// Data passed by Cursor hooks via stdin.
#[derive(Debug, Deserialize, Default)]
pub struct CursorHookInput {
    #[serde(default)]
    pub conversation_id: Option<String>,
    #[serde(default)]
    pub generation_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub cursor_version: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub modified_files: Option<Vec<String>>,
    #[serde(default)]
    pub context_tokens: Option<u64>,
    #[serde(default)]
    pub context_window_size: Option<u64>,
}

/// Parse stdin JSON from an agent hook. Returns generic key-value map.
pub fn parse_hook_stdin() -> Option<serde_json::Value> {
    use std::io::Read;
    let mut input = String::new();
    // Non-blocking: try to read stdin, but don't hang if there's no input
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    // Read with a small buffer — hooks should send data quickly
    match handle.read_to_string(&mut input) {
        Ok(0) => None,
        Ok(_) => serde_json::from_str(&input).ok(),
        Err(_) => None,
    }
}

/// Extract tool name from Claude Code's tool_input or tool_use context.
pub fn extract_tool_name(hook_input: &serde_json::Value) -> Option<String> {
    // Claude Code PreToolUse passes tool_name at top level in some versions,
    // or we can extract it from the tool_input structure
    hook_input
        .get("tool_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Extract file path from tool input (Write/Edit tools).
/// Returns a relative path when the file is inside the project root.
pub fn extract_file_path(tool_input: &serde_json::Value) -> Option<String> {
    tool_input
        .get("file_path")
        .or_else(|| tool_input.get("notebook_path"))
        .and_then(|v| v.as_str())
        .map(relativize_path)
}

/// Convert an absolute path to a relative one if it falls under the project root or CWD.
pub fn relativize_path(path: &str) -> String {
    let p = std::path::Path::new(path);
    if p.is_relative() {
        return path.to_string();
    }
    // Try project root first
    if let Some(root) = find_project_root(None) {
        if let Ok(rel) = p.strip_prefix(&root) {
            return rel.to_string_lossy().replace('\\', "/");
        }
    }
    // Fall back to CWD
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(rel) = p.strip_prefix(&cwd) {
            return rel.to_string_lossy().replace('\\', "/");
        }
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_agent_kind() {
        assert_eq!(AgentKind::parse("claude-code"), Some(AgentKind::ClaudeCode));
        assert_eq!(AgentKind::parse("claude"), Some(AgentKind::ClaudeCode));
        assert_eq!(AgentKind::parse("cursor"), Some(AgentKind::Cursor));
        assert_eq!(AgentKind::parse("copilot"), Some(AgentKind::Copilot));
        assert_eq!(AgentKind::parse("gemini"), Some(AgentKind::GeminiCli));
        assert_eq!(AgentKind::parse("codex"), Some(AgentKind::CodexCli));
        assert!(AgentKind::parse("vim").is_none());
    }

    #[test]
    fn claude_hook_entry_contains_marker() {
        let entry = claude_hook_entry("session-start", "", "chub");
        assert!(is_chub_hook(&entry));
    }

    #[test]
    fn extract_file_path_from_tool_input() {
        let input = serde_json::json!({
            "file_path": "/src/main.rs",
            "content": "fn main() {}"
        });
        assert_eq!(extract_file_path(&input), Some("/src/main.rs".to_string()));
    }
}
