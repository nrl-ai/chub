<p align="center">
  <img src="https://raw.githubusercontent.com/nrl-ai/chub/main/website/assets/logo.svg" width="80" height="80" alt="Chub">
</p>

<h1 align="center">@nrl-ai/chub</h1>

<p align="center">
  <strong>Agent-agnostic context, tracking, and cost analytics for AI-assisted development.</strong>
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/@nrl-ai/chub"><img src="https://img.shields.io/npm/v/@nrl-ai/chub?color=0ea5e9&label=npm" alt="npm"></a>
  <a href="https://pypi.org/project/chub/"><img src="https://img.shields.io/pypi/v/chub?color=0ea5e9&label=pypi" alt="PyPI"></a>
  <a href="https://crates.io/crates/chub"><img src="https://img.shields.io/crates/v/chub?color=0ea5e9&label=crates.io" alt="crates.io"></a>
  <a href="https://github.com/nrl-ai/chub/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-0ea5e9" alt="License"></a>
</p>

Chub is the all-in-one agent layer: curated context, session tracking, cost analytics, and team knowledge for AI coding agents. Built in Rust, agent-agnostic, git-native.

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

### Context
- **1,553+ curated docs** — API references served to agents via MCP and CLI
- **Doc pinning** — lock versions so every agent uses the same reference
- **Context profiles** — role-scoped context with inheritance
- **Project context** — custom architecture docs, conventions, runbooks
- **Dep auto-detection** — scan package.json, Cargo.toml, requirements.txt and more

### Tracking & Analytics
- **Session tracking** — tokens, costs, models, tool calls across Claude Code, Cursor, Copilot, Gemini CLI, Codex
- **Cost estimation** — built-in rates for Claude, GPT, Gemini, DeepSeek, o1/o3
- **Web dashboard** — charts, breakdowns, session history at localhost:4243
- **Budget alerts** — configurable thresholds with warnings

### Self-Learning
- **Team annotations** — structured bugs, fixes, practices committed to git
- **Agent config sync** — generate CLAUDE.md, .cursorrules from one source
- **8 MCP tools** — agents search, fetch, annotate, track, and query context automatically

### Performance
- **~44ms cold start** — native Rust binary, no runtime deps
- **10 MB binary** — single binary, runs on Linux, macOS, Windows, ARM64

## Usage

### Team setup

```sh
# Initialize project for team sharing
chub init

# Auto-detect dependencies and pin matching docs
chub init --from-deps

# Pin doc versions for the team
chub pin add openai/chat --lang python --version 4.0
chub pin list
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

### Track AI usage

```sh
chub track enable                        # install hooks (auto-detects agent)
chub track status                        # see active session
chub track report --days 7               # costs, tokens, models
chub track dashboard                     # web dashboard
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
