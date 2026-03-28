# Doc Bundles

Bundles are curated, shareable collections of docs that can be installed in one command. Instead of telling a new teammate "pin these 8 entries," hand them a bundle file.

## Quick Start

```sh
# Create a bundle from your current pins
chub bundle create api-stack \
  --entries "openai/chat,stripe/api,serde/derive,axum/routing" \
  --description "Backend API development stack" \
  --author "Alice"

# Install it (pins all entries)
chub bundle install api-stack

# List available bundles
chub bundle list
```

## How It Works

A bundle is a YAML file in `.chub/bundles/` that lists entry IDs. When installed, each entry is pinned with a reason linking back to the bundle. Bundles are git-tracked, so the whole team gets them on pull.

```
.chub/
  bundles/
    api-stack.yaml
    ml-pipeline.yaml
    frontend.yaml
```

## Creating Bundles

```sh
chub bundle create <name> --entries "<id>,<id>,..." [--description "..."] [--author "..."] [--notes "..."]
```

Entries are comma-separated doc IDs from the registry. The command creates `.chub/bundles/<name>.yaml`:

```yaml
# .chub/bundles/api-stack.yaml
name: api-stack
description: Backend API development stack
author: Alice
entries:
  - openai/chat
  - stripe/api
  - serde/derive
  - axum/routing
notes: Standard stack for all backend services
```

You can also create bundle files by hand — the format is simple YAML.

## Installing Bundles

```sh
# By name (looks in .chub/bundles/)
chub bundle install api-stack

# By file path (useful for shared bundles outside the project)
chub bundle install ~/company-bundles/onboarding.yaml
```

Each entry is pinned with the reason `From bundle: <name>`, so `chub pin list` shows where each pin came from. Entries that are already pinned are skipped. If an entry ID doesn't exist in the registry, a warning is printed and the remaining entries continue.

## Listing Bundles

```sh
chub bundle list
```

Shows all bundles in `.chub/bundles/` with their descriptions and entry counts:

```
api-stack       Backend API development stack              (4 entries)
ml-pipeline     ML/data processing dependencies            (6 entries)
frontend        React + styling libraries                  (5 entries)
```

## Team Workflows

### Onboarding

Create a bundle per role so new teammates get the right context immediately:

```yaml
# .chub/bundles/backend-onboarding.yaml
name: backend-onboarding
description: Everything a new backend engineer needs
author: Platform Team
entries:
  - openai/chat
  - stripe/api
  - serde/derive
  - axum/routing
  - tokio/runtime
  - sqlx/query
notes: Install on your first day. See also .chub/profiles/backend.yaml for agent rules.
```

```sh
# New teammate runs:
chub bundle install backend-onboarding
chub profile use backend
```

### Project Bootstrap

Combine `chub detect` (auto-detect from dependencies) with a bundle for entries that aren't auto-detected:

```sh
# Auto-pin from package.json / Cargo.toml / etc.
chub detect --pin

# Add the extras that detection misses
chub bundle install project-extras
```

### Sharing Across Repos

Bundle files are plain YAML, so you can share them outside `.chub/bundles/`:

```sh
# Install from an absolute path
chub bundle install /shared/company-standards.yaml

# Install from a teammate's repo
chub bundle install ../other-project/.chub/bundles/common.yaml
```

## Bundles vs. Profiles

Both help configure what docs an agent sees, but they serve different purposes:

| | Bundles | Profiles |
|---|---|---|
| **What** | A list of entry IDs to pin | Pins + context docs + agent rules |
| **When** | One-time setup (install and done) | Active selection (switch between roles) |
| **Scope** | Just pins | Pins, rules, and project context |
| **Typical use** | Onboarding, project bootstrap | Day-to-day role switching |

Use bundles to seed the initial set of pins. Use profiles to switch between different working contexts.

## JSON Output

All bundle commands support `--json` for scripting:

```sh
chub bundle list --json
chub bundle create mystack --entries "a/b,c/d" --json
chub bundle install mystack --json
```
