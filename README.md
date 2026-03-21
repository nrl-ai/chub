# Chub

> The missing context layer for AI-assisted development teams.

Built on [Context Hub](https://github.com/andrewyng/context-hub) by Andrew Ng — Chub is a high-performance Rust rewrite that extends the original with team-first features: shared doc pinning, git-tracked annotations, context profiles, and agent config sync.

**For individuals**: drop-in replacement for `chub` with a faster binary, better search, and persistent annotations.
**For teams**: commit a `.chub/` directory to your repo and every developer and every AI agent gets the same versioned context, automatically.

---

## What Chub does

Coding agents hallucinate APIs and forget what they learn between sessions. Context Hub's answer — curated, versioned markdown docs served via CLI and MCP — works well. Chub keeps that foundation and adds:

| | |
|---|---|
| **Native speed** | 27x faster builds, 26ms cold start, 1.2 MB binary — no Node.js required |
| **Team pins** | Lock docs to specific versions so every agent on the team uses the same reference |
| **Shared annotations** | Team knowledge lives in `.chub/annotations/` — committed to git, surfaced automatically |
| **Custom project context** | Your architecture docs, API conventions, and runbooks, served alongside public docs |
| **Context profiles** | Scope which docs and rules each role (backend, frontend, etc.) gets |
| **Agent config sync** | Generate and keep `CLAUDE.md`, `.cursorrules`, `AGENTS.md` in sync from one source |
| **Private registry** | Self-host internal docs alongside the public registry — no cloud required |
| **Full compatibility** | Same registry format, search index, and config schema as Context Hub |

---

## Installation

```sh
npm install -g @nrl-ai/chub
```

Or via pip:

```sh
pip install chub
```

Or download a pre-built binary from [GitHub Releases](https://github.com/vietanhdev/chub/releases).

---

## Usage

```sh
# Search and fetch docs
chub search "stripe"
chub get openai/chat --lang python
chub list

# Initialize project for team sharing
chub init
chub init --from-deps          # auto-detect dependencies

# Pin docs for the team
chub pin openai/chat --lang python --version 4.0
chub pins                      # list pins
chub get --pinned              # fetch all pinned docs

# Context profiles
chub profile use backend
chub profile list

# Team annotations
chub annotate openai/chat "Use streaming API" --team

# Auto-detect dependencies and pin matching docs
chub detect --pin

# Generate CLAUDE.md, .cursorrules from .chub/config.yaml
chub agent-config generate

# Doc freshness check
chub check --fix

# Snapshots
chub snapshot create v1.0
chub snapshot diff v1.0 v2.0

# MCP server
chub mcp

# JSON output (all commands)
chub search "stripe" --json
```

---

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

MCP tools: `chub_search`, `chub_get`, `chub_list`, `chub_annotate`, `chub_feedback`.
Registry resource: `chub://registry`.

---

## Team Features

The `.chub/` directory at the project root is committed to git and shared with the team.

```
my-project/
├── .chub/
│   ├── config.yaml          # Project-level config
│   ├── pins.yaml            # Pinned doc versions
│   ├── annotations/         # Team knowledge, git-tracked
│   ├── context/             # Custom docs: architecture, conventions
│   └── profiles/            # Role-scoped context (backend.yaml, frontend.yaml)
```

### Three-tier config inheritance

```
~/.chub/config.yaml          # Tier 1 — personal defaults
    ↓ overridden by
.chub/config.yaml            # Tier 2 — project config (shared)
    ↓ overridden by
.chub/profiles/<name>.yaml   # Tier 3 — role/task profile
```

### Implemented features

| Feature | Status | Description |
|---|---|---|
| Project init (`chub init`) | Done | Create `.chub/` with sensible defaults |
| Doc pinning | Done | Lock doc versions in `pins.yaml` |
| Team annotations | Done | Git-tracked annotations with merge |
| Context profiles | Done | Role-scoped context with inheritance |
| Project context | Done | Custom markdown docs served via MCP |
| Dep auto-detection | Done | 9 file types (npm, Cargo, pip, Go, etc.) |
| Agent config sync | Done | Generate CLAUDE.md, .cursorrules, etc. |
| Doc snapshots | Done | Point-in-time pin snapshots |
| Doc freshness | Done | Compare pinned vs installed versions |
| Usage analytics | Done | Local opt-in fetch tracking |

See [docs/plan.md](docs/plan.md) for the full roadmap.

---

## Benchmarks

Measured on the production corpus (1,553 docs, 6 skills, 1,691 files).

### Build

| Operation | JS (`chub`) | Rust (`Chub`) | Speedup |
|---|---|---|---|
| `build` (4 entries) | 1,050 ms | **38 ms** | **27x** |
| `build` (1,559 entries) | 6,300 ms | **2,500 ms** | **2.5x** |
| `build --validate-only` | 6,300 ms | **360 ms** | **17x** |
| Cold start (`--help`) | 120 ms | **26 ms** | **4.6x** |

### Resource Usage

| Metric | JS | Rust |
|---|---|---|
| Binary size | ~70 MB (with `node_modules`) | **1.2 MB** |
| Runtime dependency | Node.js 20+ | None (single binary) |
| Memory (build, 1,559 entries) | ~120 MB | **~15 MB** |

---

## Test Suite

99 tests covering behavioral parity and team features:

| Suite | Tests | Coverage |
|---|---|---|
| Tokenizer | 6 | Stop words, punctuation, edge cases |
| BM25 search | 7 | Scoring, ranking, limits, field weights |
| Inverted index | 1 | Parity with linear scan |
| Frontmatter parser | 9 | YAML, CRLF, BOM, empty, numeric |
| Language normalization | 4 | Aliases, case, unknown |
| Build integration | 15 | Output format, validation, structure |
| Search parity | 20 | Multi-word, tags, descriptions |
| Team features | 33 | Pins, profiles, annotations, snapshots, detect, freshness, agent config, analytics |

All tests use isolated temp directories — no writes to the repo's `.chub/`.

---

## Documentation

Full documentation at [chub.nrl.ai](https://chub.nrl.ai) (VitePress):

- [Getting Started](https://chub.nrl.ai/guide/getting-started)
- [Team Features](https://chub.nrl.ai/guide/pinning)
- [CLI Reference](https://chub.nrl.ai/reference/cli)
- [Configuration](https://chub.nrl.ai/reference/configuration)
- [MCP Server](https://chub.nrl.ai/reference/mcp-server)

---

## License

MIT
