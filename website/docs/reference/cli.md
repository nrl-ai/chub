# CLI Commands

All commands support `--json` for machine-readable output and `--help` for usage details.

## Core Commands

### chub search

Search for docs and skills in the registry using BM25 ranking.

```sh
chub search [<query>] [--tags <tags>] [--lang <lang>] [--type <type>] [--limit <n>] [--json]
```

| Flag | Description | Default |
|---|---|---|
| `<query>` | Search query (supports multi-word). Omit to list all. | — |
| `--tags <tags>` | Comma-separated tag filter (e.g. `openai,chat`) | all |
| `--lang <lang>` | Filter by language (e.g. `python`, `js`) | all |
| `--type <type>` | Filter by type: `doc` or `skill` | all |
| `--limit <n>` | Maximum number of results | 20 |
| `--json` | Output results as JSON | off |

**Examples:**

```sh
chub search "stripe payments"
chub search "react hooks" --limit 5
chub search --tags openai,chat --lang python --json
```

### chub get

Fetch a specific doc or skill by ID.

```sh
chub get <id>... [--lang <lang>] [--version <ver>] [--pinned] [--match-env] [-o <file>] [--json]
```

| Flag | Description |
|---|---|
| `<id>` | Doc ID (e.g., `openai/chat`). Multiple IDs allowed. |
| `--lang <lang>` | Language variant (e.g., `python`, `js`) |
| `--version <ver>` | Specific version (e.g., `4.0`) |
| `--pinned` | Fetch all pinned docs at once |
| `--match-env` | Auto-detect version from `package.json`, `Cargo.toml`, etc. |
| `--full` | Fetch all files in the entry, not just the entry point |
| `--file <file>` | Fetch a specific sub-file (comma-separated) |
| `-o <path>` | Write output to a file or directory |
| `--json` | JSON output |

**Examples:**

```sh
chub get openai/chat --lang python
chub get stripe/api --lang javascript --version 2024
chub get --pinned                            # fetch all pinned docs
chub get project/architecture                # project context doc
chub get openai/chat --match-env             # version matched from project deps
```

The fetched doc automatically includes:
- Merged annotations from all three tiers (org → team → personal) appended at the end
- A pin notice if the doc is pinned by the team

### chub list

List all available docs in the registry.

```sh
chub list [<query>] [--tags <tags>] [--lang <lang>] [--type <type>] [--limit <n>] [--json]
```

| Flag | Description | Default |
|---|---|---|
| `<query>` | Optional search query to filter results | — |
| `--tags <tags>` | Comma-separated tag filter | all |
| `--lang <lang>` | Filter by language | all |
| `--type <type>` | Filter by type: `doc` or `skill` | all |
| `--limit <n>` | Max results | 20 |
| `--json` | JSON output | off |

### chub update

Refresh the cached registry and search index.

```sh
chub update [--force] [--full]
```

| Flag | Description |
|---|---|
| `--force` | Re-download even if cache is fresh |
| `--full` | Download the full bundle for offline use |

### chub cache

Manage the local cache.

```sh
chub cache status    # show cache state and size
chub cache clear     # clear all cached data
```

## Team Commands

### chub init

Initialize a `.chub/` project directory for team sharing.

```sh
chub init [--from-deps] [--monorepo]
```

| Flag | Description |
|---|---|
| `--from-deps` | Scan dependency files and auto-pin matching docs |
| `--monorepo` | Create config with auto-profile rules for monorepo layout |

Creates `.chub/` with `config.yaml`, `pins.yaml`, `annotations/`, `context/`, and `profiles/`.

### chub pin

Manage pinned doc versions.

```sh
chub pin add <id> [--lang <lang>] [--version <ver>] [--reason <text>] [--source <name>]
chub pin remove <id>
chub pin list
chub pin get [--lang <lang>]
```

| Subcommand | Description |
|---|---|
| `add <id>` | Pin a doc to the project |
| `remove <id>` | Remove a pinned doc |
| `list` | List all active pins |
| `get` | Fetch all pinned docs at once |

**Options for `pin add`:**

