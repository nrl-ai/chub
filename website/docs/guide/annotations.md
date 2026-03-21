# Annotations & Self-Learning

Annotations are structured notes attached to docs — confirmed bugs, workarounds found, and team conventions validated. They appear automatically in every agent's view of that doc from then on.

This is how Chub builds a **self-learning knowledge base**: when one agent resolves a non-obvious problem with a library, it records it once. Every future agent sees it. The team never rediscovers the same issue twice.

## Annotation kinds

Four kinds, each with a specific meaning:

| Kind | Use when | Example |
|------|----------|---------|
| `issue` | Confirmed bug, broken param, or misleading example | "`tool_choice='none'` silently returns null" |
| `fix` | Workaround that resolves a confirmed issue | "Use `tool_choice='auto'` instead" |
| `practice` | Team convention or validated pattern | "Always set `max_tokens` to avoid unbounded cost" |
| `note` | General observation that doesn't fit above | "SDK auto-retries on 429, no manual retry needed" |

Pair `fix` with `issue` — they are most useful together.

## Three storage tiers

Annotations have three tiers with different semantics:

| Tier | Location | Semantics | Sharing |
|------|----------|-----------|---------|
| 1 — Personal | `~/.chub/annotations/<id>.json` | **Overwrite** — one note per entry, replaces previous | Local only |
| 2 — Team | `.chub/annotations/<id>.yaml` | **Append** — adds to history, preserves author + date | Git-tracked |
| 3 — Org | Remote HTTP API | **Append** — org-wide baseline, locally cached | All teams |

**Resolution order:** Org (baseline) → Team (project overlay) → Personal (most specific). When fetched via `chub get` or MCP, all three tiers are merged and appended to the doc automatically.

The team tier requires a `.chub/` project directory (created by `chub init`). The org tier requires an `annotation_server` configured in `.chub/config.yaml` or the `CHUB_ANNOTATION_SERVER` env var.

### Enabling Tier 3 (org)

```yaml
# .chub/config.yaml (URL is not sensitive — safe to commit)
annotation_server:
  url: https://annotations.internal.company.com
  auto_push: false     # set true to mirror every team write to org tier
  cache_ttl_secs: 3600 # local cache TTL (default: 1 hour)
```

```yaml
# ~/.chub/config.yaml or CHUB_ANNOTATION_TOKEN env var (token is sensitive — never commit)
annotation_token: "your-secret-token"
```

## Writing annotations

### CLI

```sh
# Team annotation — append to shared history
chub annotate openai/chat "Use v4 streaming, not completions" --team

# Record a confirmed bug with severity
chub annotate openai/chat "tool_choice='none' silently ignores tools and returns null" \
  --kind issue --severity high --team

# Record the fix for the bug
chub annotate openai/chat "Use tool_choice='auto' or remove tools from the array entirely" \
  --kind fix --team

# Record a validated team pattern
chub annotate openai/chat "Always set max_tokens to avoid unbounded streaming cost" \
  --kind practice --team

# Org annotation — write to org server (Tier 3)
chub annotate openai/chat "Always set max_tokens explicitly" --kind practice --org

# Personal annotation (local only, replaces previous note for this entry)
chub annotate openai/chat "My local WIP note"
```

### MCP (via agent)

Agents write annotations using the `chub_annotate` MCP tool. In a project with `.chub/`, writes automatically route to the team tier. Use `scope` to target a specific tier:

```json
{ "id": "openai/chat", "kind": "issue", "severity": "high",
  "note": "tool_choice='none' silently ignores tools. Confirmed on gpt-4o-mini 2024-07-18." }
```

```json
{ "id": "openai/chat", "kind": "fix",
  "note": "Pass tool_choice='auto' instead. To suppress tools, remove them from the tools array entirely." }
```

```json
{ "id": "openai/chat", "kind": "practice",
  "note": "Always set max_tokens explicitly. Omitting it causes unbounded output on streaming requests." }
```

```json
{ "id": "openai/chat", "kind": "practice",
  "note": "Always set max_tokens explicitly.", "scope": "org" }
```

## Team annotation file format

```yaml
# .chub/annotations/openai--chat.yaml
id: openai/chat
issues:
  - author: alice
    date: 2026-03-20
    severity: high
    note: "tool_choice='none' silently ignores tools and returns null. Confirmed on gpt-4o-mini 2024-07-18."
fixes:
  - author: alice
    date: 2026-03-20
    note: "Pass tool_choice='auto' instead. To suppress tools, remove them from the tools array entirely."
practices:
  - author: bob
    date: 2026-03-18
    note: "Always set max_tokens to avoid unbounded cost on streaming responses."
notes:
  - author: carol
    date: 2026-03-15
    note: "openai>=1.0 SDK auto-retries 429 and 500 with backoff. Do not wrap in a retry loop."
```

The file is committed to git. Every section is an append-only history. Use `chub annotate <id> --clear --team` to remove the entire file.

## What agents see

