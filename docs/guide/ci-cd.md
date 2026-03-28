# CI/CD Integration

Chub integrates into your development pipeline at three levels: pre-commit hooks for local checks, CI jobs for pull request validation, and post-merge actions for tracking and freshness.

## Pre-Commit Secret Scanning

Block secrets before they reach the repository. Chub's `scan secrets git --staged` checks only staged changes, making it fast enough for a pre-commit hook.

### With the pre-commit Framework

Add to `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: chub-scan-secrets
        name: Scan for secrets
        entry: chub scan secrets git --staged
        language: system
        pass_filenames: false
        stages: [pre-commit]
```

### With Husky (Node.js)

```sh
npx husky add .husky/pre-commit "chub scan secrets git --staged"
```

### With a Plain Git Hook

```sh
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/sh
chub scan secrets git --staged
EOF
chmod +x .git/hooks/pre-commit
```

If any secret is detected, the hook exits non-zero and the commit is blocked. The finding details are printed to stderr.

### Tuning for Speed

Staged-only scanning (`--staged`) is fast because it only checks the diff, not the full history. For large repos where even that is slow, narrow the scope:

```sh
# Only scan specific file types
chub scan secrets git --staged --config .chub-scan.toml
```

```toml
# .chub-scan.toml — skip generated files
[allowlist]
paths = ["dist/.*", "vendor/.*", ".*\\.min\\.js$"]
```

## GitHub Actions

### Secret Scanning on Pull Requests

```yaml
# .github/workflows/chub-scan.yml
name: Secret Scan
on: [pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # full history for git scan

      - name: Install Chub
        run: npm install -g @nrl-ai/chub

      - name: Scan for secrets
        run: chub scan secrets git --format sarif --output results.sarif

      - name: Upload SARIF
        if: always()
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: results.sarif
```

SARIF output integrates with GitHub's code scanning alerts, showing findings inline on the pull request.

### Baseline Filtering

On established repos, scanning the full history produces known findings. Use a baseline to suppress them and only flag new secrets:

```sh
# Generate baseline once (commit this file)
chub scan secrets git --format json --output .chub-scan-baseline.json

# CI: scan with baseline — only new findings fail
chub scan secrets git --baseline-path .chub-scan-baseline.json
```

### Pin Freshness Checks

Catch outdated doc pins in CI:

```yaml
      - name: Check pin freshness
        run: chub check
```

`chub check` exits non-zero if any pinned doc has a newer version available. Add `--fix` locally to update pins, then commit.

### Dependency Detection

Verify that new dependencies have matching docs pinned:

```yaml
      - name: Detect unpinned dependencies
        run: |
          chub detect
          # Fails if new deps found without pins (use chub detect --pin to fix locally)
```

## GitLab CI

```yaml
# .gitlab-ci.yml
secret-scan:
  stage: test
  image: node:20
  before_script:
    - npm install -g @nrl-ai/chub
  script:
    - chub scan secrets git --format json --output gl-secret-detection-report.json
  artifacts:
    reports:
      secret_detection: gl-secret-detection-report.json
```

## Custom Config for CI

Create a `.chub-scan.toml` at the repo root. This is auto-discovered by `chub scan secrets` (no `--config` flag needed):

```toml
title = "Project secret scanning rules"

[extend]
useDefault = true           # include all 73+ built-in rules
disabledRules = [           # suppress noisy rules
  "generic-api-key",
]

[allowlist]
paths = [
  "test/fixtures/.*",       # test data with fake secrets
  "docs/examples/.*",       # documentation examples
]
stopwords = [
  "EXAMPLE",
  "your_api_key_here",
  "xxxxxxxxxxxx",
]

# Add a project-specific rule
[[rules]]
id = "internal-service-token"
description = "Internal service token"
regex = '''(?i)internal[_-]?token\s*[:=]\s*["']?([a-z0-9]{32,})'''
keywords = ["internal_token", "internal-token"]
```

## Output Formats

| Format | Flag | Best for |
|--------|------|----------|
| JSON | `--format json` | Scripts, baseline files, gitleaks compatibility |
| SARIF | `--format sarif` | GitHub/GitLab code scanning integration |
| CSV | `--format csv` | Spreadsheets, compliance audits |
| Table | (default) | Human reading in terminal |

## Scanning AI Transcripts

AI coding agents can leak secrets through their conversation logs. Pipe transcripts through Chub's stdin scanner:

```sh
# Scan a transcript file
cat .git/chub/transcripts/session-*.jsonl | chub scan secrets stdin

# Scan with redaction — replace secrets with [REDACTED]
chub scan secrets dir .git/chub/transcripts/ --redact
```

Chub's tracking system automatically redacts secrets from session transcripts before they are committed to the session branch. See [AI Usage Tracking](/guide/tracking) for details.

## Tracking Hooks in CI

If your CI runs AI agents (e.g., Codex for automated refactoring), enable tracking so CI sessions appear in team reports:

```yaml
      - name: Enable tracking
        run: chub track enable

      # ... AI agent steps ...

      - name: Push session data
        run: git push origin chub/sessions/v1
```

## Environment Variables

These environment variables are useful in CI:

| Variable | Purpose |
|----------|---------|
| `CHUB_DIR` | Override `~/.chub` data directory (useful for CI caching) |
| `CHUB_PROJECT_DIR` | Override project root detection |
| `CHUB_SCAN_CONFIG` | Path to scan config file |
| `CHUB_TELEMETRY=0` | Disable telemetry in CI |
