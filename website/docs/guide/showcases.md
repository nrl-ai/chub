# Showcases

Real examples of using Chub with Claude Code, Cursor, and other AI coding agents.

## Express + OpenAI API project

Set up a Node.js project with auto-detected dependencies, pinned docs, team annotations, and MCP integration — in under a minute.

### 1. Create project and auto-detect deps

```sh
# Start with a standard package.json
cat package.json
```
```json
{
  "dependencies": {
    "express": "^4.21.0",
    "openai": "^4.67.0",
    "redis": "^4.7.0",
    "zod": "^3.23.0"
  }
}
```

```sh
# Initialize .chub/ and detect dependencies in one step
chub init --from-deps
```

```
Created .chub/ at ./my-app/.chub
  config.yaml
  pins.yaml
  profiles/base.yaml
  context/architecture.md
  annotations/

Detected 4 dependencies from project files:
  express     javascript (^4.21.0)
  openai      javascript (^4.67.0)
  redis       javascript (^4.7.0)
  zod         javascript (^3.23.0)

Run chub detect --pin to auto-pin detected docs.
```

### 2. Auto-pin matching docs

```sh
chub detect --pin
```

```
Detected 4 dependencies with 3 available docs:

  express  javascript (^4.21.0)  →  express/express  [pinnable]
  openai   javascript (^4.67.0)  →  openai/package   [pinnable]
  redis    javascript (^4.7.0)   →  redis/package    [pinnable]
  ✗ zod

Pinned 3 docs.
```

Chub matched 3 of 4 deps to curated docs and pinned them. The team now shares the same doc versions.

### 3. Fetch docs for the AI agent

```sh
# Search for relevant patterns
chub search "openai streaming"
```

```
5 results for "openai streaming":
  azure/openai      Azure OpenAI JavaScript SDK guide...
  openai/package    openai package guide for Python...
  kafka/streaming   KafkaJS - Apache Kafka client...
```

```sh
# Fetch the pinned OpenAI doc
chub get openai/chat --lang javascript | head -20
```

```markdown
# OpenAI API Coding Guidelines (JavaScript/TypeScript)

You are an OpenAI API coding expert. Help me with writing code
using the OpenAI API calling the official libraries and SDKs.

## Golden Rule: Use the Correct and Current SDK

Always use the official OpenAI Node.js SDK.
- NPM Package: `openai`
- Installation: npm install openai
```

### 4. Add team knowledge

```sh
# Annotate docs with team-specific guidance
chub annotate openai/chat "Always use streaming. Set max_tokens=4096." \
  --kind practice --team
```

Now when any team member or AI agent fetches this doc, they see:

```
---
⚠ USER-CONTRIBUTED ANNOTATIONS (not part of official documentation):
[Team practice — you (2026-03-21)] Always use streaming. Set max_tokens=4096.
```

### 5. Add project context

Create custom docs that the AI agent sees alongside public docs:

```sh
cat .chub/context/api-conventions.md
```

```markdown
# API Conventions

## Authentication
- All endpoints require Bearer token in Authorization header
- Tokens are JWTs signed with RS256

## Error Responses
- Always return { error: string, code: string } format
- Use HTTP status codes correctly

## OpenAI Integration
- Use streaming for all chat completions
- Set temperature=0.7 for creative tasks, 0 for deterministic
```

### 6. Connect to Claude Code

Add to `.mcp.json` in your project root:

```json
{
  "mcpServers": {
    "chub": {
      "command": "chub",
      "args": ["mcp"]
    }
  }
}
```

Claude Code now has access to these MCP tools:

| Tool | What it does |
|---|---|
| `chub_search` | Search 1,553+ curated docs |
| `chub_get` | Fetch a doc by ID with annotations |
| `chub_list` | Browse available docs |
| `chub_context` | Get pinned docs, annotations, and project context |
| `chub_pins` | Manage pinned doc versions |
| `chub_annotate` | Read/write agent notes |
| `chub_feedback` | Rate doc quality |

When Claude asks "how do I set up Stripe webhooks?", it can search, find the right doc, read your team's annotations, and follow your project conventions — all automatically.

---

## Python FastAPI + Stripe project

### 1. Auto-detect from requirements.txt

```sh
cat requirements.txt
```
```
fastapi==0.115.0
stripe==10.0.0
uvicorn==0.30.0
pydantic==2.9.0
```

```sh
chub init --from-deps
```

```
Detected 4 dependencies from project files:
  fastapi   python (0.115.0)
  stripe    python (10.0.0)
  uvicorn   python (0.30.0)
  pydantic  python (2.9.0)
```

```sh
chub detect --pin
```

```
Pinned 4 docs:
  fastapi/package    python
  stripe/payments    python
  uvicorn/package    python
  pydantic/settings  python
```

### 2. Create a backend profile

```yaml
# .chub/profiles/backend.yaml
name: backend
description: Backend API development
context:
  - api-conventions.md
rules:
  - "Use Pydantic v2 model_validator, not v1 validator"
  - "All Stripe webhook handlers must verify signatures"
  - "Use async def for all FastAPI endpoints"
```

### 3. Add Stripe-specific annotations

```sh
chub annotate stripe/payments "Use idempotency keys for all POST requests. Webhook endpoint must use raw body parsing — do not use JSONResponse middleware before it."
```

### 4. Fetch all pinned docs at once

```sh
chub get --pinned
```

This fetches all 4 pinned docs in one command — ready to paste into a prompt or serve via MCP.

---

## Rust project with Cargo.toml

Chub detects dependencies from `Cargo.toml` (including `[workspace.dependencies]`):

```sh
chub detect
```

```
Detected 8 dependencies with 5 available docs:

  tokio      rust (1.40)   →  tokio/runtime     [pinnable]
  serde      rust (1.0)    →  serde/package     [pinnable]
  reqwest    rust (0.12)   →  reqwest/package   [pinnable]
  sqlx       rust (0.8)    →  sqlx/package      [pinnable]
  clap       rust (4.5)    →  clap/package      [pinnable]
  ✗ thiserror
  ✗ anyhow
  ✗ tracing
```

```sh
chub detect --pin    # pin all 5
chub get --pinned    # fetch all at once
```

---

## Supported dependency files

Chub auto-detects from all major package managers:

| File | Language | Example |
|---|---|---|
| `package.json` | JavaScript/TypeScript | `"express": "^4.21.0"` |
| `Cargo.toml` | Rust | `[dependencies]` and `[workspace.dependencies]` |
| `requirements.txt` | Python | `fastapi==0.115.0` |
| `pyproject.toml` | Python | `[project.dependencies]` |
| `Pipfile` | Python | `[packages]` |
| `go.mod` | Go | `require github.com/gin-gonic/gin v1.10` |
| `Gemfile` | Ruby | `gem 'rails', '~> 7.0'` |
| `pom.xml` | Java (Maven) | `<dependency>` |
| `build.gradle` / `build.gradle.kts` | Java/Kotlin (Gradle) | `implementation 'org.springframework...'` |
