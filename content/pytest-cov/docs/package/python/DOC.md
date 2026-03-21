---
name: package
description: "Practical pytest-cov 7.0.0 guide for pytest coverage runs, reports, xdist, and subprocess measurement"
metadata:
  languages: "python"
  versions: "7.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest-cov,pytest,coverage,testing,python,ci"
---

# pytest-cov Python Package Guide

## Golden Rule

`pytest-cov` is a pytest plugin, not an application library. Install it into the same environment as `pytest`, run it through `pytest` command-line options such as `--cov`, and keep long-lived coverage settings in a coverage config file.

This guide targets `pytest-cov 7.0.0`.

## Install

Install the plugin into the same virtual environment as `pytest`:

```bash
python -m pip install "pytest-cov==7.0.0"
```

If you run tests in parallel with `pytest-xdist`, install that alongside the plugin:

```bash
python -m pip install pytest-xdist
```

## Runtime Setup

- Import: none in application code
- Environment variables: none required for normal usage
- Authentication: none
- Initialization: run tests with `python -m pytest` and pass coverage options there

The common setup mistake is treating `pytest-cov` like a runtime package import. In normal use, you do not add `import pytest_cov` to your app.

## Basic Commands

Run coverage for one package or source tree:

```bash
python -m pytest --cov=src/your_pkg tests
```

Show missing lines in terminal output:

```bash
python -m pytest --cov=src/your_pkg --cov-report=term-missing tests
```

Fail CI if total coverage drops below a threshold:

```bash
python -m pytest \
  --cov=src/your_pkg \
  --cov-branch \
  --cov-fail-under=90 \
  tests
```

Append coverage data across multiple runs instead of replacing it:

```bash
python -m pytest --cov=src/your_pkg --cov-append tests/unit
python -m pytest --cov=src/your_pkg --cov-append tests/integration
```

## Put Persistent Settings In `pyproject.toml`

`pytest-cov` works best when the CLI stays short and the real coverage settings live in a config file that coverage already understands.

```toml
[tool.pytest.ini_options]
addopts = [
  "--cov",
  "--cov-report=term-missing",
  "--cov-report=xml:coverage.xml",
]

[tool.coverage.run]
source = ["src/your_pkg"]
branch = true

[tool.coverage.report]
show_missing = true
skip_covered = true
```

Use bare `--cov` when `source` is already configured under coverage settings. Passing `--cov=...` on the command line overrides coverage's `source` value.

If your tests change the working directory, or you spawn subprocesses from a different location, pass the config path explicitly:

```bash
python -m pytest --cov --cov-config=pyproject.toml tests
```

## Reports

Common report outputs include terminal summaries, HTML, XML, JSON, LCOV, and annotated source output.

Generate multiple outputs in one run:

```bash
python -m pytest \
  --cov=src/your_pkg \
  --cov-report=term-missing \
  --cov-report=html:htmlcov \
  --cov-report=xml:coverage.xml \
  tests
```

If you pass any explicit `--cov-report`, the default terminal report is not added automatically. Ask for every report you want.

To generate only file output and suppress terminal coverage text:

```bash
python -m pytest --cov=src/your_pkg --cov-report= --cov-report=xml:coverage.xml tests
```

## Parallel Runs With `pytest-xdist`

For distributed or parallel pytest runs:

```bash
python -m pytest -n auto --cov=src/your_pkg tests
```

`pytest-cov` combines coverage from xdist workers automatically.

For per-test coverage contexts:

```bash
python -m pytest --cov=src/your_pkg --cov-context=test tests
```

Do not combine `--cov-context=test` with xdist. Distinct test contexts are not supported with distributed workers.

## Subprocess Coverage In 7.x

`pytest-cov 7.0.0` no longer uses the older `.pth`-based subprocess measurement approach. If your tests spawn Python subprocesses and you want their coverage included, enable coverage's subprocess patching in your coverage config:

```toml
[tool.coverage.run]
patch = ["subprocess"]
```

Keep `--cov-config` explicit when subprocesses may start from a different working directory:

```bash
python -m pytest --cov --cov-config=pyproject.toml tests
```

## Debugging And Temporary Opt-Out

Coverage tracing can interfere with debuggers. Disable coverage for that run when you need reliable breakpoint behavior:

```bash
python -m pytest --no-cov -k test_name
```

## Common Pitfalls

- `--cov=path_or_pkg` overrides coverage's configured `source`; use bare `--cov` when `source` already lives in config.
- If you specify one `--cov-report`, specify every report you need for that run.
- Config-file discovery can break when tests or subprocesses change directories; use `--cov-config=pyproject.toml` or `--cov-config=.coveragerc` explicitly.
- Older guides that describe automatic `.pth`-based subprocess handling do not match `pytest-cov 7.0.0`.
- `--cov-context=test` is useful for fine-grained analysis, but not with xdist.

## Official Sources

- Documentation: `https://pytest-cov.readthedocs.io/en/latest/`
- Configuration: `https://pytest-cov.readthedocs.io/en/latest/config.html`
- Reporting: `https://pytest-cov.readthedocs.io/en/latest/reporting.html`
- xdist: `https://pytest-cov.readthedocs.io/en/latest/xdist.html`
- Subprocess support: `https://pytest-cov.readthedocs.io/en/latest/subprocess-support.html`
- Changelog: `https://pytest-cov.readthedocs.io/en/latest/changelog.html`
- PyPI package: `https://pypi.org/project/pytest-cov/`
