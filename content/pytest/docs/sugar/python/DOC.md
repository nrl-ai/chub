---
name: sugar
description: "pytest-sugar plugin guide for prettier pytest output, progress bars, and Playwright trace hints"
metadata:
  languages: "python"
  versions: "1.1.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest,testing,terminal,ci,playwright"
---

# pytest-sugar Python Package Guide

## Golden Rule

Use `pytest-sugar` only as a pytest UI plugin. It changes terminal output, adds a progress bar, and can surface Playwright trace locations, but it does not replace normal pytest configuration or failure semantics.

As of March 12, 2026, the version used here `1.1.1` still matches the latest PyPI release. The PyPI project description is partly stale for requirements; the current maintainer repo for the `1.1.x` line has moved beyond the older `Python 3.8` / `pytest 6.2` floor shown in the long description.

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

The plugin activates automatically when pytest loads installed plugins.

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

### Disable sugar for a run

If a CI log parser or another plugin expects plain pytest output, disable `pytest-sugar` explicitly:

```bash
pytest -p no:sugar
```

## Important CLI Options

### Show the older detailed summary format

```bash
pytest --old-summary
```

Use this when the one-line instant failure display is too compact and you want a more traditional detailed summary.

### Force sugar output in non-interactive environments

```bash
pytest --force-sugar
```

Use this in CI, containers, or other environments where pytest does not think it is attached to a real terminal.

### Control Playwright trace lookup

By default, the plugin looks for Playwright traces in the default Playwright Python output directory, `test-results`.

```bash
pytest --sugar-trace-dir artifacts/playwright
```

Disable trace lookup entirely if it adds noise:

```bash
pytest --sugar-no-trace
```

## Project Configuration

### Pytest config

`pytest-sugar` does not need credentials or network configuration. For most projects, configuration lives in normal pytest config through `addopts`.

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

### Theme and progress bar config

The plugin also supports a dedicated `pytest-sugar.conf` file for UI tweaks such as theme symbols/colors and progress bar length.

```ini
[theme]
symbol_passed = ✓
symbol_failed = x

[sugar]
progressbar_length = 30
```

Use this only for presentation tweaks. Keep real test behavior in pytest config, not in the sugar-specific file.

## Practical Patterns

### Local development

```bash
pytest tests/unit
```

This is the common case: auto-loaded plugin, compact progress display, failures shown immediately.

### CI job with forced terminal-style output

```bash
pytest --force-sugar --maxfail=1
```

This is useful when CI captures stdout without a real TTY and you still want the sugar progress output.

### Playwright end-to-end tests

```bash
pytest tests/e2e --sugar-trace-dir test-results
```

If Playwright writes traces on failures, `pytest-sugar` can point you to those artifacts from the test output.

## Common Pitfalls

- Do not treat `pytest-sugar` as a reporting backend. It is a terminal UX plugin, not a results store or HTML/JUnit replacement.
- In CI or redirected output, the progress UI may not appear unless you add `--force-sugar`.
- If another tool parses raw pytest output, disable sugar with `-p no:sugar` for that job.
- The plugin's terminal formatting is most useful in human-facing runs; plain pytest is often better for machine-consumed logs.
- Windows terminals can render odd glyphs or colors depending on font/charset settings. If output looks broken, simplify theme symbols or fall back to default pytest output.
- Playwright trace hints depend on where traces are actually written. If your suite uses a non-default artifact directory, set `--sugar-trace-dir` explicitly.

## Version-Sensitive Notes

- PyPI release history shows `1.1.1` published on August 23, 2025.
- The PyPI long description still advertises `Python 3.8+` and `pytest 6.2+`, but the maintainer repository for the current line has newer packaging constraints. Verify your environment against the repo's current packaging metadata if you are pinning old interpreters or old pytest versions.
- The `1.1.x` line includes Playwright trace-related options such as `--sugar-trace-dir` and `--sugar-no-trace`. Older blog posts and screenshots from pre-`1.0` releases may not mention them.
- If you need deterministic, minimal logs for tooling, prefer plain pytest output for that specific job even if developers use `pytest-sugar` locally.

## Official Sources

- PyPI package page: `https://pypi.org/project/pytest-sugar/`
- PyPI project JSON/API page: `https://pypi.org/pypi/pytest-sugar/`
- Maintainer repository: `https://github.com/Teemu/pytest-sugar`
- Changelog/releases source: `https://github.com/Teemu/pytest-sugar/blob/main/CHANGES.rst`
