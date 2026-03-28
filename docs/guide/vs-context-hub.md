# Chub vs Context Hub

Chub is a Rust rewrite of [Context Hub](https://github.com/andrewyng/context-hub) (JS). Fully format-compatible — same registry, same search index, same config schema. Content works in both directions without modification.

## Performance

Measured on the production corpus (1,553 docs, 8 skills). Median of 5 runs.

| Operation | Context Hub (JS) | Chub (Rust) | Speedup |
|---|---|---|---|
| `search "stripe payments"` | 1,060 ms | **56 ms** | **19x** |
| `build --validate-only` | 1,920 ms | **380 ms** | **5x** |
| `build` (1,560 entries) | 3,460 ms | **1,770 ms** | **2x** |
| `get stripe/api` | 148 ms | **63 ms** | **2.3x** |
| Cold start (`--help`) | 131 ms | **44 ms** | **3x** |

| Metric | Context Hub (JS) | Chub (Rust) |
|---|---|---|
| Package size | ~22 MB (`node_modules`) | **10 MB** (single binary) |
| Peak memory (build) | ~122 MB | **~23 MB** (5.3x less) |
| Runtime dependency | Node.js 20+ | **None** |

Search is the largest improvement (19x) because Chub uses a BM25 inverted index that only scores documents containing query terms, while Context Hub does a linear scan over all entries.

## Features only in Chub

| Feature | Command | Description |
|---|---|---|
| Doc pinning | `chub pin` | Lock doc versions for team consistency |
| Context profiles | `chub profile` | Role-scoped context with inheritance |
| Team annotations | `chub annotate --team` | Git-tracked annotations in `.chub/annotations/` |
| Org annotations | `chub annotate --org` | Server-hosted annotations with local cache |
| Structured kinds | `--kind issue/fix/practice` | Categorize what agents learn |
| Project context | `chub context` | Custom markdown docs via MCP |
| Dep auto-detection | `chub detect` | Scan all major package managers |
| Agent config sync | `chub agent-config` | Generate rules for 10 agent targets |
| Doc snapshots | `chub snapshot` | Point-in-time pin captures |
| Freshness checks | `chub check` | Compare pinned vs installed versions |
| Usage analytics | `chub stats` | Local opt-in fetch tracking |
| HTTP serving | `chub serve` | Serve a content directory |
| Doc bundles | `chub bundle` | Shareable doc collections |
| AI usage tracking | `chub track` | Sessions, tokens, costs, dashboard |
| Usage telemetry | `chub telemetry` | View and manage local telemetry data |

## MCP tools

| Tool | Context Hub | Chub |
|---|---|---|
| `chub_search` | Yes | Yes |
| `chub_get` | Yes | Yes |
| `chub_list` | Yes | Yes |
| `chub_annotate` | Yes | Yes |
| `chub_feedback` | Yes | Yes |
| `chub_context` | — | **Yes** (pins, annotations, profiles, project context) |
| `chub_pins` | — | **Yes** (manage pinned docs) |
| `chub_track` | — | **Yes** (query AI usage tracking data) |

**Total: 5 tools (JS) vs 8 tools (Rust)**

## CLI commands

**7 commands (JS) vs 22 commands (Rust)**

Chub adds: `init`, `pin`, `profile`, `detect`, `agent-config`, `check`, `context`, `stats`, `serve`, `bundle`, `snapshot`, `mcp`, `list`, `track`, `telemetry`.

## Format Compatibility

Chub produces byte-compatible output with Context Hub:

- `registry.json` — identical fields and structure, all camelCase
- `search-index.json` — identical BM25 parameters and inverted index format
- `~/.chub/config.yaml` — same config schema
- Annotation files — same JSON format (Chub adds optional `kind`/`severity` fields, ignored by JS)

Content authored for Context Hub works in Chub without changes and vice versa.

## Search algorithm

Both use BM25 scoring (k1=1.5, b=0.75) with lexical boost (Levenshtein distance, prefix/contains matching).

| Aspect | Context Hub | Chub |
|---|---|---|
| Scoring | BM25 | BM25 (identical parameters) |
| Index type | Linear scan | **Inverted index** |
| Tokenizer | 56 stop words | 56 stop words (identical) |
| Lexical boost | Levenshtein + prefix/contains | Levenshtein + prefix/contains (identical) |

The inverted index is the key performance difference — Chub only scores documents that contain at least one query term.
