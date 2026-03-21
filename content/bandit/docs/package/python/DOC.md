---
name: package
description: "Bandit Python package guide for security scanning Python code with the Bandit CLI"
metadata:
  languages: "python"
  versions: "1.9.4"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "bandit,python,security,sast,static-analysis,lint,ci"
---

# Bandit Python Package Guide

## Golden Rule

Use Bandit as a CLI security scanner for Python source trees, keep the package version pinned to what your project expects, and drive scans with an explicit config file when you need anything beyond the default recursive scan. For `1.9.4`, prefer a `pyproject.toml` or YAML config passed with `-c`; only rely on `.bandit` auto-discovery when you are intentionally using recursive mode with `-r`.

## Install

Pin the package in your dev tooling:

```bash
python -m pip install "bandit==1.9.4"
```

Common alternatives:

```bash
uv add --dev "bandit==1.9.4"
poetry add --group dev "bandit==1.9.4"
```

Optional extras from the official docs:

```bash
python -m pip install "bandit[toml]==1.9.4"
python -m pip install "bandit[sarif]==1.9.4"
python -m pip install "bandit[baseline]==1.9.4"
```

Use these when:

- `toml`: you want Bandit to read `pyproject.toml`
- `sarif`: you want SARIF output for code scanning tools
- `baseline`: you want the baseline workflow documented by Bandit

## First Scan

Recursive scans are the normal starting point:

```bash
bandit -r src
```

Scan multiple roots and exclude generated or vendored paths:

```bash
bandit -r src tests -x .venv,build,dist,migrations
```

Tighten the findings Bandit reports:

```bash
bandit -r src --severity-level medium --confidence-level medium
```

The short flags still work and are common in CI:

```bash
bandit -r src -ll -ii
```

## Configuration

Bandit supports INI, YAML, and TOML config styles, but they do not behave the same way.

### `pyproject.toml`

Bandit only reads TOML if you install the `toml` extra and pass the file with `-c`:

```toml
[tool.bandit]
exclude_dirs = [".venv", "build", "dist", "tests/fixtures"]
tests = ["B201", "B301", "B602", "B608"]
skips = ["B101"]
```

Run it with:

```bash
bandit -c pyproject.toml -r src
```

### YAML config

YAML config is also passed explicitly with `-c`:

```yaml
exclude_dirs:
  - .venv
  - build
  - dist
tests:
  - B201
  - B301
  - B602
skips:
  - B101
```

Run it with:

```bash
bandit -c bandit.yaml -r src
```

### `.bandit` INI

Bandit auto-loads a `.bandit` file only when you use `-r`:

```ini
[bandit]
exclude = tests,.venv
skips = B101,B601
```

Run it with:

```bash
bandit -r src
```

Config notes:

- `tests` is the allowlist of checks to run.
- `skips` is the denylist of checks to disable.
- Use one or the other deliberately; mixing both can hide why a check did or did not run.
- Plugin override sections are supported in YAML and TOML when a plugin documents configurable keys.

## Core Usage

### Write machine-readable reports

JSON is the easiest format for pipelines:

```bash
bandit -r src -f json -o bandit-report.json
```

SARIF works well for GitHub code scanning and similar tools:

```bash
bandit -r src -f sarif -o bandit-report.sarif
```

Other built-in formatters include `csv`, `html`, `screen`, `txt`, `xml`, and `yaml`.

### Use a baseline for legacy code

Generate a baseline from the current state:

```bash
bandit -r src -f json -o bandit-baseline.json
```

Then compare later scans against it:

```bash
bandit -r src -b bandit-baseline.json
```

The baseline file must be JSON output produced by Bandit.

### Integrate with pre-commit

Bandit documents a `pre-commit` hook. When your config lives in `pyproject.toml`, pass `-c` and add the TOML extra:

```yaml
- repo: https://github.com/PyCQA/bandit
  rev: "1.9.4"
  hooks:
    - id: bandit
      args: ["-c", "pyproject.toml"]
      additional_dependencies: ["bandit[toml]"]
```

### Fail-open mode for migration periods

If you need visibility before enforcing failures:

```bash
bandit -r src --exit-zero
```

Use this only temporarily; otherwise CI will always pass even when Bandit finds issues.

## Authentication And Environment

Bandit does not use API credentials. The relevant environment setup is the Python environment where you install the package and the paths you choose to scan.

Operational setup that matters:

- Run Bandit inside the same virtual environment or toolchain lock that owns your Python dependencies.
- Exclude virtual environments, generated code, vendored code, and migration snapshots unless you explicitly want them scanned.
- In CI, make the working directory and scan roots explicit so Bandit does not miss source folders or scan build artifacts.

## Common Pitfalls

- `.bandit` is not a universal auto-discovery mechanism. The docs state it is discovered only when `-r` is used.
- `pyproject.toml` is not read automatically. Use `bandit[toml]` and pass `-c pyproject.toml`.
- Baselines only work with JSON reports. Do not try to feed Bandit a text or SARIF file with `-b`.
- `--exit-zero` is useful while introducing Bandit, but it suppresses CI failure even for high-severity findings.
- A broad skip like `B101` can hide real problems outside tests. Scope skips narrowly and document why they exist.
- Severity and confidence thresholds are independent. Raising one without the other often leads to confusing result changes in CI.
- Pre-commit and direct CLI runs should use the same config file, or agents will see inconsistent findings between local and CI runs.

## Version-Sensitive Notes For `1.9.4`

- PyPI lists `1.9.4` as the current release, published on `2026-02-25`.
- `1.9.4` requires Python `>=3.10`; do not assume older Python runtimes are still supported.
- The `1.9.4` release fixes several issues that matter in automation, including a crash in the `B613` plugin when scanning stdin, a false positive in `B615`, and incorrect line numbers for multiline cases in `B106`.
- Older Bandit examples often assume `.bandit` INI files or omit TOML support details. For current projects, `pyproject.toml` plus `bandit[toml]` is usually the cleanest setup.
