# Agent Integrations

Chub works with AI coding agents at two levels:

1. **MCP server** (`chub mcp`) — runtime doc lookup, annotations, context. Works with any MCP client.
2. **Agent config** (`chub agent-config sync`) — generates static rules files for 10 agent targets.

## Quick Start

```sh
npm install -g chub          # install
chub init --from-deps        # initialize project, detect deps
chub agent-config sync       # generate rules for all configured targets
```

## MCP Server Setup

`chub mcp` is a stdio MCP server. Add it to your agent's MCP config:

### Claude Code

`.claude/settings.json`:
```json
{ "mcpServers": { "chub": { "command": "chub", "args": ["mcp"] } } }
```

Or install the plugin: `claude /plugin install https://github.com/nrl-ai/chub`

### Cursor

`.cursor/mcp.json`:
```json
{ "mcpServers": { "chub": { "command": "chub", "args": ["mcp"] } } }
```

### Windsurf

`~/.codeium/windsurf/mcp_config.json` (via Windsurf settings UI):
```json
{ "mcpServers": { "chub": { "command": "chub", "args": ["mcp"] } } }
```

### GitHub Copilot (VS Code)

`.vscode/mcp.json`:
```json
{ "servers": { "chub": { "command": "chub", "args": ["mcp"] } } }
```

Note: Copilot uses `"servers"` not `"mcpServers"`.

### Gemini CLI

`.gemini/settings.json`:
```json
{ "mcpServers": { "chub": { "command": "chub", "args": ["mcp"] } } }
```

### Kiro

`.kiro/settings/mcp.json`:
```json
{ "mcpServers": { "chub": { "command": "chub", "args": ["mcp"] } } }
```

### Codex (OpenAI)

`.codex/config.toml`:
```toml
[[mcp_servers]]
name = "chub"
command = "chub"
args = ["mcp"]
```

Or: `codex mcp add chub -- chub mcp`

### Cline / Roo Code

Configure via the extension's MCP settings UI (VS Code sidebar). Add:
```json
{ "mcpServers": { "chub": { "command": "chub", "args": ["mcp"] } } }
```

Roo Code also supports `.roo/mcp.json` for project-scoped config.

### Continue.dev

`.continue/config.yaml`:
```yaml
mcpServers:
  - name: chub
    command: chub
    args: ["mcp"]
```

### Aider

```sh
aider --mcp-server-command "chub mcp"
```

### Any MCP Client

Stdio transport, JSON-RPC 2.0, protocol version `2024-11-05`:
```sh
chub mcp   # reads stdin, writes stdout
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

## Agent Config Generation

`chub agent-config sync` generates static rules files from `.chub/config.yaml`. This gives agents project context even without MCP.

### Supported targets

| Target name | Output file | Agent |
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

### Configuration

Add targets to `.chub/config.yaml`:

```yaml
agent_rules:
  global:
    - "Run tests before committing"
  targets:
    - cursorrules
    - claude.md
    - copilot
    - agents.md
    - gemini.md
    - kiro
```

Then:

```sh
chub agent-config sync     # generate all targets
chub agent-config diff     # preview what would change
```

## Skills (Claude Code)

Skills are slash commands in `.claude/skills/`:

| Command | Purpose |
|---------|---------|
| `/docs <query>` | Search or fetch documentation |
| `/annotate <id> <note>` | Record a team annotation |
| `/setup` | Initialize chub for the project |

Plugin users get namespaced versions: `/chub:docs`, `/chub:annotate`, `/chub:setup`.

## Cross-Agent Compatibility

Some files are read by multiple agents:

| File | Read by |
|------|---------|
| `AGENTS.md` | Codex, Roo Code, Augment Code |
| `CLAUDE.md` | Claude Code, Augment Code |
| `GEMINI.md` | Gemini CLI |

For polyglot teams, generate `agents.md` + your primary agent's target.
