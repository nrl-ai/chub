# Dependency Auto-Detection

Scan your project's dependency files and auto-pin matching docs.

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
