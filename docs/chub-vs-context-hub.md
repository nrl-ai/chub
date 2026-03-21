# Chub vs Context Hub

Chub is a Rust rewrite of [Context Hub](https://github.com/andrewyng/context-hub) (JS). This document compares the two implementations with benchmarked numbers.

## Performance

All benchmarks measured on the production corpus (1,553 docs, 7 skills) on Windows 11, Node.js v22, Rust release build. Median of 5 runs. Reproduce with `./scripts/benchmark.sh`.

| Operation | Context Hub (JS) | Chub (Rust) | Speedup |
|---|---|---|---|
| `search "stripe payments"` | 1,060 ms | **56 ms** | **19x** |
| `build --validate-only` | 1,920 ms | **380 ms** | **5x** |
| `build` (1,560 entries) | 3,460 ms | **1,770 ms** | **2x** |
| `get stripe/api` | 148 ms | **63 ms** | **2.3x** |
| Cold start (`--help`) | 131 ms | **44 ms** | **3x** |

Search is the largest improvement (19x) because Chub uses a BM25 inverted index that only scores documents containing query terms, while Context Hub does a linear scan over all entries.

## Resource Usage

| Metric | Context Hub (JS) | Chub (Rust) |
|---|---|---|
| Package size | ~22 MB (`node_modules`) | **10 MB** (single binary) |
| Peak memory (build, 1,560 entries) | ~122 MB | **~23 MB** (5.3x less) |
| Runtime dependency | Node.js 20+ | **None** |
| Installation | `npm install -g @aisuite/chub` | `npm install -g @nrl-ai/chub` / `pip install chub` / `cargo install chub` / `brew install nrl-ai/tap/chub` / direct binary download |

Chub ships as a single static binary with no runtime dependencies. Context Hub requires Node.js and downloads ~22 MB of npm packages.

## Feature Comparison

### Core features (parity)

Both implementations support all core functionality:

| Feature | Context Hub | Chub |
|---|---|---|
| `search` (BM25 + lexical boost) | Yes | Yes |
| `get` (with `--lang`, `--version`, `--file`, `--full`) | Yes | Yes |
| `build` (content directory to registry) | Yes | Yes |
| `update` / `cache` management | Yes | Yes |
| `annotate` (persistent agent notes) | Yes | Yes |
| `feedback` (doc/skill ratings) | Yes | Yes |
| MCP server (stdio transport) | Yes | Yes |
| Registry format (`registry.json`, `search-index.json`) | camelCase JSON | **Identical** |
| Multi-source registries | Yes | Yes |
| Telemetry (opt-in) | Yes | Yes |

### Features only in Chub

| Feature | Command | Description |
|---|---|---|
| Doc pinning | `chub pin` | Lock doc versions in `pins.yaml` for team consistency |
| Context profiles | `chub profile` | Role-scoped context (backend, frontend, etc.) with inheritance |
| Team annotations | `chub annotate --team` | Git-tracked annotations in `.chub/annotations/` |
| Project context | `chub context` | Custom markdown docs in `.chub/context/` served via MCP |
| Dep auto-detection | `chub detect` | Scan 9 file types (npm, Cargo, pip, Go, etc.) for matching docs |
| Agent config sync | `chub agent-config` | Generate CLAUDE.md, .cursorrules, AGENTS.md from one source |
| Doc snapshots | `chub snapshot` | Point-in-time pin captures for reproducibility |
| Freshness checks | `chub check` | Compare pinned vs installed versions, auto-update |
| Usage analytics | `chub stats` | Local opt-in fetch tracking |
| Project init | `chub init` | Create `.chub/` directory with sensible defaults |
| HTTP serving | `chub serve` | Serve a content directory as an HTTP registry |
| Doc bundles | `chub bundle` | Shareable doc collections |

### MCP tools

| Tool | Context Hub | Chub |
|---|---|---|
| `chub_search` | Yes | Yes |
| `chub_get` | Yes | Yes |
| `chub_list` | Yes | Yes |
| `chub_annotate` | Yes | Yes |
| `chub_feedback` | Yes | Yes |
| `chub_context` | — | **Yes** (pins, annotations, profiles, project context) |
| `chub_pins` | — | **Yes** (manage pinned docs with locked versions) |

### CLI commands

| Command | Context Hub | Chub |
|---|---|---|
| `search` | Yes | Yes |
| `get` | Yes | Yes |
| `build` | Yes | Yes |
| `update` | Yes | Yes |
| `cache` | Yes | Yes |
| `annotate` | Yes | Yes |
| `feedback` | Yes | Yes |
| `mcp` | — | **Yes** |
| `init` | — | **Yes** |
| `pin` / `unpin` / `pins` | — | **Yes** |
| `profile` | — | **Yes** |
| `detect` | — | **Yes** |
| `agent-config` | — | **Yes** |
| `check` | — | **Yes** |
| `context` | — | **Yes** |
| `stats` | — | **Yes** |
| `serve` | — | **Yes** |
| `bundle` | — | **Yes** |
| `snapshot` | — | **Yes** |

**Total: 7 commands (JS) vs 20 commands (Rust)**

## Format Compatibility

Chub produces byte-compatible output with Context Hub:

- `registry.json` — identical top-level keys (`docs`, `skills`, `generated`, `version`), all fields camelCase
- `search-index.json` — identical structure (`algorithm`, `avgFieldLengths`, `documents`, `idf`, `invertedIndex`, `params`, `totalDocs`, `version`)
- `~/.chub/config.yaml` — same config schema (sources, trust policy, refresh interval, telemetry, feedback)
- Annotation files — same JSON format (`id`, `note`, `updatedAt`)

Content authored for Context Hub works in Chub without changes and vice versa.

## Search Algorithm

Both use BM25 scoring (k1=1.5, b=0.75) with lexical boost (Levenshtein distance, prefix/contains matching).

| Aspect | Context Hub | Chub |
|---|---|---|
| Scoring | BM25 | BM25 (identical parameters) |
| Index type | Linear scan | **Inverted index** (only scores docs with matching terms) |
| Tokenizer | 52 stop words, punctuation stripping | 52 stop words, punctuation stripping (identical) |
| Lexical boost | Levenshtein + prefix/contains | Levenshtein + prefix/contains (identical) |
| Search speed (1,560 entries) | ~1,060 ms | **~56 ms** (19x faster) |

The inverted index is the key performance difference — Chub only scores documents that contain at least one query term, while Context Hub scores every document in the registry.

## Running Benchmarks

```sh
# Prerequisites
cargo build --release
cd references/context-hub/cli && npm install && cd -

# Run all benchmarks (5 iterations, median reported)
./scripts/benchmark.sh

# Custom iteration count
./scripts/benchmark.sh --runs 10

# Skip memory measurement (faster)
./scripts/benchmark.sh --skip-memory
```

The benchmark script measures cold start, build, validate-only, search, get, package size, peak memory, feature counts, and output format compatibility.
