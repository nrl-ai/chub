# Chub — Implementation Status & Roadmap

> **Vision**: The agent-agnostic context and tracking layer for AI-assisted development teams.
> Not just docs lookup — a shared, versioned, project-aware knowledge system with session tracking, cost analytics, and reasoning capture. Works with any AI coding agent. Makes every agent on your team as informed as your best engineer.

---

## Capability comparison

| Capability | Context Hub (JS) | Chub (Rust) |
|---|---|---|
| Public library docs | 1,553+ curated | 1,553+ curated |
| Custom/private docs | Yes (build cmd) | Yes (build cmd) |
| Offline mode | Yes (bundle) | Yes (bundle) |
| Team collaboration | No | **Yes** (pins, profiles, annotations) |
| Self-learning agents | No | **Yes** (structured annotation kinds, policy in CLAUDE.md) |
| Three-tier annotations | No | **Yes** (personal / team / org server) |
| Project awareness | No | **Yes** (auto-detect deps) |
| Agent config sync | No | **Yes** (CLAUDE.md, .cursorrules, AGENTS.md) |
| Agent integrations | No | **Yes** (MCP + agent config targets) |
| Git-tracked context | No | **Yes** (`.chub/` in repo) |
| Context profiles | No | **Yes** (role-scoped, with inheritance) |
| AI usage tracking | No | **Yes** (sessions, tokens, costs, models) |
| Offline telemetry | No | **Yes** (local JSONL journal, no external deps) |
| Self-hosted registry | Yes | Yes + `chub serve` |
| MCP server | 5 tools | **8 tools** (+ team + tracking) |
| CLI commands | 7 | **22** |
| Cold start | ~131 ms | **~44 ms** |
| Binary size | ~22 MB (node_modules) | **10 MB** (native) |

---

## Implementation Status

| Priority | Feature | Status | Notes |
|----------|---------|--------|-------|
| P0 | `chub init` + `.chub/` directory | **Done** | `init_project()` with `--from-deps` and `--monorepo` |
| P0 | Doc pinning (`pins.yaml`) | **Done** | CRUD via `chub pin add/remove/list/get` + MCP integration |
| P0 | Team annotations (git-tracked) | **Done** | Append semantics, write/read/merge + pin notices |
| P0 | Personal annotations | **Done** | Overwrite semantics, `~/.chub/annotations/` |
| P1 | Structured annotation kinds | **Done** | `issue`/`fix`/`practice`/`note`, severity, policy in agent configs |
| P1 | Org annotation server (Tier 3) | **Done** | REST API, bearer auth, TTL cache, auto-push, graceful degradation |
| P0 | Context profiles with inheritance | **Done** | `extends:` inheritance, circular detection, active profile |
| P1 | Custom project context | **Done** | Frontmatter parsing, `chub get project/<name>` |
| P1 | Dependency auto-detection | **Done** | All major package managers (see `docs/reference/cli.md` for list) |
| P1 | AGENTS.md / CLAUDE.md generation | **Done** | All targets listed in `docs/guide/agent-config.md` |
| P1 | Private registry + `chub serve` | **Done** | HTTP server via axum |
| P2 | Doc freshness monitoring | **Done** | `chub check` + `chub check --fix` |
| P2 | Doc bundles | **Partial** | Bundle struct + `create`/`install`/`list` commands; `publish` not wired |
| P2 | Smart context selection (`chub_context`) | **Done** | MCP tool returns profile rules + pinned docs + annotations in one call |
| P2 | Monorepo + path-scoped profiles | **Done** | `auto_profile` config with path globs |
| P3 | Local usage analytics | **Done** | `chub stats` with `--days`, JSONL storage, `chub telemetry` commands |
| P2 | AI usage tracking | **Done** | `chub track` — sessions, hooks, cost estimation, MCP tool, agent config |
| P3 | Doc snapshots | **Done** | `chub snapshot create/restore/diff/list` |
| P3 | Task-scoped ephemeral context | **Planned** | `chub_task_context` MCP tool |
| P3 | CI/CD integration | **Planned** | GitHub Actions, freshness checks, pin validation |
| P3 | Python/npm SDKs | **Partial** | npm wrapper done; Python CLI wrapper done; native Python API not started |
| P3 | IDE extensions | **Planned** | VS Code, JetBrains, Neovim |
| P2 | Agent integrations | **Done** | MCP server + agent config targets (see `docs/guide/agent-config.md`) |

