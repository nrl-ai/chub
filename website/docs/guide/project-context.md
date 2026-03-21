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
chub list --project
```

## Use cases

- Architecture decisions ("why we chose X over Y")
- Internal API documentation
- Coding conventions specific to this codebase
- Deployment and operational runbooks
- Module boundary definitions
