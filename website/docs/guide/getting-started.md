# Getting Started

Install Chub and start serving curated docs to your AI coding agents in under 5 minutes.

## Installation

### npm (recommended)

```sh
npm install -g @nrl-ai/chub
```

### Cargo (from source)

```sh
cargo install chub
```

### Binary download

Download prebuilt binaries from [GitHub Releases](https://github.com/nrl-ai/chub/releases).

| Platform | Package |
|---|---|
| Linux x64 | `chub-linux-x64` |
| macOS x64 | `chub-darwin-x64` |
| macOS ARM | `chub-darwin-arm64` |
| Windows x64 | `chub-win32-x64` |

### Verify installation

```sh
chub --version
```

## Quick Start

### Search for docs

```sh
chub search "stripe payments"
```

### Fetch a doc

```sh
chub get openai/chat --lang python
```

### List all available docs

```sh
chub list
```

### Initialize a project

```sh
# Create .chub/ directory with defaults
chub init

# Auto-detect dependencies and pin matching docs
chub init --from-deps
```

### Pin docs for your team

```sh
# Pin a doc with version lock
chub pin openai/chat --lang python --version 4.0

# List pinned docs
chub pins

# Fetch all pinned docs at once
chub get --pinned
```

## MCP Setup

Chub includes a built-in MCP (Model Context Protocol) server for AI agents.

### Claude Code

Add to your `.mcp.json`:

```json
{
  "mcpServers": {
    "chub": {
      "command": "chub",
      "args": ["mcp"]
    }
  }
}
```

### Cursor

Go to **Settings → MCP Servers → Add Server**:

```
Command: chub mcp
Transport: stdio
```

### With a profile

```sh
chub mcp --profile backend
```

::: tip
When using MCP, pinned doc versions are automatically applied. Agents don't need to know which version to use.
:::

## Project Setup

`chub init` creates a `.chub/` directory in your project root:

```
my-project/
├── .chub/
│   ├── config.yaml        # Project config
│   ├── pins.yaml          # Pinned docs
│   ├── annotations/       # Team-shared annotations
│   ├── context/           # Custom project docs
│   └── profiles/          # Named context profiles
```

::: info
Commit `.chub/` to git so the whole team shares the same context. Personal settings stay in `~/.chub/`.
:::

### Three-tier config inheritance

```
~/.chub/config.yaml          # Tier 1 — personal defaults
    ↓ overridden by
.chub/config.yaml            # Tier 2 — project config (shared)
    ↓ overridden by
.chub/profiles/<name>.yaml   # Tier 3 — role/task profile
```

## Next Steps

- [Doc Pinning](/guide/pinning) — lock doc versions for your team
- [Context Profiles](/guide/profiles) — role-scoped context
- [CLI Reference](/reference/cli) — all commands and flags
- [Configuration](/reference/configuration) — config file format
