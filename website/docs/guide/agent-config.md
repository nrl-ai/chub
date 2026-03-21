# Agent Config Sync

Generate and sync agent config files from a single source of truth in `.chub/config.yaml`. Solves the fragmentation problem where teams maintain separate CLAUDE.md, .cursorrules, and .windsurfrules that drift out of sync.

## Configuration

```yaml
# .chub/config.yaml
agent_rules:
  global:
    - "Always use TypeScript strict mode"
    - "Write tests for all new functions"

  modules:
    backend:
      path: "src/api/**"
      rules:
        - "Use Zod for all request validation"
    frontend:
      path: "src/components/**"
      rules:
        - "Use Tailwind CSS, no inline styles"

  include_pins: true       # Include pinned doc references
  include_context: true    # Include project context doc names

  targets:
    - claude.md            # → CLAUDE.md
    - cursorrules          # → .cursorrules
    - windsurfrules        # → .windsurfrules
    - agents.md            # → AGENTS.md
    - copilot              # → .github/copilot-instructions.md
```

## Commands

```sh
chub agent-config generate   # Generate all target files
chub agent-config sync       # Update only if source changed
chub agent-config diff       # Show what would change
```

## Generated output example

```markdown
# Project Rules

- Always use TypeScript strict mode
- Write tests for all new functions

## Pinned Documentation
Use `chub get <id>` to fetch these docs:
- openai/chat (python v4.0) — We use v4 streaming API
- stripe/api (javascript) — Latest version

## Project Context
Use `chub get project/<name>` for these:
- project/architecture — High-level system architecture

## Module: backend (src/api/**)
- Use Zod for all request validation

## Module: frontend (src/components/**)
- Use Tailwind CSS, no inline styles
```

## Auto-sync with git hooks

```sh
# .git/hooks/pre-commit
chub agent-config sync && git add CLAUDE.md .cursorrules AGENTS.md
```
