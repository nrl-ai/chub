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

Running `chub pin add` on an already-pinned ID updates it:

```sh
# Update version
chub pin add openai/chat --version 4.5

# Add a reason
chub pin add openai/chat --reason "Migrated to v4.5"
```

## Freshness checks

Pinned versions can drift behind the library version actually installed in your project. Use `chub check` to detect this:

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

---

See [Dep Auto-Detection](/guide/detect) to auto-pin from your dependency files.
