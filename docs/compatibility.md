# API and Algorithm Compatibility

Deep comparison of Chub (Rust) and Context Hub (JS) implementations. Covers algorithm parity, serialization format compatibility, interface contracts, and known divergences.

For performance benchmarks and feature-level comparison, see [chub-vs-context-hub.md](chub-vs-context-hub.md).

## Search Algorithm

### BM25 scoring

Both implementations use identical BM25 parameters and formulas.

| Parameter | Context Hub (JS) | Chub (Rust) | Match |
|---|---|---|---|
| k1 (term saturation) | 1.5 | 1.5 | Exact |
| b (length normalization) | 0.75 | 0.75 | Exact |
| IDF formula | `Math.log((N-df+0.5)/(df+0.5)+1)` | `((N-df+0.5)/(df+0.5)+1.0).ln()` | Exact |
| Score threshold | `> 0` | `> 0.0` | Exact |

Per-field BM25 score:

```
score = IDF(term) × (TF × (k1 + 1)) / (TF + k1 × (1 - b + b × (DL / avgFL)))
```

Total score is a weighted sum across four fields:

| Field | Weight | Rationale |
|---|---|---|
| `id` | 4.0 | Package identifier — strongest signal |
| `name` | 3.0 | Short human name, high specificity |
| `tags` | 2.0 | Curated keywords |
| `description` | 1.0 | Longer text, more noise |

**Source of truth**: `crates/chub-core/src/search/bm25.rs` (Rust), `references/context-hub/cli/src/lib/bm25.js` (JS).

### Tokenization

Both implementations use the same tokenization pipeline:

**Free text** (`tokenize`):
1. Lowercase
2. Replace non-`[a-z0-9\s-]` with space
3. Split on whitespace and hyphens
4. Filter: remove stop words and single-character tokens (digits exempt)

**Identifiers** (`tokenizeIdentifier`):
1. All `tokenize` results
2. Split by `/`, `_`, `.`, `-`, spaces → compact each segment
3. Full compact form (strip all non-alphanumeric)
4. Alpha-numeric boundary splits (`"auth0"` → `"auth"`, `"0"`)
5. Deduplicate

Example: `node-fetch` → `{node, fetch, nodefetch}`

**Source of truth**: `crates/chub-core/src/search/tokenizer.rs` (Rust), `bm25.js` lines 37–96 (JS).

### Stop words

Both use the same 56-word list:

```
a, about, an, and, are, as, at, be, but, by, can, do, does, for, from, has,
have, how, if, in, into, is, it, its, just, may, no, not, of, on, or, over,
so, such, than, that, the, them, then, these, this, those, through, to, too,
under, use, used, using, very, was, were, will, with, you, your
```

**Design note**: BM25's IDF already downweights common terms mathematically. The stop list is redundant for scoring but reduces index size. Some words on the list (`use`, `no`, `how`, `do`) carry meaning in technical documentation queries. A future revision may shrink the list to ~24 pure function words and let IDF handle the rest. Any change requires a coordinated index format bump since tokenization affects `search-index.json` content.

### Inverted index

| Aspect | Context Hub (JS) | Chub (Rust) |
|---|---|---|
| Stored in `search-index.json` | Yes (built at build time) | Yes (built at build time) |
| Used at search time | Yes (candidate pruning) | Yes (candidate pruning) |
| Fallback without index | Linear scan of all docs | Linear scan of all docs |

Both build the inverted index identically: for each document, collect unique terms across all four fields, then map `term → [docIndex, ...]`.

### Lexical boost (fuzzy fallback)

When BM25 alone doesn't cover all query terms, both implementations apply an identical lexical boost using compact identifier matching. The boost is **additive** to BM25 scores.

**Trigger conditions** (identical in both):
1. Query has ≥2 tokens after tokenization
2. At least one query term is missing from the inverted index

**Scoring weights** (identical in both):

