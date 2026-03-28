<p align="center">
  <img src="https://raw.githubusercontent.com/nrl-ai/chub/main/website/assets/logo.svg" width="80" height="80" alt="Chub">
</p>

<h1 align="center">Chub</h1>

<p align="center">
  <strong>The all-in-one infrastructure layer for AI-assisted development.</strong><br>
  <em>Context · Tracking · Security · Team knowledge — agent-agnostic, git-native, built in Rust.</em>
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

AI coding agents are powerful — but the infrastructure around them is missing:

- **No context** — Agents hallucinate APIs, use deprecated endpoints, and forget what they learned between sessions
- **No visibility** — You have no idea what AI is costing your team, which agents are being used, or how many tokens they consume
- **No memory** — When an agent discovers a gotcha, that knowledge evaporates. Next week, a teammate's agent hits the same issue
- **No guardrails** — Agents generate code with hardcoded secrets, paste API keys into transcripts, and commit credentials. Traditional scanners miss secrets buried in AI chat logs and agent transcripts

These aren't four separate problems. They're one: **there's no infrastructure layer for AI coding agents.**

## The Solution

Chub is the all-in-one agent layer — context, tracking, security, and analytics in a single CLI + MCP server. Built in Rust, agent-agnostic, git-native.

<p align="center">
  <img src="https://raw.githubusercontent.com/nrl-ai/chub/main/website/assets/architecture.svg" width="700" alt="Chub Architecture — Context, Tracking, Learning">
</p>

