---
layout: home

hero:
  name: Chub
  text: Fast curated docs for AI coding agents
  tagline: Team-first. Git-tracked. Built in Rust.
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
    details: ~26ms cold start. 1.2MB single binary. 27x faster builds than the JS version. Zero runtime dependencies.
  - icon: 📌
    title: Doc Pinning
    details: Lock specific doc versions for your team. Every developer and every AI agent uses the same reference material.
  - icon: 👥
    title: Context Profiles
    details: Role-scoped context with inheritance. Backend devs get API docs, frontend gets UI docs — automatically.
  - icon: 🔄
    title: Agent Config Sync
    details: Generate CLAUDE.md, .cursorrules, AGENTS.md, and more from a single source of truth in .chub/config.yaml.
  - icon: 📄
    title: Project Context
    details: Custom markdown docs in .chub/context/ — architecture decisions, conventions, runbooks — served via CLI and MCP.
  - icon: 🔍
    title: Dep Auto-Detection
    details: Scan package.json, Cargo.toml, requirements.txt and 6 more file types. Auto-pin matching docs with one command.
---

<div class="stats-bar">
  <div class="stat">
    <div class="stat-num">1,553</div>
    <div class="stat-label">Curated Docs</div>
  </div>
  <div class="stat">
    <div class="stat-num">~26ms</div>
    <div class="stat-label">Cold Start</div>
  </div>
  <div class="stat">
    <div class="stat-num">1.2MB</div>
    <div class="stat-label">Binary Size</div>
  </div>
  <div class="stat">
    <div class="stat-num">27x</div>
    <div class="stat-label">Faster Builds</div>
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

Add to your MCP config and your AI agent gets instant access to 1,553+ curated docs:

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

Measured on the production corpus (1,553 docs, 6 skills, 1,691 files):

| Operation | JS (Context Hub) | Rust (Chub) | Speedup |
|---|---|---|---|
| `build` (4 entries) | 1,050 ms | **38 ms** | **27x** |
| `build` (1,559 entries) | 6,300 ms | **2,500 ms** | **2.5x** |
| `build --validate-only` | 6,300 ms | **360 ms** | **17x** |
| Cold start (`--help`) | 120 ms | **26 ms** | **4.6x** |

| Metric | JS | Rust |
|---|---|---|
| Binary size | ~70 MB (with `node_modules`) | **1.2 MB** |
| Runtime dependency | Node.js 20+ | **None** |
| Memory (build, 1,559 entries) | ~120 MB | **~15 MB** |

## vs Context Hub

| Capability | Context Hub | Chub |
|---|---|---|
| Curated docs | 1,600+ | 1,553+ |
| MCP server | Yes | Yes |
| Format compatible | — | Byte-for-byte |
| Cold start | ~120ms | **~26ms** |
| Binary size | ~70MB (Node) | **1.2MB** |
| Doc pinning | No | **Yes** |
| Team annotations | No | **Git-tracked** |
| Context profiles | No | **With inheritance** |
| Agent config sync | No | **5 targets** |
| Dep auto-detection | No | **9 file types** |
| Doc snapshots | No | **Yes** |

Built on [Context Hub](https://github.com/andrewyng/context-hub) by Andrew Ng. Fully format-compatible.
