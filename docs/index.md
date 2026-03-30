---
layout: home

hero:
  name: Chub
  text: The all-in-one infrastructure layer for AI-assisted development
  tagline: Context, tracking, security, and team knowledge in one tool. Curated docs, session analytics, secret scanning, self-learning annotations — agent-agnostic, git-native, built in Rust.
  image:
    src: /logo.svg
    alt: Chub
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: Why Chub
      link: /guide/why-chub
    - theme: alt
      text: GitHub
      link: https://github.com/nrl-ai/chub

features:
  - icon: 📚
    title: Context
    details: "Serve curated, versioned API docs to any AI agent. 1,553+ entries, BM25 search in 56ms, MCP + CLI. Your agents stop hallucinating API signatures."
  - icon: 🧠
    title: Self-Learning
    details: "Agents write back bugs, fixes, and practices they discover. Three-tier annotations (personal → team → org) compound your team's knowledge automatically."
  - icon: 📊
    title: Tracking & Analytics
    details: "Track every AI session — tokens, costs, models, reasoning, tool calls — across Claude Code, Cursor, Copilot, Gemini CLI, Codex. One command, any agent."
  - icon: 💰
    title: Cost Intelligence
    details: "Built-in rates for Claude, GPT, Gemini, DeepSeek, o1/o3. Custom rate overrides. Budget alerts. Web dashboard with charts and breakdowns."
  - icon: 👥
    title: Team Sharing
    details: "Pin doc versions, scope context by role, sync agent configs to 10 targets, share annotations via git. The whole team sees the same truth."
  - icon: 🔒
    title: Security
    details: "260+ secret detection rules. Scans git history, directories, and stdin. AI transcript-aware — catches secrets buried in agent chat logs and prompts. Drop-in gitleaks/betterleaks replacement. 2–4x faster on typical repos. SARIF output for CI/CD."
  - icon: ⚡
    title: Fast & Portable
    details: "~44ms cold start. 10 MB single binary. Zero runtime deps. Works offline. Self-hostable. Runs on Linux, macOS, Windows, ARM64."
---

<div class="stats-bar">
  <div class="stat">
    <div class="stat-num">4</div>
    <div class="stat-label">Pillars: Context · Tracking · Security · Team</div>
  </div>
  <div class="stat">
    <div class="stat-num">1,553+</div>
    <div class="stat-label">Curated Docs</div>
  </div>
  <div class="stat">
    <div class="stat-num">8</div>
    <div class="stat-label">MCP Tools</div>
  </div>
  <div class="stat">
    <div class="stat-num">6+</div>
    <div class="stat-label">Agents Supported</div>
  </div>
</div>

## The All-in-One Agent Layer

Most tools do one thing: serve docs, or track usage, or scan secrets, or sync configs. Chub does all four — because context, tracking, security, and team knowledge are one problem, not four.

<p align="center">
  <img src="/architecture.svg" width="700" alt="Chub Architecture — Context, Tracking, Learning">
</p>

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

Or download a prebuilt binary from [GitHub Releases](https://github.com/nrl-ai/chub/releases) — single 10 MB binary, no runtime dependencies.

## Quick Start

### 1. Give your agents accurate docs

```sh
chub search "stripe payments"              # find docs
chub get openai/chat --lang python         # fetch a doc
chub mcp                                   # start MCP server for agents
```

### 2. Track what your agents do

```sh
chub track enable                          # install hooks (auto-detects agent)
# ... use your AI agent as normal ...
chub track status                          # see active session
chub track report                          # costs, tokens, models, tools
chub track dashboard                       # web dashboard at localhost:4243
```

### 3. Catch leaked secrets

```sh
chub scan secrets git                          # scan git history
chub scan secrets git --staged                 # pre-commit hook mode
chub scan secrets dir ./src                    # scan a directory
```

### 4. Build team knowledge

```sh
chub init --from-deps                      # create .chub/, auto-pin docs
chub annotate openai/chat "Use streaming"  # team annotation
  --kind practice --team
chub agent-config sync                     # generate CLAUDE.md, .cursorrules
```

## MCP Setup

<p align="center">
  <img src="/dataflow.svg" width="700" alt="Chub MCP Dataflow — Agent → Chub → Registry">
</p>

Add to your MCP config and your AI agent gets instant access to context, tracking, and team knowledge:

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

Works with Claude Code, Cursor, Windsurf, Copilot, Gemini CLI, Kiro, Codex, Cline, Roo Code, Augment, Continue.dev, and Aider. See [Agent Integrations](/guide/agent-config) for setup guides.

**8 MCP tools:** `chub_search`, `chub_get`, `chub_list`, `chub_context`, `chub_pins`, `chub_annotate`, `chub_feedback`, `chub_track`

## Benchmarks

### Context vs Context Hub — 1,553 docs, median of 5 runs

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

### Secret scanning — 10 real public repos, median of 3 runs

Directory scan (chub vs gitleaks v8.30.1):

| Repo | Files | Chub | Gitleaks | Speedup |
|---|--:|--:|--:|---|
| axios/axios | 361 | **124 ms** | 410 ms | **3.8x** |
| expressjs/express | 213 | **119 ms** | 409 ms | **3.9x** |
| tokio-rs/tokio | 843 | **132 ms** | 414 ms | **3.5x** |
| tiangolo/fastapi | 2,981 | **263 ms** | 421 ms | **1.8x** |
| django/django | 7,027 | **445 ms** | 435 ms | 1.1x |
| golang/go | 15,154 | 847 ms | **422 ms** | 0.6x |

Chub is 2–4x faster on repos up to ~7k files. gitleaks leads on very large monorepos. See [full table](/guide/scanning#performance).

## What makes Chub different

| Capability | Context Hub | Context7 | Chub |
|---|---|---|---|
| **Context** | | | |
| Curated docs | 1,600+ | hosted | 1,553+ |
| MCP server | 5 tools | 2 tools | **8 tools** |
| CLI commands | 7 | — | **22** |
| Self-hosted registry | Yes | No | **Yes** |
| Format compatible | — | — | **Identical to Context Hub** |
| **Self-Learning** | | | |
| Structured annotations (issue/fix/practice) | No | No | **Yes** |
| Three-tier storage (personal/team/org) | No | No | **Yes** |
| Annotation policy in CLAUDE.md | No | No | **Yes** |
| **Team Features** | | | |
| Doc version pinning | No | No | **Yes** |
| Context profiles with inheritance | No | No | **Yes** |
| Agent config sync | No | No | **10 targets** |
| Project context docs | No | No | **Yes** |
| **Tracking & Analytics** | | | |
| Session tracking | No | No | **Yes** |
| Cost estimation | No | No | **Yes** |
| Web dashboard | No | No | **Yes** |
| Multi-agent support | — | — | **6+ agents** |
| **Security** | | | |
| Secret scanning (260+ rules) | No | No | **Yes** |
| AI transcript scanning | No | No | **Yes** |
| Gitleaks/betterleaks compatible | No | No | **Drop-in** |
| SARIF/JSON/CSV output | No | No | **Yes** |

Built on [Context Hub](https://github.com/andrewyng/context-hub) by Andrew Ng — fully format-compatible. Complementary to [Context7](https://context7.com). [Read the full story.](/guide/why-chub)
