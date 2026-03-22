---
layout: home

hero:
  name: Chub
  text: Give your AI agents the context to get it right the first time
  tagline: Curated docs that learn from every session. Team knowledge that compounds. Built in Rust.
  image:
    src: /logo.svg
    alt: Chub
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: Installation
      link: /guide/installation
    - theme: alt
      text: GitHub
      link: https://github.com/nrl-ai/chub

features:
  - icon: ⚡
    title: Lightning Fast
    details: ~44ms cold start. 10 MB single binary. 19x faster search, 5x faster validation. Zero runtime dependencies.
  - icon: 🧠
    title: Agents That Learn
    details: Agents write back what they discover — bugs, fixes, and validated practices. The knowledge compounds. Your team's AI never hits the same gotcha twice.
  - icon: 📌
    title: Doc Pinning
    details: Lock specific doc versions for your team. Every developer and every AI agent uses the same reference material.
  - icon: 👥
    title: Context Profiles
    details: Role-scoped context with inheritance. Backend devs get API docs, frontend gets UI docs — automatically.
  - icon: 🔄
    title: Agent Config Sync
    details: Generate CLAUDE.md, .cursorrules, AGENTS.md, and more from a single source of truth. Add annotation policy to instruct agents automatically.
  - icon: 📄
    title: Project Context
    details: Custom markdown docs in .chub/context/ — architecture decisions, conventions, runbooks — served via CLI and MCP.
---

<div class="stats-bar">
  <div class="stat">
    <div class="stat-num">1,560</div>
    <div class="stat-label">Curated Docs</div>
  </div>
  <div class="stat">
    <div class="stat-num">~44ms</div>
    <div class="stat-label">Cold Start</div>
  </div>
  <div class="stat">
    <div class="stat-num">10MB</div>
    <div class="stat-label">Binary Size</div>
  </div>
  <div class="stat">
    <div class="stat-num">19x</div>
    <div class="stat-label">Faster Search</div>
  </div>
</div>

## Install

::: code-group

```sh [npm]
npm install -g @nrl-ai/chub
```

```sh [pip]
pip install chub
```

```sh [Cargo]
cargo install chub
```

```sh [Homebrew]
brew install nrl-ai/tap/chub
```

:::

## Quick Start

```sh
# Search for docs
chub search "stripe payments"

# Fetch a doc
chub get openai/chat --lang python

# Initialize project for team sharing
chub init

# Auto-detect dependencies and pin matching docs
chub detect --pin

# Start MCP server for AI agents
chub mcp
```

## MCP Setup

Add to your MCP config and your AI agent gets instant access to 1,560+ curated docs:

```json
{
  "mcpServers": {
    "chub": {
      "command": "chub",
      "args": ["mcp"]
    }
  }
}
```

## Benchmarks

Measured on the production corpus (1,560 docs, 7 skills). Median of 5 runs. Reproduce with `./scripts/benchmark.sh`.

| Operation | Context Hub (JS) | Chub (Rust) | Speedup |
|---|---|---|---|
| `search "stripe payments"` | 1,060 ms | **56 ms** | **19x** |
| `build --validate-only` | 1,920 ms | **380 ms** | **5x** |
| `build` (1,560 entries) | 3,460 ms | **1,770 ms** | **2x** |
| `get stripe/api` | 148 ms | **63 ms** | **2.3x** |
| Cold start (`--help`) | 131 ms | **44 ms** | **3x** |

| Metric | Context Hub (JS) | Chub (Rust) |
|---|---|---|
| Package size | ~22 MB (`node_modules`) | **10 MB** (single binary) |
| Runtime dependency | Node.js 20+ | **None** |
| Peak memory (build) | ~122 MB | **~23 MB** |

## vs Context Hub & Context7

| Capability | Context Hub | Context7 | Chub |
|---|---|---|---|
| Curated docs | 1,600+ | hosted | 1,560+ |
| MCP server | 5 tools | 2 tools | **7 tools** |
| CLI commands | 7 | — | **20** |
| Self-learning agents | No | **No** | **Yes** |
| Structured annotations (issue/fix/practice) | No | **No** | **Yes** |
| Annotation policy in CLAUDE.md | No | **No** | **Yes** |
| Doc pinning | No | No | **Yes** |
| Team annotations | No | No | **Git-tracked** |
| Context profiles | No | No | **With inheritance** |
| Agent config sync | No | No | **10 targets** |
| Self-hosted registry | Yes | No | **Yes** |
| Format compatible | — | — | **Identical to Context Hub** |

Built on [Context Hub](https://github.com/andrewyng/context-hub) by Andrew Ng. Fully format-compatible. Complementary to Context7. The difference: Chub's knowledge base grows smarter with every session — because learning is a natural skill for coding agents. [Read the full story.](/guide/why-chub)
