# Why Chub

[Context Hub](https://github.com/andrewyng/context-hub) by Andrew Ng solves a real problem: coding agents hallucinate APIs and forget what they learn between sessions. Its solution — curated, versioned markdown docs served via CLI and MCP — works.

Chub is built on that foundation and adds three things:

## 1. Performance

A native Rust binary replaces the Node.js runtime:
- **27x faster builds** for content registry
- **~26ms cold start** vs 120ms
- **1.2MB binary** vs ~70MB with `node_modules`

Every operation under 50ms. Speed matters because agents shouldn't wait for context.

## 2. Team Features

Things an individual tool doesn't need but a team does:
- **Doc pinning** — lock versions so agents can't use outdated APIs
- **Shared annotations** — "don't use the v3 pattern" committed to git
- **Context profiles** — backend devs get API docs, frontend gets UI docs
- **Agent config sync** — one source of truth for CLAUDE.md, .cursorrules, etc.
- **Project context** — architecture docs, conventions, runbooks served alongside public docs

## 3. Full Compatibility

Identical registry format, search index, and config schema. Content authored for Context Hub works in Chub without changes. The switch is a drop-in replacement.

## Who Is It For

- **Teams** where multiple developers and AI agents need consistent context
- **Projects** that want git-tracked, reviewable doc configuration
- **Monorepos** that need path-scoped context profiles
- **Anyone** who wants a faster alternative to Context Hub

## Design Principles

1. **Git-first** — team config lives in the repo. If it's not in git, it doesn't exist for the team.
2. **Gradual adoption** — works for a solo developer today; adds team value when `.chub/` is committed.
3. **Three-tier inheritance** — personal → project → profile. No tier is required.
4. **Agent-native** — every feature is accessible via MCP. CLI is for humans, MCP is for agents.
5. **Zero cloud dependency** — everything works offline and locally.
6. **Fast** — every operation under 50ms.
