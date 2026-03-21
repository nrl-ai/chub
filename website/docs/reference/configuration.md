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
pins:
  - openai/chat
context:
  - api-conventions.md
```

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
