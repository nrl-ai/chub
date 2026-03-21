use std::sync::Arc;

use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::transport::stdio;
use rmcp::{tool_handler, ServerHandler, ServiceExt};

use super::tools::ChubMcpServer;

#[tool_handler]
impl ServerHandler for ChubMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("chub", env!("CARGO_PKG_VERSION")))
        .with_instructions(
            "Context Hub MCP Server - search and retrieve LLM-optimized docs and skills",
        )
    }

    async fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListResourcesResult, rmcp::model::ErrorData> {
        use rmcp::model::{Annotated, RawResource};

        let resource = RawResource::new("chub://registry", "Context Hub Registry")
            .with_mime_type("application/json")
            .with_description("Browse the full Context Hub registry of docs and skills");

        Ok(rmcp::model::ListResourcesResult::with_all_items(vec![
            Annotated::new(resource, None),
        ]))
    }

    async fn read_resource(
        &self,
        request: rmcp::model::ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ReadResourceResult, rmcp::model::ErrorData> {
        use chub_core::registry::list_entries;
        use chub_core::registry::SearchFilters;

        if request.uri.as_str() != "chub://registry" {
            return Err(rmcp::model::ErrorData::invalid_params(
                "Resource not found",
                None,
            ));
        }

        let entries = list_entries(&SearchFilters::default(), &self.merged);
        let simplified: Vec<serde_json::Value> = entries
            .iter()
            .map(|entry| {
                let mut val = serde_json::json!({
                    "id": entry.id(),
                    "name": entry.name(),
                    "type": entry.entry_type,
                    "description": entry.description(),
                    "tags": entry.tags(),
                });
                if let Some(languages) = entry.languages() {
                    val["languages"] = serde_json::json!(languages
                        .iter()
                        .map(|l| serde_json::json!({
                            "language": l.language,
                            "versions": l.versions.iter().map(|v| &v.version).collect::<Vec<_>>(),
                            "recommended": l.recommended_version,
                        }))
                        .collect::<Vec<_>>());
                }
                val
            })
            .collect();

        let text = serde_json::to_string_pretty(&serde_json::json!({
            "entries": simplified,
            "total": simplified.len(),
        }))
        .unwrap_or_default();

        Ok(rmcp::model::ReadResourceResult::new(vec![
            rmcp::model::ResourceContents::text(request.uri, text),
        ]))
    }
}

/// Run the MCP stdio server.
pub async fn run_mcp_server() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[chub-mcp] Starting server...");

    // Best-effort registry load
    if let Err(e) = chub_core::fetch::ensure_registry().await {
        eprintln!("[chub-mcp] Warning: Registry not loaded: {}", e);
    }

    let merged = Arc::new(chub_core::registry::load_merged());
    let server = ChubMcpServer::new(merged);

    eprintln!("[chub-mcp] Server started (v{})", env!("CARGO_PKG_VERSION"));

    let transport = stdio();
    let running = server.serve(transport).await?;

    // Wait for either: the MCP server to finish, or a shutdown signal.
    // This prevents orphan processes when the parent MCP host terminates.
    // When the parent closes stdin, rmcp's transport should detect EOF and
    // cause `waiting()` to return. The ctrl_c handler catches explicit signals.
    tokio::select! {
        result = running.waiting() => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            eprintln!("[chub-mcp] received interrupt, shutting down.");
        }
    }

    Ok(())
}
