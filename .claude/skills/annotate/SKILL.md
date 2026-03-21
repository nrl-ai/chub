---
name: annotate
description: Record a team annotation (issue, fix, or practice) on a documentation entry.
user-invocable: true
argument-hint: <entry-id> <note>
---

# Annotate Documentation

Record annotation: $ARGUMENTS

First token is the entry ID, rest is the note. Use `chub_annotate` with `scope: "team"`.

Auto-detect `kind` from the note content: bugs/gotchas = "issue", workarounds = "fix", conventions = "practice", everything else = "note".
