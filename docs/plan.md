# Chub — Product Roadmap

> **Vision**: The missing context layer for AI-assisted development teams.
> Not just docs lookup — a shared, versioned, project-aware knowledge system that makes every agent on your team as informed as your best engineer.

---

## Why Chub

[Context Hub](https://github.com/andrewyng/context-hub) by Andrew Ng solves a real problem: coding agents hallucinate APIs and forget what they learn between sessions. Its solution — curated, versioned markdown docs served via CLI and MCP — works well. Chub is built on that foundation.

**What Chub adds**:
1. **Performance** — a native Rust binary replaces the Node.js runtime: 5× faster validation, 19× faster search, ~44ms cold start vs 131ms, 10 MB binary vs ~22 MB `node_modules`.
2. **Team features** — doc pinning, shared annotations, custom project context, and agent config sync; things an individual tool doesn't need but a team does.
3. **Full compatibility** — identical registry format, search index, and config schema. Content authored for Context Hub works in Chub without changes.

> **Reference**: Context7 (~50K GitHub stars) validates that there is strong demand for up-to-date docs in AI coding tools. Chub targets the same problem space from the open-source, self-hosted, team-first angle.

### Capability comparison

| Capability | Context Hub (JS) | Chub (Rust) |
|---|---|---|
| Public library docs | 1,600+ curated | 1,600+ curated |
| Custom/private docs | Yes (build cmd) | Yes (build cmd) |
| Offline mode | Yes (bundle) | Yes (bundle) |
| Team collaboration | No | **Yes** (pins, profiles, annotations) |
| Project awareness | No | **Yes** (auto-detect deps) |
| Agent config sync | No | **Yes** (CLAUDE.md, .cursorrules, AGENTS.md) |
| Git-tracked context | No | **Yes** (`.chub/` in repo) |
| Context profiles | No | **Yes** (role-scoped) |
| Self-hosted registry | Yes | Yes + `chub serve` |
| MCP server | 5 tools | **7 tools** (+ team tools) |
| CLI commands | 7 | **20** |
| Cold start | ~131 ms | **~44 ms** |
| Binary size | ~22 MB (node_modules) | **10 MB** (native) |

---

## Flexibility Model

Chub uses a **three-tier config inheritance** chain. Later tiers override earlier ones:

```
~/.chub/config.yaml          # Tier 1 — personal defaults (machine-wide)
    ↓ overridden by
.chub/config.yaml            # Tier 2 — project config (git-tracked, shared)
    ↓ overridden by
.chub/profiles/<name>.yaml   # Tier 3 — role/task profile (opt-in per session)
```

This means:
- A developer can work solo without any `.chub/` directory.
- A team can add `.chub/` to the repo and everyone benefits immediately on next pull.
- Power users can switch profiles for focused tasks without changing shared config.
- Orgs can layer a company-wide config via a shared source, overridden per project.

---

## Phase 6: Team Foundation

The minimum viable team feature set. Everything in this phase lives in `.chub/` and is committed to git.

### 6.1 — Project-level `.chub/` directory

```
my-project/
├── .chub/
│   ├── config.yaml          # Project-level config (overrides ~/.chub/config.yaml)
│   ├── pins.yaml            # Pinned docs with locked versions
│   ├── annotations/         # Team-shared annotations (git-tracked)
│   │   ├── openai--chat.yaml
│   │   └── stripe--api.yaml
│   ├── context/             # Custom project docs (auto-served via MCP)
│   │   ├── architecture.md
│   │   ├── api-conventions.md
│   │   └── auth-flow.md
│   └── profiles/            # Named context profiles (role or task-scoped)
│       ├── base.yaml        # Shared base — all profiles can extend this
│       ├── backend.yaml
│       └── frontend.yaml
```

**Commands**:
```bash
chub init                    # Create .chub/ with sensible defaults
chub init --from-deps        # Auto-detect from package.json/Cargo.toml/etc.
chub init --monorepo         # Scaffold root + per-package .chub/ dirs
```

**Why**: Every team member and every AI agent gets the same context. No "did you read the wiki?" — it's in the repo, versioned, reviewable.

### 6.2 — Doc Pinning

Pin specific docs and versions so every team member (and agent) uses the same reference material. Pins are declarative and version-controlled.

**`.chub/pins.yaml`**:
```yaml
pins:
  - id: openai/chat
    lang: python
    version: "4.0.0"
    reason: "We use v4 streaming API — do NOT suggest v3 patterns"

  - id: stripe/api
    lang: javascript
    # version omitted = always latest

  - id: nextjs/app-router
    lang: javascript
    version: "15.0.0"
    reason: "Locked until migration to v15 app router is complete"

  - id: internal/auth-service
    source: private          # served from team's private registry
    reason: "Internal auth microservice docs"
```

**Commands**:
```bash
chub pin openai/chat --lang python --version 4.0.0
chub pin stripe/api
chub unpin openai/chat
chub pins                    # List all pins with versions and reasons
chub get --pinned            # Fetch all pinned docs at once
```

**MCP integration**: When an agent calls `chub_get`, pinned version/language is automatically applied — the agent does not need to know which version to use.

When serving a pinned doc, Chub appends a team notice:
```
---
[Team pin] Locked to v4.0.0 (python). Reason: We use v4 streaming API — do NOT suggest v3 patterns.
```

### 6.3 — Team Annotations (Git-tracked)

Current annotations are per-machine (`~/.chub/annotations/`). Team annotations live in `.chub/annotations/` and are shared via git.

**`.chub/annotations/openai--chat.yaml`**:
```yaml
id: openai/chat
notes:
  - author: alice
    date: 2026-03-15
    note: "Webhook endpoint requires raw body parsing — do NOT use express.json() before it"
  - author: bob
    date: 2026-03-18
    note: "Rate limits hit at 500 RPM on our plan. Use exponential backoff with jitter."
```

**Resolution order** (last wins):
1. Public doc content (from registry)
2. Team annotations (`.chub/annotations/`)
3. Personal annotations (`~/.chub/annotations/`)

**Commands**:
```bash
chub annotate openai/chat "Use v4 streaming, not completions" --team
chub annotate openai/chat --team --list   # Show team annotations
chub annotate openai/chat --personal      # Personal-only (current behavior)
```

### 6.4 — Custom Project Context

Teams author custom markdown docs in `.chub/context/` that are served automatically alongside public docs via MCP and CLI.

**Use cases**:
- Architecture decisions ("why we chose X over Y")
- Internal API documentation
- Coding conventions and patterns specific to this codebase
- Deployment and operational runbooks
- Module boundary definitions

**`.chub/context/architecture.md`**:
```markdown
---
name: Project Architecture
description: "High-level architecture of our payment processing system"
tags: architecture, payments, microservices
---

# Architecture Overview

Our system uses an event-driven microservices architecture...
```

Project docs appear in `chub search` and `chub_search` results alongside public docs, with a `[project]` badge. Fetched via `chub get project/architecture`.

### 6.5 — Context Profiles

Different roles need different context. Profiles let you scope which docs, annotations, and rules an agent loads — without changing shared pins.

**Profile inheritance**: profiles can extend a base, so shared rules are written once.

**`.chub/profiles/base.yaml`**:
```yaml
name: Base
description: "Shared rules for all roles"
rules:
  - "Follow the coding conventions in .chub/context/conventions.md"
  - "Run tests before committing"
context:
  - conventions.md
  - architecture.md
```

**`.chub/profiles/backend.yaml`**:
```yaml
name: Backend Developer
extends: base                 # inherits base rules and context
description: "Context for backend/API development"
pins:
  - openai/chat
  - stripe/api
  - redis/cache
  - postgresql/queries
context:
  - api-conventions.md
  - auth-flow.md
rules:
  - "Use Zod for all request validation"
  - "All endpoints must have OpenAPI annotations"
```

**`.chub/profiles/frontend.yaml`**:
```yaml
name: Frontend Developer
extends: base
description: "Context for UI/component development"
pins:
  - nextjs/app-router
  - tailwindcss/core
  - react/hooks
context:
  - component-patterns.md
rules:
  - "Use Tailwind CSS — no inline styles"
  - "All components must be server components unless they need interactivity"
```

**Commands**:
```bash
chub profile use backend     # Activate profile for this session
chub profile use none        # Clear active profile
chub profile list            # Show available profiles with descriptions
chub get --profile backend   # Fetch all profile docs at once (no session change)
```

**MCP integration**: `chub mcp --profile backend` starts the MCP server scoped to that profile. Agents get focused, relevant context instead of the full registry.

### 6.6 — Dependency Auto-Detection

Scan the project's dependency files and suggest/pin relevant docs automatically.

**Supported**:
- `package.json` (npm/pnpm/yarn)
- `requirements.txt` / `pyproject.toml` / `Pipfile` (Python)
- `Cargo.toml` (Rust)
- `go.mod` (Go)
- `Gemfile` (Ruby)
- `pom.xml` / `build.gradle` (Java)

**Commands**:
```bash
chub detect                  # Scan deps, show available docs
chub detect --pin            # Auto-pin all detected docs
chub detect --diff           # Show new deps since last detect
```

**Example output**:
```
Detected 12 dependencies with available docs:

  openai (python)          → openai/chat [pinnable]
  stripe (python)          → stripe/api [pinnable]
  fastapi (python)         → fastapi/app [pinnable]
  redis (python)           → redis/cache [pinnable]
  pydantic (python)        → pydantic/models [pinnable]
  ✗ custom-internal-lib    → no match

Pin all? chub detect --pin
```

---

## Phase 7: Agent Config Sync

### 7.1 — AGENTS.md / CLAUDE.md Generation

The fragmentation problem: teams maintain separate `CLAUDE.md`, `.cursorrules`, `.windsurfrules`, `copilot-instructions.md`, `AGENTS.md` files that drift out of sync. Each agent tool reads a different file.

Chub generates and syncs all of them from a single source of truth in `.chub/config.yaml`.

**`.chub/config.yaml`** (agent rules section):
```yaml
agent_rules:
  global:
    - "Always use TypeScript strict mode"
    - "Use pnpm, not npm or yarn"
    - "Write tests for all new functions"
    - "Follow the error handling pattern in src/lib/errors.ts"

  modules:
    backend:
      path: "src/api/**"
      rules:
        - "Use Zod for all request validation"
        - "All endpoints must have OpenAPI annotations"
    frontend:
      path: "src/components/**"
      rules:
        - "Use Tailwind CSS, no inline styles"
        - "All components must be server components unless they need interactivity"

  include_pins: true          # Auto-include pinned doc references
  include_context: true       # Auto-include .chub/context/ doc names

  targets:
    - claude.md               # → CLAUDE.md
    - cursorrules             # → .cursorrules
    - windsurfrules           # → .windsurfrules
    - agents.md               # → AGENTS.md
    - copilot                 # → .github/copilot-instructions.md
```

**Commands**:
```bash
chub agent-config generate   # Generate all target files
chub agent-config sync       # Update targets only if source changed (idempotent)
chub agent-config diff       # Show what would change without writing
```

**Generated CLAUDE.md** (example):
```markdown
# Project Rules

- Always use TypeScript strict mode
- Use pnpm, not npm or yarn
- Write tests for all new functions

## Pinned Documentation
Use `chub get <id>` to fetch these docs when working with these libraries:
- openai/chat (python v4.0.0) — We use v4 streaming API
- stripe/api (javascript) — Latest version

## Project Context
Use `chub get project/<name>` or ask Chub for these:
- project/architecture — High-level system architecture
- project/api-conventions — REST API conventions and patterns

## Module: backend (src/api/**)
- Use Zod for all request validation
- All endpoints must have OpenAPI annotations

## Module: frontend (src/components/**)
- Use Tailwind CSS, no inline styles
- All components must be server components unless they need interactivity
```

**Auto-sync hook**: Add a git pre-commit or CI step to keep generated files in sync:
```bash
# .git/hooks/pre-commit  (or husky / lefthook)
chub agent-config sync && git add CLAUDE.md .cursorrules AGENTS.md
```

---

## Phase 8: Private Registry & Sharing

### 8.1 — Private Registry (`chub serve`)

Teams can self-host a registry for internal documentation — APIs, services, internal SDKs — and combine it with the public registry.

**`chub serve`** starts an HTTP registry server from a local content directory:
```bash
chub serve ./internal-docs --port 4242
# Now serves: http://localhost:4242/registry.json
```

For a shared/persistent deployment, build and deploy to any static host:
```bash
chub build ./internal-docs --output ./dist --base-url https://docs.internal.company.com/chub
# Deploy dist/ to Nginx, S3, GitHub Pages, etc.
```

**Configuring multiple sources** in `.chub/config.yaml`:
```yaml
sources:
  - name: official
    url: https://cdn.chub.dev/v1            # public registry
  - name: company
    url: https://docs.internal.company.com/chub
    auth: bearer
    token_env: CHUB_COMPANY_TOKEN           # secret from env, not committed
  - name: local-dev
    path: /shared/nfs/chub-registry         # NFS or local path
```

All sources merge transparently — `chub search "auth-service"` returns results from all sources ranked together. Private docs are clearly badged `[company]` or `[local]`.

### 8.2 — Doc Bundles (Shareable Collections)

A bundle is a curated list of docs packaged as a single shareable unit — like a "stack starter" for documentation.

**`.chub/bundles/ai-chat-starter.yaml`**:
```yaml
name: "AI Chat App Starter"
description: "Everything you need to build an AI chat application"
author: vietanhdev
entries:
  - openai/chat
  - anthropic/messages
  - nextjs/app-router
  - vercel/ai-sdk
  - tailwindcss/core
notes: "Recommended stack for production AI chat apps"
```

**Commands**:
```bash
chub bundle create ai-chat-starter    # Interactive bundle builder
chub bundle install ai-chat-starter   # Pin all entries from bundle
chub bundle publish ai-chat-starter   # Share to public registry
chub bundle list                      # Browse community bundles
```

Bundles can be referenced from a URL too — teams can share bundles as GitHub Gists or raw files:
```bash
chub bundle install https://gist.github.com/user/abc123/bundle.yaml
```

### 8.3 — Doc Freshness Monitoring

Detect when pinned doc versions lag behind the library version actually installed in the project.

```bash
chub check                   # Compare pinned versions vs installed versions
chub check --fix             # Auto-update outdated pins to current installed version
```

**Output**:
```
⚠  openai/chat pinned to v4.0.0 docs, but openai==4.52.0 is installed
   → chub pin openai/chat --version 4.52.0

✓  stripe/api docs are current (v2025.03)
✓  redis/cache docs are current

1 outdated pin found. Run `chub check --fix` to update.
```

---

## Phase 9: Intelligence Layer

### 9.1 — Smart Context Selection (`chub_context`)

Instead of agents fetching docs one-by-one, Chub analyzes the current task and returns an optimal, token-budget-aware bundle in one call.

**MCP tool**: `chub_context`
```json
{
  "task": "Add Stripe webhook handler for subscription events",
  "files_open": ["src/api/webhooks.ts", "src/lib/stripe.ts"],
  "profile": "backend",
  "max_tokens": 50000
}
```

**Returns**: A ranked, deduplicated bundle:
1. Pinned docs relevant to the task (scored by BM25 against the task description)
2. Team annotations on those docs
3. Project context docs mentioning relevant keywords
4. Active profile rules

This reduces agent "exploration cost" — instead of 3–4 `search → get` round trips, one call returns everything.

### 9.2 — Monorepo & Path-Scoped Profiles

In monorepos, context should change depending on which package or service the agent is working in.

**Directory structure**:
```
my-monorepo/
├── .chub/                   # Root-level: shared pins, annotations
│   ├── config.yaml
│   └── profiles/
│       └── base.yaml
├── packages/
│   ├── api-service/
│   │   └── .chub/
│   │       └── config.yaml  # Extends root; adds api-service-specific pins
│   └── web-app/
│       └── .chub/
│           └── config.yaml  # Extends root; adds frontend-specific pins
```

**Path-based auto-profile** in root `.chub/config.yaml`:
```yaml
auto_profile:
  - path: "packages/api-service/**"
    profile: backend
  - path: "packages/web-app/**"
    profile: frontend
  - path: "packages/shared/**"
    profile: base
```

When an agent opens a file in `packages/api-service/`, `chub mcp` automatically loads the `backend` profile without any manual switching.

### 9.3 — Task-Scoped Context (`chub context`)

Ephemeral context for a specific task — fetches relevant docs for the task without modifying pins or the active profile.

```bash
chub context "implement OAuth2 PKCE flow"
# → Returns: oauth2/pkce, openid-connect/core, fastapi/security
# → Does NOT add to pins, does NOT change active profile
```

**MCP tool**: `chub_task_context`
```json
{ "task": "implement OAuth2 PKCE flow" }
```

Useful for: one-off tasks, debugging sessions, exploring unfamiliar APIs — anything where you want relevant docs without committing to a pin.

### 9.4 — Usage Analytics (Local, Opt-in)

Track which docs the team actually uses to inform curation and pin cleanup decisions. All data stays local in SQLite.

```bash
chub stats                   # Show usage analytics
chub stats --json            # Machine-readable output for dashboards
```

**Output**:
```
Most fetched docs (last 30 days):
  1. openai/chat          — 142 fetches (8 unique agents)
  2. stripe/api           — 89 fetches
  3. nextjs/app-router    — 67 fetches

Never fetched (pinned but unused):
  - redis/cache           — pinned 45 days ago, 0 fetches
  - postgresql/queries    — pinned 30 days ago, 0 fetches

Suggestion: unpin unused docs to reduce noise.
```

**Storage**: `~/.chub/analytics.db` — opt-in, never leaves the machine.

### 9.5 — Doc Snapshots for CI/CD

Lock all documentation to a point-in-time snapshot for reproducible builds and regression audits.

```bash
chub snapshot create v2.1.0        # Capture all pinned doc versions
chub snapshot restore v2.1.0       # Restore exact doc versions
chub snapshot diff v2.0.0 v2.1.0   # What changed between releases
```

**Use case**: "The agent generated correct code in staging but wrong code in production" — snapshot diffs identify whether a doc update caused the regression.

---

## Phase 10: Distribution & Ecosystem

### 10.1 — Multi-language SDK

| Package | Registry | Use case |
|---------|----------|----------|
| `@nrl-ai/chub` | npm | Node.js projects, JS/TS tooling |
| `chub` | PyPI | Python projects, Jupyter notebooks |
| `chub` | crates.io | Rust projects, embedded use |
| `chub` | Homebrew | macOS users |
| `chub` | GitHub Releases | Direct binary download |

**Python SDK** (beyond CLI wrapper):
```python
from chub import ChubClient

client = ChubClient(profile="backend")
docs = client.get("openai/chat", lang="python")
results = client.search("payment processing")
context = client.task_context("implement OAuth2 PKCE flow")
client.annotate("stripe/api", "Use idempotency keys for all charges", team=True)
```

### 10.2 — IDE Extensions

- **VS Code extension**: Sidebar showing pinned docs, inline doc previews, profile switcher, annotation editing
- **JetBrains plugin**: Same for IntelliJ/PyCharm/WebStorm
- **Neovim plugin**: Telescope integration for `chub search` and `chub get`

### 10.3 — CI/CD Integration

```yaml
# GitHub Actions
- uses: nrl/chub-action@v1
  with:
    check-freshness: true      # Fail if pinned docs are stale vs package.json versions
    validate-pins: true        # Ensure all deps have pinned docs
    sync-agent-config: true    # Regenerate CLAUDE.md etc. if .chub/config.yaml changed
    fail-on-drift: true        # Fail if generated files differ from committed files
```

---

## Implementation Progress

| Priority | Feature | Phase | Status | Notes |
|----------|---------|-------|--------|-------|
| P0 | `chub init` + `.chub/` directory | 6.1 | **Done** | `init_project()` with `--from-deps` and `--monorepo` |
| P0 | Doc pinning (`pins.yaml`) | 6.2 | **Done** | CRUD + `--pinned` flag on `chub get` + MCP integration |
| P0 | Team annotations (git-tracked) | 6.3 | **Done** | Write/read/append/merge + pin notices |
| P0 | Context profiles with inheritance | 6.5 | **Done** | `extends:` inheritance, circular detection, active profile |
| P1 | Custom project context | 6.4 | **Done** | Frontmatter parsing, `chub get project/<name>` |
| P1 | Dependency auto-detection | 6.6 | **Done** | 9 file types (npm, Cargo, pip, pyproject, Pipfile, go.mod, Gemfile, pom.xml, Gradle) |
| P1 | AGENTS.md / CLAUDE.md generation | 7.1 | **Done** | 5 targets: claude.md, cursorrules, windsurfrules, agents.md, copilot |
| P1 | Private registry + `chub serve` | 8.1 | **Done** | HTTP server via axum |
| P2 | Doc freshness monitoring | 8.3 | **Done** | `check_freshness()` + `auto_fix_freshness()` |
| P2 | Doc bundles | 8.2 | Partial | Bundle struct defined, install/publish not wired |
| P2 | Smart context selection | 9.1 | Planned | |
| P2 | Monorepo + path-scoped profiles | 9.2 | **Done** | `auto_profile` config with path globs |
| P3 | Task-scoped ephemeral context | 9.3 | Planned | |
| P3 | Local usage analytics | 9.4 | **Done** | `record_fetch()` + `get_stats()` with JSONL storage |
| P3 | Doc snapshots | 9.5 | **Done** | Create/restore/diff/list |
| P3 | CI/CD integration | 10.3 | Planned | |
| P3 | Python/npm SDKs | 10.1 | Partial | npm wrapper done, Python not started |
| P3 | IDE extensions | 10.2 | Planned | |

### Test coverage

99 tests across 4 test suites, all passing:
- 31 unit tests (tokenizer, BM25, frontmatter, normalize)
- 15 build parity tests
- 20 search parity tests
- 33 team feature integration tests (isolated temp dirs, no repo pollution)

---

## Design Principles

1. **Git-first**: Team config lives in the repo. If it's not in git, it doesn't exist for the team.
2. **Gradual adoption**: Works for a solo developer today; adds team value when `.chub/` is committed. No big-bang migration.
3. **Three-tier inheritance**: Personal (`~/.chub/`) → project (`.chub/`) → profile. Later tiers override; no tier is required.
4. **Agent-native**: Every feature is accessible via MCP. CLI is for humans, MCP is for agents. Same data, same logic.
5. **Zero cloud dependency**: Everything works offline and locally. Cloud features (private registry, bundle publishing) are opt-in.
6. **Convention over configuration**: `chub init` gives a working setup in one command. Customization is there when needed.
7. **Fast**: Every operation under 50ms. Speed is a feature — agents must not wait for context.
