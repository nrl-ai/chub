# Configuration

## Config file locations

| Tier | Location | Scope |
|---|---|---|
| 1 | `~/.chub/config.yaml` | Personal defaults (machine-wide) |
| 2 | `.chub/config.yaml` | Project config (git-tracked) |
| 3 | `.chub/profiles/<name>.yaml` | Role/task profile |

Later tiers override earlier ones. No tier is required.

## Full config example

```yaml
# .chub/config.yaml

# Registry sources
sources:
  - name: official
    url: https://cdn.aichub.org/v1
  - name: company
    url: https://docs.internal.company.com/chub

cdn_url: "https://cdn.aichub.org/v1"
source: "official"            # default source
output_dir: "./dist"
output_format: "markdown"     # or json
refresh_interval: 86400       # cache TTL in seconds
telemetry: false
feedback: false

# Tier 3 org annotation server (URL is not sensitive — safe to commit)
annotation_server:
  url: https://annotations.internal.company.com
  auto_push: false     # set true to mirror every team write to org tier
  cache_ttl_secs: 3600 # local cache TTL in seconds (default: 3600)

# Agent rules
agent_rules:
  global:
    - "Follow the project coding conventions"
  modules:
    backend:
      path: "src/api/**"
      rules:
        - "Use Zod for validation"
  include_pins: true
  include_context: true
  include_annotation_policy: true  # inject annotation instructions for agents
  targets:
    - claude.md
    - cursorrules

# Monorepo auto-profile
auto_profile:
  - path: "packages/api/**"
    profile: backend
  - path: "packages/web/**"
    profile: frontend
```

## Environment variables

| Variable | Description |
|---|---|
| `CHUB_DIR` | Override `~/.chub` data directory |
| `CHUB_BUNDLE_URL` | Override the default CDN URL |
| `CHUB_PROJECT_DIR` | Override project root (for testing) |
| `CHUB_ANNOTATION_SERVER` | Override org annotation server URL |
| `CHUB_ANNOTATION_TOKEN` | Auth token for org annotation server (personal only, never commit) |

## annotation_server config fields

The `annotation_server` key is part of `.chub/config.yaml` (project config, safe to commit). The token is **not** stored here — use `~/.chub/config.yaml` `annotation_token` or the `CHUB_ANNOTATION_TOKEN` env var.

| Field | Type | Default | Description |
|---|---|---|---|
| `url` | string | — | Base URL of the annotation server (required) |
| `auto_push` | bool | `false` | Mirror every team (Tier 2) write to the org server |
| `cache_ttl_secs` | integer | `3600` | How long to cache org annotations locally (seconds) |

## pins.yaml

```yaml
pins:
  - id: openai/chat
    lang: python          # optional
    version: "4.0"        # optional
    reason: "Use v4 API"  # optional
    source: official      # optional
```

## Profile format

```yaml
name: Backend Developer
extends: base              # optional inheritance
description: "Backend dev context"
rules:
  - "Use Zod for validation"
pins:                      # list of entry ID strings, not objects
  - openai/chat
  - stripe/api
context:
  - api-conventions.md
```

The `pins:` field in a profile is a list of entry ID strings (e.g. `openai/chat`). For per-pin version and language overrides, use `.chub/pins.yaml` instead.

## Team annotation format

Team annotations live in `.chub/annotations/` as per-entry YAML files. The filename uses `--` as a path separator (e.g. `openai--chat.yaml` for the `openai/chat` entry).

```yaml
# .chub/annotations/openai--chat.yaml
id: openai/chat
issues:
  - author: alice
    date: 2026-03-20
    severity: high        # high | medium | low
    note: "Rate limit errors occur above 50 req/s — add exponential backoff"
fixes:
  - author: alice
    date: 2026-03-20
    note: "Use exponential backoff with jitter; see utils/retry.ts"
practices:
  - author: bob
    date: 2026-03-18
    note: "Always use streaming for chat completions; set max_tokens=4096"
notes:
  - author: carol
    date: 2026-03-15
    note: "v4 SDK required — v3 patterns will not work with our setup"
```

Annotation kinds: `issues` (known problems), `fixes` (workarounds), `practices` (team conventions), `notes` (general observations). All annotations are appended to the doc when fetched via `chub get` or MCP.

## Context doc frontmatter

```yaml
---
name: Project Architecture
description: "Architecture overview"
tags: architecture, microservices
---
```

| Field | Required | Description |
|---|---|---|
| `name` | Yes | Display name |
| `description` | No | Short description |
| `tags` | No | Comma-separated search tags |
