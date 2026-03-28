# MCP Server

Chub includes a built-in MCP (Model Context Protocol) stdio server. Connect it to any MCP-compatible agent (Claude, Cursor, Windsurf, Copilot, etc.) and the agent can search, fetch, annotate, and get optimal context without any manual commands.

## Starting the server

```sh
chub mcp
```

To scope the session to a context profile, activate it first:

```sh
chub profile use backend
chub mcp
```

## Agent configuration

::: code-group

```json [Claude Code (.mcp.json)]
{
  "mcpServers": {
    "chub": { "command": "chub", "args": ["mcp"] }
  }
}
```

```json [Cursor (.cursor/mcp.json)]
{
  "mcpServers": {
    "chub": { "command": "chub", "args": ["mcp"] }
  }
}
```

```json [Windsurf (settings UI)]
{
  "mcpServers": {
    "chub": { "command": "chub", "args": ["mcp"] }
  }
}
```

```json [Copilot (.vscode/mcp.json)]
{
  "servers": {
    "chub": { "command": "chub", "args": ["mcp"] }
  }
}
```

```json [Gemini CLI (.gemini/settings.json)]
{
  "mcpServers": {
    "chub": { "command": "chub", "args": ["mcp"] }
  }
}
```

```json [Kiro (.kiro/settings/mcp.json)]
{
  "mcpServers": {
    "chub": { "command": "chub", "args": ["mcp"] }
  }
}
```

```toml [Codex (.codex/config.toml)]
[[mcp_servers]]
name = "chub"
command = "chub"
args = ["mcp"]
```

```json [Cline / Roo Code (extension UI)]
{
  "mcpServers": {
    "chub": { "command": "chub", "args": ["mcp"] }
  }
}
```

```yaml [Continue.dev (.continue/config.yaml)]
mcpServers:
  - name: chub
    command: chub
    args: ["mcp"]
```

```sh [Aider]
aider --mcp-server-command "chub mcp"
```

:::

## MCP Tools

### chub_search

Search for docs and skills by query, tags, or language.

**Parameters:**

| Parameter | Type | Description |
|---|---|---|
| `query` | string? | Search query. Omit to list all entries. |
| `tags` | string? | Comma-separated tag filter (e.g. `"openai,chat"`) |
| `lang` | string? | Filter by language (e.g. `"python"`, `"js"`) |
| `limit` | number? | Max results (default 20) |

**Examples:**

```json
{ "query": "stripe payments" }
{ "query": "openai embeddings", "lang": "python" }
{ "tags": "auth,oauth", "limit": 5 }
```

**Response:**

```json
{
  "results": [
    { "id": "openai/chat", "name": "OpenAI Chat", "type": "doc", "description": "...", "languages": ["python", "js"] }
  ],
  "total": 42,
  "showing": 20
}
```

---

### chub_get

Fetch the content of a doc or skill by ID. Returns the full markdown, with any team annotations and pin notices appended automatically.

**Parameters:**

| Parameter | Type | Description |
|---|---|---|
| `id` | string | Entry ID (e.g. `"openai/chat"`, `"stripe/api"`) |
| `lang` | string? | Language variant (e.g. `"python"`, `"js"`) |
| `version` | string? | Specific version (e.g. `"4.0"`) |
| `full` | boolean? | Fetch all files in the entry, not just the entry point |
| `file` | string? | Fetch a specific sub-file (e.g. `"references/streaming.md"`) |
| `match_env` | boolean? | Auto-detect version from `package.json`, `Cargo.toml`, etc. |

**Examples:**

```json
{ "id": "openai/chat" }
{ "id": "openai/chat", "lang": "python", "version": "4.0" }
{ "id": "openai/chat", "match_env": true }
{ "id": "stripe/api", "full": true }
{ "id": "nextjs/app-router", "file": "references/caching.md" }
```

**Automatic behaviors:**
- If the entry is pinned, the pinned `lang`/`version` is applied as a default.
- Merged annotations (org → team → personal) are appended under a `⚠ USER-CONTRIBUTED ANNOTATIONS` separator.
- A `[Team pin]` notice is appended when the doc is pinned.

---

### chub_list

List all available docs and skills in the registry.

**Parameters:**

| Parameter | Type | Description |
|---|---|---|
| `tags` | string? | Comma-separated tag filter |
| `lang` | string? | Filter by language |
| `limit` | number? | Max entries (default 50) |

**Example:**

```json
{ "tags": "stripe", "lang": "python" }
```

---

### chub_annotate

Read, write, clear, or list structured annotations. Agents should use this proactively to build persistent team knowledge.

**Parameters:**

| Parameter | Type | Description |
|---|---|---|
| `id` | string? | Entry ID to annotate. Required unless `list=true`. |
| `note` | string? | Annotation text. Omit to read existing. |
| `kind` | string? | `"note"` (default), `"issue"`, `"fix"`, or `"practice"` |
| `severity` | string? | `"high"`, `"medium"`, or `"low"` — only applies when `kind="issue"` |
| `scope` | string? | `"auto"` (default), `"personal"`, `"team"`, or `"org"` |
| `clear` | boolean? | Remove the annotation for this entry |
| `list` | boolean? | List all annotations. `id` is not needed. |

**Modes:**

| Call | Effect |
|---|---|
| `{ "id": "openai/chat" }` | Read merged annotations (org + team + personal) |
| `{ "id": "openai/chat", "note": "...", "kind": "issue" }` | Write annotation (auto-routed) |
| `{ "id": "openai/chat", "note": "...", "scope": "org" }` | Write to org server specifically |
| `{ "id": "openai/chat", "clear": true }` | Remove annotation (auto-routed) |
| `{ "list": true }` | List all annotations (auto-routed) |
| `{ "list": true, "scope": "org" }` | List all org annotations |

