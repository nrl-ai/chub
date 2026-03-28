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

## Writing effective context docs

### Keep them focused

Each context doc should cover one topic. Agents work better with several focused docs than one massive file:

```
.chub/context/
  architecture.md       # system overview and module boundaries
  conventions.md        # coding standards and patterns
  api-conventions.md    # API-specific rules (status codes, auth, pagination)
  deployment.md         # how to deploy, environments, rollback
  data-model.md         # database schema, relationships, migration rules
```

### Write for agents, not humans

Context docs are injected into agent prompts. Write them as instructions the agent should follow, not documentation a human would browse:

```markdown
---
name: API Conventions
description: "Rules for REST API endpoints in this project"
tags: api, rest, conventions
---

# API Conventions

## Response format

All endpoints return JSON with this envelope:
- `{ "data": ... }` for success (200, 201)
- `{ "error": { "code": "...", "message": "..." } }` for errors

## Authentication

All routes under `/api/v1/` require a Bearer token. Public routes
live under `/api/public/`. Never add auth-required routes to `/api/public/`.

## Naming

- Resource names are plural: `/users`, `/invoices`, not `/user`, `/invoice`
- Use kebab-case for multi-word paths: `/payment-methods`
- IDs are UUIDs, never sequential integers
```

### Use tags for discovery

Tags help agents find relevant context. Keep them short and consistent:

```yaml
tags: api, rest, conventions      # good — specific and searchable
tags: project documentation       # bad — too generic
```

## Templates

Here are starter templates for common context doc types:

### Architecture

```markdown
---
name: Architecture
description: "System architecture, module boundaries, and key decisions"
tags: architecture, design
---

# Architecture

## System overview
<!-- High-level diagram or description -->

## Module boundaries
<!-- Which crate/package owns what, and rules for cross-module imports -->

## Key decisions
<!-- Why we chose X over Y, with enough context to prevent agents from "improving" it -->
```

### Conventions

```markdown
---
name: Coding Conventions
description: "Code style, patterns, and practices for this project"
tags: conventions, style
---

# Coding Conventions

## Error handling
<!-- How errors propagate, which types to use, what to log -->

## Testing
<!-- Test naming, fixtures, mocking policy, coverage expectations -->

## Dependencies
<!-- Rules for adding deps, approved vs. prohibited libraries -->
```

### Deployment

```markdown
---
name: Deployment
description: "Deployment process, environments, and rollback procedures"
tags: deployment, ops
---

# Deployment

## Environments
<!-- staging, production, how they differ -->

## Deploy process
<!-- Steps to deploy, what to verify before and after -->

## Rollback
<!-- How to roll back, who can authorize, SLA expectations -->
```

## Cross-referencing

Context docs can reference each other and link to pinned registry docs:

```markdown
See the API conventions in `api-conventions.md` for endpoint rules.
For the OpenAI client patterns, refer to the pinned `openai/chat` doc.
```

Agents resolve these naturally when all context is loaded via `chub_context`.

## Use cases

- Architecture decisions ("why we chose X over Y")
- Internal API documentation
- Coding conventions specific to this codebase
- Deployment and operational runbooks
- Module boundary definitions
- Onboarding guides for new team members or agents
- Security policies (what agents must never do)
