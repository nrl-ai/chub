# AI Usage Tracking

Track AI coding agent activity per-project — sessions, tool calls, models, tokens, reasoning, and estimated costs. Agent-agnostic: works with Claude Code, Cursor, Copilot, Gemini CLI, Codex, and more. Team-visible summaries are committed alongside code; full transcripts stay local.

## Quick Start

```sh
chub track enable          # install hooks (auto-detects your agent)
# ... use your AI agent as normal ...
chub track status          # see active session
chub track log             # session history
chub track report          # aggregate usage report
```

## How It Works

`chub track enable` installs lightweight hooks into your AI agent's config. These hooks fire on lifecycle events and record session data automatically — no changes to your workflow. Chub is **agent-agnostic**: the same tracking pipeline works across Claude Code, Cursor, Copilot, Gemini CLI, and Codex.

```mermaid
sequenceDiagram
    participant Agent as AI Agent
    participant Hook as chub track hook
    participant State as Session State
    participant Branch as Git Branches

    Agent->>Hook: session-start
    Hook->>State: Create session + entire.io state
    loop Each Turn
        Agent->>Hook: prompt
        Hook->>State: Increment turns
        Agent->>Hook: pre-tool (Read, Edit, ...)
        Hook->>State: Track tool calls
        Agent->>Hook: post-tool
        Hook->>State: Track files, tokens, reasoning
    end
    Agent->>Hook: stop / session-end
    Hook->>State: Parse transcript, calculate cost
    Hook->>Branch: Write session YAML to chub/sessions/v1
    Hook->>Branch: Write checkpoint to entire/checkpoints/v1
```

## Data Architecture