| Flag | Description |
|---|---|
| `--lang <lang>` | Pin to a specific language |
| `--version <ver>` | Pin to a specific version |
| `--reason <text>` | Human-readable reason (shown to agents) |
| `--source <name>` | Source name for private registry |

**Examples:**

```sh
chub pin add openai/chat --lang python --version 4.0 --reason "Use v4 streaming API"
chub pin add stripe/api --lang javascript
chub pin remove openai/chat
chub pin list                                # list all active pins
chub pin get                                 # fetch all pinned docs at once
```

### chub profile

Manage context profiles for role-scoped context.

```sh
chub profile use <name>    # activate a profile ("none" to clear)
chub profile list          # list available profiles
chub profile current       # show the currently active profile
```

### chub annotate

Read, write, clear, or list annotations for a doc entry.

```sh
chub annotate [<id>] [<note>] [OPTIONS]
```

**Modes:**

| Invocation | Effect |
|---|---|
| `chub annotate <id>` | Read all annotations (org + team + personal merged) |
| `chub annotate <id> "<note>"` | Write personal annotation (overwrites previous) |
| `chub annotate <id> "<note>" --team` | Append team annotation (add to history) |
| `chub annotate <id> "<note>" --org` | Append to org annotation server (Tier 3) |
| `chub annotate <id> --personal` | Read personal tier only |
| `chub annotate <id> --team` | Read team tier only |
| `chub annotate <id> --org` | Read org tier only |
| `chub annotate <id> --clear` | Remove personal annotation |
| `chub annotate <id> --clear --team` | Remove team annotation file |
| `chub annotate <id> --clear --org` | Remove org annotation from server |
| `chub annotate --list` | List all personal annotations |
| `chub annotate --list --team` | List all team annotations |
| `chub annotate --list --org` | List all org annotations |

**Options:**

| Flag | Description | Default |
|---|---|---|
| `--kind <KIND>` | `note`, `issue`, `fix`, `practice` | `note` |
| `--severity <LEVEL>` | `high`, `medium`, `low` (issue kind only) | none |
| `--personal` | Target personal tier (`~/.chub/annotations/`, overwrite) | default |
| `--team` | Target team tier (`.chub/annotations/`, git-tracked, append) | off |
| `--org` | Target org server (requires `annotation_server` in config) | off |
| `--author <name>` | Author name for team/org annotations | `$USER` |
| `--clear` | Remove the annotation | |
| `--list` | List all annotations | |

**Examples:**

```sh
# Write a team note (default kind)
chub annotate openai/chat "Use v4 streaming, not completions" --team

# Record a confirmed bug with severity
chub annotate openai/chat "tool_choice='none' silently ignores tools and returns null" \
  --kind issue --severity high --team

# Record the fix for the bug above
chub annotate openai/chat "Use tool_choice='auto' or remove tools from the array" \
  --kind fix --team

# Record a validated team pattern
chub annotate openai/chat "Always set max_tokens to avoid unbounded streaming cost" \
  --kind practice --team

# Org-level annotation (requires annotation_server in .chub/config.yaml)
chub annotate openai/chat "Always set max_tokens explicitly" \
  --kind practice --org

# Read all annotations for an entry (org + team + personal merged)
chub annotate openai/chat

# List all team annotations
chub annotate --list --team
```

::: tip Storage tiers
- **Personal** (`--personal`, default): overwrite semantics — one note per entry, stored in `~/.chub/annotations/`, local only.
- **Team** (`--team`): append semantics — adds to history with author + date, stored in `.chub/annotations/`, git-tracked.
- **Org** (`--org`): append semantics — sent to the org annotation server, requires `annotation_server.url` in `.chub/config.yaml`.

Reading without a tier flag shows all three tiers merged.
:::

### chub feedback

Submit feedback (thumbs up/down) about a doc.

```sh
chub feedback <id> <rating> [<comment>] [OPTIONS]
```

