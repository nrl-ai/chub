<p align="center">
  <img src="https://raw.githubusercontent.com/nrl-ai/chub/main/website/assets/logo.svg" width="80" height="80" alt="Chub">
</p>

<h1 align="center">Chub</h1>

<p align="center">
  <strong>Agent-agnostic context, tracking, and cost analytics for AI-assisted development.</strong><br>
  <em>Curated docs · Session tracking · Cost analytics · Team knowledge — all git-native.</em>
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/@nrl-ai/chub"><img src="https://img.shields.io/npm/v/@nrl-ai/chub?color=0ea5e9&label=npm" alt="npm"></a>
  <a href="https://pypi.org/project/chub/"><img src="https://img.shields.io/pypi/v/chub?color=0ea5e9&label=pypi" alt="PyPI"></a>
  <a href="https://crates.io/crates/chub"><img src="https://img.shields.io/crates/v/chub?color=0ea5e9&label=crates.io" alt="crates.io"></a>
  <a href="https://github.com/nrl-ai/chub/actions"><img src="https://img.shields.io/github/actions/workflow/status/nrl-ai/chub/ci.yml?color=0ea5e9&label=CI" alt="CI"></a>
  <a href="https://github.com/nrl-ai/chub/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-0ea5e9" alt="License"></a>
  <a href="https://www.npmjs.com/package/@nrl-ai/chub"><img src="https://img.shields.io/npm/dm/@nrl-ai/chub?color=0ea5e9&label=downloads" alt="Downloads"></a>
</p>

<p align="center">
  <a href="https://chub.nrl.ai">Docs</a> · <a href="https://chub.nrl.ai/guide/getting-started">Getting Started</a> · <a href="https://github.com/nrl-ai/chub/releases">Releases</a>
</p>

---

## The Problem

AI coding agents (Claude, Cursor, Copilot) hallucinate API signatures, use deprecated endpoints, and forget what they learned between sessions. You paste docs into chat, but they get lost in context. Your teammates paste different docs. Nobody's on the same page.

## The Solution

Chub is a CLI + MCP server that serves curated, versioned API documentation directly to your AI agents. Think of it as a package manager for AI context: you pin the docs you need, your agents fetch them on demand, and your whole team shares the same source of truth — tracked in git.

```
You type:   chub get openai/chat --lang python
Agent sees: The complete, accurate OpenAI Chat API reference for Python
           + any bugs, fixes, and best practices your team has discovered
```

