# CLI Commands

All commands support `--json` for machine-readable output and `--help` for usage details.

## Core Commands

### chub search

Search for docs and skills in the registry using BM25 ranking.

```sh
chub search <query> [--limit <n>] [--source <name>] [--json]
```

| Flag | Description | Default |
|---|---|---|
| `<query>` | Search query (supports multi-word) | — |
| `--limit <n>` | Maximum number of results | 10 |
| `--source <name>` | Search within a specific source | all sources |
| `--json` | Output results as JSON | off |

**Examples:**

```sh
chub search "stripe payments"
chub search "react hooks" --limit 5
chub search "auth" --source official --json
```

### chub get

Fetch a specific doc or skill by ID.

```sh
chub get <id> [--lang <language>] [--version <ver>] [--source <name>] [--pinned] [--match-env] [--json]
```

| Flag | Description |
|---|---|
| `<id>` | Doc ID (e.g., `openai/chat`) |
| `--lang <language>` | Language variant (e.g., `python`, `javascript`) |
| `--version <ver>` | Specific version (e.g., `4.0`) |
| `--source <name>` | From a specific source |
| `--pinned` | Fetch all pinned docs at once |
| `--match-env` | Auto-detect version from `package.json`, `Cargo.toml`, etc. |
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
- Team annotations (issues, fixes, practices) appended at the end
- A pin notice if the doc is pinned by the team

### chub list

List all available docs in the registry.

```sh
chub list [--source <name>] [--project] [--json]
```

| Flag | Description |
|---|---|
| `--source <name>` | Filter by source |
| `--project` | List project context docs only |
| `--json` | JSON output |

### chub update

Refresh the cached registry and search index.

```sh
chub update
```

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

### chub pin / unpin / pins

Manage pinned doc versions.

```sh
chub pin <id> [--lang <lang>] [--version <ver>] [--reason <text>]
chub unpin <id>
chub pins
```

| Flag | Description |
|---|---|
| `--lang <lang>` | Pin to a specific language |
| `--version <ver>` | Pin to a specific version |
| `--reason <text>` | Human-readable reason (shown to agents) |

**Examples:**

```sh
chub pin openai/chat --lang python --version 4.0 --reason "Use v4 streaming API"
chub pin stripe/api --lang javascript
chub unpin openai/chat
chub pins                                # list all active pins
```

### chub profile

Manage context profiles for role-scoped context.

```sh
chub profile use <name>    # activate a profile ("none" to clear)
chub profile list          # list available profiles
```

### chub annotate

Read, write, clear, or list annotations for a doc entry.

```sh
chub annotate [<id>] [<note>] [OPTIONS]
```

**Modes:**

| Invocation | Effect |
|---|---|
| `chub annotate <id>` | Read existing annotation(s) for entry |
| `chub annotate <id> "<note>"` | Write personal annotation (overwrites previous) |
| `chub annotate <id> "<note>" --team` | Append team annotation (add to history) |
| `chub annotate <id> --clear` | Remove personal annotation |
| `chub annotate <id> --clear --team` | Remove team annotation |
| `chub annotate --list` | List all personal annotations |
| `chub annotate --list --team` | List all team annotations |

**Options:**

| Flag | Description | Default |
|---|---|---|
| `--kind <KIND>` | `note`, `issue`, `fix`, `practice` | `note` |
| `--severity <LEVEL>` | `high`, `medium`, `low` (issue kind only) | none |
| `--team` | Write to `.chub/annotations/` (git-tracked, append semantics) | off |
| `--personal` | Write to `~/.chub/annotations/` (local, overwrite semantics) | default |
| `--author <name>` | Author name for team annotations | `$USER` |
| `--clear` | Remove the annotation |  |
| `--list` | List all annotations |  |

**Examples:**

```sh
# Write a team note (default kind)
chub annotate openai/chat "Use v4 streaming, not completions" --team

# Record a confirmed bug
chub annotate openai/chat "tool_choice='none' silently ignores tools and returns null" \
  --kind issue --severity high --team

# Record the fix for the bug above
chub annotate openai/chat "Use tool_choice='auto' or remove tools from the array" \
  --kind fix --team

# Record a validated team pattern
chub annotate openai/chat "Always set max_tokens to avoid unbounded streaming cost" \
  --kind practice --team

# Read current annotations for an entry
chub annotate openai/chat --team

# List all team annotations
chub annotate --list --team
```

::: tip Personal vs team semantics
Personal annotations use **overwrite** semantics — each write replaces the previous note for that entry. Team annotations use **append** semantics — each write adds a new entry to the history, preserving author and date. Team annotations live in `.chub/annotations/` and are committed to git.
:::

### chub feedback

Submit feedback (thumbs up/down) about a doc.

```sh
chub feedback <id> <rating> [--comment <text>]
```

| Flag | Description |
|---|---|
| `<rating>` | `up` or `down` |
| `--comment <text>` | Optional explanation |

### chub detect

Scan dependency files and find matching docs.

```sh
chub detect [--pin]
```

| Flag | Description |
|---|---|
| `--pin` | Auto-pin all detected matches |

Supported dependency files: `package.json`, `Cargo.toml`, `requirements.txt`, `pyproject.toml`, `Pipfile`, `go.mod`, `Gemfile`, `pom.xml`, `build.gradle(.kts)`.

### chub check

Check pinned doc versions against installed library versions.

```sh
chub check [--fix]
```

| Flag | Description |
|---|---|
| `--fix` | Auto-update outdated pins to match installed versions |

### chub agent-config

Generate and sync agent configuration files from `.chub/config.yaml`.

```sh
chub agent-config generate   # generate all target files
chub agent-config sync       # update only if source changed
chub agent-config diff       # show what would change
```

Supported targets: `CLAUDE.md`, `.cursorrules`, `.windsurfrules`, `AGENTS.md`, `.github/copilot-instructions.md`.

### chub snapshot

Manage point-in-time pin snapshots.

```sh
chub snapshot create <name>          # save current pins
chub snapshot list                   # list all snapshots
chub snapshot restore <name>         # restore pin state
chub snapshot diff <name-a> <name-b> # compare two snapshots
```

### chub stats

Show local usage analytics (opt-in).

```sh
chub stats [--json]
```

## Server Commands

### chub mcp

Start the MCP (Model Context Protocol) stdio server.

```sh
chub mcp [--profile <name>]
```

| Flag | Description |
|---|---|
| `--profile <name>` | Load a specific context profile |

### chub serve

Serve a built content directory as an HTTP registry.

```sh
chub serve <content-dir> [--port <n>]
```

| Flag | Description | Default |
|---|---|---|
| `--port <n>` | HTTP port | 4242 |

## Build Commands

### chub build

Build a content directory into `registry.json` and `search-index.json`.

```sh
chub build <content-dir> [-o <output>] [--base-url <url>] [--validate-only]
```

| Flag | Description | Default |
|---|---|---|
| `-o, --output <dir>` | Output directory | `dist/` |
| `--base-url <url>` | CDN base URL for doc paths | none |
| `--validate-only` | Validate content without building | off |

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
