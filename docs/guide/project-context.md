# Project Context

Author custom markdown docs in `.chub/context/` with YAML frontmatter. They are served automatically alongside public docs via MCP and CLI.

## Creating context docs

```markdown
<!-- .chub/context/architecture.md -->
---
name: Project Architecture
description: "High-level architecture overview"
tags: architecture, microservices
---

# Architecture Overview

Our system uses an event-driven microservices architecture...
```

## Frontmatter fields

| Field | Required | Description |
|---|---|---|
| `name` | Yes | Display name |
| `description` | No | Short description |
| `tags` | No | Comma-separated tags for search |

## Fetching context docs

```sh
# Fetch a project doc
chub get project/architecture

# List project docs
chub context --list
```

## MCP access

Agents can fetch project context docs directly via the `chub_get` MCP tool using the `project/` prefix:

```json
{ "id": "project/architecture" }
```

The `chub_context` MCP tool returns all project context doc names along with the active profile rules, pinned docs, and annotations in a single call — so agents get full situational awareness without multiple requests.

## Including context in profiles

Profile YAML can reference context docs by filename in the `context:` list. When the profile is active, those docs are automatically injected alongside public registry docs:

```yaml
# .chub/profiles/backend.yaml
name: Backend Developer
extends: base
context:
  - api-conventions.md    # from .chub/context/
  - architecture.md       # from .chub/context/
rules:
  - "Follow the API conventions defined in api-conventions.md"
```

See [Context Profiles](/guide/profiles) for the full profile format and inheritance model.

## Use cases

- Architecture decisions ("why we chose X over Y")
- Internal API documentation
- Coding conventions specific to this codebase
- Deployment and operational runbooks
- Module boundary definitions
