<p align="center">
  <img src="website/assets/logo.svg" width="80" height="80" alt="Chub">
</p>

<h1 align="center">Chub</h1>

<p align="center">
  <strong>The missing context layer for AI-assisted development teams.</strong>
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/@nrl-ai/chub"><img src="https://img.shields.io/npm/v/@nrl-ai/chub?color=0ea5e9&label=npm" alt="npm"></a>
  <a href="https://pypi.org/project/chub/"><img src="https://img.shields.io/pypi/v/chub?color=0ea5e9&label=pypi" alt="PyPI"></a>
  <a href="https://crates.io/crates/chub"><img src="https://img.shields.io/crates/v/chub?color=0ea5e9&label=crates.io" alt="crates.io"></a>
  <a href="https://github.com/nrl-ai/chub/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-0ea5e9" alt="License"></a>
</p>

<p align="center">
  <a href="https://chub.nrl.ai">Docs</a> · <a href="https://chub.nrl.ai/guide/getting-started">Getting Started</a> · <a href="https://github.com/nrl-ai/chub/releases">Releases</a>
</p>

---

Built on [Context Hub](https://github.com/andrewyng/context-hub) by Andrew Ng — Chub is a high-performance Rust rewrite that extends the original with team-first features: shared doc pinning, git-tracked annotations, context profiles, and agent config sync.

**For individuals**: drop-in replacement for `chub` with a faster binary, better search, and persistent annotations.
**For teams**: commit a `.chub/` directory to your repo and every developer and every AI agent gets the same versioned context, automatically.

## Why Chub

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

### npm (recommended)

```sh
npm install -g @nrl-ai/chub
```

### pip

```sh
pip install chub
```

Pre-built wheels for Linux (x64, ARM64), macOS (x64, Apple Silicon), and Windows (x64).

### Cargo (build from source)

```sh
cargo install chub
```

### Homebrew (macOS / Linux)

```sh
brew install nrl-ai/tap/chub
```

### Binary download

Download prebuilt binaries from [GitHub Releases](https://github.com/nrl-ai/chub/releases):

| Platform | Binary |
|---|---|
| Linux x64 | `chub-linux-x64` |
| Linux ARM64 | `chub-linux-arm64` |
| macOS x64 | `chub-darwin-x64` |
| macOS ARM (Apple Silicon) | `chub-darwin-arm64` |
| Windows x64 | `chub-win32-x64.exe` |

### Verify installation

```sh
chub --version
```

---

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

### Initialize a project for team sharing

```sh
chub init                    # create .chub/ directory
chub init --from-deps        # auto-detect dependencies and pin matching docs
```

---

## Usage

### Search and fetch

```sh
chub search "stripe"                    # BM25 search across all docs
chub search "auth" --limit 5            # limit results
chub search "react" --source official   # search specific source
chub get openai/chat --lang python      # fetch doc by ID
chub get stripe/api --lang javascript   # language-specific
chub get openai/chat --version 4.0      # specific version
chub list                               # list all available docs
chub list --json                        # JSON output (works with all commands)
```

### Doc pinning

```sh
chub pin openai/chat --lang python --version 4.0 --reason "Use v4 API"
chub pin stripe/api --lang javascript
chub pins                               # list all pins
chub unpin openai/chat                  # remove a pin
chub get --pinned                       # fetch all pinned docs at once
```

### Context profiles

```sh
chub profile use backend                # activate a profile
chub profile use none                   # clear profile
chub profile list                       # list available profiles
```

### Team annotations

```sh
chub annotate openai/chat "Use streaming API" --team       # git-tracked
chub annotate openai/chat "My local note" --personal       # local only
```

### Dependency auto-detection

```sh
chub detect                             # show detected deps with matching docs
chub detect --pin                       # auto-pin all matches
```

### Agent config sync

```sh
chub agent-config generate              # generate CLAUDE.md, .cursorrules, etc.
chub agent-config sync                  # update only if changed
chub agent-config diff                  # preview changes
```

### Snapshots and freshness

```sh
chub snapshot create v1.0               # save current pins
chub snapshot list                      # list snapshots
chub snapshot restore v1.0              # restore pin state
chub snapshot diff v1.0 v2.0            # compare snapshots
chub check                              # check pinned vs installed versions
chub check --fix                        # auto-update outdated pins
```

### Cache management

```sh
chub update                             # refresh cached registry
chub cache status                       # show cache state
chub cache clear                        # clear local cache
```

### Usage analytics

```sh
chub stats                              # show fetch analytics (local, opt-in)
chub stats --json                       # JSON output
```

---

## MCP Integration

Chub includes a built-in MCP (Model Context Protocol) server that lets AI agents search and fetch docs directly.

```sh
chub mcp                                # start MCP stdio server
chub mcp --profile backend              # with a profile
```

### Claude Code

Add to `.mcp.json` in your project root:

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

Add to `.cursor/mcp.json`:

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

### Windsurf / Other agents

Any MCP-compatible agent can use Chub. The transport is stdio — just point the agent at `chub mcp`.

### MCP tools

| Tool | Description |
|---|---|
| `chub_search` | Search docs by query |
| `chub_get` | Fetch a doc by ID |
| `chub_list` | List all available docs |
| `chub_annotate` | Add an annotation |
| `chub_feedback` | Submit doc feedback |

Registry resource: `chub://registry`

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

### Feature overview

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

### Build performance

| Operation | JS (`chub`) | Rust (`Chub`) | Speedup |
|---|---|---|---|
| `build` (4 entries) | 1,050 ms | **38 ms** | **27x** |
| `build` (1,559 entries) | 6,300 ms | **2,500 ms** | **2.5x** |
| `build --validate-only` | 6,300 ms | **360 ms** | **17x** |
| Cold start (`--help`) | 120 ms | **26 ms** | **4.6x** |

### Resource usage

| Metric | JS | Rust |
|---|---|---|
| Binary size | ~70 MB (with `node_modules`) | **1.2 MB** |
| Runtime dependency | Node.js 20+ | **None** (single binary) |
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

```sh
cargo test --all                     # run all tests
```

---

## Content Registry

### Building from source content

```sh
chub build ./content -o ./dist                             # build registry
chub build ./content --validate-only                       # validate only
chub build ./content --base-url https://cdn.example.com/v1 # with CDN URL
```

### Serving a local registry

```sh
chub serve ./dist --port 4242        # serve as HTTP registry
```

### Content format

```
content/
  <author>/
    docs/<entry-name>/
      <lang>/DOC.md                  # YAML frontmatter + markdown
      <lang>/<version>/DOC.md        # versioned variant
    skills/<entry-name>/
      SKILL.md
```

---

## Documentation

Full documentation at [chub.nrl.ai](https://chub.nrl.ai):

- [Getting Started](https://chub.nrl.ai/guide/getting-started) — install and first commands
- [Installation](https://chub.nrl.ai/guide/installation) — all platforms and package managers
- [Why Chub](https://chub.nrl.ai/guide/why-chub) — comparison with Context Hub
- [Doc Pinning](https://chub.nrl.ai/guide/pinning) — lock doc versions
- [Context Profiles](https://chub.nrl.ai/guide/profiles) — role-scoped context
- [Team Annotations](https://chub.nrl.ai/guide/annotations) — shared knowledge
- [Project Context](https://chub.nrl.ai/guide/project-context) — custom docs
- [CLI Reference](https://chub.nrl.ai/reference/cli) — all commands and flags
- [Configuration](https://chub.nrl.ai/reference/configuration) — config file format
- [MCP Server](https://chub.nrl.ai/reference/mcp-server) — agent integration

---

## Contributing

```sh
cargo build                          # debug build
cargo test --all                     # run tests
cargo fmt --all                      # format
cargo clippy --all -- -D warnings    # lint
```

---

## License

MIT
