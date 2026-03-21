# Chub Plugin for Claude Code

Claude Code plugin that gives your AI agent access to the [chub](https://github.com/nrl-ai/chub) documentation registry via MCP.

## Install

```sh
npm install -g chub                                    # prerequisite
claude /plugin install https://github.com/nrl-ai/chub  # install plugin
```

## What you get

**MCP tools** (Claude calls these automatically):

| Tool | Purpose |
|------|---------|
| `chub_search` | Search docs by query, tags, or language |
| `chub_get` | Fetch a doc by ID (e.g. `serde/derive`) |
| `chub_list` | List all available docs |
| `chub_context` | Get pinned docs + profile rules + project context |
| `chub_pins` | Add/remove/list pinned docs |
| `chub_annotate` | Read/write team annotations |
| `chub_feedback` | Submit doc quality feedback |

**Skills** (slash commands):

| Command | Purpose |
|---------|---------|
| `/chub:docs <query>` | Search or fetch documentation |
| `/chub:annotate <id> <note>` | Record a team annotation |
| `/chub:setup` | Initialize chub for the current project |

## Local development

```sh
claude --plugin-dir ./claude-plugin
```
