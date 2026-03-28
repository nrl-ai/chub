---
name: Architecture
description: "Crate layout, data flow, and key design decisions"
tags: architecture, design, crates
---

# Chub Architecture

## Crate Layout

```
chub-core   — library: all business logic, no CLI concerns
chub-cli    — binary: CLI commands, MCP server, output formatting
```

`chub-cli` depends on `chub-core`; nothing else crosses crate boundaries.

## Data Flow

### `chub get` / `chub search`

```
Config (~/.chub/config.yaml + .chub/config.yaml + env vars)
  └─ sources: Vec<SourceConfig>

fetch::ensure_registry()          — fetches registry.json + search-index.json if stale
registry::load_merged()           — loads all sources, merges into MergedRegistry
registry::search_entries()        — BM25 via inverted index + lexical boost
registry::get_entry()             — exact lookup with source:id disambiguation
registry::resolve_doc_path()      — picks language/version, returns CDN path
fetch::fetch_doc()                — cache → CDN fallback
```

### Team Features (`.chub/` directory)

```
.chub/config.yaml       → project-level config (overrides ~/.chub/)
.chub/pins.yaml         → locked doc versions for the team
.chub/annotations/      → git-tracked team annotations
.chub/context/          → custom project docs (served via MCP + CLI)
.chub/profiles/         → role-scoped context profiles with inheritance
.chub/snapshots/        → point-in-time pin snapshots
.chub/bundles/          → shareable doc collections
```

Three-tier config inheritance: `~/.chub/` → `.chub/` → `.chub/profiles/<name>.yaml`

## Search Pipeline

`search/tokenizer.rs` — shared tokenizer (56 stop words, punctuation stripping, `compact_identifier` strips all non-alphanumeric for fuzzy matching).

`search/bm25.rs` — BM25 scoring (k1=1.5, b=0.75). Fields: `id`, `name`, `description`, `tags`.

`search/index.rs` — inverted index built at load time from `search-index.json`. Only docs containing ≥1 query term are scored.

`registry::search_entries()` layers BM25 with a lexical boost pass (Levenshtein distance, prefix/contains matching on compact identifiers). The lexical scan is global only when BM25 returns 0 results or a multi-word query has terms missing from the index.

## MCP Server

`chub mcp` runs an stdio server via the `rmcp` crate. It has its own `mcp::server::run_mcp_server()` entry point and loads the registry independently (not through the CLI flow).

Tools: `chub_search`, `chub_get`, `chub_list`, `chub_context`, `chub_pins`, `chub_annotate`, `chub_feedback`. Tool parameter structs use `schemars::JsonSchema` for schema generation. Registry exposed as a resource at `chub://registry`.

## Secret Scanning

`scan/` module — gitleaks/betterleaks-compatible secret scanner.

```
scan/config.rs    — ScanConfig: .gitleaks.toml compatible config loading (TOML/YAML)
scan/finding.rs   — Finding struct with gitleaks-compatible PascalCase JSON
scan/report.rs    — JSON, SARIF 2.1.0, CSV output
scan/scanner.rs   — Scanner: orchestrates scanning of git repos, directories, stdin
```

The scanner reuses the `team/tracking/redact.rs` rule engine (73+ rules with Shannon entropy, stopwords, base64 decoding). `Scanner::scan_text()` wraps `Redactor::scan_text()` to produce `Finding` structs with line/column locations and fingerprints.

Scan modes:
- **Git** — parses `git log -p` output or `git diff --cached` (staged mode)
- **Dir** — walks directory, skips binary/large/.git files
- **Reader** — any `Read` source (stdin)

## Key Design Decisions

1. **Format compatibility**: All on-disk formats are byte-for-byte identical with JS Context Hub (`serde(rename)` enforces camelCase)
2. **Search**: BM25 scoring + inverted index (not linear scan) for O(1) term lookup
3. **MCP server**: Separate entry point via `rmcp` crate, not through CLI flow
4. **Team features**: All git-tracked in `.chub/`, graceful degradation when absent
5. **Incremental build**: SHA-256 manifest skips unchanged files during `chub build`
6. **Shared rule engine**: Scanner and transcript redactor share the same 73+ detection rules in `redact.rs`
