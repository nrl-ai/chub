# @nrl-ai/chub

> The missing context layer for AI-assisted development teams.

Chub is a high-performance CLI + MCP server that serves curated, versioned API documentation to AI coding agents. Built in Rust, it is a drop-in replacement for [Context Hub](https://github.com/andrewyng/context-hub) with team-first features.

## Installation

```sh
npm install -g @nrl-ai/chub
```

## Quick Start

```sh
# Search for docs
chub search "stripe payments"

# Fetch a doc
chub get openai/chat --lang python

# List all available docs
chub list
```

## Features

- **1,560+ curated docs** — API references for popular libraries and frameworks
- **~44ms cold start** — native Rust binary, no Node.js runtime needed
- **10 MB binary** — vs ~22 MB node_modules
- **MCP server** — AI agents search and fetch docs automatically
- **Doc pinning** — lock versions so every agent uses the same reference
- **Team annotations** — shared knowledge committed to git
- **Context profiles** — role-scoped context with inheritance
- **Agent config sync** — generate CLAUDE.md, .cursorrules from one source
- **Dep auto-detection** — scan package.json, Cargo.toml, requirements.txt and more

## Usage

### Team setup

```sh
# Initialize project for team sharing
chub init

# Auto-detect dependencies and pin matching docs
chub init --from-deps

# Pin doc versions for the team
chub pin openai/chat --lang python --version 4.0
chub pins
```

### MCP server

Start the built-in MCP server for AI agents:

```sh
chub mcp
```

Add to your MCP config (`.mcp.json` for Claude Code, `.cursor/mcp.json` for Cursor):

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

### More commands

```sh
chub search "auth" --limit 5            # search with limit
chub get stripe/api --lang javascript    # language-specific doc
chub profile use backend                 # activate context profile
chub annotate openai/chat "note" --team  # team annotation
chub detect --pin                        # auto-pin from dependencies
chub agent-config generate               # generate CLAUDE.md, .cursorrules
chub check --fix                         # update outdated pins
chub snapshot create v1.0                # snapshot current pins
chub update                              # refresh cached registry
```

## How It Works

This package is a thin JavaScript wrapper that resolves the correct platform-specific binary from `optionalDependencies`. No Node.js runtime is needed at execution time — it runs the native Rust binary directly.

Supported platforms:

| Package | Platform |
|---|---|
| `@nrl-ai/chub-linux-x64` | Linux x86_64 |
| `@nrl-ai/chub-linux-arm64` | Linux ARM64 |
| `@nrl-ai/chub-darwin-x64` | macOS Intel |
| `@nrl-ai/chub-darwin-arm64` | macOS Apple Silicon |
| `@nrl-ai/chub-win32-x64` | Windows x86_64 |

## Also available via

```sh
pip install chub          # PyPI
cargo install chub        # crates.io
brew install nrl-ai/tap/chub  # Homebrew
```

## Links

- [Documentation](https://chub.nrl.ai)
- [GitHub](https://github.com/nrl-ai/chub)
- [PyPI](https://pypi.org/project/chub/)
- [crates.io](https://crates.io/crates/chub)

## License

MIT
