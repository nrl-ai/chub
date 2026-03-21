---
name: setup
description: Initialize chub for this project — detect dependencies, pin matching docs, generate agent configs.
user-invocable: true
argument-hint: (no arguments needed)
---

# Setup Chub

1. Run `chub init --from-deps`
2. Run `chub detect --json` and pin each match via `chub_pins`
3. Run `chub agent-config sync`
4. Show summary of what was pinned and generated
