# Secret Scanning

Chub includes a built-in secret scanner — a drop-in replacement for [gitleaks](https://github.com/gitleaks/gitleaks) and [betterleaks](https://github.com/nicholasgasior/betterleaks), with native awareness of AI agent transcripts.

## Why another scanner?

Traditional secret scanners focus on source code and git history. But AI coding agents introduce new leak vectors:

- Developers paste API keys into agent prompts
- Agents include credentials in tool call outputs and chat responses
- Transcripts with secrets get committed or stored locally
- Agent-generated code hardcodes secrets copied from environment

Chub's scanner catches all of these. The same 260-rule detection engine that automatically redacts secrets from stored session transcripts also powers the `chub scan` command — one engine, two applications.

## Quick start

```sh
# Scan git history
chub scan secrets git

# Scan only staged files (pre-commit hook)
chub scan secrets git --staged

# Scan a directory
chub scan secrets dir ./src

# Scan from stdin (agent transcript, pipe, etc.)
cat transcript.log | chub scan secrets stdin --label "claude-session"
```

## Detection rules

Chub ships with 260 built-in rules covering:

| Category | Examples |
|---|---|
| **Cloud providers** | AWS access key/secret, GCP API key, Azure AD client secret |
| **AI/LLM services** | OpenAI, Anthropic, DeepSeek, Mistral, xAI, Cerebras, TogetherAI, OpenRouter |
| **Source control** | GitHub PAT/fine-grained/OAuth, GitLab PAT/pipeline, Bitbucket |
| **DevOps** | Heroku, Netlify, Fly.io, Vercel, HashiCorp Terraform/Vault, Pulumi, Doppler |
| **Payments** | Stripe secret/publishable key |
| **Observability** | Grafana API/cloud/service, Sentry DSN, New Relic, Datadog, Databricks |
| **SaaS** | Slack (bot/app/webhook), Twilio, Shopify, Postman, Linear, Notion, Snyk |
| **Package registries** | npm, PyPI, RubyGems, Clojars |
| **Auth/crypto** | Private keys (PEM), JWT, age secret keys |

Each rule uses regex matching combined with:

- **Keyword pre-filtering** — only scan lines containing relevant keywords (fast path)
- **Shannon entropy** — measures randomness of captured secrets to filter low-entropy false positives
- **Stopword filtering** — ignores common placeholders like `your_api_key_here`, `${VARIABLE}`, `changeme`

## Output formats

### JSON (default)

Gitleaks-compatible JSON array of findings:

```sh
chub scan secrets git -f json
chub scan secrets git -f json -r report.json    # write to file
```

### SARIF

Static Analysis Results Interchange Format — for CI/CD integration (GitHub Code Scanning, Azure DevOps, etc.):

```sh
chub scan secrets git -f sarif -r report.sarif
```

### CSV

Spreadsheet-friendly format:

```sh
chub scan secrets git -f csv -r report.csv
```

## Redaction

Mask secrets in output:

```sh
chub scan secrets git --redact          # 100% redacted (default)
chub scan secrets git --redact 50       # 50% redacted
```

## Pre-commit hook

Add to `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: chub-scan
        name: chub secret scan
        entry: chub scan secrets git --staged --no-banner
        language: system
        pass_filenames: false
```

Or use a git hook directly in `.git/hooks/pre-commit`:

```sh
#!/bin/sh
chub scan secrets git --staged --exit-code 1 --no-banner
```

## Baseline management

Filter out known, accepted findings:

```sh
# Generate a baseline from current state
chub scan secrets git -r baseline.json

# Future scans ignore baseline findings (matched by fingerprint)
chub scan secrets git --baseline-path baseline.json
```

## Configuration

Chub reads gitleaks-compatible TOML config files. It searches for config in this order:

1. Explicit `--config` path
2. Environment variables: `CHUB_SCAN_CONFIG`, `BETTERLEAKS_CONFIG`, `GITLEAKS_CONFIG`
3. `.chub-scan.toml` in the target directory
4. `.betterleaks.toml` in the target directory
5. `.gitleaks.toml` in the target directory

### Example config

```toml
title = "My project scan config"

[extend]
useDefault = true          # include built-in rules (default: true)
disabledRules = ["jwt"]    # disable specific built-in rules

[allowlist]
description = "Global allowlist"
paths = ["test/fixtures/.*", ".*_test\\.go"]
regexes = ["EXAMPLE", "PLACEHOLDER"]
stopwords = ["dummy", "fake"]

[[rules]]
id = "internal-service-token"
description = "Internal service API token"
regex = '''myco-svc-[a-zA-Z0-9]{40}'''
keywords = ["myco-svc-"]
tags = ["internal"]
entropy = 3.5

[[rules.allowlists]]
description = "Test fixtures"
condition = "OR"
paths = [".*test.*"]
regexes = ["test-token"]
```

### Config fields

| Field | Description |
|---|---|
| `title` | Config title (informational) |
| `extend.useDefault` | Include built-in rules (default: `true`) |
| `extend.disabledRules` | Rule IDs to disable from built-in set |
| `allowlist.paths` | Regex patterns for paths to skip |
| `allowlist.regexes` | Regex patterns to allowlist in content |
| `allowlist.stopwords` | Substrings that suppress a finding |
| `allowlist.commits` | Commit SHAs to skip |
| `rules[].id` | Rule identifier |
| `rules[].regex` | Detection regex |
| `rules[].secretGroup` | Capture group index for the secret (default: 1; 0 = full match fallback) |
| `rules[].entropy` | Minimum Shannon entropy for the secret |
| `rules[].keywords` | Keywords for fast pre-filtering |
| `rules[].path` | File path regex filter |
| `rules[].allowlists` | Per-rule allowlists |

## CI/CD integration

### GitHub Actions

```yaml
- name: Scan for secrets
  run: chub scan secrets git -f sarif -r results.sarif --no-banner

- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: results.sarif
```

### Generic CI

```sh
chub scan secrets git --exit-code 1 --no-banner
```

The scanner exits with code 1 (configurable via `--exit-code`) when secrets are found, and 0 when clean.

## Writing custom rules

The 260 built-in rules cover common services, but you'll often need project-specific rules for internal tokens, service keys, or proprietary formats.

### Rule anatomy

```toml
[[rules]]
id = "internal-service-token"          # Unique ID (used in allowlists and disabledRules)
description = "Internal service token" # Human-readable description
regex = '''myco-svc-[a-zA-Z0-9]{40}''' # Detection regex
keywords = ["myco-svc-"]              # Fast pre-filter (must appear in line for regex to run)
tags = ["internal"]                    # Optional tags for categorization
entropy = 3.5                          # Optional: minimum Shannon entropy for the match
secretGroup = 1                        # Optional: regex capture group containing the secret (default: 1; betterleaks convention)
```

### Step-by-step example: detecting an internal API key

Suppose your company issues API keys like `ACME-KEY-a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4`:

```toml
[[rules]]
id = "acme-api-key"
description = "ACME Corp internal API key"
regex = '''ACME-KEY-[a-f0-9]{32}'''
keywords = ["ACME-KEY-"]
```

**How it works:**
1. **Keyword pre-filter** — only lines containing `ACME-KEY-` are tested (fast path, skips 99%+ of lines)
2. **Regex match** — the full pattern validates the key format
3. **No entropy needed** — the `ACME-KEY-` prefix is distinctive enough

### Using entropy filtering

Entropy measures randomness. A string like `aaaaaa` has low entropy; `a8Kf2mQ9` has high entropy. Set an entropy threshold to filter out placeholder values that match the regex pattern but aren't real secrets:

```toml
[[rules]]
id = "generic-token-header"
description = "Bearer token in Authorization header"
regex = '''(?i)authorization:\s*bearer\s+([a-zA-Z0-9._\-]{20,})'''
keywords = ["authorization", "bearer"]
secretGroup = 1             # capture group 1 = the token value
entropy = 3.5               # reject low-entropy matches like "xxxxxxxxxxxxxxxxxxxx"
```

**Entropy guidelines:**
- `3.0` — loose (catches most real secrets, some false positives)
- `3.5` — balanced (good default for token-like patterns)
- `4.0` — strict (may miss some real secrets with repetitive patterns)

### Per-rule allowlists

Suppress false positives for specific rules without affecting other rules:

```toml
[[rules]]
id = "internal-service-token"
description = "Internal service token"
regex = '''myco-svc-[a-zA-Z0-9]{40}'''
keywords = ["myco-svc-"]

[[rules.allowlists]]
description = "Test fixtures use fake tokens"
condition = "OR"                    # Match ANY of the following
paths = [".*test.*", ".*fixture.*"]
regexes = ["fake-token", "test-token"]
```

### Testing custom rules

Validate your rules before committing by scanning a known file:

```sh
# Create a test file with a known secret
echo 'token = "ACME-KEY-a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4"' > /tmp/test-secret.txt  # gitleaks:allow

# Scan it with your config
chub scan secrets dir /tmp/test-secret.txt --config .chub-scan.toml

# Clean up
rm /tmp/test-secret.txt
```

### Disabling built-in rules

If a built-in rule produces false positives in your codebase, disable it rather than fighting it:

```toml
[extend]
useDefault = true
disabledRules = [
  "generic-api-key",        # too broad for our codebase
  "jwt",                    # we store test JWTs in fixtures
]
```

## Troubleshooting

### Too many false positives

1. **Check which rule fires** — the JSON output includes the `ruleID` field for each finding
2. **Add to allowlist** — add the pattern or path to `[allowlist]`
3. **Add stopwords** — for placeholder patterns like `example_key_here`
4. **Raise entropy** — increase the rule's `entropy` threshold
5. **Disable the rule** — as a last resort, add to `disabledRules`

### Scanning is slow

- **Use `--staged`** for pre-commit hooks (scans only the diff, not full history)
- **Add path allowlists** — skip `vendor/`, `dist/`, `node_modules/`, generated files
- **Use keyword pre-filtering** — rules with `keywords` skip lines that don't contain the keyword

### Scanner exits 0 but secrets exist

- Check if the secret pattern matches the rule's regex
- Check if entropy filtering is rejecting the match (try lowering or removing `entropy`)
- Check if an allowlist is suppressing the finding

## Relationship to transcript redaction

Chub's tracking system automatically redacts secrets from stored session transcripts using the same 260-rule engine. The scanner and the redactor share their rule definitions — when a new detection rule is added, both scanning and redaction benefit immediately.

| Feature | `chub scan` | Transcript redaction |
|---|---|---|
| Rule engine | Shared (260 rules) | Shared (260 rules) |
| Entropy filtering | Yes | Yes |
| Stopword filtering | Yes | Yes |
| Invocation | Explicit CLI command | Automatic during tracking |
| Output | JSON/SARIF/CSV findings | Redacted transcript text |

## Performance

Benchmarked against [gitleaks](https://github.com/gitleaks/gitleaks) v8.30.1 and [betterleaks](https://github.com/nicholasgasior/betterleaks) on 10 real public GitHub repositories. Median of 3 runs on each.

### Directory scan

| Repo | Files | Chub | Gitleaks | Betterleaks | Speedup |
|---|--:|--:|--:|--:|---|
| axios/axios | 361 | **124 ms** | 410 ms | 468 ms | **3.8x** |
| expressjs/express | 213 | **119 ms** | 409 ms | 461 ms | **3.9x** |
| tokio-rs/tokio | 843 | **132 ms** | 414 ms | 466 ms | **3.5x** |
| pallets/flask | 236 | **179 ms** | 425 ms | 474 ms | **2.6x** |
| openai/openai-python | 1,280 | **185 ms** | 414 ms | 469 ms | **2.5x** |
| tiangolo/fastapi | 2,981 | **263 ms** | 421 ms | 486 ms | **1.8x** |
| django/django | 7,027 | **445 ms** | 435 ms | 488 ms | **1.1x** |
| hashicorp/vault | 8,611 | 527 ms | **414 ms** | 462 ms | 0.9x |
| denoland/deno | 11,618 | 711 ms | **414 ms** | 462 ms | 0.6x |
| golang/go | 15,154 | 847 ms | **422 ms** | 471 ms | 0.6x |

Chub is **2–4x faster** on repositories up to ~7,000 files. For very large repositories (10k+ files) gitleaks' fixed-overhead goroutine pool pulls ahead — Rust's rayon threads amortise well across I/O-bound work but the Go tool's lower startup cost wins at scale.

### Git history scan

| Repo | Commits | Chub | Gitleaks | Betterleaks | Speedup |
|---|--:|--:|--:|--:|---|
| expressjs/express | 357 | **233 ms** | 310 ms | 526 ms | **2.3x** |
| pallets/flask | 688 | **272 ms** | 320 ms | 490 ms | **1.8x** |
| openai/openai-python | 200 | **290 ms** | 306 ms | 480 ms | **1.7x** |
| tokio-rs/tokio | 625 | **370 ms** | 318 ms | 504 ms | **1.4x** |
| axios/axios | 452 | 457 ms | **298 ms** | 474 ms | 1.0x |
| tiangolo/fastapi | 200 | 478 ms | **322 ms** | 529 ms | 1.1x |
| django/django | 200 | 980 ms | **315 ms** | 489 ms | 0.5x |
| denoland/deno | 200 | 1,143 ms | **312 ms** | 495 ms | 0.4x |
| hashicorp/vault | 477 | 1,946 ms | **318 ms** | 493 ms | 0.3x |
| golang/go | 200 | 1,615 ms | **307 ms** | 482 ms | 0.3x |

Git history scanning is fastest on small-to-medium histories. For large or dense commit trees (vault, golang/go) gitleaks' C-backed libgit2 blob diffing outperforms chub's pure-Rust `git2` walker. Improvements to pack-file streaming are tracked in the roadmap.

Run `bash scripts/benchmark-scan-repos.sh` to reproduce locally (requires the 10 repos pre-cloned — see the script header).
