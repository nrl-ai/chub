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
chub get <id> [--lang <language>] [--version <ver>] [--source <name>] [--pinned] [--json]
```

| Flag | Description |
|---|---|
| `<id>` | Doc ID (e.g., `openai/chat`) |
| `--lang <language>` | Language variant (e.g., `python`, `javascript`) |
| `--version <ver>` | Specific version (e.g., `4.0`) |
| `--source <name>` | From a specific source |
| `--pinned` | Fetch all pinned docs at once |
| `--json` | JSON output |

**Examples:**

```sh
chub get openai/chat --lang python
chub get stripe/api --lang javascript --version 2024
chub get --pinned                        # fetch all pinned docs
chub get project/architecture            # fetch a project context doc
```

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
| `--monorepo` | Create config with auto-profile rules for monorepo |

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

Add annotations to docs. Team annotations are git-tracked; personal ones are local.

```sh
chub annotate <id> <note> [--team] [--personal] [--author <name>]
```

| Flag | Description |
|---|---|
| `--team` | Save to `.chub/annotations/` (git-tracked, shared) |
| `--personal` | Save to `~/.chub/annotations/` (local only) |
| `--author <name>` | Author name (defaults to `$USER`) |

### chub feedback

Submit feedback about a doc.

```sh
chub feedback <id> <message>
```

### chub detect

Scan dependency files and find matching docs.

```sh
chub detect [--pin]
```

| Flag | Description |
|---|---|
| `--pin` | Auto-pin all detected matches |

Supported: `package.json`, `Cargo.toml`, `requirements.txt`, `pyproject.toml`, `Pipfile`, `go.mod`, `Gemfile`, `pom.xml`, `build.gradle(.kts)`.

### chub agent-config

Generate and sync agent configuration files from `.chub/config.yaml`.

```sh
chub agent-config generate   # generate all target files
chub agent-config sync       # update only if source changed
chub agent-config diff       # show what would change
```

Targets: `CLAUDE.md`, `.cursorrules`, `.windsurfrules`, `AGENTS.md`, `.github/copilot-instructions.md`.

### chub snapshot

Manage point-in-time pin snapshots.

```sh
chub snapshot create <name>          # save current pins
chub snapshot list                   # list all snapshots
chub snapshot restore <name>         # restore pin state
chub snapshot diff <name-a> <name-b> # compare two snapshots
```

### chub check

Check pinned doc versions against installed library versions.

```sh
chub check [--fix]
```

| Flag | Description |
|---|---|
| `--fix` | Auto-update outdated pins to match installed versions |

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
