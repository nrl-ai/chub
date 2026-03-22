# CLI Reference

Full command reference for Context Hub (`chub`).

## Global Flags

| Flag | Purpose |
|------|---------|
| `--json` | Structured JSON output (for agents and piping) |
| `--version` | Print CLI version |
| `--help` | Show help |

## chub search [query]

Search docs and skills. No query lists all entries.

| Flag | Purpose |
|------|---------|
| `--tags <csv>` | Filter by comma-separated tags |
| `--lang <language>` | Filter by language |
| `--type <type>` | Filter by type (`doc` or `skill`) |
| `--limit <n>` | Max results (default: 20) |

```bash
chub search                          # list everything
chub search "stripe"                 # fuzzy search by name/description
chub search stripe/payments          # exact id — shows full detail
chub search --tags automation        # filter by tag
chub search --type skill             # list only skills
```

**Exact ID match** returns the full entry detail (versions, languages, files). **Fuzzy search** returns a list of matches ranked by relevance.

## chub get \<ids...\>

Fetch one or more docs or skills by ID. Auto-detects type (doc vs skill). Auto-infers language when only one variant exists.

| Flag | Purpose |
|------|---------|
| `--lang <language>` | Language variant (js, py, ts, etc.) |
| `--version <version>` | Specific doc version |
| `--full` | Fetch all files, not just the entry point |
| `--file <paths>` | Fetch specific file(s) by path (comma-separated) |
| `-o, --output <path>` | Write to file or directory |
| `--pinned` | Fetch all pinned docs at once |

```bash
chub get stripe/api                  # single doc (auto-infers lang)
chub get openai/chat --lang py   # specific language
chub get pw-community/login-flows    # fetch a skill
chub get stripe/api openai/chat  # multiple entries
chub get stripe/api -o .context/     # save to file
```

### Incremental Fetch

When a doc has reference files beyond the main entry point, the output includes a footer:

```
---
Additional files available (use --file to fetch):
  references/advanced.md
  references/errors.md
Example: chub get acme/widgets --file references/advanced.md
```

Fetch only what you need:

```bash
chub get acme/widgets --file references/advanced.md       # one file
chub get acme/widgets --file advanced.md,errors.md         # multiple
chub get acme/widgets --full                               # everything
```

With `--json`, the response includes an `additionalFiles` array listing available reference files.

### Multi-Language Docs

If a doc is available in multiple languages and `--lang` is not specified, the CLI lists available languages and asks you to choose.

If a doc has only one language, `--lang` is not required — it's auto-inferred. If `--lang` is specified but unavailable for an entry, the CLI falls back to auto-selection rather than erroring.

## chub annotate [id] [note]

Attach persistent notes to a doc or skill. See [Feedback and Annotations](feedback-and-annotations.md) and [Agent Annotations](features/agent-annotations.md) for the full guide.

| Flag | Purpose |
|------|---------|
| `--clear` | Remove annotation (respects `--team`/`--org` flag) |
| `--list` | List all annotations |
| `--team` | Save as team annotation (git-tracked in `.chub/annotations/`) |
| `--personal` | Save as personal annotation only (default) |
| `--org` | Write to org annotation server (Tier 3) |
| `--author <name>` | Author name for team/org annotations |
| `--kind <kind>` | Annotation kind: `note` (default), `issue`, `fix`, `practice` |
| `--severity <level>` | Severity for issue annotations: `high`, `medium`, `low` |

```bash
chub annotate stripe/api "Use idempotency keys for POST requests"
chub annotate stripe/api                   # view current note (all tiers merged)
chub annotate stripe/api "new note"        # replaces previous
chub annotate stripe/api --clear           # remove personal annotation
chub annotate --list                       # list all personal annotations
chub annotate --team stripe/api "Team note"  # team annotation
chub annotate --list --org                 # list org annotations
chub annotate --kind issue --severity high stripe/api "Webhook sig fails"
```

## chub feedback [id] [rating] [comment]

Rate a doc or skill. Feedback is sent to the registry for maintainers. See [Feedback and Annotations](feedback-and-annotations.md) for details.

| Flag | Purpose |
|------|---------|
| `--label <label>` | Feedback label (repeatable) |
| `--lang <language>` | Language variant |
| `--file <file>` | Specific file within the entry |
| `--agent <name>` | AI tool name |
| `--model <model>` | LLM model name |
| `--status` | Show feedback and telemetry status |

Valid labels: `accurate`, `well-structured`, `helpful`, `good-examples`, `outdated`, `inaccurate`, `incomplete`, `wrong-examples`, `wrong-version`, `poorly-structured`.

```bash
chub feedback stripe/api up "Clear examples, well structured"
chub feedback openai/chat down --label outdated --label wrong-examples
```

## chub update

Download or refresh the cached registry from remote sources.

| Flag | Purpose |
|------|---------|
| `--force` | Re-download even if cache is fresh |
| `--full` | Download full bundle for offline use |

## chub cache status\|clear

Manage the local cache.

- `cache status` — shows cache info (sources, registries, sizes, last updated)
- `cache clear` — removes cached content

## chub build \<content-dir\>

Build a registry from a local content directory. See the [Content Guide](content-guide.md) for how to structure your content.

| Flag | Purpose |
|------|---------|
| `-o, --output <path>` | Output directory (default: `<content-dir>/dist`) |
| `--base-url <url>` | Base URL for remote serving |
| `--validate-only` | Validate content without building |
| `--no-incremental` | Disable incremental builds (copy all files) |