**Write examples:**

```json
// Confirmed bug with severity
{ "id": "openai/chat", "kind": "issue", "severity": "high",
  "note": "tool_choice='none' silently ignores tools and returns null. Confirmed on gpt-4o-mini 2024-07-18." }

// Workaround for the bug above
{ "id": "openai/chat", "kind": "fix",
  "note": "Pass tool_choice='auto' instead. To suppress tools, remove them from the tools array entirely." }

// Validated team pattern
{ "id": "openai/chat", "kind": "practice",
  "note": "Always set max_tokens explicitly. Omitting it causes unbounded output on streaming requests." }

// Write directly to org tier
{ "id": "openai/chat", "kind": "practice",
  "note": "Always set max_tokens explicitly.", "scope": "org" }
```

**Auto-routing:** By default (`scope="auto"`), writes go to the **team tier** when a `.chub/` project directory is present, and to the **personal tier** otherwise. Use `scope` to target a tier explicitly:

| `scope` | Target | Semantics |
|---|---|---|
| `"auto"` | Team (if `.chub/` exists), else personal | — |
| `"personal"` | `~/.chub/annotations/` | Overwrite — one note per entry |
| `"team"` | `.chub/annotations/` | Append — full history with author + date |
| `"org"` | Remote annotation server | Append — requires `annotation_server` config |

Reads (no `note`) always return all three tiers merged. If the team write fails (e.g., no `.chub/` when `scope="team"`), an explicit error is returned — there is no silent fallback.

**Recommended workflow:**

1. Always read first to avoid duplicates: `{ "id": "openai/chat" }`
2. Write one fact per annotation with exact params/values
3. Pair `kind="fix"` with its `kind="issue"` annotation

---

### chub_context

Get a combined context bundle for a task in one call: pinned docs, merged annotations (org + team + personal), active profile rules, and project context docs. Use at session start or when switching to a new area of the codebase.

**Parameters:**

| Parameter | Type | Description |
|---|---|---|
| `task` | string? | Task description (passed through to response for framing) |
| `files_open` | string[]? | Currently open files (for auto-profile detection) |
| `profile` | string? | Profile name to scope context to |
| `max_tokens` | number? | Soft token budget |

**Example:**

```json
{ "task": "implement Stripe payment flow", "files_open": ["src/payments/stripe.ts"] }
```

**Response:**

```json
{
  "pins": [{ "id": "stripe/api", "lang": "javascript", "reason": "Use v4 API" }],
  "profile": { "name": "backend", "rules": ["Use Zod for validation"], "context": ["api-conventions.md"] },
  "project_context": [{ "file": "architecture.md", "name": "Architecture", "description": "..." }],
  "annotations": [{ "id": "stripe/api", "annotation": "[Team issue (high) — alice (2026-03-20)] ..." }],
  "task": "implement Stripe payment flow"
}
```

---

### chub_pins

List, add, or remove pinned docs.

**Parameters:**

| Parameter | Type | Description |
|---|---|---|
| `id` | string? | Entry ID to pin or unpin |
| `lang` | string? | Language variant |
| `version` | string? | Version to lock |
| `reason` | string? | Reason for pinning |
| `remove` | boolean? | Remove the pin |
| `list` | boolean? | List all pins (default when no `id` provided) |

**Examples:**

```json
// List all pins
{}

// Add a pin
{ "id": "openai/chat", "lang": "python", "version": "4.0", "reason": "Use v4 streaming" }

// Remove a pin
{ "id": "openai/chat", "remove": true }
```

---

### chub_feedback

Send quality feedback (thumbs up/down) for a doc or skill to help authors improve content.

**Parameters:**

| Parameter | Type | Description |
|---|---|---|
| `id` | string | Entry ID |
| `rating` | string | `"up"` or `"down"` |
| `comment` | string? | Optional explanation |
| `lang` | string? | Language variant rated |
| `version` | string? | Version rated |
| `labels` | string[]? | Structured labels (e.g. `["outdated", "missing-example"]`) |

**Example:**

```json
{ "id": "openai/chat", "rating": "down",
  "comment": "Missing streaming example for Python", "labels": ["missing-example"] }
```

---

### chub_track

Query AI usage tracking data — session status, cost reports, session history, and session details.

**Parameters:**

| Parameter | Type | Description |
|---|---|---|
| `action` | string | `"status"`, `"report"`, `"log"`, or `"show"` |
| `days` | number? | Time range in days (default 30) |
| `session_id` | string? | Session ID for `action="show"` |

**Examples:**

```json
{ "action": "status" }
{ "action": "report", "days": 7 }
{ "action": "log", "days": 14 }
{ "action": "show", "session_id": "2026-03-22T10-05-abc123" }
```

**Response (status):**

```json
{
  "active_session": {
    "session_id": "2026-03-22T10-05-abc123",
    "agent": "claude-code",
    "model": "claude-opus-4-6",
    "turns": 14,
    "tokens": { "input": 45000, "output": 12000 }
  }
}
```

---

## MCP Resources

| URI | Description |
|---|---|
| `chub://registry` | Full merged registry (all entries) |

---

## Team-aware behavior

When running as an MCP server, Chub automatically:

- Applies pinned versions and languages when fetching docs
- Serves project context docs (via `project/<name>`)
- Appends merged annotations (org + team + personal) to doc content
- Appends pin notices to pinned docs
- Scopes results to the active profile (if set via `--profile`)
- Routes annotation writes to the team tier when `.chub/` exists (or org tier when `scope="org"`)
