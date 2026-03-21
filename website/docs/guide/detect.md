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

## Version matching

Use `chub get --match-env` to auto-detect the version of a dependency from your project's dep files and fetch the matching doc version:

```sh
# Reads openai version from requirements.txt / pyproject.toml
# and fetches the closest matching doc version
chub get openai/chat --lang python --match-env
```

This is especially useful when upgrading a library — you get the doc that matches what's actually installed, not what's pinned.

## Freshness

After pinning, use `chub check` to detect when pinned doc versions lag behind the library version installed in your project:

```sh
chub check         # Compare pinned vs installed versions
chub check --fix   # Auto-update outdated pins
```

See [Snapshots & Freshness](/guide/snapshots) for the full freshness and auditing workflow.