Chub stores tracking data using orphan git branches (similar to [entire.io](https://entire.io)):

### Team-Visible Summaries

Session summaries are stored on the `chub/sessions/v1` orphan branch and pushed to remote via the `pre-push` git hook. Checkpoints (with prompts and attribution) go to the `entire/checkpoints/v1` branch. Contains metadata only, never full transcripts.

```yaml
session_id: "2026-03-22T10-05-abc123"
agent: "claude-code"
model: "claude-opus-4-6"
started_at: "2026-03-22T10:05:00Z"
ended_at: "2026-03-22T10:42:00Z"
duration_s: 2220
turns: 14
tokens:
  input: 45000
  output: 12000
  cache_read: 8000
  cache_write: 3000
  reasoning: 5500        # extended thinking / reasoning tokens (if used)
tool_calls: 23
tools_used: ["Read", "Edit", "Bash", "Grep"]
files_changed: ["src/main.rs", "src/lib.rs"]
commits: ["abc1234", "def5678"]
est_cost_usd: 0.85
env:
  os: "windows"
  arch: "x86_64"
  branch: "main"
  repo: "my-project"
  git_user: "Jane Developer"
  chub_version: "0.1.15"
  extended_thinking: true    # set when reasoning tokens are detected
```

### Local-Only Data

- **Transcripts** — `.git/chub/transcripts/` — archived conversations, viewable in the dashboard
- **Session journals** — `.git/chub-sessions/` — JSONL event logs with prompts and tool details
- **Session state** — `.git/entire-sessions/` — entire.io-compatible session state

None of these are pushed. Use `chub track clear` to delete at any time.

### Git Trailers & Hooks

Commits during a tracked session automatically get trailers:
- `Chub-Session: <id>` — links the commit to its session
- `Chub-Checkpoint: <id>` — links the commit to its checkpoint on the orphan branch

The `pre-push` hook pushes `chub/sessions/v1` and `entire/checkpoints/v1` alongside your code. Rebase operations are detected and skipped.

## Supported Agents

| Agent | Config File | Status |
|-------|------------|--------|
| **Claude Code** | `.claude/settings.json` | Full support |
| **Cursor** | `.cursor/hooks.json` | Full support |
| **GitHub Copilot** | `.github/hooks/chub-tracking.json` | Full support |
| **Gemini CLI** | `.gemini/settings.json` | Full support |
| **Codex CLI** | `.codex/config.toml` | Full support |
| **Aider** | `.aider.conf.yml` | Detection only |
| **Windsurf** | IDE config | Detection only |
| **Cline** | `.clinerules/hooks/` | Detection only |

::: tip Auto-Detection
When you run `chub track enable` without specifying an agent, chub detects which agents are set up in your project by checking for `.claude/`, `.cursor/`, `.github/`, `.gemini/`, and `.codex/` directories.
:::

## Hook Events

### Claude Code

| Hook | What it tracks |
|------|---------------|
| `SessionStart` | Creates session, links transcript file |
| `Stop` / `SessionEnd` | Finalizes session, parses transcript for tokens, calculates cost |
| `UserPromptSubmit` | Records first prompt, increments turn count |
| `PreToolUse` | Increments step count and tool call counter |
| `PostToolUse` | Tracks file changes from Write/Edit tools |

### Cursor

| Hook | What it tracks |
|------|---------------|
| `sessionStart` | Creates session |
| `sessionEnd` / `stop` | Finalizes session |
| `beforeSubmitPrompt` | Tracks prompts |

### Gemini CLI

| Hook | What it tracks |
|------|---------------|
| `SessionStart` | Creates session |
| `SessionEnd` | Finalizes session |
| `BeforeTool` | Tracks tool calls |
| `AfterTool` | Tracks file changes |

### GitHub Copilot

| Hook | What it tracks |
|------|---------------|
| `sessionStart` | Creates session |
| `sessionEnd` | Finalizes session |
| `userPromptSubmitted` | Tracks prompts |
| `preToolUse` | Counts tool calls |
| `postToolUse` | Tracks file changes |

### Codex CLI

| Hook | What it tracks |
|------|---------------|
| `SessionStart` | Creates session |
| `Stop` | Finalizes session |
| `UserPromptSubmit` | Tracks prompts |
| `AfterToolUse` | Tracks tool usage |

### Git Hooks

| Hook | Purpose |
|------|---------|
| `prepare-commit-msg` | Adds `Chub-Session: <id>` trailer |
| `post-commit` | Snapshots session summary |

Existing git hooks are preserved — chub backs them up and chains execution.

## Cost Estimation

Chub estimates costs using built-in rates for popular models:

| Model | Input / 1M tokens | Output / 1M tokens | Reasoning / 1M tokens |
|-------|-------------------|-------------------|---------------------|
| Claude Opus | $15.00 | $75.00 | $75.00 |
| Claude Sonnet | $3.00 | $15.00 | $15.00 |
| Claude Haiku | $0.80 | $4.00 | $4.00 |
| GPT-4o | $2.50 | $10.00 | $10.00 |
| GPT-4o Mini | $0.15 | $0.60 | $0.60 |
| o3 / o1 | $10.00 | $40.00 | $40.00 |
| Gemini Pro | $1.25 | $5.00 | $5.00 |
| Gemini Flash | $0.075 | $0.30 | $0.30 |
| DeepSeek | $0.27 | $1.10 | $1.10 |

Reasoning tokens (extended thinking in Claude, chain-of-thought in o1/o3/Gemini) are tracked separately and priced at the output rate.

Override with custom rates in `.chub/config.yaml`:

```yaml
tracking:
  cost_rates:
    - model: "custom-model"
      input_per_m: 5.0
      output_per_m: 20.0
      cache_read_per_m: 0.5
      cache_write_per_m: 6.25
```

## Custom Cost Rates

The built-in rate table covers major models. For self-hosted models, fine-tuned variants, or providers with custom pricing, add your own rates in `.chub/config.yaml`:

```yaml
tracking:
  cost_rates:
    - model: "llama-3.1-70b"
      input_per_m: 0.88
      output_per_m: 0.88

    - model: "gpt-4-turbo-company"
      input_per_m: 10.0
      output_per_m: 30.0
      cache_read_per_m: 2.5       # defaults to input * 0.1 if omitted
      cache_write_per_m: 12.5     # defaults to input * 1.25 if omitted
```

Model matching is case-insensitive and matches on substring — `"gpt-4-turbo"` matches any model ID containing that string. Custom rates take priority over built-in rates.

### Reading cost reports

```sh
chub track report
```

```
Cost report (last 30 days):
  Total: $47.82 across 156 sessions

  By model:
    claude-opus-4-6     $32.10  (67%)  — 89 sessions
    claude-sonnet-4-6    $12.40  (26%)  — 52 sessions
    gpt-4o               $3.32   (7%)  — 15 sessions

  By agent:
    claude-code         $38.20  — 112 sessions
    cursor               $9.62  —  44 sessions

  Token breakdown:
    Input: 12.4M  Output: 3.2M  Reasoning: 1.8M  Cache read: 5.1M
```

### Cost optimization tips

- **Watch reasoning tokens** — extended thinking in Claude Opus costs $75/M tokens. If a session has high reasoning tokens for a simple task, consider using Sonnet instead
- **Check cache hit rates** — high `cache_read` relative to `input` means the agent is efficiently reusing context
- **Compare agents** — if Cursor and Claude Code produce similar results on your codebase, prefer the one with lower per-session cost

## Web Dashboard

Launch a local dashboard for visual session tracking:

```sh
chub track dashboard               # http://localhost:4243
chub track dashboard --port 8080   # custom port
```

### Dashboard features

- **Stat cards** — total sessions, total cost, token usage, and active session indicator at the top
- **Agent and model breakdown charts** — see which agents and models your team uses most, with cost proportions
- **Session history table** — sortable list of all sessions with agent, model, duration, tokens, and estimated cost
- **Conversation viewer** — click a session to view its transcript (from local data only)
- **Time range selector** — filter by last 7, 30, or 90 days
- **Theme toggle** — light and dark mode
- **Auto-refresh** — updates every 10 seconds while open

### JSON API

The dashboard exposes a REST API for scripting and custom integrations:

| Endpoint | Description |
|----------|-------------|
| `GET /api/status` | Current tracking status |
| `GET /api/sessions?days=N` | Session list |
| `GET /api/report?days=N` | Aggregate report |
| `GET /api/session?id=X` | Single session detail |
| `GET /api/entire-states` | entire.io session states |

```sh
# Fetch last 7 days of sessions as JSON
curl http://localhost:4243/api/sessions?days=7

# Get aggregate report
curl http://localhost:4243/api/report?days=30
```

## Multi-Agent Tracking

When multiple agents are enabled on the same project, each agent's sessions are tracked independently with their own session IDs. The report aggregates across all agents:

```sh
chub track enable               # installs hooks for all detected agents
chub track report               # shows combined stats, broken down by agent
```

Sessions are linked to commits via git trailers regardless of which agent created them, so `git log` shows which sessions produced which code.

## entire.io Compatibility

Chub writes session state compatible with [entire.io](https://entire.io), stored in `.git/entire-sessions/`. This means:

- `entire status` reads chub-tracked sessions
- `entire doctor` validates chub session states
- Both tools can coexist without conflict

## Privacy

- **Summaries** (`.chub/sessions/`): Metadata only. No prompts, no code. Safe to share.
- **Journals** (`.git/chub-sessions/`): Contains prompts and tool data. Local-only.
- **entire.io states** (`.git/entire-sessions/`): Local-only.

Run `chub track clear` to delete all local transcripts.

## CLI Reference

| Command | Description |
|---------|-------------|
| `chub track enable [agent]` | Install hooks (auto-detect if omitted) |
| `chub track enable --force` | Overwrite existing hooks |
| `chub track disable` | Remove all hooks |
| `chub track status` | Show active session and tracking state |
| `chub track log [--days N]` | Session history (default 30 days) |
| `chub track show <id>` | Full session details |
| `chub track report [--days N]` | Aggregate cost/token/model report |
| `chub track export [--days N]` | JSON export for external tools |
| `chub track clear` | Delete local transcripts |
| `chub track dashboard` | Launch web dashboard |

All commands support `--json` for structured output.
