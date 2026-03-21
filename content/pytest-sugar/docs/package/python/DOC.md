---
name: package
description: "pytest-sugar package guide for prettier pytest output, progress bars, and Playwright trace hints"
metadata:
  languages: "python"
  versions: "1.1.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest-sugar,pytest,testing,terminal,ci,playwright"
---

# pytest-sugar Python Package Guide

## What It Is

Use `pytest-sugar` as a pytest terminal UI plugin. It changes console output, adds a progress bar, and can show Playwright trace locations, but it does not change pytest's core collection, fixture, or assertion behavior.

This entry covers package version `1.1.1`.

## Install

Install it into the same environment as `pytest`:

```bash
python -m pip install "pytest-sugar==1.1.1"
```

Common alternatives:

```bash
uv add "pytest-sugar==1.1.1"
poetry add "pytest-sugar==1.1.1"
```

Run tests normally after install:

```bash
pytest
```

The plugin auto-loads when pytest discovers installed plugins.

## Initialization And Configuration Model

There is no package-specific Python client to initialize, no authentication flow, and no package-specific environment variable required for normal usage.

Use the same pytest entrypoint you already use:

```bash
pytest
```

For persistent defaults, put options in normal pytest config such as `pyproject.toml` or `pytest.ini`.

## Core Usage

### Default behavior

After installation, `pytest-sugar` replaces pytest's standard progress/output view with a progress bar and prints failures as they happen.

```bash
pytest
```

### Verbose mode

Use verbose mode if you want one test per line instead of the condensed progress UI:

```bash
pytest --verbose
```

### Disable sugar for one run

If CI tooling or another plugin expects plain pytest output, disable `pytest-sugar` explicitly:

```bash
pytest -p no:sugar
```

## Important CLI Options

### Use the older detailed summary

```bash
pytest --old-summary
```

Use this when the default instant-failure display is too compact and you want a more traditional detailed summary.

### Force sugar output in non-interactive environments

```bash
pytest --force-sugar
```

Use this in CI, containers, or redirected output where pytest does not think it is attached to a real terminal.

### Control Playwright trace lookup

By default, the plugin looks for Playwright traces in Playwright Python's default output directory, `test-results`.

```bash
pytest --sugar-trace-dir artifacts/playwright
```

Disable trace lookup entirely if it adds noise:

```bash
pytest --sugar-no-trace
```

## Project Configuration

### Pytest config

For most projects, keep defaults in regular pytest config through `addopts`.

`pyproject.toml`:

```toml
[tool.pytest.ini_options]
addopts = "-ra --force-sugar"
```

`pytest.ini`:

```ini
[pytest]
addopts = -ra --force-sugar
```

### `pytest-sugar.conf`

Use the optional `pytest-sugar.conf` file only for presentation tweaks such as theme symbols, colors, and progress bar length.

```ini
[theme]
symbol_passed = ✓
symbol_failed = x

[sugar]
progressbar_length = 30
```

Keep real test behavior in pytest config, not in the sugar-specific file.

## Common Workflows

### Local development

```bash
pytest tests/unit
```

This is the common case: auto-loaded plugin, compact progress display, and failures shown immediately.

### CI with forced terminal-style output

```bash
pytest --force-sugar --maxfail=1
```

This is useful when CI captures stdout without a real TTY and you still want the sugar progress UI.

### Playwright end-to-end tests

```bash
pytest tests/e2e --sugar-trace-dir test-results
```

If Playwright writes traces on failures, `pytest-sugar` can point to those artifacts from test output.

## Common Pitfalls

- Do not treat `pytest-sugar` as a reporting backend. It is a terminal UX plugin, not an HTML, JUnit, or results storage system.
- In CI or redirected output, the progress UI may not appear unless you add `--force-sugar`.
- If another tool parses raw pytest output, disable sugar with `-p no:sugar` for that job.
- The formatting is most useful in human-facing runs; plain pytest output is often better for machine-consumed logs.
- Windows terminals can render odd glyphs or colors depending on terminal font and charset settings. If output looks wrong, simplify theme symbols or fall back to plain pytest output.
- Playwright trace hints depend on where traces are actually written. If your suite uses a non-default artifact directory, set `--sugar-trace-dir` explicitly.

## Version-Sensitive Notes

- This entry targets `1.1.1`.
- PyPI release history shows `1.1.1` published on August 23, 2025.
- The PyPI long description still mentions older compatibility floors, while the maintainer repository for the current `1.1.x` line has newer packaging constraints. If you are pinning older Python or pytest versions, check the repo packaging metadata instead of relying on stale PyPI prose.
- The `1.1.x` line includes Playwright trace-related options such as `--sugar-trace-dir` and `--sugar-no-trace`. Older blog posts and screenshots from pre-`1.0` releases may not mention them.

## Official Sources

- PyPI package page: `https://pypi.org/project/pytest-sugar/`
- PyPI project JSON/API page: `https://pypi.org/pypi/pytest-sugar/`
- Maintainer repository: `https://github.com/Teemu/pytest-sugar`
- README: `https://github.com/Teemu/pytest-sugar/blob/main/README.md`
- Changelog/releases source: `https://github.com/Teemu/pytest-sugar/blob/main/CHANGES.rst`
