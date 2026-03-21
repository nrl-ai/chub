# Why Chub

[Context Hub](https://github.com/andrewyng/context-hub) by Andrew Ng solves a real problem: coding agents hallucinate APIs and forget what they learn between sessions. Its solution — curated, versioned markdown docs served via CLI and MCP — works.

Chub is built on that foundation and adds three things:

## 1. Performance

A native Rust binary replaces the Node.js runtime:

| Metric | Context Hub (JS) | Chub (Rust) | Improvement |
|---|---|---|---|
| Build (4 entries) | 1,050 ms | **38 ms** | **27x faster** |
| Build (1,559 entries) | 6,300 ms | **2,500 ms** | **2.5x faster** |
| Validate only | 6,300 ms | **360 ms** | **17x faster** |
| Cold start (`--help`) | 120 ms | **26 ms** | **4.6x faster** |
| Binary size | ~70 MB | **1.2 MB** | **58x smaller** |
| Memory (1,559 entries) | ~120 MB | **~15 MB** | **8x less** |
| Runtime dependency | Node.js 20+ | **None** | Single binary |

Every operation completes under 50ms. Speed matters because agents shouldn't wait for context.

## 2. Team Features

Things an individual tool doesn't need but a team does:

- **Doc pinning** — lock versions so agents can't use outdated APIs
- **Shared annotations** — "don't use the v3 pattern" committed to git, surfaced automatically
- **Context profiles** — backend devs get API docs, frontend gets UI docs, inherited from a shared base
- **Agent config sync** — one source of truth for CLAUDE.md, .cursorrules, AGENTS.md, and more
- **Project context** — architecture docs, conventions, runbooks served alongside public docs
- **Dep auto-detection** — scan 9 file types and auto-pin matching docs
- **Doc snapshots** — point-in-time pin captures for reproducible builds
- **Freshness checks** — detect when pinned doc versions lag behind installed packages

## 3. Full Compatibility

Identical registry format, search index, and config schema. Content authored for Context Hub works in Chub without changes. The switch is a drop-in replacement.

## Who is it for

- **Teams** where multiple developers and AI agents need consistent context
- **Projects** that want git-tracked, reviewable doc configuration
- **Monorepos** that need path-scoped context profiles
- **Anyone** who wants a faster alternative to Context Hub

## Design Principles

1. **Git-first** — team config lives in the repo. If it's not in git, it doesn't exist for the team.
2. **Gradual adoption** — works for a solo developer today; adds team value when `.chub/` is committed.
3. **Three-tier inheritance** — personal -> project -> profile. No tier is required.
4. **Agent-native** — every feature is accessible via MCP. CLI is for humans, MCP is for agents.
5. **Zero cloud dependency** — everything works offline and locally.
6. **Fast** — every operation under 50ms.
