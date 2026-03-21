# Context Profiles

Different roles need different context. Profiles scope which docs, rules, and context an agent loads — without changing shared pins.

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

Start the MCP server with a profile:

```sh
chub mcp --profile backend
```

Agents get focused, relevant context instead of the full registry.
