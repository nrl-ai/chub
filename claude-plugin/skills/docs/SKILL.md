---
name: docs
description: Search or fetch API documentation from the chub registry. Handles both search queries and direct ID lookups.
user-invocable: true
argument-hint: <query or doc-id> [language]
---

# Documentation Lookup

Look up: $ARGUMENTS

If the input contains `/` (e.g. `serde/derive`), use `chub_get` to fetch it directly. Otherwise use `chub_search`.

After fetching, check `chub_annotate` (read mode, no `note`) for any team-recorded issues or practices on that entry.
