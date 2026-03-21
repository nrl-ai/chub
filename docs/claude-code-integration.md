# Claude Code Integration

Chub integrates with Claude Code via three layers: **MCP tools** (runtime), **skills** (workflows), and a **plugin** (distribution).

## Setup (this repo)

Already configured in `.claude/settings.json`. Clone and go — Claude Code picks up MCP tools, skills, and hooks automatically.

## Setup (other projects)

### Option A: Plugin (recommended)

```sh
claude /plugin install https://github.com/nrl-ai/chub
```

Gives you MCP tools + skills (`/chub:docs`, `/chub:annotate`, `/chub:setup`).

### Option B: MCP only

Add to `.claude/settings.json`:

```json
{
  "mcpServers": {
    "chub": { "command": "chub", "args": ["mcp"] }
  }
}
```

## MCP Tools

| Tool | Purpose |
|------|---------|
| `chub_search` | Search docs by query, tags, or language |
| `chub_get` | Fetch a doc by ID (e.g. `serde/derive`) |
| `chub_list` | List all available docs |
| `chub_context` | Get pinned docs + profile rules + project context |
| `chub_pins` | Add/remove/list pinned docs |
| `chub_annotate` | Read/write team annotations |
| `chub_feedback` | Submit doc quality feedback |

## Skills

| Command | What it does |
|---------|-------------|
| `/docs <query>` | Search or fetch documentation |
| `/annotate <id> <note>` | Record a team annotation |
| `/setup` | Initialize chub for the current project |

When installed as a plugin, skills are namespaced: `/chub:docs`, `/chub:annotate`, `/chub:setup`.

## Hooks

- **`chub-freshness-check.sh`** — Warns before `git commit` if pinned docs may be stale. Non-blocking.

## Architecture

```
Claude Code starts
  ├── .claude/settings.json → starts chub MCP server
  ├── .claude/skills/       → registers /docs, /annotate, /setup
  └── CLAUDE.md             → project rules, pinned docs, context docs

User asks about an API
  ├── chub_search → find docs
  ├── chub_get    → fetch content
  └── chub_annotate (read) → check for team notes
```
