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

## How inheritance works

Inheritance resolves from root to leaf, up to 10 levels deep. Circular references are detected and rejected.

- **`rules`** — parent rules come first, child rules are appended after
- **`pins`** and **`context`** — unioned (no duplicates)
- **`description`** — first non-empty value from child to root wins

```yaml
# base.yaml rules: ["Run tests"]
# backend.yaml extends base, rules: ["Use Zod"]
# Resolved backend rules: ["Run tests", "Use Zod"]
```

::: warning Gotcha
If a parent and child both pin the same entry, the child's version wins. But `rules` are always appended, never replaced — if you need to override a parent rule, add a contradicting rule in the child rather than trying to remove the parent's.
:::

## Monorepo auto-profile

In `.chub/config.yaml`, set `auto_profile` to automatically switch profiles based on file paths:

```yaml
auto_profile:
  - path: "packages/api/**"
    profile: backend
  - path: "packages/web/**"
    profile: frontend
```

When an agent opens a file matching a path pattern, the corresponding profile is loaded automatically. Patterns use glob syntax (powered by the `globset` crate). If no pattern matches, the profile falls back to prefix matching against the path.

### Priority

Rules are evaluated top-to-bottom. The **first match wins** — if a file matches multiple patterns, the profile from the first matching rule is used. Put more specific paths before broader ones:

```yaml
auto_profile:
  # Specific paths first
  - path: "packages/api/auth/**"
    profile: auth-specialist
  - path: "packages/api/**"
    profile: backend
  # Broader paths last
  - path: "packages/**"
    profile: fullstack
```

### Monorepo examples

**npm / pnpm workspaces:**

```yaml
# .chub/config.yaml
auto_profile:
  - path: "packages/api/**"
    profile: backend
  - path: "packages/web/**"
    profile: frontend
  - path: "packages/shared/**"
    profile: shared-libs
  - path: "apps/mobile/**"
    profile: mobile
```

```yaml
# .chub/profiles/backend.yaml
name: Backend
extends: base
pins:
  - express/routing
  - prisma/client
context:
  - api-conventions.md
rules:
  - "All endpoints must validate input with Zod schemas"
  - "Use repository pattern for database access"
```

```yaml
# .chub/profiles/frontend.yaml
name: Frontend
extends: base
pins:
  - react/hooks
  - nextjs/app-router
context:
  - ui-conventions.md
rules:
  - "Use server components by default, client components only when needed"
  - "All UI text must go through the i18n system"
```

**Cargo workspaces:**

```yaml
auto_profile:
  - path: "crates/core/**"
    profile: core-lib
  - path: "crates/cli/**"
    profile: cli
  - path: "crates/server/**"
    profile: server
```

```yaml
# .chub/profiles/core-lib.yaml
name: Core Library
extends: base
pins:
  - serde/derive
  - tokio/runtime
rules:
  - "No IO in this crate — all side effects go through trait abstractions"
  - "Every public type must implement Serialize and Deserialize"
```

**Python monorepo (src layout):**

```yaml
auto_profile:
  - path: "src/api/**"
    profile: api
  - path: "src/ml/**"
    profile: ml-pipeline
  - path: "src/workers/**"
    profile: workers
  - path: "tests/**"
    profile: testing
```

### Debugging auto-profile

Check which profile is active:

```sh
chub profile current
# → backend (auto-detected from packages/api/src/routes.ts)
```

If the wrong profile activates, verify your glob patterns. Common mistakes:
- Missing `**` — `packages/api` only matches the directory itself, not files inside it
- Overlapping patterns — remember first match wins
- The `CHUB_PROFILE` env var overrides auto-profile entirely

## MCP integration

Activate a profile before starting the MCP server, or use the `auto_profile` config to switch automatically based on open files:

```sh
chub profile use backend
chub mcp
```

The `chub_context` MCP tool returns the full active context in a single call — profile rules, context docs, pinned docs, and annotations. The agent receives everything it needs to work correctly for the active role without making multiple separate requests.

## Related features

- [Project Context](/guide/project-context) — author the custom docs referenced in `context:`
- [Agent Config Sync](/guide/agent-config) — sync profile rules to `CLAUDE.md` / `.cursorrules`
- [Doc Bundles](/guide/bundles) — install a curated set of pins in one command (useful with profiles)
