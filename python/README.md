# Chub

> The missing context layer for AI-assisted development teams.

Chub is a high-performance CLI + MCP server that serves curated, versioned API documentation to AI coding agents. It is a Rust rewrite of [Context Hub](https://github.com/andrewyng/context-hub) with team-first features: shared doc pinning, git-tracked annotations, context profiles, and agent config sync.

## Installation

```sh
pip install chub
```

Pre-built wheels are available for:

- Linux x86_64, aarch64
- macOS x86_64, Apple Silicon
- Windows x86_64

## Usage

```sh
# Search and browse
chub search                          # list all entries
chub search "stripe"                 # BM25 search
chub search --tags openai --lang py  # filtered search

# Fetch docs and skills
chub get stripe/api --lang python    # fetch a doc
chub get openai/chat stripe/api      # fetch multiple
chub get openai/chat --full          # all files
chub get openai/chat -o doc.md       # save to file

# Registry management
chub update                          # refresh registries
chub cache status
chub cache clear

# Annotations (persist across sessions)
chub annotate stripe/api "Webhook needs raw body"
chub annotate --list

# MCP server (for Claude Code, Cursor, Windsurf, etc.)
chub mcp
```

You can also invoke it as a Python module:

```sh
python -m chub search "stripe"
```

## MCP Integration

Add to your MCP config (`claude_desktop_config.json` or `.cursor/mcp.json`):

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

Available MCP tools: `chub_search`, `chub_get`, `chub_list`, `chub_annotate`, `chub_feedback`.

## How It Works

This Python package is a thin wrapper around the native Rust binary. When you run `chub` or `python -m chub`, it delegates to the platform-specific compiled binary bundled in the wheel. No runtime dependencies required.

## Links

- [GitHub](https://github.com/vietanhdev/chub)
- [Issues](https://github.com/vietanhdev/chub/issues)

## License

MIT
