---
name: package
description: "Safety CLI for scanning Python dependencies for vulnerabilities and license issues"
metadata:
  languages: "python"
  versions: "3.7.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "safety,security,vulnerability-scanning,license-compliance,python,cli,dependencies"
---

# Safety Python Package Guide

## Golden Rule

Use `safety` as a CLI tool, not as an importable runtime library. For Safety `3.x`, the normal flow is authenticate first, then run `safety scan` against a project directory. Many older examples still use Safety `2.x` commands like `safety check -r requirements.txt`; treat those as migration material, not the default 3.x workflow.

## Install

Pin the CLI version your workflow expects:

```bash
python -m pip install "safety==3.7.0"
```

Useful alternatives:

```bash
pipx install "safety==3.7.0"
uv tool install "safety==3.7.0"
```

Use `pipx` or `uv tool` when you want Safety isolated from the project environment. Install it into the project environment only if your CI or dev tooling expects that layout.

## Authenticate And Initialize

Safety 3 scans are tied to the Safety Platform account model. Start with interactive login on a developer machine:

```bash
safety auth login
```

For headless machines or SSH sessions without a browser:

```bash
safety auth login --headless
```

For CI or other non-interactive environments, use an API key:

```bash
export SAFETY_API_KEY="your-api-key"
safety auth login --key
```

Prefer an environment variable in CI so the key stays out of shell history and repo files.

## Core Usage

### Scan the current project

```bash
safety scan
```

Safety 3 scans the current project directory and detects supported dependency files and Python environments automatically.

### Scan a specific directory

```bash
safety scan --target ./services/api
```

Use `--target` when the repository root is not the Python project root.

### Save machine-readable output

```bash
safety scan --output json --save-as safety-report.json
```

The docs list multiple output formats, including `screen`, `text`, `json`, `html`, `bare`, `xunit`, `spdx-json`, and `gitlab-sast`.

### Generate and validate a policy file

```bash
safety generate policy_file
safety validate policy_file --path .safety-policy.yml
```

Use a policy file when you need to tune scan behavior, ignore rules, or reporting settings for a repository-level workflow.

## Project Configuration

Safety looks for a `.safety-policy.yml` file in the scan target or current working directory unless you point the CLI at a different file path.

Common policy uses from the official docs:

- define `report` settings such as `dependency-vulnerabilities`, `licenses`, and `json-output`
- set scan inputs such as `include-files`
- manage ignores with IDs, expiration dates, and reasons

If you want deterministic automation, keep the policy file in the repo root and validate it in CI before relying on it.

## CI And Automation Pattern

A practical non-interactive pattern is:

```bash
export SAFETY_API_KEY="$SAFETY_API_KEY"
safety auth login --key
safety scan --target . --output json --save-as safety-report.json
```

In CI, treat the scan report as an artifact and let the command's exit code fail the job when policy or vulnerability conditions require it. Safety documents command-specific exit codes, so do not assume every non-zero exit means the CLI crashed.

## Common Pitfalls

- Safety `3.x` is not the same CLI contract as Safety `2.x`. Old examples that use `safety check`, `--file`, or `-r requirements.txt` are usually outdated for the current docs.
- `safety scan` targets directories, not a single requirements file. The official migration guide says to scan the project directory or use policy-file `include-files` if you need to narrow inputs.
- Authentication is required for the main Safety 3 scan workflow. If `safety scan` behaves unexpectedly in CI, check auth state before debugging the project files.
- Safety Platform project policy can override or supersede local policy behavior. The current docs discuss both local `.safety-policy.yml` usage and platform-managed policy, so verify the effective behavior in the account you are using.
- Save reports explicitly with `--save-as` when another job step needs the results. Printing JSON to stdout is easy to lose in CI logs.

## Version-Sensitive Notes For 3.7.0

- The official docs banner and quickstart pages currently track Safety `3.7.0`, and PyPI also lists `3.7.0` for the package version used here.
- The official migration guide says `safety check` from v2 should be replaced with `safety scan` in v3, and that `scan` no longer targets individual files directly.
- Some official docs snippets still mention installing `safety==3.4.0` on authentication pages. Treat those as stale examples and pin `3.7.0` if you are matching this curated entry.
- Safety 3.5.0 introduced the minimum version needed for Safety Firewall support according to the migration docs, so older 3.x examples may miss newer platform behavior.

## Official Links

- Docs root: `https://docs.safetycli.com/safety-docs/`
- Quick start: `https://docs.safetycli.com/safety-docs/getting-started/quick-start`
- Authentication: `https://docs.safetycli.com/safety-docs/getting-started/authenticate-yourself`
- Policy files: `https://docs.safetycli.com/safety-docs/safety-cli/scanning-for-vulnerable-and-malicious-packages/using-a-safety-policy-file`
- Output formats: `https://docs.safetycli.com/safety-docs/output/formats`
- Exit codes: `https://docs.safetycli.com/safety-docs/support/exit-codes`
- Migration guide: `https://docs.safetycli.com/safety-docs/miscellaneous/release-notes/breaking-changes-in-safety-3`
- PyPI: `https://pypi.org/project/safety/`
