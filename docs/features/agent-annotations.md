# Feature: Agent Annotations

## Overview

Agents can annotate docs and skills with gotchas, tips, and experiences learned while building with them. Annotations persist across sessions and appear automatically when the agent fetches the same doc again.

Chub supports a **3-tier annotation system** with escalating scope:

| Tier | Scope | Storage | Sharing |
|------|-------|---------|---------|
| **Personal** (Tier 1) | Per-user, per-machine | `~/.chub/annotations/<id>.json` | None |
| **Team** (Tier 2) | Per-project, git-tracked | `.chub/annotations/<id>.yaml` | Via git |
| **Org** (Tier 3) | Org-wide, server-hosted | Remote HTTP API with local cache | Via annotation server |

When reading annotations, all tiers are merged automatically. On `chub get`, annotations from all tiers appear appended to the doc.

## Annotation Kinds

Each annotation has a `kind` that classifies what the agent learned:

| Kind | Purpose |
|------|---------|
| `note` | General observation (default) |
| `issue` | Undocumented bug, broken param, or misleading example |
| `fix` | Workaround that resolved an issue |
| `practice` | Team convention or validated pattern |

Issue annotations can optionally carry a `severity`: `high`, `medium`, or `low`.

## CLI Commands

### `chub annotate <id> <note>`

Write a personal annotation (default).

```bash
# Personal annotation (default)
chub annotate openai/chat "streaming requires explicit close when using function calling"

# Team annotation (git-tracked)
chub annotate --team openai/chat "Use batch endpoint for our pipeline"

# Org annotation (server-hosted)
chub annotate --org openai/chat "Company-wide: always use v2 API"

# With kind and severity
chub annotate --kind issue --severity high stripe/api "Webhook sig fails with raw body middleware"
chub annotate --kind fix stripe/api "Use express.raw() middleware before webhook handler"
chub annotate --kind practice openai/chat "Always set max_tokens to avoid runaway costs"

# Author attribution (team/org)
chub annotate --team --author "alice" openai/chat "Verified: streaming works with function calling"
```

### Read annotations

```bash
# View merged annotations for a doc (all tiers)
chub annotate openai/chat

# View personal only
chub annotate --personal openai/chat

# View team only
chub annotate --team openai/chat
```

### List all annotations

```bash
chub annotate --list               # personal annotations
chub annotate --list --team        # team annotations
chub annotate --list --org         # org annotations
```

### Clear an annotation

```bash
chub annotate --clear stripe/api             # clear personal
chub annotate --clear --team stripe/api      # clear team
chub annotate --clear --org stripe/api       # clear org (requires server)
```

### Annotations in `chub get`

When annotations exist, `chub get` appends them after the doc content:

```
# Stripe API
...doc content...

---
[Agent note — 2026-03-15T10:30:00Z]
Webhook verification requires raw body — do not parse JSON before verifying
```

With `--json`, annotations are included in the response object:

```json
{
  "id": "stripe/api",
  "type": "doc",
  "content": "...",
  "annotation": {
    "id": "stripe/api",
    "note": "Webhook verification requires raw body...",
    "kind": "issue",
    "severity": "high",
    "updatedAt": "2026-03-15T10:30:00.000Z"
  }
}
```

## MCP Tool

The `chub_annotate` MCP tool provides the same functionality for AI coding agents:

```json
{
  "name": "chub_annotate",
  "arguments": {
    "id": "stripe/api",
    "note": "Use idempotency keys for POST requests",
    "kind": "practice",
    "scope": "team"
  }
}
```

Parameters:
- `id` — entry ID (required for read/write; optional for list)
- `note` — annotation text (omit to read)
- `clear` — set `true` to remove the annotation
- `list` — set `true` to list all annotations
- `scope` — `"personal"` (default), `"team"`, `"org"`, or `"auto"`
- `kind` — `"note"`, `"issue"`, `"fix"`, or `"practice"`
- `severity` — `"high"`, `"medium"`, or `"low"` (only for `kind: "issue"`)
- `author` — author name for team/org annotations

## Storage Details

### Tier 1 — Personal (`~/.chub/annotations/`)

- One JSON file per entry (e.g., `stripe--api.json`)
- **Overwrite semantics**: writing replaces the previous note entirely
- Not shared or synced

### Tier 2 — Team (`.chub/annotations/`)

- YAML files inside the project's `.chub/` directory
- **Append semantics**: each write adds a new entry with author + timestamp
- Git-tracked — team members share annotations via version control
- Requires `chub init` to set up the project directory

### Tier 3 — Org (`annotation_server`)

- Annotations stored on a remote HTTP server
- Locally cached with configurable TTL (default: 1 hour)
- Requires configuration in `.chub/config.yaml`:

```yaml
annotation_server:
  url: https://annotations.example.com
  auto_push: false
  cache_ttl_secs: 3600
```

- Auth token: set `annotation_token` in `~/.chub/config.yaml` or `CHUB_ANNOTATION_TOKEN` env var
- Server URL: can also be set via `CHUB_ANNOTATION_SERVER` env var
