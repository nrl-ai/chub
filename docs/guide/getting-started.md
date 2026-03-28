# Getting Started

Get up and running with Chub in under 5 minutes. This guide covers the three pillars — context, tracking, and team knowledge — and how to set them all up.

## Install

Pick your preferred method:

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

See the full [Installation guide](/guide/installation) for binary downloads and platform-specific instructions.

Verify it works:

```sh
chub --version
```

## Search for docs

Chub serves curated API documentation from a public registry of 1,553+ docs. Search for anything:

```sh
chub search "stripe payments"
```

```
  1. stripe/api            Stripe API reference           ★ 0.92
  2. stripe/webhooks       Stripe webhook handling         ★ 0.78
  3. stripe/checkout       Stripe Checkout integration     ★ 0.71
```

## Fetch a doc

Grab a specific doc by ID. Use `--lang` to get language-specific content:

```sh
chub get openai/chat --lang python
```

This outputs the full markdown doc — ready to be consumed by an AI agent or read by a human.

```sh
# Other examples
chub get stripe/api --lang javascript
chub get nextjs/app-router --version 15.0
chub get openai/chat --lang python --version 4.0
```

## List all docs

```sh
chub list
```

Use `--json` with any command for machine-readable output:

```sh
chub list --json
```

## Set up MCP for your AI agent

Chub includes a built-in MCP (Model Context Protocol) server. This is how AI agents like Claude and Cursor access docs automatically.

::: code-group

```json [Claude Code (.mcp.json)]
{
  "mcpServers": {
    "chub": {
      "command": "chub",
      "args": ["mcp"]
    }
  }
}
```

```json [Cursor (.cursor/mcp.json)]
{
  "mcpServers": {
    "chub": {
      "command": "chub",
      "args": ["mcp"]
    }
  }
}
```

:::

Once configured, your AI agent can search and fetch docs without any manual commands.

## Initialize a project

Set up team sharing by creating a `.chub/` directory in your project:

```sh
chub init
```

This creates:

```
my-project/
├── .chub/
│   ├── config.yaml        # Project config
│   ├── pins.yaml          # Pinned docs
│   ├── annotations/       # Team-shared annotations
│   ├── context/           # Custom project docs
│   └── profiles/          # Named context profiles
```

::: tip Auto-detect dependencies
Use `--from-deps` to scan `package.json`, `Cargo.toml`, `requirements.txt`, etc. and auto-pin matching docs:

```sh
chub init --from-deps
```
:::

Commit `.chub/` to git so the whole team shares the same context. Personal settings stay in `~/.chub/`.

## Pin docs for your team

Lock specific doc versions so every team member and AI agent uses the same reference:

```sh
chub pin add openai/chat --lang python --version 4.0 --reason "Use v4 streaming API"
chub pin add stripe/api --lang javascript
```

List and fetch pinned docs:

```sh
chub pin list          # list all pins
chub pin get           # fetch all pinned docs at once
```

## Track your AI usage

Enable session tracking to see tokens, costs, and tool usage across any AI agent:

```sh
chub track enable                  # auto-detects Claude Code, Cursor, Copilot, etc.
```

Then use your AI agent as normal. After a session:

```sh
chub track status                  # see active session
chub track report                  # aggregate usage report (costs, tokens, models)
chub track dashboard               # web dashboard at localhost:4243
```

Session summaries are shared with your team via git. Full transcripts stay local. See [AI Usage Tracking](/guide/tracking) for details.

## Config inheritance

Chub uses a layered config file system — no layer is required. Settings cascade from personal defaults through project config to active role profiles:

```
~/.chub/config.yaml          # Personal defaults (machine-wide)
    ↓ overridden by
.chub/config.yaml            # Project config (git-tracked, shared with team)
    ↓ overridden by
.chub/profiles/<name>.yaml   # Role/task profile (e.g. backend, frontend)
```

## What to learn next

Now that you have Chub installed, explore the features that matter to your workflow:

| If you want to... | Read |
|---|---|
| Understand the vision | [Why Chub](/guide/why-chub) |
| **Context** | |
| Lock doc versions for your team | [Doc Pinning](/guide/pinning) |
| Give different roles different context | [Context Profiles](/guide/profiles) |
| Add custom project docs | [Project Context](/guide/project-context) |
| Auto-detect deps and pin docs | [Dep Auto-Detection](/guide/detect) |
| **Self-Learning** | |
| Share team knowledge in git | [Annotations & Self-Learning](/guide/annotations) |
| Rate doc quality for maintainers | [Feedback](/guide/feedback) |
| Sync CLAUDE.md / .cursorrules | [Agent Config Sync](/guide/agent-config) |
| **Tracking & Analytics** | |
| Track sessions, tokens, and costs | [AI Usage Tracking](/guide/tracking) |
| Monitor doc freshness | [Snapshots & Freshness](/guide/snapshots) |
| **Reference** | |
| See all CLI commands | [CLI Reference](/reference/cli) |
| Configure Chub | [Configuration](/reference/configuration) |
| Connect AI agents via MCP | [MCP Server](/reference/mcp-server) |
| Author docs and skills | [Content Guide](/guide/content-guide) |
| Build a custom private registry | [Self-Hosting](/guide/self-hosting) |
| Compare with Context Hub or Context7 | [Comparisons](/guide/vs-context-hub) |
