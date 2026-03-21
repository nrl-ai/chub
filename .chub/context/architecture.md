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

## Key Design Decisions

1. **Format compatibility**: All on-disk formats are byte-for-byte identical with JS Context Hub
2. **Search**: BM25 scoring + inverted index (not linear scan) for O(1) term lookup
3. **MCP server**: Separate entry point via `rmcp` crate, not through CLI flow
4. **Team features**: All git-tracked in `.chub/`, graceful degradation when absent