| Match type | Name score | ID score | ID segment | Name segment |
|---|---|---|---|---|
| Exact | 620 | 600 | 580 | 560 |
| Prefix | 560 | 540 | 530 | 520 |
| Contains | 520 | 500 | 490 | 480 |
| Fuzzy (Levenshtein) | 500 | 470 | 460 | 450 |

**Levenshtein thresholds** (identical in both):

| Query length | Max edit distance |
|---|---|
| < 5 chars | No fuzzy matching |
| 5–8 chars | 1 |
| > 8 chars | 2–3 |

Fuzzy penalty: 20 points per edit distance unit.

**Positional bonuses** (identical in both):
- First segment of ID: +10
- Last segment of ID: +10
- Query matches first segment: +60
- Query matches last segment: +25
- Query matches both first and last (multi-segment): +40

**Source of truth**: `crates/chub-core/src/registry.rs` (Rust), `references/context-hub/cli/src/lib/registry.js` (JS).

## Serialization Formats

### registry.json

Top-level structure:

```json
{
  "version": "1.0.0",
  "generated": "2026-01-15T00:00:00.000Z",
  "base_url": "https://cdn.aichub.org/v1",
  "docs": [...],
  "skills": [...]
}
```

All field names match exactly between JS and Rust output. Fields requiring camelCase use `#[serde(rename)]` in Rust.

**Doc entry fields**:

| JSON field | JS type | Rust type | Rename needed |
|---|---|---|---|
| `id` | string | `String` | No |
| `name` | string | `String` | No |
| `description` | string | `String` | No |
| `source` | string | `String` | No |
| `tags` | string[] | `Vec<String>` | No |
| `languages` | array | `Vec<LanguageEntry>` | No |
| `languages[].language` | string | `String` | No |
| `languages[].recommendedVersion` | string | `String` | `#[serde(rename)]` |
| `languages[].versions` | array | `Vec<VersionEntry>` | No |
| `languages[].versions[].version` | string | `String` | No |
| `languages[].versions[].path` | string | `String` | No |
| `languages[].versions[].files` | string[] | `Vec<String>` | No |
| `languages[].versions[].size` | number | `u64` | No |
| `languages[].versions[].lastUpdated` | string | `String` | `#[serde(rename)]` |
| `languages[].versions[].contentHash` | string? | `Option<String>` | `#[serde(rename)]` |

**Skill entry fields**:

| JSON field | JS type | Rust type | Rename needed |
|---|---|---|---|
| `id` | string | `String` | No |
| `name` | string | `String` | No |
| `description` | string | `String` | No |
| `source` | string | `String` | No |
| `tags` | string[] | `Vec<String>` | No |
| `path` | string | `String` | No |
| `files` | string[] | `Vec<String>` | No |
| `size` | number | `u64` | No |
| `lastUpdated` | string | `String` | `#[serde(rename)]` |
| `contentHash` | string? | `Option<String>` | `#[serde(rename)]` |

**Note**: `base_url` uses snake_case in **both** implementations (JS writes `registry.base_url = opts.baseUrl`). No rename needed.

**Source of truth**: `crates/chub-core/src/types.rs` (Rust structs), `references/context-hub/cli/src/commands/build.js` (JS output).

### search-index.json

```json
{
  "version": "1.0.0",
  "algorithm": "bm25",
  "params": { "k1": 1.5, "b": 0.75 },
  "totalDocs": 42,
  "avgFieldLengths": { "id": 1.2, "name": 1.5, "description": 8.3, "tags": 2.1 },
  "idf": { "payment": 3.456, "stripe": 4.789 },
  "documents": [
    { "id": "stripe/api", "tokens": { "id": [...], "name": [...], "description": [...], "tags": [...] } }
  ],
  "invertedIndex": { "payment": [0, 5], "stripe": [0] }
}
```

