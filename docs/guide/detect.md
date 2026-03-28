# Dependency Auto-Detection

Scan your project's dependency files and auto-pin matching docs. Detection runs automatically during project init when you use `chub init --from-deps`, or you can run it standalone at any time.

## Usage

```sh
# Show detected dependencies with matching docs
chub detect

# Auto-pin all detected docs
chub detect --pin
```

## Supported files

| File | Language |
|---|---|
| `package.json` | JavaScript/TypeScript |
| `Cargo.toml` | Rust |
| `requirements.txt` | Python |
| `pyproject.toml` | Python |
| `Pipfile` | Python |
| `go.mod` | Go |
| `Gemfile` | Ruby |
| `pom.xml` | Java (Maven) |
| `build.gradle` / `build.gradle.kts` | Java/Kotlin (Gradle) |

## Example output

```
Detected 6 dependencies with available docs:

  openai (python)     → openai/chat [pinnable]
  stripe (python)     → stripe/api [pinnable]
  fastapi (python)    → fastapi/app [pinnable]
  ✗ custom-lib        → no match

Pin all? chub detect --pin
```

## How matching works

Detection uses **name-based matching** against the Chub registry:

1. **Exact match** — dependency name matches the first segment of an entry ID (e.g. `openai` matches `openai/chat`) — confidence 1.0
2. **Full ID match** — dependency name matches the full entry ID
3. **Partial match** — substring match (minimum 4 characters to avoid false positives) — confidence 0.5

Detection does not resolve versions — it identifies which docs are available for your dependencies. Use `--pin` to pin them, then set specific versions with `chub pin add ... --version`.

### Unmatched dependencies

Not every dependency has a matching doc in the registry. This is normal:

```
Detected 12 dependencies:
  openai (python)     → openai/chat [pinnable]
  stripe (python)     → stripe/api [pinnable]
  fastapi (python)    → fastapi/app [pinnable]
  ✗ python-dotenv     → no match
  ✗ custom-lib        → no match
```

Unmatched dependencies are skipped silently with `--pin`. You can author docs for internal libraries and host them on a [private registry](/guide/self-hosting).

## Workspace and monorepo support

Detection scans dependency files at the project root. Specific ecosystem support:

| Ecosystem | Workspace support |
|---|---|
| **Cargo** | Reads `[workspace.dependencies]` from root `Cargo.toml` |
| **npm / pnpm** | Reads root `package.json` only |
| **Python** | Reads root `requirements.txt`, `pyproject.toml`, `Pipfile` |
| **Go** | Reads root `go.mod` |
| **Ruby** | Reads root `Gemfile` |
| **Java** | Reads root `pom.xml`, `build.gradle`, `build.gradle.kts` |

For monorepos with per-package dependency files, run `chub detect` from each package directory, or consolidate shared dependencies in the root manifest.

## Version matching with `--match-env`

Use `chub get --match-env` to auto-detect the version of a dependency from your project's dep files and fetch the matching doc version:

```sh
# Reads openai version from requirements.txt / pyproject.toml
# and fetches the closest matching doc version
chub get openai/chat --lang python --match-env
```

This is especially useful when upgrading a library — you get the doc that matches what's actually installed, not what's pinned.

## Combining with other features

```sh
# Detect and pin in one step
chub detect --pin

# Then check freshness as dependencies are updated
chub check
chub check --fix

# Or install a bundle for deps that detection misses
chub bundle install project-extras
```

See [Snapshots & Freshness](/guide/snapshots) for the full freshness and auditing workflow.