### Test coverage

148 tests across 4 test suites, all passing:
- 47 unit tests (tokenizer, BM25, frontmatter, normalize, annotations, agent config)
- 15 build parity tests
- 20 search parity tests
- 66 team feature integration tests (isolated temp dirs, no repo pollution)

---

## Future Work

### AI Usage Tracking (`chub track`)

Track all AI coding agent activity per-project — sessions, prompts, tool calls, models, tokens, and costs.

**Done (Phase 1)**:
- Session model (`sessions.rs`): start/stop sessions, YAML summaries in `.chub/sessions/`, active state in `.git/chub-sessions/`
- Event journal (`session_journal.rs`): JSONL transcript in `.git/chub-sessions/` (local-only)
- Cost estimation (`cost.rs`): token-to-USD for Claude, GPT, Gemini, DeepSeek model families
- Hook handler (`chub track hook`): session-start, stop, prompt, pre-tool, post-tool, model-update
- CLI commands: `track status`, `track log`, `track show`, `track report`, `track export`, `track clear`

**Done (Phase 2)**:
- `chub track enable [agent]` — auto-install hooks in `.claude/settings.json`, `.cursor/hooks.json`
- `chub track disable` — cleanly remove hooks, preserving non-chub hooks
- Git hooks: `prepare-commit-msg` (add `Chub-Session:` trailer), `post-commit` (record commit hash)
- Resolved binary path in hook commands (no dependency on PATH)
- Error suppression (`2>/dev/null || true`) — broken hooks never block the IDE
- Hook chaining: existing git hooks backed up to `.pre-chub` and called after chub hooks
- Stdin JSON parsing from Claude Code / Cursor hook payloads (session_id, model, prompt, tool_input, tool_response)

**Done (Phase 3)**:
- Configurable cost rates in `.chub/config.yaml` (`tracking.cost_rates`)
- Custom rates override built-in rates (model name substring matching)
- Budget alert threshold (`tracking.budget_alert_usd`) with 80%/100% warnings in reports
- Config merges via three-tier system (personal → project → profile)

**Done (Phase 4)**:
- `chub_track` MCP tool — agents can query status, report, log, and session details
- AI Usage Tracking section in generated agent configs (`chub agent-config sync`)
- 8 MCP tools total (was 7)

### Task-scoped ephemeral context

`chub_task_context` — fetch relevant docs for a one-off task without modifying pins or the active profile:

```json
{ "task": "implement OAuth2 PKCE flow" }
```

Returns ranked docs relevant to the task description. Does NOT add to pins, does NOT change active profile. Useful for debugging sessions, exploring unfamiliar APIs, one-off tasks.

### CI/CD integration

```yaml
# GitHub Actions
- uses: nrl/chub-action@v1
  with:
    check-freshness: true      # fail if pinned docs lag installed packages
    validate-pins: true        # ensure all deps have pinned docs
    sync-agent-config: true    # regenerate CLAUDE.md if .chub/config.yaml changed
    fail-on-drift: true        # fail if generated files differ from committed
```

### Agent integrations (Done)

Two integration layers: MCP server (`chub mcp`) and agent config generation (`chub agent-config sync`). Claude Code also gets skills and a distributable plugin.

See `docs/guide/agent-config.md` for the full setup guide, tool list, and supported targets.

### IDE extensions

- **VS Code**: Sidebar panel, inline doc previews, profile switcher, annotation editor
- **JetBrains**: Same for IntelliJ/PyCharm/WebStorm
- **Neovim**: Telescope integration for `chub search` and `chub get`

### Bundle publish

`chub bundle publish <name>` — share bundles to the public registry as GitHub Gists or raw URLs:

```sh
chub bundle publish my-stack          # publish to registry
chub bundle install https://...       # install from URL
```

---

## Design Principles

1. **Git-first** — team config lives in the repo. If it's not in git, it doesn't exist for the team.
2. **Gradual adoption** — works for a solo developer today; adds team value when `.chub/` is committed.
3. **Three-tier inheritance** — personal → project → profile. Later tiers override; no tier is required.
4. **Agent-native** — every feature is accessible via MCP. CLI is for humans, MCP is for agents.
5. **Zero cloud dependency** — everything works offline and locally.
6. **Fast** — search in ~56ms, cold start in ~44ms.