| JSON field | Rust field | Rename |
|---|---|---|
| `totalDocs` | `total_docs` | `#[serde(rename)]` |
| `avgFieldLengths` | `avg_field_lengths` | `#[serde(rename)]` |
| `invertedIndex` | `inverted_index` | `#[serde(rename)]` |

All other fields are naturally camelCase or single-word. Roundtrip serialization is verified by `search_index_json_roundtrip` test in `crates/chub-core/tests/search_parity.rs`.

### Frontmatter (DOC.md / SKILL.md)

Both parse YAML between `---` delimiters. Rust handles a strict superset of inputs.

| Capability | Context Hub (JS) | Chub (Rust) |
|---|---|---|
| `---\n...\n---\n` delimiters | Yes | Yes |
| CRLF line endings | Via regex `\r?\n` | Explicit normalization |
| UTF-8 BOM stripping | No | Yes |
| Numeric value coercion to string | Via JS loose typing | Explicit `Number → String` |
| Boolean value coercion to string | Via JS loose typing | Explicit `Bool → String` |
| Trailing `---` without final newline | No (regex requires `\n?` after) | Yes |

**Parsed fields** (identical schema):

```yaml
---
name: string                    # display name (required for build)
description: string             # searchable summary
metadata:
  languages: string             # comma-separated
  versions: string              # comma-separated semver
  revision: number              # monotonically increasing
  source: string                # official | maintainer | community
  tags: string                  # comma-separated
  updated-on: string            # YYYY-MM-DD
---
```

**Source of truth**: `crates/chub-core/src/frontmatter.rs` (Rust), `references/context-hub/cli/src/lib/frontmatter.js` (JS).

## MCP Interface

### Tools present in both implementations

All five original MCP tools are fully compatible. Parameter names, types, and required/optional status match exactly.

#### chub_search

| Parameter | Type | Required | Both |
|---|---|---|---|
| `query` | string | No (omit to list all) | Yes |
| `tags` | string | No (comma-separated) | Yes |
| `lang` | string | No | Yes |
| `limit` | number | No (default: 20) | Yes |

#### chub_get

| Parameter | Type | Required | Both | Rust-only |
|---|---|---|---|---|
| `id` | string | Yes | Yes | |
| `lang` | string | No | Yes | |
| `version` | string | No | Yes | |
| `full` | boolean | No | Yes | |
| `file` | string | No | Yes | |
| `match_env` | boolean | No | | Yes |

#### chub_list

| Parameter | Type | Required | Both |
|---|---|---|---|
| `tags` | string | No | Yes |
| `lang` | string | No | Yes |
| `limit` | number | No (default: 50) | Yes |

#### chub_annotate

| Parameter | Type | Required | Both | Rust-only |
|---|---|---|---|---|
| `id` | string | No (required except list mode) | Yes | |
| `note` | string | No | Yes | |
| `clear` | boolean | No | Yes | |
| `list` | boolean | No | Yes | |
| `kind` | string | No | | Yes (`note`, `issue`, `fix`, `practice`) |
| `severity` | string | No | | Yes (`high`, `medium`, `low`) |
| `scope` | string | No | | Yes (`auto`, `personal`, `team`, `org`) |

#### chub_feedback

| Parameter | Type | Required | Both |
|---|---|---|---|
| `id` | string | Yes | Yes |
| `rating` | string | Yes (`up` / `down`) | Yes |
| `comment` | string | No | Yes |
| `type` | string | No (`doc` / `skill`) | Yes |
| `lang` | string | No | Yes |
| `version` | string | No | Yes |
| `file` | string | No | Yes |
| `labels` | array | No | Yes |

### Tools only in Chub

| Tool | Purpose |
|---|---|
| `chub_context` | Returns pinned docs, profile rules, project context, and annotations scoped to a task |
| `chub_pins` | Manage pinned doc versions (add, remove, list) |

These are additive — they don't conflict with existing tools or change their behavior.

### MCP output format

Both wrap results in MCP-compatible format:

```json
{ "content": [{ "type": "text", "text": "..." }], "isError": false }
```

