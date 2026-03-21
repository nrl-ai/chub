# Why Chub

[Context Hub](https://github.com/andrewyng/context-hub) by Andrew Ng solves a real problem: coding agents hallucinate APIs and forget what they learn between sessions. Its solution — curated, versioned markdown docs served via CLI and MCP — works.

Chub is built on that foundation and adds four things:

## 1. Performance

A native Rust binary replaces the Node.js runtime:

| Metric | Context Hub (JS) | Chub (Rust) | Improvement |
|---|---|---|---|
| Search | 1,060 ms | **56 ms** | **19x faster** |
| Validate only | 1,920 ms | **380 ms** | **5x faster** |
| Build (1,560 entries) | 3,460 ms | **1,770 ms** | **2x faster** |
| Get (cached doc) | 148 ms | **63 ms** | **2.3x faster** |
| Cold start (`--help`) | 131 ms | **44 ms** | **3x faster** |
| Package size | ~22 MB | **10 MB** | **2.2x smaller** |
| Peak memory (build) | ~122 MB | **~23 MB** | **5.3x less** |
| Runtime dependency | Node.js 20+ | **None** | Single binary |

Measured on Windows 11 with the production corpus (1,553 docs, 7 skills). Median of 5 runs. Reproduce with `./scripts/benchmark.sh`.

## 2. Self-Learning Agents

This is the key differentiator from both Context Hub and Context7.

Context7 and Context Hub serve docs. Chub also collects knowledge back from agents.

Every time an agent resolves something non-obvious — a broken parameter, an undocumented gotcha, a workaround that actually works — it writes a structured annotation. Future agents that fetch the same doc see it automatically. The knowledge base improves with every session.

**Structured annotation kinds:**

| Kind | What it captures |
|------|-----------------|
| `issue` | Confirmed bug, broken param, misleading example |
| `fix` | Workaround that resolves the issue |
| `practice` | Convention or pattern the team has validated |
| `note` | General observation |

**What Context7 shows an agent:**
```
[openai/chat docs — official content only]
```

**What Chub shows an agent:**
```
[openai/chat docs — official content]
---
[Team issue (high) — bob] tool_choice='none' silently ignores tools — returns null
[Team fix — bob] Use tool_choice='auto' or remove tools from the array
[Team practice — alice] Always set max_tokens; omitting it causes unbounded streaming cost
```

The second agent never has to discover any of this the hard way.

**Annotation policy in CLAUDE.md:** set `include_annotation_policy: true` in your agent config to give every agent standing instructions to write back what it discovers — without repeating yourself in every session.

**Three storage tiers:** personal (`~/.chub/`), team/repo (`.chub/`), and hosted org-wide (Phase 8). Each tier is additive; personal annotations always win.

## 3. Team Features

Things an individual tool doesn't need but a team does:

- **Doc pinning** — lock versions so agents can't use outdated APIs
- **Shared annotations** — structured by kind (issue/fix/practice), committed to git, surfaced automatically
- **Context profiles** — backend devs get API docs, frontend gets UI docs, inherited from a shared base
- **Agent config sync** — one source of truth for CLAUDE.md, .cursorrules, AGENTS.md, and more
- **Project context** — architecture docs, conventions, runbooks served alongside public docs
- **Dep auto-detection** — scan 9 file types and auto-pin matching docs
- **Doc snapshots** — point-in-time pin captures for reproducible builds
- **Freshness checks** — detect when pinned doc versions lag behind installed packages

## 4. Full Compatibility

Identical registry format, search index, and config schema. Content authored for Context Hub works in Chub without changes. The switch is a drop-in replacement.

## Chub vs Context7

Context7 (~50K GitHub stars) validates strong market demand for up-to-date docs in AI coding tools. It solves the same core problem: agents hallucinating APIs.

The key differences:

| | Context7 | Chub |
|---|---|---|
| Doc serving | Yes | Yes |
| Self-hosted | No | **Yes** |
| Team sharing | No | **Yes** |
| Self-learning agents | **No** | **Yes** |
| Annotation kinds (issue/fix/practice) | **No** | **Yes** |
| Agent config sync (CLAUDE.md etc.) | No | **Yes** |
| Git-tracked knowledge | No | **Yes** |
| Custom/private docs | No | **Yes** |

Context7 is a hosted service you call. Chub is infrastructure you own — with the knowledge base growing smarter over time as your own agents use it.

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
6. **Fast** — search in ~56ms, cold start in ~44ms.
