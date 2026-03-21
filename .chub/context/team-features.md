---
name: Team Features Guide
description: "How team features work and how to extend them"
tags: team, features, guide
---

# Team Features

## Overview

Team features enable shared, git-tracked context for AI coding agents. Everything lives in `.chub/` and is committed to the repo.

## Feature Map

| Feature | Module | CLI Command |
|---------|--------|-------------|
| Doc Pinning | `team::pins` | `chub pin add/remove/list` |
| Team Annotations | `team::team_annotations` | `chub annotate --team` |
| Project Context | `team::context` | `chub context --list`, `chub get project/<name>` |
| Profiles | `team::profiles` | `chub profile use/list/current` |
| Dep Detection | `team::detect` | `chub detect [--pin]` |
| Agent Config | `team::agent_config` | `chub agent-config generate/sync/diff` |
| Freshness | `team::freshness` | `chub check [--fix]` |
| Analytics | `team::analytics` | `chub stats [--days N]` |
| Snapshots | `team::snapshots` | `chub snapshot create/restore/diff/list` |
| Bundles | `team::bundles` | `chub bundle create/install/list` |
| HTTP Server | CLI only | `chub serve <dir> [-p port]` |

## Adding a New Team Feature

1. Create module in `crates/chub-core/src/team/new_feature.rs`
2. Add `pub mod new_feature;` to `team/mod.rs`
3. Create CLI command in `crates/chub-cli/src/commands/new_feature.rs`
4. Add to `Commands` enum in `main.rs`
5. Add MCP tool if agents need access

## Config Inheritance

```
~/.chub/config.yaml          # Personal defaults
    ↓ overridden by
.chub/config.yaml            # Project config (git-tracked)
    ↓ overridden by
.chub/profiles/<name>.yaml   # Role/task profile
```
