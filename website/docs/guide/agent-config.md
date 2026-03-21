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

  include_pins: true              # Include pinned doc references
  include_context: true           # Include project context doc names
  include_annotation_policy: true # Instruct agents to write back what they discover

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

## Annotation Policy

When you encounter something non-obvious while using a library, record it:

- `chub_annotate id="<id>" kind="issue" note="..."` — undocumented bugs, broken params, misleading examples
- `chub_annotate id="<id>" kind="fix" note="..."` — workarounds that resolved an issue
- `chub_annotate id="<id>" kind="practice" note="..."` — patterns the team prefers or has validated

Rules:
- Annotate after confirming, not speculatively — only write what you have verified works or fails
- One fact per annotation — do not bundle multiple issues into one note
- Be reproducible — include the exact call, param, or value, not vague descriptions
- Check first — read existing annotations (`chub_annotate id=<id>`) before writing to avoid duplicates
- Do not annotate what is already in the official docs — only capture what the docs missed or got wrong

## Module: backend (src/api/**)
- Use Zod for all request validation

## Module: frontend (src/components/**)
- Use Tailwind CSS, no inline styles
```

The `include_annotation_policy: true` section is only emitted when that flag is set. It gives agents a standing order to annotate — without repeating it in every session prompt.

## Auto-sync with git hooks

```sh
# .git/hooks/pre-commit
chub agent-config sync && git add CLAUDE.md .cursorrules AGENTS.md
```