Built on [Context Hub](https://github.com/andrewyng/context-hub) by Andrew Ng — Chub is a high-performance Rust rewrite that extends the original with team features: shared doc pinning, git-tracked annotations, context profiles, agent config sync, content integrity verification, and **self-learning agents** that write back what they discover.

---

## How It Works

```
┌──────────────┐     ┌─────────────┐     ┌──────────────────┐
│  AI Agent    │────▶│  Chub MCP   │────▶│  Doc Registry    │
│  (Claude,    │     │  Server     │     │  (CDN / local)   │
│   Cursor)    │◀────│             │◀────│                  │
└──────────────┘     └─────────────┘     └──────────────────┘
   Gets accurate        Searches,             1,553+ curated
   API docs on          caches, and           docs covering
   demand               verifies docs         major APIs
```

1. **Agent needs API docs** — asks Chub via MCP (or you run `chub get`)
2. **Chub searches the registry** — BM25 search with lexical boosting, 19x faster than the JS version
3. **Returns the right version** — respects your team's pins, language preference, and version locks
4. **Appends team knowledge** — annotations, project context, and profile rules travel with the doc

---

## Quick Start

### Install

```sh
npm install -g @nrl-ai/chub     # npm (recommended)
pip install chub                 # pip
cargo install chub                # cargo (or: cargo install chub-cli)
brew install nrl-ai/tap/chub     # homebrew (macOS / Linux)
```

Or download a prebuilt binary from [GitHub Releases](https://github.com/nrl-ai/chub/releases) — single 10 MB binary, no runtime dependencies.

### Use

```sh
chub search "stripe payments"                  # find docs
chub get openai/chat --lang python             # fetch a doc
chub get stripe/api --match-env                # auto-detect version from your package.json
chub list                                      # browse everything
```

### Connect to Your AI Agent

Add to `.claude/settings.json` (Claude Code), `.cursor/mcp.json` (Cursor), or the equivalent for your agent:

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

Your agent now has access to `chub_search`, `chub_get`, `chub_list`, `chub_annotate`, `chub_context`, `chub_pins`, `chub_feedback`, and `chub_track` tools.

Works with Claude Code, Cursor, Windsurf, GitHub Copilot, Gemini CLI, Kiro, Cline, Roo Code, Augment, Codex, Continue.dev, and Aider. See [Agent Integrations](docs/guide/agent-config.md) for setup guides.

---

## Team Sharing

This is where Chub goes beyond Context Hub. Initialize a `.chub/` directory in your repo and commit it — every developer and every AI agent gets the same versioned context, automatically.

```sh
chub init --from-deps            # auto-detect deps, create .chub/, pin matching docs
```

```
my-project/
├── .chub/                       # committed to git
│   ├── config.yaml              # project-level config
│   ├── pins.yaml                # locked doc versions
│   ├── annotations/             # team knowledge (e.g., "use streaming API for this endpoint")
│   ├── context/                 # your own docs: architecture, conventions, runbooks
│   └── profiles/                # role-scoped context (backend.yaml, frontend.yaml)
```

### Pin docs to specific versions

```sh
chub pin add openai/chat --lang python --version 4.0 --reason "Use v4 API"
chub pin add stripe/api --lang javascript
chub pin get                     # fetch all pinned docs at once
```

### Build a self-learning knowledge base

Annotations live in three tiers — each scoped for different audiences:

| Tier | Storage | Visibility |
|---|---|---|
| **Personal** | `~/.chub/annotations/` | You only |
| **Team** | `.chub/annotations/` (git-tracked) | Your repo |
| **Org** | Remote HTTP API | Entire organization |

Agents can write back what they discover — structured by kind so the knowledge is findable, not just buried in a notes field:

```sh
# Human adds a note
chub annotate openai/chat "Always use streaming for chat completions" --team

# Agent discovers a bug and records it (kind=issue + kind=fix)
chub annotate openai/chat "tool_choice='none' silently ignores tools" --kind issue --severity high --team
chub annotate openai/chat "use tool_choice='auto' or remove tools from array" --kind fix --team

# Agent validates a best practice
chub annotate openai/chat "Always set max_tokens to avoid unbounded cost" --kind practice --team
```

Annotation kinds: **note**, **issue** (with severity), **fix**, **practice**. When any agent fetches these docs, all annotations appear alongside the official content — grouped by kind, clearly marked as team-contributed. Every debugging session becomes permanent team knowledge.

Add to `.chub/config.yaml` to automatically instruct agents to annotate:

```yaml
agent_rules:
  include_annotation_policy: true   # adds Annotation Policy section to CLAUDE.md / AGENTS.md
```

### Scope context by role

```sh
chub profile use backend         # backend devs get backend-relevant docs
chub profile use frontend        # frontend devs get frontend-relevant docs
```

### Keep versions fresh

```sh
chub detect                      # scan package.json, requirements.txt, Cargo.toml, etc.
chub check                       # compare pinned doc versions vs installed package versions
chub check --fix                 # auto-update outdated pins
```

### Sync agent config files

Generate rules files from a single `.chub/config.yaml` — one config, every agent in sync:

```sh
chub agent-config sync           # generate/update all configured targets
```

Supported targets: `claude.md`, `cursorrules`, `windsurfrules`, `agents.md`, `copilot`, `gemini.md`, `clinerules`, `roorules`, `augmentrules`, `kiro`.

---

## CLI Reference

### Search and Fetch

```sh
chub search "stripe"                    # BM25 search across all docs
chub search "auth" --limit 5            # limit results
chub get openai/chat --lang python      # fetch doc by ID
chub get stripe/api --version 2.0       # specific version
chub get stripe/api --match-env         # auto-detect version from project deps
chub get openai/chat --full             # fetch all files in the entry
chub get openai/chat --file refs.md     # fetch a specific file
chub list                               # list all available docs
chub list --json                        # JSON output (works with all commands)
```

### Doc Pinning

```sh
chub pin add openai/chat --lang python --version 4.0 --reason "Use v4 API"
chub pin list                           # list all pins
chub pin remove openai/chat             # remove a pin
chub pin get                            # fetch all pinned docs at once
```

### Context Profiles

```sh
chub profile use backend                # activate a profile
chub profile use none                   # clear profile
chub profile list                       # list available profiles
```

### Team Annotations

```sh
chub annotate openai/chat "Use streaming API" --team       # git-tracked
chub annotate openai/chat "My local note" --personal       # local only
```

### Dependency Detection

```sh
chub detect                             # show detected deps with matching docs
chub detect --pin                       # auto-pin all matches
```

Supports: `package.json`, `requirements.txt`, `pyproject.toml`, `Cargo.toml`, `go.mod`, `Gemfile`, `Pipfile`, `pom.xml`, `build.gradle`, `build.gradle.kts`.

### Agent Config Sync

```sh
chub agent-config generate              # generate rules for all configured targets
chub agent-config sync                  # update only if changed
chub agent-config diff                  # preview changes
```

### Snapshots and Freshness

```sh
chub snapshot create v1.0               # save current pins
chub snapshot list                      # list snapshots
chub snapshot restore v1.0              # restore pin state
chub snapshot diff v1.0 v2.0            # compare snapshots
chub check                              # check pinned vs installed versions
chub check --fix                        # auto-update outdated pins
```

### Cache Management

```sh
chub update                             # refresh cached registry
chub cache status                       # show cache state
chub cache clear                        # clear local cache
```

---

## MCP Server

```sh
chub mcp                                # start MCP stdio server
```

To scope the session to a profile, activate it first: `chub profile use backend && chub mcp`.

### Available tools

| Tool | Description |
|---|---|
| `chub_search` | Search docs by query, tags, or language |
| `chub_get` | Fetch a doc by ID (supports `match_env` for auto version detection) |
| `chub_list` | List all available docs and skills |
| `chub_annotate` | Read, write, or list annotations |
| `chub_context` | Get optimal context for a task (pins + annotations + profile) |
| `chub_pins` | List, add, or remove pinned docs |
| `chub_feedback` | Submit quality feedback for a doc |
| `chub_track` | Query AI usage tracking data (sessions, costs, tokens) |

Registry resource: `chub://registry`

Works with any MCP-compatible agent: Claude Code, Cursor, Windsurf, and others. The transport is stdio.

---

## Security

Chub includes several security measures for safe use in team environments:

- **Content integrity verification** — `chub build` computes SHA-256 hashes of all doc content, stored in the registry. Fetched content is verified against these hashes to detect CDN tampering.
- **Annotation trust framing** — User-contributed annotations are clearly marked as non-official content when served to agents, mitigating prompt injection risks.
- **Annotation length limits** — Notes are capped at 4,000 characters to prevent context flooding.
- **Path traversal protection** — File path parameters are validated and normalized.
- **Graceful process lifecycle** — The MCP server handles signals cleanly to prevent orphan processes.

---

## Benchmarks

Measured on the production corpus (1,553 docs, 8 skills). Median of 5 runs. Full methodology in [Chub vs Context Hub](docs/guide/vs-context-hub.md).

### Performance

| Operation | Context Hub (JS) | Chub (Rust) | Speedup |
|---|---|---|---|
| `search "stripe payments"` | 1,060 ms | **56 ms** | **19x** |
| `build --validate-only` | 1,920 ms | **380 ms** | **5x** |
| `build` (full registry) | 3,460 ms | **1,770 ms** | **2x** |
| `get stripe/api` | 148 ms | **63 ms** | **2.3x** |
| Cold start (`--help`) | 131 ms | **44 ms** | **3x** |

### Resources

| Metric | Context Hub (JS) | Chub (Rust) |
|---|---|---|
| Package size | ~22 MB (`node_modules`) | **10 MB** (single binary) |
| Runtime dependency | Node.js 20+ | **None** |
| Peak memory (build) | ~122 MB | **~23 MB** (5.3x less) |

### Feature comparison

| | Context Hub (JS) | Context7 | Chub (Rust) |
|---|---|---|---|
| Team features (pins, profiles, snapshots) | — | — | **Yes** |
| Self-learning agents (structured annotations) | — | — | **Yes** |
| 3-tier annotations (personal, team, org) | — | — | **Yes** |
| Agent config sync | — | — | **Yes** |
| Content integrity verification | — | — | **Yes** |
| Auto version detection (`--match-env`) | — | — | **Yes** |
| Self-hosted registry | Yes | — | **Yes** |
| Registry format compatibility | — | — | **Identical** |

---

## Content Registry

### Build your own docs

```sh
chub build ./content -o ./dist                             # build registry
chub build ./content --validate-only                       # validate only
chub build ./content --base-url https://cdn.example.com/v1 # with CDN URL
```

### Content format

```
content/
  <author>/
    docs/<entry-name>/
      <lang>/DOC.md                  # YAML frontmatter + markdown
      <lang>/<version>/DOC.md        # versioned variant
    skills/<entry-name>/
      SKILL.md
```

### Self-host

```sh
chub serve ./dist --port 4242        # serve as HTTP registry
```

Add your private registry as an additional source in `~/.chub/config.yaml` — no cloud required. See [Self-Hosting a Registry](docs/guide/self-hosting.md) for details.

---

## Test Suite

Comprehensive test coverage across unit, parity, and integration tests:

| Suite | Coverage |
|---|---|
| Search | BM25 scoring, tokenizer, inverted index, lexical boost |
| Frontmatter | YAML parsing, CRLF, BOM, edge cases |
| Annotations | Kind validation, severity, 3-tier storage |
| Build parity | Output format matches JS Context Hub byte-for-byte |
| Search parity | Multi-word, tags, descriptions match JS results |
| Team features | Pins, profiles, snapshots, detect, freshness, org HTTP |

```sh
cargo test --all                     # run all tests
```

---

## Documentation

Full documentation at [chub.nrl.ai](https://chub.nrl.ai):

- [Getting Started](https://chub.nrl.ai/guide/getting-started) — install and first commands
- [Installation](https://chub.nrl.ai/guide/installation) — all platforms and package managers
- [Why Chub](https://chub.nrl.ai/guide/why-chub) — comparison with Context Hub
- [Doc Pinning](https://chub.nrl.ai/guide/pinning) — lock doc versions
- [Context Profiles](https://chub.nrl.ai/guide/profiles) — role-scoped context
- [Team Annotations](https://chub.nrl.ai/guide/annotations) — shared knowledge
- [Project Context](https://chub.nrl.ai/guide/project-context) — custom docs
- [CLI Reference](https://chub.nrl.ai/reference/cli) — all commands and flags
- [Configuration](https://chub.nrl.ai/reference/configuration) — config file format
- [MCP Server](https://chub.nrl.ai/reference/mcp-server) — agent integration
- [Showcases](https://chub.nrl.ai/guide/showcases) — real-world usage examples

---

## Contributing

```sh
cargo build                          # debug build
cargo test --all                     # run tests
cargo fmt --all                      # format
cargo clippy --all -- -D warnings    # lint
```

---

## License

MIT
