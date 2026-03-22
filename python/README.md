# Chub

> The missing context layer for AI-assisted development teams.

Chub is a high-performance CLI + MCP server that serves curated, versioned API documentation to AI coding agents. It is a Rust rewrite of [Context Hub](https://github.com/andrewyng/context-hub) with team-first features: shared doc pinning, git-tracked annotations, context profiles, and agent config sync.

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
chub pin openai/chat --lang python      # pin a doc version
chub pins                               # list pinned docs
chub profile use backend                # activate a profile
chub annotate openai/chat "note" --team # team annotation
chub detect --pin                       # auto-pin from deps
chub agent-config generate              # generate CLAUDE.md, .cursorrules
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

MCP tools: `chub_search`, `chub_get`, `chub_list`, `chub_context`, `chub_pins`, `chub_annotate`, `chub_feedback`.

## How It Works

This Python package is a thin wrapper around the native Rust binary. When you run `chub` or `python -m chub`, it delegates to the platform-specific compiled binary bundled in the wheel. No Python runtime dependencies are required.

## Links

- [Documentation](https://chub.nrl.ai)
- [GitHub](https://github.com/nrl-ai/chub)
- [npm package](https://www.npmjs.com/package/@nrl-ai/chub)
- [Issues](https://github.com/nrl-ai/chub/issues)

## License

MIT
