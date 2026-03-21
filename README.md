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
| **Native speed** | 27× faster builds, 26ms cold start, 1.2 MB binary — no Node.js required |
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

Or download a pre-built binary from [GitHub Releases](https://github.com/vietanhdev/chub/releases).

---

## Usage

```sh
# Search and browse
chub search                          # list all entries
chub search "stripe"                 # BM25 search
chub search --tags openai --lang py  # filtered search

# Fetch docs and skills
chub get stripe/api --lang js        # fetch a doc
chub get openai/chat stripe/api      # fetch multiple
chub get openai/chat --full          # all files
chub get openai/chat -o doc.md       # save to file

# Registry management
chub update                          # refresh registries
chub update --full                   # download full bundles
chub cache status
chub cache clear

# Annotations (persist across sessions)
chub annotate stripe/api "Webhook needs raw body"
chub annotate --list
chub annotate stripe/api --clear

# Feedback
chub feedback stripe/api up
chub feedback stripe/api down --label outdated

# MCP server (for Claude Code, Cursor, Windsurf, etc.)
chub mcp

# JSON output (all commands support --json)
chub search "stripe" --json
```

---

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
Registry resource: `chub://registry`.

---

## Team Features (Roadmap)

The next phase adds a `.chub/` directory at the project root — committed to git, shared with the team.

```
my-project/
├── .chub/
│   ├── config.yaml          # Project-level config (overrides ~/.chub/config.yaml)
│   ├── pins.yaml            # Pinned doc versions — locked for the whole team
│   ├── annotations/         # Team knowledge, git-tracked
│   ├── context/             # Custom docs: architecture, conventions, runbooks
│   └── profiles/            # Role-scoped context (backend.yaml, frontend.yaml)
```

**Three-tier inheritance**: personal (`~/.chub/`) → project (`.chub/`) → profile. Later tiers override, none are required. A solo developer works without any `.chub/`; a team adds it when ready.

See [docs/plan.md](docs/plan.md) for the full roadmap.

---

## Benchmarks

Measured on the production corpus (1,553 docs, 6 skills, 1,691 files). Best-of-3, warm filesystem cache, Windows 11 / AMD64.

### Build

| Operation | JS (`chub`) | Rust (`Chub`) | Speedup |
|-----------|-------------|---------------|---------|
| `build` (4 entries) | 1,050 ms | **38 ms** | **27×** |
| `build` (1,559 entries) | 6,300 ms | **2,500 ms** | **2.5×** |
| `build --validate-only` (1,559 entries) | 6,300 ms | **360 ms** | **17×** |
| Cold start (`--help`) | 120 ms | **26 ms** | **4.6×** |

> The full build includes copying 1,691 files to disk — pure I/O that both versions must do. `--validate-only` isolates computation: parsing, indexing, and validation run **17× faster**.

### Resource Usage

| Metric | JS | Rust |
|--------|-----|------|
| Binary size | ~70 MB (with `node_modules`) | **1.2 MB** |
| Runtime dependency | Node.js 20+ | None (single binary) |
| Memory (build, 1,559 entries) | ~120 MB | **~15 MB** |

---

## Architecture

### vs. Context Hub (JS)

| Area | JS | Rust (Chub) |
|------|----|-------------|
| **Search** | Linear scan of all documents | Inverted index — scores only matching docs |
| **Directory walking** | 3 walks per entry | Single-pass walkdir |
| **JSON output** | `JSON.stringify` → string → write | Streaming `serde_json` with `BufWriter` |
| **Frontmatter** | Regex + `yaml` package | `serde_yaml` with CRLF/BOM handling |
| **ID collision check** | `Map<string, bool>` | `HashSet` (half the memory) |
| **Startup** | Node.js VM boot (~120ms) | Native binary (~26ms) |
| **Distribution** | Requires Node.js runtime | Single static binary via npm optional deps |
| **Path handling** | OS-native separators (broken on Windows) | Always forward slashes |

### Data Format Parity

Chub reads and writes the exact same formats as the JS version:

- **`registry.json`** — identical structure, all camelCase field names via `serde(rename)`
- **`search-index.json`** — identical BM25 index (same tokenizer, same IDF values, same field weights)
- **`config.yaml`** — same defaults, same env var overrides (`CHUB_DIR`, `CHUB_BUNDLE_URL`)
- **DOC.md / SKILL.md frontmatter** — same YAML parser, same field extraction
- **Annotation JSONs** — same `~/.chub/annotations/` directory layout

Verified by building the production corpus with both versions and diffing output:
- 1,553 doc IDs: **identical**
- 6 skill IDs: **identical**
- 3,864 IDF terms: **max value difference = 0.0**
- Average field lengths: **identical**

### Project Structure

```
chub/
├── Cargo.toml                     # Workspace root
├── crates/
│   ├── chub-core/                 # Library: all business logic
│   │   └── src/
│   │       ├── types.rs           # Registry, Entry, SearchIndex structs
│   │       ├── config.rs          # YAML config + env var overrides
│   │       ├── frontmatter.rs     # YAML frontmatter parser
│   │       ├── normalize.rs       # Language alias map (js/ts/py/rb/cs)
│   │       ├── cache.rs           # Source dirs, meta.json, cache stats
│   │       ├── fetch.rs           # reqwest HTTP, registry/bundle/doc fetch
│   │       ├── registry.rs        # Multi-source merge, search, entry resolution
│   │       ├── annotations.rs     # JSON CRUD at ~/.chub/annotations/
│   │       ├── identity.rs        # Platform machine UUID → SHA-256
│   │       ├── telemetry.rs       # Feedback API + agent detection
│   │       ├── search/
│   │       │   ├── tokenizer.rs   # Shared tokenizer + 52 stop words
│   │       │   ├── bm25.rs        # BM25 scoring (k1=1.5, b=0.75)
│   │       │   └── index.rs       # Inverted index for fast search
│   │       └── build/
│   │           ├── discovery.rs   # Single-pass entry discovery
│   │           └── builder.rs     # Registry + index generation
│   └── chub-cli/                  # Binary: CLI + MCP server
│       └── src/
│           ├── main.rs            # tokio runtime, clap dispatch
│           ├── output.rs          # JSON/human dual-mode output
│           ├── commands/
│           │   ├── build.rs
│           │   ├── search.rs
│           │   ├── get.rs
│           │   ├── update.rs
│           │   ├── cache.rs
│           │   ├── annotate.rs
│           │   └── feedback.rs
│           └── mcp/
│               ├── server.rs      # MCP stdio server (rmcp)
│               └── tools.rs       # 5 tool handlers + registry resource
├── npm/                           # npm distribution
│   ├── chub/                      # @nrl-ai/chub (thin JS wrapper)
│   ├── chub-linux-x64/
│   ├── chub-linux-arm64/
│   ├── chub-darwin-x64/
│   ├── chub-darwin-arm64/
│   └── chub-win32-x64/
└── docs/
    └── plan.md                    # Full product roadmap
```

---

## Test Suite

62 tests covering full behavioral parity with the JS implementation:

| Suite | Tests | Coverage |
|-------|-------|----------|
| Tokenizer | 6 | Stop words, punctuation, edge cases |
| BM25 search | 7 | Scoring, ranking, limits, field weights |
| Inverted index | 1 | Parity with linear scan across 10 queries |
| Frontmatter parser | 9 | YAML, CRLF, BOM, empty, numeric values |
| Language normalization | 4 | Aliases, case, unknown languages |
| Build integration (CLI) | 15 | Output format, validation, structure, versioning |
| Search parity | 20 | Quality corpus, multi-word, tags, descriptions |

---

## Implementation Status

### Phase 1: Foundation + Build — Done
Types, config, frontmatter, normalize, tokenizer, BM25, discovery, builder. Full `chub build` with all flags. 62 tests, zero warnings.

### Phase 2: Search + Get + Cache — Done
`cache.rs`, `fetch.rs`, `registry.rs`, `annotations.rs`. CLI commands: `search`, `get`, `update`, `cache`.

### Phase 3: Feedback + Identity — Done
`identity.rs` (machine UUID → SHA-256), `telemetry.rs` (fire-and-forget feedback POST, agent detection). CLI commands: `feedback`, `annotate`.

### Phase 4: MCP Server — Done
`chub mcp` stdio server via `rmcp`. 5 tool handlers: `chub_search`, `chub_get`, `chub_list`, `chub_annotate`, `chub_feedback`. Registry resource at `chub://registry`. Path traversal + ID validation.

### Phase 5: npm Distribution — Done
Platform packages for 5 targets. Thin JS wrapper at `@nrl-ai/chub` with `optionalDependencies + bin/chub.js`. Same `npx chub` UX as SWC/Biome.

### Phase 6+: Team Features — Planned
See [docs/plan.md](docs/plan.md).

---

## License

MIT
