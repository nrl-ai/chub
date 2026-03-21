use std::sync::Arc;

use chub_core::annotations::{
    clear_annotation, list_annotations, read_annotation, write_annotation,
};
use chub_core::fetch::{fetch_doc, fetch_doc_full};
use chub_core::registry::{
    get_entry, list_entries, resolve_doc_path, resolve_entry_file, search_entries, MergedRegistry,
    ResolvedPath, SearchFilters, TaggedEntry,
};
use chub_core::team;
use chub_core::telemetry::{is_feedback_enabled, send_feedback, FeedbackOpts, VALID_LABELS};

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{schemars, tool, tool_router};

fn text_result(data: impl serde::Serialize) -> String {
    serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string())
}

fn simplify_entry(entry: &TaggedEntry) -> serde_json::Value {
    let mut val = serde_json::json!({
        "id": entry.id(),
        "name": entry.name(),
        "type": entry.entry_type,
        "description": entry.description(),
        "tags": entry.tags(),
    });
    if let Some(languages) = entry.languages() {
        val["languages"] =
            serde_json::json!(languages.iter().map(|l| &l.language).collect::<Vec<_>>());
    }
    val
}

// --- Tool parameter structs ---

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    /// Search query. Omit to list all entries.
    #[schemars(default)]
    pub query: Option<String>,
    /// Comma-separated tag filter (e.g. "openai,chat")
    #[schemars(default)]
    pub tags: Option<String>,
    /// Filter by language (e.g. "python", "js")
    #[schemars(default)]
    pub lang: Option<String>,
    /// Max results (default 20)
    #[schemars(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetParams {
    /// Entry ID (e.g. "openai/chat", "stripe/api"). Use source:id for disambiguation.
    pub id: String,
    /// Language variant (e.g. "python", "js"). Auto-selected if only one.
    #[schemars(default)]
    pub lang: Option<String>,
    /// Specific version (e.g. "1.52.0"). Defaults to recommended.
    #[schemars(default)]
    pub version: Option<String>,
    /// Fetch all files, not just the entry point (default false)
    #[schemars(default)]
    pub full: Option<bool>,
    /// Fetch a specific file by path (e.g. "references/streaming.md")
    #[schemars(default)]
    pub file: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListParams {
    /// Comma-separated tag filter
    #[schemars(default)]
    pub tags: Option<String>,
    /// Filter by language
    #[schemars(default)]
    pub lang: Option<String>,
    /// Max entries (default 50)
    #[schemars(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AnnotateParams {
    /// Entry ID to annotate. Required unless using list mode.
    #[schemars(default)]
    pub id: Option<String>,
    /// Annotation text to save. Omit to read existing annotation.
    #[schemars(default)]
    pub note: Option<String>,
    /// Remove the annotation for this entry (default false)
    #[schemars(default)]
    pub clear: Option<bool>,
    /// List all annotations (default false). When true, id is not needed.
    #[schemars(default)]
    pub list: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FeedbackParams {
    /// Entry ID to rate (e.g. "openai/chat")
    pub id: String,
    /// Thumbs up or down
    pub rating: String,
    /// Optional comment explaining the rating
    #[schemars(default)]
    pub comment: Option<String>,
    /// Entry type. Auto-detected if omitted.
    #[serde(rename = "type")]
    #[schemars(default)]
    pub entry_type: Option<String>,
    /// Language variant rated
    #[schemars(default)]
    pub lang: Option<String>,
    /// Version rated
    #[schemars(default)]
    pub version: Option<String>,
    /// Specific file rated
    #[schemars(default)]
    pub file: Option<String>,
    /// Structured feedback labels
    #[schemars(default)]
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[allow(dead_code)] // Fields exposed via MCP JSON schema, read by serde
pub struct ContextParams {
    /// Task description to find relevant context for
    #[schemars(default)]
    pub task: Option<String>,
    /// Files currently open (for auto-profile detection)
    #[schemars(default)]
    pub files_open: Option<Vec<String>>,
    /// Profile name to scope context to
    #[schemars(default)]
    pub profile: Option<String>,
    /// Maximum token budget (soft limit)
    #[schemars(default)]
    pub max_tokens: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PinsParams {
    /// Entry ID to pin or unpin
    #[schemars(default)]
    pub id: Option<String>,
    /// Language variant
    #[schemars(default)]
    pub lang: Option<String>,
    /// Version to lock
    #[schemars(default)]
    pub version: Option<String>,
    /// Reason for pinning
    #[schemars(default)]
    pub reason: Option<String>,
    /// Remove the pin (default false)
    #[schemars(default)]
    pub remove: Option<bool>,
    /// List all pins (default false)
    #[schemars(default)]
    pub list: Option<bool>,
}

// --- MCP Server ---

#[derive(Debug, Clone)]
pub struct ChubMcpServer {
    pub merged: Arc<MergedRegistry>,
    pub tool_router: ToolRouter<Self>,
}

impl ChubMcpServer {
    pub fn new(merged: Arc<MergedRegistry>) -> Self {
        Self {
            merged,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl ChubMcpServer {
    #[tool(
        name = "chub_search",
        description = "Search Context Hub for docs and skills by query, tags, or language"
    )]
    async fn handle_search(&self, Parameters(params): Parameters<SearchParams>) -> String {
        let limit = params.limit.unwrap_or(20);
        let filters = SearchFilters {
            tags: params.tags,
            lang: params.lang,
        };

        let entries = if let Some(ref query) = params.query {
            search_entries(query, &filters, &self.merged)
        } else {
            list_entries(&filters, &self.merged)
        };

        let sliced: Vec<_> = entries.iter().take(limit).collect();
        text_result(serde_json::json!({
            "results": sliced.iter().map(|e| simplify_entry(e)).collect::<Vec<_>>(),
            "total": entries.len(),
            "showing": sliced.len(),
        }))
    }

    #[tool(
        name = "chub_get",
        description = "Fetch the content of a doc or skill by ID from Context Hub"
    )]
    async fn handle_get(&self, Parameters(params): Parameters<GetParams>) -> String {
        // Validate file parameter (path traversal) — normalize and reject suspicious paths
        if let Some(ref file) = params.file {
            let normalized = std::path::Path::new("/")
                .join(file)
                .to_string_lossy()
                .to_string();
            let normalized = normalized.trim_start_matches('/').to_string();
            if normalized != *file || file.contains("..") {
                return text_result(serde_json::json!({
                    "error": format!("Invalid file path: \"{}\". Path traversal is not allowed.", file),
                }));
            }
        }

        let result = get_entry(&params.id, &self.merged);

        if result.ambiguous {
            return text_result(serde_json::json!({
                "error": format!("Ambiguous entry ID \"{}\". Be specific:", params.id),
                "alternatives": result.alternatives,
            }));
        }

        let entry = match result.entry {
            Some(e) => e,
            None => {
                return text_result(serde_json::json!({
                    "error": format!("Entry \"{}\" not found.", params.id),
                    "suggestion": "Use chub_search to find available entries.",
                }));
            }
        };

        let entry_type = entry.entry_type;

        // Apply pin overrides: if the entry is pinned, use pinned lang/version as defaults
        let mut effective_lang = params.lang.clone();
        let mut effective_version = params.version.clone();
        let mut pin_notice = String::new();
        if let Some(pin) = team::pins::get_pin(entry.id()) {
            if effective_lang.is_none() {
                effective_lang = pin.lang.clone();
            }
            if effective_version.is_none() {
                effective_version = pin.version.clone();
            }
            pin_notice = team::team_annotations::get_pin_notice(
                entry.id(),
                pin.version.as_deref(),
                pin.lang.as_deref(),
                pin.reason.as_deref(),
            );
        }

        let resolved = resolve_doc_path(
            &entry,
            effective_lang.as_deref(),
            effective_version.as_deref(),
        );

        let resolved = match resolved {
            Some(r) => r,
            None => {
                return text_result(serde_json::json!({
                    "error": format!("Could not resolve path for \"{}\".", params.id),
                }));
            }
        };

        match &resolved {
            ResolvedPath::VersionNotFound {
                requested,
                available,
            } => {
                return text_result(serde_json::json!({
                    "error": format!("Version \"{}\" not found for \"{}\".", requested, params.id),
                    "available": available,
                }));
            }
            ResolvedPath::NeedsLanguage { available } => {
                return text_result(serde_json::json!({
                    "error": format!("Multiple languages available for \"{}\". Specify the lang parameter.", params.id),
                    "available": available,
                }));
            }
            ResolvedPath::Ok { .. } => {}
        }

        let (file_path, base_path, files) = match resolve_entry_file(&resolved, entry_type) {
            Some(r) => r,
            None => {
                return text_result(serde_json::json!({
                    "error": format!("\"{}\": unresolved", params.id),
                }));
            }
        };

        let source = match &resolved {
            ResolvedPath::Ok { source, .. } => source.clone(),
            _ => unreachable!(),
        };

        let mut content = if let Some(ref file) = params.file {
            if !files.contains(&file.to_string()) {
                let entry_file_name = if entry_type == "skill" {
                    "SKILL.md"
                } else {
                    "DOC.md"
                };
                let available: Vec<_> = files
                    .iter()
                    .filter(|f| f.as_str() != entry_file_name)
                    .collect();
                return text_result(serde_json::json!({
                    "error": format!("File \"{}\" not found in {}.", file, params.id),
                    "available": if available.is_empty() { vec!["(none)".to_string()] } else { available.iter().map(|s| s.to_string()).collect() },
                }));
            }
            let path = format!("{}/{}", base_path, file);
            match fetch_doc(&source, &path).await {
                Ok(c) => c,
                Err(e) => {
                    return text_result(serde_json::json!({
                        "error": format!("Failed to fetch \"{}\": {}", params.id, e),
                    }));
                }
            }
        } else if params.full.unwrap_or(false) && !files.is_empty() {
            match fetch_doc_full(&source, &base_path, &files).await {
                Ok(all_files) => all_files
                    .iter()
                    .map(|(name, content)| format!("# FILE: {}\n\n{}", name, content))
                    .collect::<Vec<_>>()
                    .join("\n\n---\n\n"),
                Err(e) => {
                    return text_result(serde_json::json!({
                        "error": format!("Failed to fetch \"{}\": {}", params.id, e),
                    }));
                }
            }
        } else {
            match fetch_doc(&source, &file_path).await {
                Ok(c) => c,
                Err(e) => {
                    return text_result(serde_json::json!({
                        "error": format!("Failed to fetch \"{}\": {}", params.id, e),
                    }));
                }
            }
        };

        // Append pin notice if this doc is pinned
        if !pin_notice.is_empty() {
            content.push_str(&pin_notice);
        }

        // Append merged annotations (team + personal)
        if let Some(merged_ann) = team::team_annotations::get_merged_annotation(entry.id()) {
            content.push_str(&format!("\n\n---\n{}\n", merged_ann));
        } else if let Some(annotation) = read_annotation(entry.id()) {
            content.push_str(&format!(
                "\n\n---\n[Agent note — {}]\n{}\n",
                annotation.updated_at, annotation.note
            ));
        }

        // Record analytics
        team::analytics::record_fetch(entry.id(), Some("mcp-server"));

        content
    }

    #[tool(
        name = "chub_list",
        description = "List all available docs and skills in Context Hub"
    )]
    async fn handle_list(&self, Parameters(params): Parameters<ListParams>) -> String {
        let limit = params.limit.unwrap_or(50);
        let filters = SearchFilters {
            tags: params.tags,
            lang: params.lang,
        };

        let entries = list_entries(&filters, &self.merged);
        let sliced: Vec<_> = entries.iter().take(limit).collect();

        text_result(serde_json::json!({
            "entries": sliced.iter().map(|e| simplify_entry(e)).collect::<Vec<_>>(),
            "total": entries.len(),
            "showing": sliced.len(),
        }))
    }

    #[tool(
        name = "chub_annotate",
        description = "Read, write, clear, or list agent annotations. Modes: (1) list=true to list all, (2) id+note to write, (3) id+clear=true to delete, (4) id alone to read."
    )]
    async fn handle_annotate(&self, Parameters(params): Parameters<AnnotateParams>) -> String {
        if params.list.unwrap_or(false) {
            let annotations = list_annotations();
            return text_result(serde_json::json!({
                "annotations": annotations,
                "total": annotations.len(),
            }));
        }

        let id = match params.id {
            Some(id) => id,
            None => {
                return text_result(serde_json::json!({
                    "error": "Missing required parameter: id. Provide an entry ID or use list mode.",
                }));
            }
        };

        // Validate entry ID
        if id.len() > 200 {
            return text_result(serde_json::json!({
                "error": "Entry ID too long (max 200 characters).",
            }));
        }
        if !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '/')
        {
            return text_result(serde_json::json!({
                "error": "Entry ID contains invalid characters. Use only alphanumeric, hyphens, underscores, dots, and slashes.",
            }));
        }

        if params.clear.unwrap_or(false) {
            let removed = clear_annotation(&id);
            return text_result(serde_json::json!({
                "status": if removed { "cleared" } else { "not_found" },
                "id": id,
            }));
        }

        if let Some(note) = params.note {
            let saved = write_annotation(&id, &note);
            return text_result(serde_json::json!({
                "status": "saved",
                "annotation": saved,
            }));
        }

        // Read mode
        if let Some(annotation) = read_annotation(&id) {
            text_result(serde_json::json!({ "annotation": annotation }))
        } else {
            text_result(serde_json::json!({ "status": "no_annotation", "id": id }))
        }
    }

    #[tool(
        name = "chub_context",
        description = "Get optimal context for a task: returns relevant pinned docs, team annotations, project context, and profile rules in one call"
    )]
    async fn handle_context(&self, Parameters(params): Parameters<ContextParams>) -> String {
        let mut result = serde_json::json!({});

        // Pinned docs
        let pins = team::pins::list_pins();
        if !pins.is_empty() {
            result["pins"] = serde_json::json!(pins);
        }

        // Active profile rules
        if let Some(ref profile_name) = params.profile {
            if let Ok(resolved) = team::profiles::resolve_profile(profile_name) {
                result["profile"] = serde_json::json!({
                    "name": resolved.name,
                    "rules": resolved.rules,
                    "context": resolved.context,
                });
            }
        } else if let Some(profile_name) = team::profiles::get_active_profile() {
            if let Ok(resolved) = team::profiles::resolve_profile(&profile_name) {
                result["profile"] = serde_json::json!({
                    "name": resolved.name,
                    "rules": resolved.rules,
                    "context": resolved.context,
                });
            }
        }

        // Project context docs
        let context_docs = team::context::list_context_docs();
        if !context_docs.is_empty() {
            result["project_context"] = serde_json::json!(context_docs);
        }

        // Team annotations for pinned docs
        let mut annotations = Vec::new();
        for pin in &pins {
            if let Some(merged_ann) = team::team_annotations::get_merged_annotation(&pin.id) {
                annotations.push(serde_json::json!({
                    "id": pin.id,
                    "annotation": merged_ann,
                }));
            }
        }
        if !annotations.is_empty() {
            result["annotations"] = serde_json::json!(annotations);
        }

        // Task relevance scoring (if task provided)
        if let Some(ref task) = params.task {
            result["task"] = serde_json::json!(task);
        }

        text_result(result)
    }

    #[tool(
        name = "chub_pins",
        description = "List, add, or remove pinned docs. Pinned docs have locked versions for the team."
    )]
    async fn handle_pins(&self, Parameters(params): Parameters<PinsParams>) -> String {
        if params.list.unwrap_or(false) || (params.id.is_none() && params.remove.is_none()) {
            let pins = team::pins::list_pins();
            return text_result(serde_json::json!({
                "pins": pins,
                "total": pins.len(),
            }));
        }

        let id = match params.id {
            Some(id) => id,
            None => {
                return text_result(serde_json::json!({
                    "error": "Missing required parameter: id",
                }));
            }
        };

        if params.remove.unwrap_or(false) {
            match team::pins::remove_pin(&id) {
                Ok(true) => text_result(serde_json::json!({"status": "unpinned", "id": id})),
                Ok(false) => text_result(serde_json::json!({"status": "not_found", "id": id})),
                Err(e) => text_result(serde_json::json!({"error": e.to_string()})),
            }
        } else {
            match team::pins::add_pin(&id, params.lang, params.version, params.reason, None) {
                Ok(()) => text_result(serde_json::json!({"status": "pinned", "id": id})),
                Err(e) => text_result(serde_json::json!({"error": e.to_string()})),
            }
        }
    }

    #[tool(
        name = "chub_feedback",
        description = "Send quality feedback (thumbs up/down) for a doc or skill to help authors improve content"
    )]
    async fn handle_feedback(&self, Parameters(params): Parameters<FeedbackParams>) -> String {
        if !is_feedback_enabled() {
            return text_result(serde_json::json!({
                "status": "skipped",
                "reason": "feedback_disabled",
            }));
        }

        // Auto-detect entry type
        let mut entry_type = params.entry_type.clone();
        if entry_type.is_none() {
            let result = get_entry(&params.id, &self.merged);
            if let Some(ref entry) = result.entry {
                entry_type = Some(entry.entry_type.to_string());
            }
        }
        let entry_type = entry_type.unwrap_or_else(|| "doc".to_string());

        // Validate labels
        let labels = params.labels.map(|ls| {
            ls.into_iter()
                .filter(|l| VALID_LABELS.contains(&l.as_str()))
                .collect::<Vec<_>>()
        });

        let result = send_feedback(
            &params.id,
            &entry_type,
            &params.rating,
            FeedbackOpts {
                comment: params.comment,
                doc_lang: params.lang,
                doc_version: params.version,
                target_file: params.file,
                labels,
                agent: Some("mcp-server".to_string()),
                model: None,
                cli_version: Some(env!("CARGO_PKG_VERSION").to_string()),
                source: None,
            },
        )
        .await;

        text_result(result)
    }
}
