---
name: package
description: "tox package guide for Python test environment orchestration and automation"
metadata:
  languages: "python"
  versions: "4.49.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "tox,python,testing,virtualenv,ci,pytest"
---

# tox Python Package Guide

## Golden Rule

Use `tox` 4 as a config-driven CLI for repeatable local and CI automation across multiple Python interpreters and tool environments. As of March 12, 2026, official upstream sources show `tox 4.49.1` on PyPI, released on March 9, 2026, with current documentation at `https://tox.wiki/en/stable/`.

## Install

`tox` is primarily a CLI tool, so install it as a developer tool instead of bundling it into your runtime dependencies.

Recommended install from the official docs:

```bash
pipx install tox
tox --help
```

Alternative install with `pip`:

```bash
python -m pip install "tox==4.49.1"
python -m tox --help
```

If you want shell completion support, PyPI publishes the `completion` extra:

```bash
python -m pip install "tox[completion]==4.49.1"
```

Verify the actual executable on your path before debugging config issues:

```bash
tox --version
tox --help
```

## Initialize And Configure A Project

Generate a starter config:

```bash
tox quickstart
```

For tox 4 projects, prefer native TOML when you do not need older INI-only features:

- `tox.toml`
- `pyproject.toml` with `[tool.tox]`

tox still supports these config locations, in priority order:

1. `tox.ini`
2. `setup.cfg`
3. `pyproject.toml` with `tool.tox.legacy_tox_ini`
4. native `pyproject.toml` under `tool.tox`
5. `tox.toml`

Minimal `tox.toml`:

```toml
requires = ["tox>=4.49"]
env_list = ["3.12", "3.11", "lint"]

[env_run_base]
description = "run the test suite"
deps = ["pytest>=8"]
commands = [["pytest", "-q", "tests"]]

[env.lint]
description = "lint with ruff"
labels = ["check"]
skip_install = true
deps = ["ruff>=0.9"]
commands = [["ruff", "check", "."]]
```

What matters here:

- `env_list` defines what plain `tox` runs by default
- `[env_run_base]` is shared configuration for normal run environments
- `skip_install = true` avoids building and installing your package for tool-only environments
- `labels` lets you group environments under a stable name such as `check`

## Core Usage

Run the default environment list:

```bash
tox
```

Run one environment:

```bash
tox run -e 3.12
```

Run multiple environments explicitly:

```bash
tox run -e 3.12,3.11,lint
```

Run by label:

```bash
tox run -m check
```

Inspect resolved config for an environment:

```bash
tox config -e 3.12
```

List known environments:

```bash
tox list
```

Run a one-off command inside a tox environment:

```bash
tox exec -e 3.12 -- python -V
```

Run environments in parallel:

```bash
tox run-parallel -e 3.12,3.11
```

Recreate an environment after interpreter, dependency, or packaging changes:

```bash
tox run -e 3.12 -r
```

Stop early after the first failing environment when you want faster CI feedback:

```bash
tox run -e 3.12,3.11,lint --fail-fast
```

## Configuration Notes

### TOML vs INI

tox 4 supports both INI and TOML. Current docs describe native TOML as more robust, but note that some advanced features still remain stronger in INI. Do not paste multiline INI syntax directly into `[tool.tox]`; native TOML uses structured arrays and tables.

For example, TOML `commands` is a list of argument lists:

```toml
commands = [["pytest", "-q", "tests"]]
```

not:

```ini
commands = pytest -q tests
```

### Packaging behavior

tox normally creates an environment, installs dependencies, then builds and installs your package before running commands. That is what you want for real test environments that should exercise the installed package.

For tool-only environments such as lint, docs, or formatting, prefer:

- `skip_install = true` when you do not need the project package installed
- `package = "skip"` when you want packaging disabled explicitly

### Dependency groups

