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
    - agents.md            # → AGENTS.md (Codex, Roo Code, Augment)
    - copilot              # → .github/copilot-instructions.md
    - gemini.md            # → GEMINI.md
    - clinerules           # → .clinerules
    - roorules             # → .roo/rules/chub-rules.md
    - augmentrules         # → .augment/rules/chub-rules.md
    - kiro                 # → .kiro/steering/chub-rules.md
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

When you encounter something non-obvious while using a library, record it via the `chub_annotate` MCP tool (or `chub annotate` CLI):

- **Issue** (confirmed bug): `chub_annotate` with `id`, `kind="issue"`, `severity="high|medium|low"`, `note`
- **Fix** (workaround): `chub_annotate` with `id`, `kind="fix"`, `note`
- **Practice** (team convention): `chub_annotate` with `id`, `kind="practice"`, `note`

Rules:
- Annotate after confirming, not speculatively — only write what you have verified works or fails
- One fact per annotation — do not bundle multiple issues into one note
- Be reproducible — include the exact call, param, or value, not vague descriptions
- Check first — call `chub_annotate` with only `id` to read existing annotations before writing to avoid duplicates
- Do not annotate what is already in the official docs — only capture what the docs missed or got wrong

## Module: backend (src/api/**)
- Use Zod for all request validation

## Module: frontend (src/components/**)
- Use Tailwind CSS, no inline styles
```

The `include_annotation_policy: true` section is only emitted when that flag is set. It gives agents a standing order to annotate — without repeating it in every session prompt.

## Writing effective rules

Good agent rules are **specific, actionable, and scoped**. The agent reads them as instructions — vague guidance gets vague behavior.

### Do

```yaml
global:
  - "Use TypeScript strict mode in all .ts files"
  - "Run `npm test` before committing"
  - "API responses must include `requestId` for tracing"

modules:
  api:
    path: "src/api/**"
    rules:
      - "Validate all request bodies with Zod schemas in src/api/schemas/"
      - "Return 422 for validation errors, not 400"
```

### Don't

```yaml
global:
  - "Write good code"              # too vague — what does "good" mean?
  - "Follow best practices"        # which practices? be explicit
  - "Be careful with errors"       # not actionable
```

### Tips

1. **Lead with the verb**: "Use", "Run", "Return", "Validate" — not "You should consider..."
2. **Include paths**: Module-scoped rules with `path:` globs keep rules relevant to what the agent is editing
3. **One fact per rule**: "Use Zod for validation" not "Use Zod for validation and also make sure to..."
4. **Reference tools**: Agents know how to run `chub get <id>` — tell them which docs to fetch
5. **Pin + annotate**: Use `include_pins: true` so agents see which libraries are endorsed, and `include_annotation_policy: true` so they write back what they discover

### Cross-agent compatibility

Some files are read by multiple agents — generate these for the widest reach:

| File | Read by |
|------|---------|
| `AGENTS.md` | Codex, Roo Code, Augment Code |
| `CLAUDE.md` | Claude Code, Augment Code |
| `GEMINI.md` | Gemini CLI |
| `.cursorrules` | Cursor |
| `.clinerules` | Cline, Roo Code (partial) |

For polyglot teams, generate `agents.md` + your primary agent's target.

### Supported targets (full list)

| Config name | Output file | Agent |
|-------------|------------|-------|
| `claude.md` | `CLAUDE.md` | Claude Code |
| `cursorrules` | `.cursorrules` | Cursor |
| `windsurfrules` | `.windsurfrules` | Windsurf |
| `agents.md` | `AGENTS.md` | Codex, Roo Code, Augment |
| `copilot` | `.github/copilot-instructions.md` | GitHub Copilot |
| `gemini.md` | `GEMINI.md` | Gemini CLI |
| `clinerules` | `.clinerules` | Cline |
| `roorules` | `.roo/rules/chub-rules.md` | Roo Code |
| `augmentrules` | `.augment/rules/chub-rules.md` | Augment Code |
| `kiro` | `.kiro/steering/chub-rules.md` | Kiro |

## Auto-sync with git hooks

```sh
# .git/hooks/pre-commit
chub agent-config sync && git add CLAUDE.md .cursorrules AGENTS.md
```