| Flag | Description |
|---|---|
| `<rating>` | `up` or `down` |
| `<comment>` | Optional explanation (positional) |
| `--lang <lang>` | Language variant rated |
| `--doc-version <ver>` | Version rated |
| `--file <file>` | Specific file within the entry |
| `--label <label>` | Structured label, repeatable (e.g. `outdated`, `missing-example`) |
| `--agent <name>` | AI coding tool name |
| `--model <model>` | LLM model name |
| `--entry-type <type>` | Explicit type: `doc` or `skill` |
| `--status` | Show feedback and telemetry status |

### chub detect

Scan dependency files and find matching docs.

```sh
chub detect [--pin] [--diff]
```

| Flag | Description |
|---|---|
| `--pin` | Auto-pin all detected matches |
| `--diff` | Show new deps since last detect |

Supported dependency files: `package.json`, `Cargo.toml`, `requirements.txt`, `pyproject.toml`, `Pipfile`, `go.mod`, `Gemfile`, `pom.xml`, `build.gradle(.kts)`.

### chub check

Check pinned doc versions against installed library versions.

```sh
chub check [--fix]
```

| Flag | Description |
|---|---|
| `--fix` | Auto-update outdated pins to match installed versions |

### chub context

Browse and query project context docs.

```sh
chub context [<query>] [--list]
```

| Flag | Description |
|---|---|
| `<query>` | Task description to find relevant context for |
| `--list` | List all project context docs |

### chub agent-config

Generate and sync agent configuration files from `.chub/config.yaml`.

```sh
chub agent-config generate   # generate all target files
chub agent-config sync       # update only if source changed
chub agent-config diff       # show what would change
```

Supported targets: `CLAUDE.md`, `.cursorrules`, `.windsurfrules`, `AGENTS.md`, `.github/copilot-instructions.md`, `GEMINI.md`, `.clinerules`, `.roo/rules/chub-rules.md`, `.augment/rules/chub-rules.md`, `.kiro/steering/chub-rules.md`.

### chub snapshot

Manage point-in-time pin snapshots.

```sh
chub snapshot create <name>          # save current pins
chub snapshot list                   # list all snapshots
chub snapshot restore <name>         # restore pin state
chub snapshot diff <name-a> <name-b> # compare two snapshots
```

### chub bundle

Manage shareable doc collections.

```sh
chub bundle create <name> --entries <ids> [--description <text>] [--author <name>] [--notes <text>]
chub bundle install <name>           # pin all entries in the bundle
chub bundle list                     # list available bundles
```

| Flag | Description |
|---|---|
| `--entries <ids>` | Comma-separated entry IDs (required for `create`) |
| `--description <text>` | Bundle description |
| `--author <name>` | Author name |
| `--notes <text>` | Additional notes |

### chub stats

Show local usage analytics (opt-in).

```sh
chub stats [--days <n>] [--json]
```

| Flag | Description | Default |
|---|---|---|
| `--days <n>` | Number of days to include | 30 |

## Server Commands

### chub mcp

Start the MCP (Model Context Protocol) stdio server.

```sh
chub mcp
```

### chub serve

Serve a built content directory as an HTTP registry.

```sh
chub serve <content-dir> [-p <port>] [-o <output-dir>]
```

| Flag | Description | Default |
|---|---|---|
| `-p, --port <n>` | HTTP port | 4242 |
| `-o, --output <dir>` | Output directory for built content | temp dir |

## Build Commands

### chub build

Build a content directory into `registry.json` and `search-index.json`.

```sh
chub build <content-dir> [-o <output>] [--base-url <url>] [--validate-only] [--no-incremental]
```

| Flag | Description | Default |
|---|---|---|
| `-o, --output <dir>` | Output directory | `<content-dir>/dist` |
| `--base-url <url>` | CDN base URL for doc paths | none |
| `--validate-only` | Validate content without building | off |
| `--no-incremental` | Rebuild all files, skip change detection | off |

**Examples:**

```sh
chub build ./content -o ./dist
chub build ./content --validate-only
chub build ./content --base-url https://cdn.example.com/v1
```

## Global Flags

| Flag | Description |
|---|---|
| `--help` | Show help for any command |
| `--version` | Show version |
| `--json` | JSON output (most commands) |
