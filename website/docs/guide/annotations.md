# Annotations & Self-Learning

Annotations are structured notes attached to docs — confirmed bugs, workarounds found, and team conventions validated. They live alongside the doc registry in git, and appear automatically in every agent's view of that doc from then on.

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

## Two storage tiers

Annotations have two tiers with different semantics:

| Tier | Location | Semantics | Sharing |
|------|----------|-----------|---------|
| Personal | `~/.chub/annotations/<id>.json` | **Overwrite** — one note per entry, replaces previous | Local only |
| Team | `.chub/annotations/<id>.yaml` | **Append** — adds to history, preserves author + date | Git-tracked |

The team tier requires a `.chub/` project directory (created by `chub init`). When both tiers have notes for the same entry, both are shown — team notes first.

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

# Personal annotation (local only, replaces previous note for this entry)
chub annotate openai/chat "My local WIP note"
```

### MCP (via agent)

Agents write annotations using the `chub_annotate` MCP tool. In a project with `.chub/`, the tool automatically routes to the team tier:

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

When any agent fetches `openai/chat`, annotations are appended to the doc content automatically:

```
[official doc content]

---
⚠ USER-CONTRIBUTED ANNOTATIONS (not part of official documentation):
[Team issue (high) — alice (2026-03-20)] tool_choice='none' silently ignores tools and returns null
[Team fix — alice (2026-03-20)] Pass tool_choice='auto' or remove tools from the array entirely
[Team practice — bob (2026-03-18)] Always set max_tokens to avoid unbounded cost
[Team — carol (2026-03-15)] openai>=1.0 SDK auto-retries 429. Do not wrap in a retry loop.
```

The framing makes it clear this is team-contributed knowledge, not official docs.

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

```sh
# Read annotations for a specific entry
chub annotate openai/chat --team        # team annotations
chub annotate openai/chat               # personal annotation

# List all annotations
chub annotate --list --team             # all team annotations, grouped by kind
chub annotate --list                    # all personal annotations

# Remove annotations
chub annotate openai/chat --clear --team
chub annotate openai/chat --clear
```

Via MCP:
```json
{ "id": "openai/chat" }          // read merged (team + personal)
{ "list": true }                 // list all (auto-routes to team tier in projects)
{ "id": "openai/chat", "clear": true }  // remove
```

## Pin notices

When a pinned doc is served, a team notice is appended automatically:

```
---
[Team pin] Locked to v4.0 (python). Reason: We use v4 streaming API.
```

This ensures agents don't accidentally reference a different version than the team has validated.
