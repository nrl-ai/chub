# Context Profiles

Different roles need different context. Profiles scope which docs, rules, and context an agent loads — without changing shared pins.

## Profile files

Profiles live in `.chub/profiles/` and are plain YAML files committed to git. Each file defines the context for a specific role or task.

```
.chub/profiles/
  base.yaml       # shared rules for all roles
  backend.yaml    # backend/API development
  frontend.yaml   # UI/frontend development
  data.yaml       # data engineering and ML
```

## Profile inheritance

Profiles can extend a base so shared rules are written once:

```yaml
# .chub/profiles/base.yaml
name: Base
description: "Shared rules for all roles"
rules:
  - "Follow coding conventions in .chub/context/conventions.md"
  - "Run tests before committing"
context:
  - conventions.md
  - architecture.md
```

```yaml
# .chub/profiles/backend.yaml
name: Backend Developer
extends: base                 # inherits base rules and context
description: "Context for backend/API development"
pins:
  - openai/chat
  - stripe/api
context:
  - api-conventions.md
rules:
  - "Use Zod for all request validation"
```

## Profile fields

| Field | Type | Description |
|---|---|---|
| `name` | string | Display name for the profile |
| `extends` | string | Name of another profile to inherit from |
| `description` | string | Short description shown in `chub profile list` |
| `rules` | list of strings | Instructions injected into agent context |
| `pins` | list of strings | Entry IDs to include when this profile is active (e.g. `openai/chat`) |
| `context` | list of strings | Project context doc filenames from `.chub/context/` to include |

Inherited fields from `extends` are merged with the child profile's fields. Child rules are appended after parent rules. Child `context` and `pins` lists are unioned with the parent's.

## Commands

```sh
chub profile use backend     # Activate profile
chub profile use none        # Clear active profile
chub profile list            # Show available profiles
```

## Monorepo auto-profile

In `.chub/config.yaml`, set `auto_profile` to automatically switch profiles based on file paths:

```yaml
auto_profile:
  - path: "packages/api/**"
    profile: backend
  - path: "packages/web/**"
    profile: frontend
```

When an agent opens a file in `packages/api/`, the `backend` profile is loaded automatically.

## MCP integration

Activate a profile before starting the MCP server, or use the `auto_profile` config to switch automatically based on open files:

```sh
chub profile use backend
chub mcp
```

The `chub_context` MCP tool returns the full active context in a single call — profile rules, context docs, pinned docs, and annotations:

```json
{ "task": "implement payment flow" }
// Returns profile rules, context docs, pinned docs, and annotations in one call
```

The agent receives everything it needs to work correctly for the active role without making multiple separate requests.

## Related features

- [Project Context](/guide/project-context) — author the custom docs referenced in `context:`
- [Agent Config Sync](/guide/agent-config) — sync profile rules to `CLAUDE.md` / `.cursorrules`
