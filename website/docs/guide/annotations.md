# Team Annotations

Annotations are team knowledge attached to docs — bugs discovered, workarounds found, and conventions validated. They live in `.chub/annotations/`, are git-tracked, and appear automatically alongside the official doc content whenever any agent fetches that doc.

This is how Chub builds a **self-learning knowledge base**: every time an agent resolves something non-obvious, it records it once, and every future agent benefits automatically.

## Annotation kinds

| Kind | Purpose |
|------|---------|
| `note` | General observation (default) |
| `issue` | Confirmed bug, broken param, or misleading example |
| `fix` | Workaround that resolves a confirmed issue |
| `practice` | Team convention or validated pattern |

Pair `fix` with `issue` — they are most useful together.

## Writing annotations

```sh
# Team annotation (git-tracked, shared) — default kind=note
chub annotate openai/chat "Use v4 streaming, not completions" --team

# Record a confirmed bug
chub annotate openai/chat "tool_choice='none' silently ignores tools" \
  --kind issue --severity high --team

# Record the fix for the bug above
chub annotate openai/chat "Use tool_choice='auto' or remove tools from array" \
  --kind fix --team

# Record a best practice
chub annotate openai/chat "Always set max_tokens to avoid unbounded streaming cost" \
  --kind practice --team

# Personal annotation (local only, not shared)
chub annotate openai/chat "My local note" --personal
```

Agents can also write annotations via the MCP `chub_annotate` tool — see [Self-learning agents](#self-learning-agents).

## File format

```yaml
# .chub/annotations/openai--chat.yaml
id: openai/chat
notes:
  - author: alice
    date: 2026-03-15
    note: "Use v4 streaming, not completions"
issues:
  - author: bob
    date: 2026-03-20
    severity: high
    note: "tool_choice='none' silently ignores tools and returns null"
fixes:
  - author: bob
    date: 2026-03-20
    note: "Use tool_choice='auto' or remove tools from the array entirely"
practices:
  - author: alice
    date: 2026-03-18
    note: "Always set max_tokens to avoid unbounded cost on streaming responses"
```

## What agents see

When any agent fetches `openai/chat`, annotations are appended to the doc:

```
---
⚠ USER-CONTRIBUTED ANNOTATIONS (not part of official documentation):
[Team issue (high) — bob (2026-03-20)] tool_choice='none' silently ignores tools and returns null
[Team fix — bob (2026-03-20)] Use tool_choice='auto' or remove tools from the array entirely
[Team practice — alice (2026-03-18)] Always set max_tokens to avoid unbounded cost on streaming
[Team — alice (2026-03-15)] Use v4 streaming, not completions
```

## Self-learning agents

Agents write back what they discover using the MCP `chub_annotate` tool. When a `.chub/` project dir is present, agents automatically write to the team tier (git-tracked). When it is absent, annotations go to personal (`~/.chub/annotations/`).

```json
// Agent discovers a bug:
{ "id": "stripe/api", "kind": "issue", "severity": "high",
  "note": "idempotency_key silently ignored when confirm=true in PaymentIntent.create()" }

// Agent records the fix:
{ "id": "stripe/api", "kind": "fix",
  "note": "Use two-step create + confirm. idempotency_key works on each step independently." }
```

### Annotation policy in CLAUDE.md / AGENTS.md

Add `include_annotation_policy: true` to your `agent_rules` in `.chub/config.yaml` to automatically inject annotation instructions into generated `CLAUDE.md`, `AGENTS.md`, and `.cursorrules` files:

```yaml
agent_rules:
  include_annotation_policy: true
  targets:
    - claude.md
    - agents.md
```

This adds a standing instruction block to every generated agent config file, so agents know to annotate without being told in every session.

### Chub workflow skill

Fetch `chub/skills/chub-workflow` to give an agent the full annotation discipline: querying docs efficiently, when to annotate, which kind to use, how to write actionable notes, and the end-of-task checklist.

```sh
chub get chub/skills/chub-workflow
```

## Resolution order

When serving a doc, annotations are merged from all configured tiers:

1. Hosted / org annotations (Tier 3 — planned Phase 8)
2. Team / repo annotations (`.chub/annotations/` — Tier 2)
3. Personal annotations (`~/.chub/annotations/` — Tier 1, wins)

## Listing annotations

```sh
chub annotate --list --team      # show all team annotations, grouped by kind
chub annotate --list             # show all personal annotations
chub annotate openai/chat --team # show annotations for a specific entry
```

## Pin notices

When a pinned doc is served, a team notice is appended automatically:

```
---
[Team pin] Locked to v4.0 (python). Reason: We use v4 streaming API.
```
