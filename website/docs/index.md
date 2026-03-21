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
      text: View on GitHub
      link: https://github.com/nrl-ai/chub

features:
  - icon: ⚡
    title: Lightning Fast
    details: ~26ms cold start. 1.2MB binary. 27x faster builds than the JS version. Written in Rust.
  - icon: 📌
    title: Doc Pinning
    details: Lock specific doc versions for your team. Every agent uses the same reference material.
  - icon: 👥
    title: Context Profiles
    details: Role-scoped context with inheritance. Backend devs get API docs, frontend gets UI docs.
  - icon: 🔄
    title: Agent Config Sync
    details: Generate CLAUDE.md, .cursorrules, AGENTS.md from a single source of truth.
  - icon: 📄
    title: Project Context
    details: Custom markdown docs in .chub/context/ — architecture, conventions, runbooks — served via MCP.
  - icon: 🔍
    title: Dep Auto-Detection
    details: Scan package.json, Cargo.toml, requirements.txt and auto-pin matching docs. 9 file types supported.
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

## Quick Install

```sh
npm install -g @nrl-ai/chub
```

```sh
# Search for docs
chub search "stripe payments"

# Fetch a doc
chub get openai/chat --lang python

# Initialize project for team sharing
chub init
```

## vs Context Hub

| Capability | Context Hub | Chub |
|---|---|---|
| Curated docs | 1,600+ | 1,553+ |
| MCP server | Yes | Yes |
| Format compatible | - | Byte-for-byte |
| Cold start | ~120ms | **~26ms** |
| Binary size | ~70MB (Node) | **1.2MB** |
| Doc pinning | No | **Yes** |
| Team annotations | No | **Git-tracked** |
| Context profiles | No | **With inheritance** |
| Agent config sync | No | **5 targets** |
| Dep auto-detection | No | **9 file types** |
| Doc snapshots | No | **Yes** |

Built on [Context Hub](https://github.com/andrewyng/context-hub) by Andrew Ng. Fully format-compatible.
