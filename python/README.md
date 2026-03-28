<p align="center">
  <img src="https://raw.githubusercontent.com/nrl-ai/chub/main/website/assets/logo.svg" width="80" height="80" alt="Chub">
</p>

<h1 align="center">Chub</h1>

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
pip install chub
```

Pre-built wheels are available for:

- Linux x86_64, ARM64
- macOS x86_64, Apple Silicon
- Windows x86_64

## Quick Start

```sh
# Search for docs
chub search "stripe payments"

# Fetch a doc
chub get openai/chat --lang python

# List all available docs
chub list

# Initialize project for team sharing
chub init
```

You can also invoke Chub as a Python module:

```sh
python -m chub search "stripe"
python -m chub get openai/chat --lang python
```

## Usage

### Search and fetch

```sh
chub search "stripe"                    # BM25 search
chub search "auth" --limit 5            # limit results
chub get stripe/api --lang python       # fetch a doc
chub get openai/chat --version 4.0      # specific version
chub list                               # list all docs
chub list --json                        # JSON output
```

### Team features

```sh
chub init                               # create .chub/ project directory
chub init --from-deps                   # auto-detect dependencies
chub pin add openai/chat --lang python  # pin a doc version
chub pin list                           # list pinned docs
chub profile use backend                # activate a profile
chub annotate openai/chat "note" --team # team annotation
chub detect --pin                       # auto-pin from deps
chub agent-config generate              # generate CLAUDE.md, .cursorrules
```

### Tracking & analytics

```sh
chub track enable                       # install hooks (auto-detects agent)
chub track status                       # see active session
chub track report --days 7              # costs, tokens, models
chub track dashboard                    # web dashboard at localhost:4243
```

### Cache management

```sh
chub update                             # refresh cached registry
chub cache status                       # show cache state
chub cache clear                        # clear local cache
```

## MCP Integration

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

MCP tools: `chub_search`, `chub_get`, `chub_list`, `chub_context`, `chub_pins`, `chub_annotate`, `chub_feedback`, `chub_track`.

## How It Works

This Python package is a thin wrapper around the native Rust binary. When you run `chub` or `python -m chub`, it delegates to the platform-specific compiled binary bundled in the wheel. No Python runtime dependencies are required.

## Links

- [Documentation](https://chub.nrl.ai)
- [GitHub](https://github.com/nrl-ai/chub)
- [npm package](https://www.npmjs.com/package/@nrl-ai/chub)
- [Issues](https://github.com/nrl-ai/chub/issues)

## License

MIT