When any agent fetches `openai/chat`, all three tiers of annotations are merged and appended to the doc content automatically:

```
[official doc content]

---
⚠ USER-CONTRIBUTED ANNOTATIONS (not part of official documentation):
[Org practice — platform-team (2026-03-10)] Always set max_tokens to avoid unbounded cost
[Team issue (high) — alice (2026-03-20)] tool_choice='none' silently ignores tools and returns null
[Team fix — alice (2026-03-20)] Pass tool_choice='auto' or remove tools from the array entirely
[Team practice — bob (2026-03-18)] Always set max_tokens to avoid unbounded cost
[Team — carol (2026-03-15)] openai>=1.0 SDK auto-retries 429. Do not wrap in a retry loop.
[Personal note — 2026-03-21] My local WIP note
```

The framing makes it clear this is team-contributed knowledge, not official docs. Tiers are labeled (`Org`, `Team`, `Personal`) so agents know the provenance.

## Self-learning agents

The power of annotations comes from agents writing them back automatically. After resolving something non-obvious with a library, an agent writes it once — and every future agent sees it without any human action.

### The annotation workflow

**Step 1 — Check first to avoid duplicates:**
```json
{ "id": "openai/chat" }
```

**Step 2 — Write what you confirmed:**
```json
{ "id": "openai/chat", "kind": "issue", "severity": "high",
  "note": "tool_choice='none' silently ignores tools and returns null." }
```

**Step 3 — Write the fix if you found one:**
```json
{ "id": "openai/chat", "kind": "fix",
  "note": "Use tool_choice='auto' or remove tools from the array entirely." }
```

**Rules for good annotations:**
- Annotate after confirming, not speculatively
- One fact per annotation — don't bundle multiple issues
- Include exact params, values, or versions — not vague descriptions
- Don't annotate what's already in the official docs

### Annotation policy in CLAUDE.md / AGENTS.md

Add `include_annotation_policy: true` to `agent_rules` in `.chub/config.yaml` to automatically inject annotation instructions into every generated agent config file:

```yaml
# .chub/config.yaml
agent_rules:
  include_annotation_policy: true
  targets:
    - claude.md
    - agents.md
```

This adds a standing instruction block to CLAUDE.md so agents know to annotate without being told in every session. See [Agent Config Sync](/guide/agent-config).

### Chub workflow skill

Fetch the `chub/skills/chub-workflow` skill to give any agent the complete annotation discipline: when to annotate, which kind to use, how to write actionable notes, and the end-of-task checklist.

```sh
chub get chub/skills/chub-workflow
```

Or via MCP:
```json
{ "id": "chub/skills/chub-workflow" }
```

## Reading and listing annotations

### CLI

```sh
# Read all annotations for an entry (org + team + personal merged)
chub annotate openai/chat

# Read a specific tier only
chub annotate openai/chat --org           # org annotations
chub annotate openai/chat --team          # team annotations
chub annotate openai/chat --personal      # personal annotation

# List all annotations
chub annotate --list --org                # list all org annotations
chub annotate --list --team               # all team annotations, grouped by kind
chub annotate --list                      # all personal annotations

# Remove annotations
chub annotate openai/chat --clear --org
chub annotate openai/chat --clear --team
chub annotate openai/chat --clear
```

### MCP

```json
{ "id": "openai/chat" }                              // read merged (org + team + personal)
{ "id": "openai/chat", "scope": "org" }              // read org annotations only
{ "id": "openai/chat", "scope": "team" }             // read team annotations only
{ "list": true }                                     // list all (auto-routes: team in projects, personal otherwise)
{ "list": true, "scope": "org" }                     // list all org annotations
{ "id": "openai/chat", "clear": true }               // remove (auto-routes)
{ "id": "openai/chat", "clear": true, "scope": "org" }  // remove org annotation
```

## Server API contract

Teams who want to implement a compatible org annotation server must provide these endpoints:

```text
GET    /api/v1/annotations              → 200 [{TeamAnnotation}, ...]
GET    /api/v1/annotations/:id          → 200 TeamAnnotation | 404
POST   /api/v1/annotations/:id          → 200 TeamAnnotation
       Body: {"note":"..","kind":"..","severity":"..","author":".."}
DELETE /api/v1/annotations/:id          → 200 | 404

Auth: Authorization: Bearer <token>   (optional if server doesn't require it)
Content-Type: application/json
Entry ID encoding: replace "/" with "--" in URL path segment
                   e.g. "openai/chat" → "/api/v1/annotations/openai--chat"
```

The `TeamAnnotation` response shape is the same as the team annotation YAML file format, serialized as JSON. The client caches responses locally with a configurable TTL and falls back to the stale cache if the server is unreachable.

## Pin notices

When a pinned doc is served, a team notice is appended automatically:

```
---
[Team pin] Locked to v4.0 (python). Reason: We use v4 streaming API.
```

This ensures agents don't accidentally reference a different version than the team has validated.
