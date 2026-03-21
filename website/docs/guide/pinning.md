# Doc Pinning

Pin specific docs and versions so every team member and AI agent uses the same reference material. Pins are declarative and version-controlled in `.chub/pins.yaml`.

## Pin a doc

```sh
chub pin openai/chat --lang python --version 4.0 --reason "Use v4 streaming API"
chub pin stripe/api --lang javascript
chub pin nextjs/app-router --version 15.0
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
chub pin <id>          # Pin a doc
chub unpin <id>        # Remove a pin
chub pins              # List all pins
chub get --pinned      # Fetch all pinned docs at once
```

## MCP Integration

When an agent calls `chub_get`, pinned version and language are automatically applied. The agent doesn't need to know which version to use.

A team notice is appended to pinned docs:

```
---
[Team pin] Locked to v4.0 (python). Reason: We use v4 streaming API.
```

## Update a pin

Running `chub pin` on an already-pinned ID updates it:

```sh
# Update version
chub pin openai/chat --version 4.5

# Add a reason
chub pin openai/chat --reason "Migrated to v4.5"
```
