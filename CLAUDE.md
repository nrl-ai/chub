# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commit style

Do not add `Co-Authored-By` trailers to commits. Commit messages should be plain, without AI attribution lines.

## What this repo is

Chub is a high-performance Rust rewrite of [Context Hub](https://github.com/andrewyng/context-hub) — a CLI + MCP server that serves curated, versioned API documentation to AI coding agents. It is fully format-compatible with the original JS version and extends it with git-tracked team features (pinning, annotations, profiles, agent config sync). See `docs/plan.md` for roadmap.

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
```

Unit tests are inline `#[cfg(test)]` blocks. Integration tests: `crates/chub-core/tests/team_features.rs`, `crates/chub-core/tests/search_parity.rs`.

### Lint & format
```sh
cargo fmt --all                      # format all crates
cargo fmt --all -- --check           # check only (CI mode)
cargo clippy --all -- -D warnings    # lint; warnings are errors
```

### Pre-commit (installed)
```sh
pre-commit run --all-files           # run all hooks manually
```

### Version bump
```sh
./scripts/set-version.sh 0.2.0      # set version across all packages
```

The version is defined in 9 files across Rust, npm, and Python. **Always use the script** — never edit version strings by hand. After bumping, run `cargo check` to regenerate `Cargo.lock`.

### Build the content registry (content → dist/)
```sh
cargo run --release -- build ./content -o ./dist
cargo run --release -- build ./content --validate-only
cargo run --release -- build ./content --base-url https://cdn.aichub.org/v1
```

## Architecture

Two crates: `chub-core` (library — all business logic) and `chub-cli` (binary — CLI, MCP server, output). `chub-cli` depends on `chub-core`; nothing else crosses crate boundaries.

For detailed architecture, data flow, conventions, and team feature design, use chub's own project context:
- `chub get project/architecture` — crate layout, data flow, design decisions
- `chub get project/conventions` — error handling, serialization, CLI output, testing patterns
- `chub get project/team-features` — feature map, adding new features, config inheritance

### Key paths

| Area | Location |
|------|----------|
| Core library | `crates/chub-core/src/` |
| CLI commands | `crates/chub-cli/src/commands/` (one file per command) |
| Team features | `crates/chub-core/src/team/` |
| MCP server | `crates/chub-cli/src/mcp/` |
| Search pipeline | `crates/chub-core/src/search/` (tokenizer → BM25 → inverted index → lexical boost) |
| Shared helpers | `crates/chub-core/src/util.rs` |
| Content registry | `content/<author>/docs/<entry>/<lang>/DOC.md` |
| npm wrapper | `npm/chub/bin/chub.js` (thin platform binary resolver) |

### Key env vars

| Var | Purpose |
|-----|---------|
| `CHUB_DIR` | Override `~/.chub` data directory |
| `CHUB_BUNDLE_URL` | Override the default CDN URL |
| `CHUB_PROJECT_DIR` | Override project root |
| `CHUB_PROFILE` | Override active context profile |
| `CHUB_ANNOTATION_SERVER` | Override org annotation server URL |
| `CHUB_ANNOTATION_TOKEN` | Auth token for org annotation server |
| `CHUB_TELEMETRY` | Set to `0` to disable telemetry |
| `CHUB_FEEDBACK` | Set to `0` to disable feedback |

Config: `~/.chub/config.yaml` (personal) → `.chub/config.yaml` (project, git-tracked) → `.chub/profiles/<name>.yaml` (role).

### Format compatibility

All on-disk formats (`registry.json`, `search-index.json`, annotation JSONs) are byte-for-byte identical with the original JS Context Hub. The `serde(rename)` attributes on `types.rs` structs enforce camelCase field names.

## Integrations

Chub integrates with AI agents via MCP (runtime tools) and agent config generation (static rules). See `docs/integrations.md` for full setup guides.

### MCP tools

Available via `chub mcp` (stdio server): `chub_search`, `chub_get`, `chub_list`, `chub_context`, `chub_pins`, `chub_annotate`, `chub_feedback`.

### Agent config generation

`chub agent-config sync` generates rules files from `.chub/config.yaml`. See `docs/integrations.md` for the full target list and setup guides.

### Skills (slash commands)

| Command | What it does |
|---------|-------------|
| `/docs <query>` | Search or fetch documentation |
| `/annotate <id> <note>` | Record a team annotation |
| `/setup` | Initialize chub for the current project |

### Project context

Pinned docs (`.chub/pins.yaml`): `serde/derive`, `clap/derive`, `tokio/runtime`, `axum/routing`.

Project context docs (`.chub/context/`): `architecture.md`, `conventions.md`, `team-features.md`. Access via `chub get project/<name>` or `chub_context` MCP tool.

## Doc review checklist

When asked to review docs, verify claims against source code. Hard numbers (counts, lists) should live in one canonical doc; other docs should reference it instead of duplicating.

| Fact | Source of truth | Canonical doc |
|---|---|---|
| MCP tool names | `crates/chub-cli/src/mcp/tools.rs` | `docs/integrations.md` |
| CLI commands | `crates/chub-cli/src/main.rs` (`Commands` enum) | `docs/cli-reference.md` |
| Agent config targets | `crates/chub-core/src/team/agent_config.rs` | `docs/integrations.md` |
| Dep detection file types | `crates/chub-core/src/team/detect.rs` | `docs/cli-reference.md` |
| BM25 params, search fields | `crates/chub-core/src/search/bm25.rs` | `.chub/context/architecture.md` |
| Frontmatter schema | `crates/chub-core/src/frontmatter.rs` | `docs/content-guide.md` |
| Benchmark numbers | `scripts/benchmark.sh` output | `docs/chub-vs-context-hub.md` |
| Version string | `Cargo.toml` workspace version | (use `./scripts/set-version.sh`) |