```bash
chub build my-content/                           # build to my-content/dist/
chub build my-content/ -o dist/                  # custom output dir
chub build my-content/ --validate-only           # validate only
```

## chub serve \<content-dir\>

Build and serve a content directory as a local HTTP registry.

| Flag | Purpose |
|------|---------|
| `-p, --port <port>` | Port to listen on (default: 4242) |
| `--host <host>` | Host to bind to (default: 127.0.0.1) |
| `-o, --output <path>` | Output directory for built content |

```bash
chub serve my-content/                           # serve at http://localhost:4242
chub serve my-content/ -p 8080                   # custom port
chub serve my-content/ --host 0.0.0.0            # expose to network
```

## chub init

Initialize a `.chub/` project directory in the current working directory. Creates the directory structure for pins, profiles, annotations, bundles, and snapshots.

## chub pin add\|remove\|list\|get

Manage pinned docs for the project.

```bash
chub pin add stripe/api                  # pin a doc
chub pin add openai/chat --lang python   # pin with language preference
chub pin list                            # list all pins
chub pin get                             # fetch all pinned docs at once
chub pin remove stripe/api               # unpin
```

## chub profile use\|list\|current

Manage context profiles with inheritance.

```bash
chub profile list                        # list available profiles
chub profile use backend                 # activate a profile
chub profile current                     # show active profile
```

## chub detect

Detect project dependencies and match them to available docs.

| Flag | Purpose |
|------|---------|
| `--pin` | Auto-pin all detected docs |

Supports: package.json, Cargo.toml, requirements.txt, pyproject.toml, Pipfile, go.mod, Gemfile, pom.xml, build.gradle, build.gradle.kts.

```bash
chub detect                              # show detected deps and matching docs
chub detect --pin                        # auto-pin matches
```

## chub agent-config

Generate or sync agent config files from `.chub/config.yaml` rules. See [Agent Integrations](integrations.md) for the full list of supported targets.

```bash
chub agent-config sync       # generate all configured targets
chub agent-config diff       # preview changes without writing
```

## chub check

Check freshness of pinned doc versions against installed dependency versions.

## chub context

Browse and query project context docs stored in `.chub/context/`.

## chub stats

Show usage analytics (fetch counts, most-used docs).

## chub bundle create\|install\|list

Manage doc bundles — shareable collections of docs.

```bash
chub bundle create my-stack --entries "openai/chat,stripe/api"
chub bundle install my-stack             # pin all entries from a bundle
chub bundle list                         # list available bundles
```

## chub snapshot create\|restore\|diff\|list

Manage doc snapshots for reproducible builds.

```bash
chub snapshot create v1.0                # capture current pins
chub snapshot list                       # list snapshots
chub snapshot diff v1.0 v1.1             # show what changed
chub snapshot restore v1.0               # restore exact pin versions
```

## chub track

Track AI coding agent sessions — tokens, costs, models, tools. See [Tracking](tracking.md) for full documentation.

```bash
chub track enable              # install hooks (auto-detect agent)
chub track enable claude-code  # install for specific agent
chub track enable --force      # overwrite existing hooks
chub track disable             # remove all hooks
chub track status              # show active session
chub track log                 # session history (30 days)
chub track log --days 7        # session history (7 days)
chub track show <session-id>   # session detail
chub track report              # aggregate usage report
chub track report --days 7     # report for last 7 days
chub track export              # JSON export for dashboards
chub track clear               # delete local transcripts
chub track dashboard           # web dashboard at localhost:4243
chub track dashboard --port 8080  # custom port
```

Supported agents: `claude-code`, `cursor`, `copilot`, `gemini-cli`, `codex`.

## chub mcp

Start the MCP stdio server for AI coding agents. See the [MCP Server reference](../website/docs/reference/mcp-server.md) for setup instructions.

## Piping Patterns

```bash
# Search, pick first result, fetch
ID=$(chub search "stripe" --json | jq -r '.results[0].id')
chub get "$ID" -o .context/stripe.md

# Fetch multiple docs at once
chub get openai/chat stripe/api -o .context/

# Check what additional files are available
chub get acme/widgets --json | jq '.additionalFiles'

# Fetch a specific reference file
chub get acme/widgets --file references/advanced.md

# List all annotations as JSON
chub annotate --list --json
```

## Configuration

Config lives at `~/.chub/config.yaml`:

```yaml
sources:
  - name: community
    url: https://cdn.aichub.org/v1
  - name: internal
    path: /path/to/local/docs

source: "official,maintainer,community"   # trust policy
refresh_interval: 86400                   # cache TTL in seconds (24h)
telemetry: true                           # anonymous usage analytics (passive)
feedback: true                            # allow chub feedback to send ratings (explicit)
```

### Telemetry

Anonymous usage analytics help improve the registry. No personally identifiable information is collected.

Opt out:
```yaml
telemetry: false
```
Or via environment variable: `CHUB_TELEMETRY=0`

### Feedback

The `chub feedback` command sends doc/skill ratings to maintainers. This is separate from telemetry — you can disable passive analytics while still being able to rate docs.

Opt out:
```yaml
feedback: false
```
Or via environment variable: `CHUB_FEEDBACK=0`

### Multi-Source

When multiple sources define the same entry ID, prefix with the source name to disambiguate:

```bash
chub get internal:openai/chat
```
