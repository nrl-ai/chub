---
name: Chub Workflow
description: "Complete workflow for AI agents using Chub: how to query docs efficiently and how to write structured annotations back to build persistent team knowledge"
tags: chub, docs, annotations, workflow, knowledge-management
---

# Chub Workflow

This skill covers the complete Chub usage pattern for AI agents: fetching the right docs efficiently, and writing back what you learn so future agents benefit from it.

---

## Part 1 — Querying Docs

### Finding docs

Search by keyword, library name, or topic:

```
chub_search(query="stripe payments")
chub_search(query="openai embeddings", lang="python")
chub_search(query="auth", tags="oauth")
```

Omit the query to browse all available docs:

```
chub_list()
```

### Fetching a doc

Fetch by the ID returned from search:

```
chub_get(id="openai/chat")
chub_get(id="openai/chat", lang="python")
chub_get(id="stripe/api", version="2024-11-20")
```

Use `match_env=true` to auto-detect the version from your project's package.json / Cargo.toml / requirements.txt:

```
chub_get(id="openai/chat", match_env=true)
```

### What you get back

The response contains:
1. The official doc content
2. Any team annotations (issues, fixes, practices) your team has written
3. Pin notices if the team has locked a specific version

Read the annotations — they contain the non-obvious knowledge your team has accumulated. Do not repeat what is already annotated.

### Efficient querying pattern

Do not search then get separately when you already know the entry ID. If you know you need `openai/chat`, call `chub_get` directly. Search is for discovery, not for known IDs.

For a task requiring multiple docs, fetch them in parallel if your framework supports it.

---

## Part 2 — Writing Annotations

After resolving something non-obvious with a library, record it. This turns a one-time debugging session into permanent team knowledge.

### When to annotate

Ask yourself after finishing a task involving a library:

- Did I encounter behaviour not in the official docs?
- Did I need a workaround to make something work?
- Did I discover a pattern that will save future agents time?

If yes, annotate. Only annotate after confirming — never speculatively.

### Annotation kinds

| Kind | Use when | Example |
|------|----------|---------|
| `issue` | Confirmed bug, broken param, or misleading example | "`n=5` silently returns 1 result" |
| `fix` | Workaround that resolves a confirmed issue | "Use `n=1`, call 5 times" |
| `practice` | Validated pattern the team should consistently use | "Always pass `user` param for rate attribution" |
| `note` | General observation that doesn't fit above | "SDK auto-retries on 429, no manual retry needed" |

Pair `fix` with `issue` — they are most useful together.

### The annotation flow

**Step 1 — Check first to avoid duplicates**

```
chub_annotate(id="openai/chat")
```

If the issue is already there, skip.

**Step 2 — Write**

Issue (confirmed bug):
```
chub_annotate(
  id="openai/chat",
  kind="issue",
  severity="high",
  note="tool_choice='none' silently ignores tools and returns null. Confirmed on gpt-4o-mini 2024-07-18."
)
```

Fix (workaround for the issue above):
```
chub_annotate(
  id="openai/chat",
  kind="fix",
  note="Pass tool_choice='auto' instead. To suppress tools, remove them from the tools array entirely."
)
```

Practice (validated team pattern):
```
chub_annotate(
  id="openai/chat",
  kind="practice",
  note="Always set max_tokens explicitly. Omitting it causes unbounded output on streaming requests."
)
```

Note (general observation):
```
chub_annotate(
  id="openai/chat",
  kind="note",
  note="Python SDK openai>=1.0 auto-retries 429 and 500 with backoff. Do not wrap in a retry loop."
)
```

### Severity for issues

| Severity | Use when |
|----------|----------|
| `high` | Silent failure, data loss, security, or production-breaking |
| `medium` | Incorrect output or unexpected exception under common usage |
| `low` | Edge case, minor inconsistency, or confusing-but-harmless |

### Writing rules

1. **One fact per annotation** — do not bundle multiple issues
2. **Exact params/values** — not "streaming is broken" but "`stream=True` with `n>1` raises a 400"
3. **Active voice, no preamble** — start with the fact, not "Note that..." or "Be aware that..."
4. **Include version if relevant** — "confirmed on stripe-python 3.12.0"

---

## Part 3 — Full Example

You implement a Stripe payment flow and discover:
- `idempotency_key` is silently ignored when `confirm=True` is set
- Fix: use separate create + confirm calls
- Best practice: always use two-step flow for production

```
# 1. Check for existing annotations
chub_annotate(id="stripe/api")
# → no existing annotations for this issue

# 2. Log the issue
chub_annotate(
  id="stripe/api",
  kind="issue",
  severity="high",
  note="idempotency_key silently ignored when confirm=True in PaymentIntent.create(). Accepted without error, no idempotency."
)

# 3. Log the fix
chub_annotate(
  id="stripe/api",
  kind="fix",
  note="Two-step: PaymentIntent.create() without confirm=True, then PaymentIntent.confirm(id). idempotency_key works on each step."
)

# 4. Log the practice
chub_annotate(
  id="stripe/api",
  kind="practice",
  note="Always use separate create and confirm for PaymentIntents. One-step confirm=True is incompatible with idempotency and 3DS."
)
```

Future agents fetching `stripe/api` see these annotations automatically — they never rediscover this the hard way.

---

## End-of-task checklist

After any task involving library calls:

- [ ] Did I confirm any undocumented bugs? → `kind=issue`
- [ ] Did I find a workaround? → `kind=fix` (paired with the issue)
- [ ] Did I validate a useful pattern? → `kind=practice`
- [ ] Did I check for existing annotations first?
- [ ] Is each annotation one fact with exact values?