tox 4 supports `dependency_groups`, which maps cleanly to PEP 735 dependency groups defined in `pyproject.toml`. Use this when your project already declares grouped dependencies and you want to avoid repeating them in `tox`.

### Environment selection

tox infers Python versions from environment names such as `3.12` or `py312`. If you declare environments for interpreters that are not installed locally, runs will fail unless you configure missing-interpreter behavior intentionally.

## Config And Credentials

tox does not have its own auth model. The practical issue is environment isolation.

Commands run inside tox-managed environments, and only selected host environment variables are passed through. If your tests need credentials, private index settings, or CI flags, pass them deliberately.

Example:

```toml
[env_run_base]
pass_env = ["CI", "PIP_*", "UV_*", "AWS_*", "GITHUB_*"]
set_env = { PYTHONUTF8 = "1" }
```

Common cases that need `pass_env`:

- private package indexes such as `PIP_INDEX_URL` and `PIP_EXTRA_INDEX_URL`
- cloud credentials used by integration tests
- CI-only feature flags or tokens

If a command calls an executable that is not installed inside the tox environment, declare it with `allowlist_externals`; otherwise tox may block it as an external command.

## Common Patterns

Test multiple Python versions:

```toml
env_list = ["3.11", "3.12", "3.13"]
```

Separate tooling from package-installing test envs:

```toml
[env.format]
skip_install = true
deps = ["ruff>=0.9"]
commands = [["ruff", "format", "--check", "."]]
```

Use labels for CI groupings:

```toml
[env.typecheck]
labels = ["check"]
skip_install = true
deps = ["mypy>=1.11"]
commands = [["mypy", "src"]]
```

Use config output for debugging:

```bash
tox config -e 3.12 --format json
```

## Common Pitfalls

- Missing interpreters are the most common cause of environment creation failures. Keep `env_list` aligned with the Python versions actually available on the host or CI image.
- Tool-only environments should usually set `skip_install = true` or `package = "skip"`. Otherwise a packaging failure can block lint or docs jobs that do not need an installed package.
- TOML syntax is not a copy-paste replacement for `tox.ini`; `commands`, tables, and nested structures use different shapes.
- Secrets from the parent shell are not automatically available everywhere you expect. Pass needed variables explicitly with `pass_env`.
- External commands are restricted unless they are installed in the tox environment or allowlisted.
- Stale environments can hide dependency or interpreter changes. Use `tox run -e <env> -r` after changing package metadata, build backend settings, or Python versions.
- tox is a CLI orchestrator, not your test runner itself. Put the real work in `commands`, `deps`, `dependency_groups`, and packaging settings.

## Version-Sensitive Notes For tox 4.49.1

- tox 4 is not a small update over tox 3. Many older blog posts still use removed or renamed concepts such as `whitelist_externals` instead of `allowlist_externals`, or assume INI-only configuration.
- Native TOML support is now first-class. Current docs recommend TOML unless you specifically need advanced features that TOML still does not support yet.
- The `schema` command was added in tox `4.24.0`, which is useful for tooling that wants machine-readable tox configuration schema output.
- tox `4.26.0` dropped Python 3.8 support for running tox itself and added support for free-threaded Python builds.
- tox `4.28.0` deprecated `min_version` in favor of `requires`, and added `constraints` support.
- tox `4.33.0` added conditional `set_env` support with environment markers.
- tox `4.48.0` added machine-readable formatting options for `tox config`, including `--format` and `--output-file`.
- tox `4.49.0` added TOML factor-label substitution support, and `4.49.1` is the current patch release on PyPI.

## Official Links

- Stable docs: `https://tox.wiki/en/stable/`
- User guide: `https://tox.wiki/en/stable/user_guide.html`
- Configuration reference: `https://tox.wiki/en/stable/config.html`
- CLI reference: `https://tox.wiki/en/stable/cli_interface.html`
- Changelog: `https://tox.wiki/en/stable/changelog.html`
- PyPI: `https://pypi.org/project/tox/`
