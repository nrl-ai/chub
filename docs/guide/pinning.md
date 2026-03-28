# Doc Pinning

Pin specific docs and versions so every team member and AI agent uses the same reference material. Pins are declarative and version-controlled in `.chub/pins.yaml`.

## Pin a doc

```sh
chub pin add openai/chat --lang python --version 4.0 --reason "Use v4 streaming API"
chub pin add stripe/api --lang javascript
chub pin add nextjs/app-router --version 15.0
```

## pins.yaml format

```yaml
pins:
  - id: openai/chat
    lang: python
    version: "4.0"
    reason: "We use v4 streaming API — do NOT suggest v3 patterns"

  - id: stripe/api
    lang: javascript
    # version omitted = always latest

  - id: internal/auth-service
    source: private
    reason: "Internal auth microservice docs"
```

## Commands

```sh
chub pin add <id>      # Pin a doc
chub pin remove <id>   # Remove a pin
chub pin list          # List all pins
chub pin get           # Fetch all pinned docs at once
```

## Update a pin

Running `chub pin add` on an already-pinned ID updates it. Only the fields you specify are changed — existing fields are preserved:

```sh
# Update version
chub pin add openai/chat --version 4.5

# Add a reason
chub pin add openai/chat --reason "Migrated to v4.5"

# Change language
chub pin add openai/chat --lang typescript
```

## Pin fields

| Field | Flag | Description |
|---|---|---|
| `id` | (positional) | Registry entry ID, e.g. `openai/chat` |
| `lang` | `--lang` | Language variant to serve (e.g. `python`, `javascript`) |
| `version` | `--version` | Doc version to lock to (stored as-is, no semver resolution) |
| `reason` | `--reason` | Human-readable note explaining why this pin exists |
| `source` | `--source` | Registry source override for private registries |

## Version matching

Pinned versions are stored as plain strings and compared with exact string matching after stripping common prefixes (`^`, `~`, `=`, `>`, `<`, `v`). There is no semver range resolution — if you pin `4.0`, it matches the `4.0` doc version, not `4.0.5` or `4.x`.

```yaml
# These are distinct versions — exact match only
- id: openai/chat
  version: "4.0"     # matches doc version "4.0" only

- id: stripe/api
  version: "2024-01"  # API date versions work too
```

When you omit `--version`, the pin uses whatever the latest version is at fetch time. To lock to a specific version, always set it explicitly.

## Language selection

Most registry entries ship multiple language variants. The `--lang` flag controls which one is served:

```sh
# Without --lang, chub serves the entry's default language
chub pin add openai/chat

# Lock to Python variant
chub pin add openai/chat --lang python
```

If your project uses multiple languages (e.g. Python backend + TypeScript frontend), use [profiles](/guide/profiles) with per-role pin languages rather than fighting over one global pin.

## Private registries

Pin docs from a private registry using `--source`:

```sh
chub pin add internal/auth-service --source private
```

The source name must match a registry configured in `.chub/config.yaml`:

```yaml
sources:
  private:
    url: https://docs.internal.company.com/v1
```

See [Self-Hosting a Registry](/guide/self-hosting) for how to run your own.

## Bulk operations

```sh
# Auto-pin everything detected in your dependency files
chub detect --pin

# Install a curated bundle of pins
chub bundle install api-stack

# Fetch all pinned docs in one shot
chub pin get
```

See [Dep Auto-Detection](/guide/detect) and [Doc Bundles](/guide/bundles) for details.

## Freshness checks

Pinned versions can drift behind the library version actually installed in your project. Freshness checking compares pin versions against your actual dependency files (`package.json`, `Cargo.toml`, etc.) using prefix-normalized string matching:

```sh
chub check         # Compare pinned doc versions vs installed library versions
chub check --fix   # Auto-update outdated pins
```

See [Snapshots & Freshness](/guide/snapshots) for the full freshness workflow and snapshot-based auditing.

## MCP integration

When an agent calls `chub_get`, the pinned version and language are automatically applied — the agent never needs to know which version to use.

A team notice is appended to pinned docs:

```
---
[Team pin] Locked to v4.0 (python). Reason: We use v4 streaming API.
```

The `chub_pins` MCP tool lets agents read and manage pins directly:

```json
// List all pins
{}

// Add a pin
{ "id": "openai/chat", "lang": "python", "version": "4.0", "reason": "Use v4 streaming" }

// Remove a pin
{ "id": "openai/chat", "remove": true }
```
