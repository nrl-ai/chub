# Team Annotations

Shared annotations in `.chub/annotations/` are git-tracked and visible to all team members and agents.

## Add annotations

```sh
# Team annotation (git-tracked, shared)
chub annotate openai/chat "Use v4 streaming, not completions" --team

# Personal annotation (local only)
chub annotate openai/chat "My local note" --personal
```

## File format

```yaml
# .chub/annotations/openai--chat.yaml
id: openai/chat
notes:
  - author: alice
    date: 2026-03-15
    note: "Webhook endpoint requires raw body parsing"
  - author: bob
    date: 2026-03-18
    note: "Rate limits hit at 500 RPM on our plan"
```

## Resolution order

When serving a doc, annotations are merged in this order (last wins):

1. Public doc content (from registry)
2. Team annotations (`.chub/annotations/`)
3. Personal annotations (`~/.chub/annotations/`)

## Pin notices

When a pinned doc is served, a team notice is appended automatically:

```
---
[Team pin] Locked to v4.0 (python). Reason: We use v4 streaming API.
```
