---
name: package
description: "Pylint 4.0.5 package guide for Python linting, config setup, CI integration, and common false-positive traps"
metadata:
  languages: "python"
  versions: "4.0.5"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pylint,python,linting,static-analysis,quality,ci"
---

# Pylint Python Package Guide

## What It Is

`pylint` is a static analysis and linting tool for Python. It checks code style, probable bugs, import problems, refactor suggestions, and convention issues, then reports them as named messages such as `unused-import`, `import-error`, or `missing-function-docstring`.

Use it when you want:

- one command that can lint files, packages, or directories
- project-wide configuration in `pyproject.toml` or rc/config files
- CI- and editor-friendly output formats
- plugin support for framework-specific checks

## Install

Add it as a development dependency and keep it pinned with the rest of your tooling:

```bash
python -m pip install "pylint==4.0.5"
```

Common alternatives:

```bash
uv add --dev "pylint==4.0.5"
poetry add --group dev "pylint==4.0.5"
```

If your project uses framework-specific or domain-specific Pylint plugins, install them in the same environment as `pylint`.

## Initialize And Configure

For new repos, prefer `pyproject.toml`. For existing repos already using INI-style tool config, `.pylintrc` is still fine.

Generate a starter config:

```bash
pylint --generate-toml-config > /tmp/pylint-generated.toml
```

or:

```bash
pylint --generate-rcfile > .pylintrc
```

Notes:

- Do not redirect `--generate-toml-config` straight into an existing `pyproject.toml`; generate to a temporary file and merge the `tool.pylint` sections you want.
- In monorepos or repos with more than one possible config file, pass `--rcfile=/absolute/or/relative/path` in CI so discovery is deterministic.
- If you lint code that targets a different Python version than the interpreter running Pylint, set `py-version` in config explicitly.

Minimal setup workflow:

1. Generate a starter config.
2. Remove broad disables you do not actually want.
3. Run Pylint against the main package and tests.
4. Add narrow message disables only after you see real noise from your codebase.

## Core Usage

Lint a package or directory:

```bash
pylint src/ tests/
```

Lint a specific module or file:

```bash
pylint my_package my_package/api.py
```

Machine-readable output for tooling:

```bash
pylint src/ --output-format=json2 > pylint-report.json
```

GitHub Actions annotations:

```bash
pylint src/ --output-format=github
```

Useful habits:

- Run Pylint from the project root so imports resolve the same way they do in normal development.
- Lint the package root in CI, not only individual changed files, because many messages depend on import context.
- Prefer message symbols like `missing-function-docstring` over numeric IDs when disabling or triaging checks.

## Message Control

The usual pattern is to keep project defaults in config and reserve inline disables for local exceptions.

Command-line example:

```bash
pylint src/ --disable=missing-module-docstring,missing-function-docstring
```

Inline example:

```python
message = "value: {}".format(value)  # pylint: disable=consider-using-f-string
```

Use inline pragmas sparingly and keep them attached to the specific line or block that needs the exception.

## Config, Plugins, And Environment

Pylint is a local analyzer. There is no auth flow, API key, or remote service setup. The important environment concern is import resolution.

Practical rules:

- Run Pylint in the same virtualenv as the project.
- Install your package and its runtime dependencies before trusting `import-error` results.
- If you use a `src/` layout, make sure the package is importable in the lint environment, typically via editable install.
- If you need framework-specific checks, configure `load-plugins` and install those plugins in the same environment.

For pre-commit, remember that hooks run in isolated environments. If your lint run depends on plugins or project-only imports, add the required packages to the hook's `additional_dependencies`.

## CI And Automation

Typical CI command:

```bash
pylint src/ tests/ --output-format=text
```

If another tool needs structured output, use `json2` instead of parsing the default text format.

In automation:

- pin the Pylint version so message behavior does not drift unexpectedly
- pin plugin versions alongside Pylint
- pass `--rcfile` explicitly in monorepos
- treat new major versions as behavior changes, not just bugfix upgrades

## Common Pitfalls

### `import-error` false positives

This usually means the lint environment does not match the app environment. Install the package in editable mode and include optional dependencies or plugins that the code imports.

### Config discovery surprises

Pylint can read config from multiple file formats. In larger repos, this can make local runs and CI disagree. Pick one canonical config file and pass it explicitly in scripted runs.

### Over-disabling checks

Generated starter configs can be noisy, but broad disables reduce the value of linting quickly. Disable by message symbol, keep the scope narrow, and prefer fixing the root cause when possible.

### Pre-commit differs from local runs

A pre-commit hook does not automatically reuse your project virtualenv. Missing plugins or missing app dependencies inside the hook environment are a common reason for inconsistent results.

### Python-version mismatch

Pylint inspects code using the interpreter and target-version settings it sees. If your CI runs on Python 3.12 but the codebase targets 3.10, set `py-version` so checks match the code you actually support.

## Version-Sensitive Notes For 4.0.x

- As of `2026-03-12`, both PyPI and the stable docs show `pylint 4.0.5`.
- The `4.0` line requires Python `>=3.10` and moved to the Astroid `4.x` line.
- The 4.0 release changed some naming-check behavior, especially around constants, type aliases, and `ClassVar` handling, so older suppressions may need review after upgrading from `3.x`.
- If you depend on framework plugins or import-order behavior, keep the full lint toolchain pinned together during upgrades instead of upgrading Pylint alone.

## Official Sources Used

- Docs root: `https://pylint.readthedocs.io/en/stable/`
- Installation guide: `https://pylint.readthedocs.io/en/stable/user_guide/installation/index.html`
- Running Pylint: `https://pylint.readthedocs.io/en/stable/user_guide/usage/run.html`
- Configuration guide: `https://pylint.readthedocs.io/en/stable/user_guide/configuration/index.html`
- What's new in 4.0: `https://pylint.readthedocs.io/en/v4.0.0/whatsnew/4/4.0/`
- PyPI package page: `https://pypi.org/project/pylint/`