Built on [Context Hub](https://github.com/andrewyng/context-hub) by Andrew Ng — Chub is a high-performance Rust rewrite that extends the original with team features, self-learning agents, session tracking, and cost analytics.

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

Add to `.mcp.json` (Claude Code), `.cursor/mcp.json` (Cursor), or the equivalent for your agent:

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

## AI Usage Tracking

Track every AI coding session — tokens, costs, models, tool calls, reasoning — across all your agents. One command to enable, works with any supported agent.

```sh
chub track enable                        # auto-detect and install hooks
# ... use your AI agent as normal ...
chub track status                        # see active session
chub track report --days 7               # last week's usage: costs, tokens, models
chub track dashboard                     # web dashboard at localhost:4243
```

Supported agents: **Claude Code**, **Cursor**, **GitHub Copilot**, **Gemini CLI**, **Codex**.

- **Session lifecycle** — start, prompts, tool calls, commits, stop — all recorded automatically
- **Cost estimation** — built-in rates for Claude, GPT, Gemini, DeepSeek, o1/o3; custom rate overrides
- **Budget alerts** — configurable thresholds with 80%/100% warnings
- **Team visibility** — session summaries shared via git; full transcripts stay local
- **Web dashboard** — charts, breakdowns, session history, transcript viewer
- **entire.io compatible** — session states readable by `entire status`

---

## Security Scanning

Drop-in replacement for gitleaks and betterleaks — with built-in strength for AI agent transcript scanning. 73+ secret detection rules, Shannon entropy analysis, stopword filtering, and structured output.

```sh
chub scan secrets git                        # scan git history for secrets
chub scan secrets git --staged               # pre-commit hook mode
chub scan secrets dir ./src                  # scan a directory
echo "$CONTENT" | chub scan secrets stdin    # scan from stdin
```

### Why another scanner?

AI coding agents introduce a new class of secret leak: keys pasted into prompts, API tokens in agent transcripts, credentials in chat logs. Traditional scanners focus on source files and git commits — they miss the AI layer entirely. Chub scans all of it.

### Features

- **73+ built-in rules** — AWS, GCP, Azure, GitHub, GitLab, Anthropic, OpenAI, Stripe, and dozens more
- **AI transcript-aware** — detects secrets in agent chat logs, prompts, and tool call outputs
- **Shannon entropy** — reduces false positives by measuring randomness of captured secrets
- **Stopword filtering** — ignores placeholders like `your_api_key_here` and `${VARIABLE}`
- **Gitleaks-compatible output** — JSON, SARIF, CSV formats; same Finding schema
- **Gitleaks-compatible config** — reads `.gitleaks.toml`, `.betterleaks.toml`, `.chub-scan.toml`
- **Baseline management** — filter out known findings with `--baseline-path`
- **Pre-commit integration** — `--staged` flag for git pre-commit hooks

### Output formats

```sh
chub scan secrets git -f json -r report.json     # JSON (default)
chub scan secrets git -f sarif -r report.sarif    # SARIF (for CI/CD)
chub scan secrets git -f csv -r report.csv        # CSV
chub scan secrets git --redact                     # redact secrets in output
```

### Config file

Create `.chub-scan.toml` (or `.gitleaks.toml`) in your repo:

```toml
title = "My project scan config"

[allowlist]
paths = ["test/fixtures/.*", ".*_test\\.go"]
regexes = ["EXAMPLE"]

[[rules]]
id = "custom-internal-token"
regex = '''myco-tk-[a-zA-Z0-9]{32}'''
keywords = ["myco-tk-"]
```

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

### Security Scanning

```sh
chub scan secrets git                           # scan git history
chub scan secrets git --staged                  # scan staged files (pre-commit)
chub scan secrets dir ./src                     # scan directory
chub scan secrets stdin --label "transcript"    # scan stdin
chub scan secrets git -f sarif -r report.sarif  # SARIF output for CI
chub scan secrets git --redact                  # redact secrets in output
chub scan secrets git -c .gitleaks.toml         # use custom config
chub scan secrets git --baseline-path known.json # filter known findings
```

### Cache Management

```sh
chub update                             # refresh cached registry
chub cache status                       # show cache state
chub cache clear                        # clear local cache
```

---

## MCP Server

<p align="center">
  <img src="https://raw.githubusercontent.com/nrl-ai/chub/main/website/assets/dataflow.svg" width="700" alt="Chub MCP Dataflow — Agent → Chub → Registry">
</p>

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

Chub includes proactive security scanning and defensive measures for AI-assisted development:

- **Secret scanning** — 73+ rules detect leaked credentials in source code, git history, directories, and stdin. Shannon entropy and stopword filtering reduce false positives. Drop-in replacement for gitleaks/betterleaks with native AI transcript awareness.
- **Agent transcript protection** — Secrets pasted into AI agent prompts, chat logs, and tool outputs are detected — a blind spot for traditional scanners.
- **Automatic redaction** — Session tracking automatically redacts secrets from stored transcripts using the same 73-rule engine.
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
| **Context** | | | |
| Team features (pins, profiles, snapshots) | — | — | **Yes** |
| Agent config sync (10 targets) | — | — | **Yes** |
| Auto version detection (`--match-env`) | — | — | **Yes** |
| Content integrity verification | — | — | **Yes** |
| Self-hosted registry | Yes | — | **Yes** |
| Registry format compatibility | — | — | **Identical** |
| **Self-Learning** | | | |
| Structured annotations (issue/fix/practice) | — | — | **Yes** |
| 3-tier annotations (personal, team, org) | — | — | **Yes** |
| **Tracking & Analytics** | | | |
| Session tracking | — | — | **Yes** |
| Cost estimation & budget alerts | — | — | **Yes** |
| Web dashboard | — | — | **Yes** |
| Multi-agent support (6+ agents) | — | — | **Yes** |
| **Security** | | | |
| Secret scanning (git/dir/stdin) | — | — | **Yes** |
| AI transcript scanning | — | — | **Yes** |
| Gitleaks/betterleaks compatible | — | — | **Drop-in** |
| SARIF output for CI/CD | — | — | **Yes** |
| Pre-commit hook support | — | — | **Yes** |

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
| Secret scanning | 73+ rules, entropy, false positives, AI transcripts, git/dir/stdin, SARIF/CSV/JSON output, baseline filtering |

```sh
cargo test --all                     # run all tests
```

---

## Documentation

Full documentation at [chub.nrl.ai](https://chub.nrl.ai):

- [Getting Started](https://chub.nrl.ai/guide/getting-started) — install and first commands
- [Installation](https://chub.nrl.ai/guide/installation) — all platforms and package managers
- [Why Chub](https://chub.nrl.ai/guide/why-chub) — the vision: context + tracking + learning
- [Doc Pinning](https://chub.nrl.ai/guide/pinning) — lock doc versions
- [Context Profiles](https://chub.nrl.ai/guide/profiles) — role-scoped context
- [Team Annotations](https://chub.nrl.ai/guide/annotations) — shared knowledge
- [Project Context](https://chub.nrl.ai/guide/project-context) — custom docs
- [CLI Reference](https://chub.nrl.ai/reference/cli) — all commands and flags
- [Configuration](https://chub.nrl.ai/reference/configuration) — config file format
- [MCP Server](https://chub.nrl.ai/reference/mcp-server) — agent integration
- [AI Usage Tracking](https://chub.nrl.ai/guide/tracking) — session tracking and cost analytics
- [Secret Scanning](https://chub.nrl.ai/guide/scanning) — security scanning for secrets in code, git, and AI transcripts
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
