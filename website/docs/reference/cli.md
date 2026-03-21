# CLI Commands

## chub search

Search for docs in the registry.

```sh
chub search <query> [--limit <n>] [--source <name>] [--json]
```

| Flag | Description | Default |
|---|---|---|
| `--limit <n>` | Max results | 10 |
| `--source <name>` | Search specific source | all |
| `--json` | JSON output | off |

## chub get

Fetch a specific doc by ID.

```sh
chub get <id> [--lang <language>] [--version <ver>] [--source <name>] [--pinned] [--json]
```

| Flag | Description |
|---|---|
| `--lang <language>` | Filter by language |
| `--version <ver>` | Specific version |
| `--pinned` | Fetch all pinned docs |

Special prefix: `chub get project/<name>` fetches a project context doc.

## chub list

```sh
chub list [--source <name>] [--json]
```

## chub build

Build a content directory into a registry.

```sh
chub build <content-dir> [-o <output>] [--base-url <url>] [--validate-only]
```

| Flag | Description | Default |
|---|---|---|
| `-o, --output <dir>` | Output directory | dist/ |
| `--base-url <url>` | CDN base URL | none |
| `--validate-only` | Validate only | off |

## chub init

```sh
chub init [--from-deps] [--monorepo]
```

## chub pin / unpin / pins

```sh
chub pin <id> [--lang <lang>] [--version <ver>] [--reason <text>]
chub unpin <id>
chub pins
```

## chub profile

```sh
chub profile use <name>    # Set active profile ("none" to clear)
chub profile list
```

## chub annotate

```sh
chub annotate <id> <note> [--team] [--personal] [--author <name>]
```

| Flag | Description |
|---|---|
| `--team` | Write to .chub/annotations/ (git-tracked) |
| `--personal` | Write to ~/.chub/annotations/ (local only) |
| `--author <name>` | Author name (defaults to $USER) |

## chub detect

```sh
chub detect [--pin]
```

## chub agent-config

```sh
chub agent-config generate   # Generate all target files
chub agent-config sync       # Update only if changed
chub agent-config diff       # Show pending changes
```

## chub snapshot

```sh
chub snapshot create <name>
chub snapshot list
chub snapshot restore <name>
chub snapshot diff <name-a> <name-b>
```

## chub check

```sh
chub check [--fix]
```

## chub stats

```sh
chub stats [--json]
```

## chub mcp

```sh
chub mcp [--profile <name>]
```

Start the MCP stdio server.

## chub serve

```sh
chub serve <content-dir> [--port <n>]
```

Start an HTTP registry server. Default port: 4242.