**Source of truth**: `crates/chub-cli/src/mcp/tools.rs` (Rust), `references/context-hub/cli/src/mcp/tools.js` (JS).

## CLI Interface

### Shared commands (identical flags)

| Command | Key flags |
|---|---|
| `search [query]` | `--tags`, `--lang`, `--limit`, `--json` |
| `get <ids...>` | `--lang`, `--version`, `--full`, `--file`, `-o`, `--json` |
| `build <dir>` | `-o`, `--base-url`, `--validate-only`, `--json` |
| `update` | `--force`, `--full`, `--json` |
| `cache status\|clear` | `--json` |
| `annotate [id] [note]` | `--clear`, `--list`, `--json` |
| `feedback [id] [rating] [comment]` | `--type`, `--lang`, `--label`, `--agent`, `--model`, `--json` |

### Commands only in Chub

`init`, `pin`, `profile`, `detect`, `agent-config`, `check`, `context`, `stats`, `serve`, `bundle`, `snapshot`, `mcp`, `list`.

See [cli-reference.md](cli-reference.md) for full documentation.

## Known Divergences

These are intentional differences that do not break compatibility.

### 1. Inverted index is optional in Rust deserialization

Rust declares `inverted_index: Option<HashMap<...>>` with `skip_serializing_if = "Option::is_none"`. JS always includes it. When Rust reads a JS-built index, the field is present and used. When the field is absent (e.g., a hand-crafted index), Rust falls back to linear scan. No behavioral difference for standard builds.

### 2. Rust frontmatter handles more edge cases

BOM stripping, explicit CRLF normalization, and trailing `---` without final newline. JS does not handle these cases. Content authored on Windows or with BOM-inserting editors works in Rust but may fail in JS.

### 3. Rust build produces a `.build-manifest.json`

Used for incremental builds (skip unchanged files). JS does not produce this file. It is ignored by both implementations when reading a content directory.

### 4. Annotation schema extensions

Rust annotations support `kind` (note/issue/fix/practice), `severity`, and `scope` fields not present in JS. These fields are stored in JSON but ignored by the JS reader (which only looks for `id`, `note`, `updatedAt`). Forward-compatible: JS silently ignores unknown fields.

### 5. Content hash verification

Rust computes SHA-256 over entry-point files at build time and stores it as `contentHash`. At fetch time, Rust verifies the hash (warning only, content still returned). JS computes the hash at build time but does not verify at fetch time.

## Compatibility Verification

### Automated tests

| Test | Location | What it verifies |
|---|---|---|
| Tokenizer parity | `crates/chub-core/tests/search_parity.rs` | `tokenize` output matches JS for shared test vectors |
| BM25 index structure | Same file | Index version, params, field lengths, IDF values |
| Search result ranking | Same file | Same top results for identical queries on identical corpus |
| Inverted index parity | Same file | Inverted index scores match linear scan within 1e-6 |
| JSON roundtrip | Same file | Serialize → deserialize → search produces identical results |
| Team features | `crates/chub-core/tests/team_features.rs` | Pins, profiles, annotations, agent config |

### Manual verification

```sh
# Build with both implementations, diff the output
cd references/context-hub/cli && node src/index.js build ../../content -o /tmp/js-out
cargo run --release -- build content -o /tmp/rs-out

# Compare registry structure (field names, types)
diff <(jq 'keys' /tmp/js-out/registry.json) <(jq 'keys' /tmp/rs-out/registry.json)

# Compare search index structure
diff <(jq 'keys' /tmp/js-out/search-index.json) <(jq 'keys' /tmp/rs-out/search-index.json)

# Compare search results
node references/context-hub/cli/src/index.js search "stripe" --json > /tmp/js-search.json
cargo run --release -- search "stripe" --json > /tmp/rs-search.json
diff <(jq '.[].id' /tmp/js-search.json) <(jq '.[].id' /tmp/rs-search.json)
```
