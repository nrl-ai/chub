# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commit style

Do not add `Co-Authored-By` trailers to commits. Commit messages should be plain, without AI attribution lines.

## What this repo is

Chub is a high-performance Rust rewrite of [Context Hub](https://github.com/andrewyng/context-hub) — a CLI + MCP server that serves curated, versioned API documentation to AI coding agents. It is fully format-compatible with the original JS version and extends it with team features (planned; see `docs/plan.md`).

## Commands

### Build & run
```sh
cargo build                          # debug build
cargo build --release                # optimised binary → target/release/chub
cargo run -- search "stripe"         # run directly (debug)
cargo run -- get openai/chat --lang python
```

### Test
```sh
cargo test --all                     # all tests in all crates
cargo test -p chub-core              # core library only
cargo test -p chub-core search       # tests whose name contains "search"
cargo test -p chub-core bm25         # specific module
```

Tests are inline (`#[cfg(test)]` blocks) in: `search/tokenizer.rs`, `search/bm25.rs`, `search/index.rs`, `frontmatter.rs`, `normalize.rs`, `annotations.rs`. Integration tests for the CLI (`build`, search parity) live in `crates/chub-cli/src/commands/build.rs` and related files. Team feature integration tests (pins, profiles, snapshots, bundles, org annotations) are in `crates/chub-core/tests/team_features.rs`.

### Lint & format
```sh
cargo fmt --all                      # format all crates
cargo fmt --all -- --check           # check only (CI mode)
cargo clippy --all -- -D warnings    # lint; warnings are errors
```

### Pre-commit (installed)
```sh
pre-commit run --all-files           # run all hooks manually
pre-commit run cargo-fmt             # run a single hook
```

### Version bump
```sh
./scripts/set-version.sh 0.2.0      # set version across all packages
```

The version is defined in 9 files across Rust, npm, and Python. **Always use the script** — never edit version strings by hand. The files it updates:
- `Cargo.toml` — workspace `version` and `chub-core` dependency version
- `npm/chub/package.json` — package version + 5 `optionalDependencies` versions
- `npm/chub-{linux-x64,linux-arm64,darwin-x64,darwin-arm64,win32-x64}/package.json` — package version
- `python/pyproject.toml` — package version
- `python/chub/__init__.py` — `__version__`

After bumping, run `cargo check` to regenerate `Cargo.lock`.

### Build the content registry (content → dist/)
```sh
cargo run --release -- build ./content -o ./dist
cargo run --release -- build ./content --validate-only
cargo run --release -- build ./content --base-url https://cdn.aichub.org/v1
```

## Architecture

### Crate layout

```
chub-core   — library: all business logic, no CLI concerns
chub-cli    — binary: CLI commands, MCP server, output formatting
```

`chub-cli` depends on `chub-core`; nothing else crosses crate boundaries.

### Data flow for `chub get` / `chub search`

```
Config (~/.chub/config.yaml + env vars)
  └─ sources: Vec<SourceConfig>   (URL or local path per source)

fetch::ensure_registry()          — fetches registry.json + search-index.json if stale
                                     cached at ~/.chub/sources/<name>/
registry::load_merged()           — loads all sources, merges into MergedRegistry
  ├─ docs: Vec<TaggedEntry>
  ├─ skills: Vec<TaggedEntry>
  └─ search_index: Option<SearchIndex>  (merged, document IDs namespaced as source:id)

registry::search_entries()        — BM25 via inverted index + lexical boost (Levenshtein)
registry::get_entry()             — exact lookup; handles source:id disambiguation
registry::resolve_doc_path()      — picks language/version, returns CDN path
fetch::fetch_doc()                — cache → CDN fallback; gzip-compressed above 10 KB
```

### Content format (for `chub build`)

```
content/
  <author>/
    docs/<entry-name>/
      <lang>/DOC.md            # YAML frontmatter + markdown
      <lang>/<version>/DOC.md  # versioned variant
    skills/<entry-name>/
      SKILL.md
```

`chub build` runs `build::discovery::discover_author()` + `build::builder::build_registry()` to produce `registry.json` and `search-index.json` in the output directory. The build is incremental by default (SHA-256 manifest skips unchanged files).

### Search pipeline

`search/tokenizer.rs` — shared tokenizer (52 stop words, punctuation stripping, `compact_identifier` strips all non-alphanumeric for fuzzy matching).

`search/bm25.rs` — BM25 scoring (k1=1.5, b=0.75). Fields: `id`, `name`, `description`, `tags`.

`search/index.rs` — inverted index built at load time from `search-index.json`. On a search, only docs containing ≥1 query term are scored (vs. linear scan in the JS version).

`registry::search_entries()` layers BM25 results with a lexical boost pass (Levenshtein distance, prefix/contains matching on compact identifiers). The lexical scan is global only when BM25 returns 0 results or a multi-word query has terms missing from the index.

### MCP server

`chub mcp` runs an stdio server via the `rmcp` crate. It does **not** go through the normal CLI flow — it has its own `mcp::server::run_mcp_server()` entry point and loads the registry independently.

MCP tools: `chub_search`, `chub_get`, `chub_list`, `chub_context`, `chub_pins`, `chub_annotate`, `chub_feedback`. Tool parameter structs use `schemars::JsonSchema` for schema generation. The registry is exposed as a resource at `chub://registry`.

### Shared utilities (`util.rs`)

`util.rs` contains shared helpers to avoid duplication: `days_to_date()`, `now_iso8601()`, `today_date()`, `sanitize_entry_id()` (replaces `/` with `--` for filenames), and `validate_filename()` (path traversal guard for snapshot/bundle names).

### Key env vars / config

| Var | Purpose |
|-----|---------|
| `CHUB_DIR` | Override `~/.chub` data directory |
| `CHUB_BUNDLE_URL` | Override the default CDN URL |

Config file: `~/.chub/config.yaml`. Multiple sources supported via `sources:` list; each source caches independently under `~/.chub/sources/<name>/`.

### npm distribution

`npm/chub/` is a thin JS wrapper (`bin/chub.js`) that resolves the platform-specific binary from `optionalDependencies`. No logic lives in the JS layer. The five platform packages (`chub-linux-x64`, etc.) are populated with the compiled Rust binary by CI.

### Format compatibility

All on-disk formats (`registry.json`, `search-index.json`, annotation JSONs) are byte-for-byte identical with the original JS Context Hub. The `serde(rename)` attributes on `types.rs` structs enforce camelCase field names to maintain this parity.

## Integrations

Chub integrates with AI coding agents via MCP (runtime tools) and agent config generation (static rules). See `docs/integrations.md` for full setup guides.

### MCP tools

Available via `chub mcp` (stdio server). Works with any MCP-compatible client.

| Tool | Purpose |
|------|---------|
| `chub_search` | Search docs by query, tags, or language |
| `chub_get` | Fetch a doc by ID (e.g. `serde/derive`) |
| `chub_list` | List all available docs |
| `chub_context` | Get pinned docs + profile rules + project context |
| `chub_pins` | Add/remove/list pinned docs |
| `chub_annotate` | Read/write team annotations |
| `chub_feedback` | Submit doc quality feedback |

### Agent config generation

`chub agent-config sync` generates rules files from `.chub/config.yaml` for 10 targets:

`claude.md`, `cursorrules`, `windsurfrules`, `agents.md`, `copilot`, `gemini.md`, `clinerules`, `roorules`, `augmentrules`, `kiro`

See `docs/integrations.md` for full reference.

### Skills (slash commands)

| Command | What it does |
|---------|-------------|
| `/docs <query>` | Search or fetch documentation |
| `/annotate <id> <note>` | Record a team annotation |
| `/setup` | Initialize chub for the current project |

### Project context

Pinned docs (`.chub/pins.yaml`): `serde/derive`, `clap/derive`, `tokio/runtime`, `axum/routing`.

Project context docs (`.chub/context/`): `architecture.md`, `conventions.md`, `team-features.md`. Access via `chub get project/<name>` or `chub_context` MCP tool.
