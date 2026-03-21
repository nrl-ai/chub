---
name: cov
description: "pytest-cov package guide for running pytest coverage, reports, xdist, and coverage config in Python projects"
metadata:
  languages: "python"
  versions: "7.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest,coverage,testing,python,ci,xdist"
---

# pytest-cov Python Package Guide

## Golden Rule

Use `pytest-cov` as a pytest plugin, not as an application import. Install it into the same environment as `pytest`, run coverage through `pytest --cov ...`, and keep the real coverage settings in a coverage config file.

As of March 12, 2026, PyPI lists `pytest-cov 7.0.0`, while the upstream `latest` docs already include a `7.0.1` changelog entry. This guide is anchored to the released PyPI version `7.0.0`.

## Install

Install the plugin into the same virtualenv as your test runner:

```bash
python -m pip install "pytest-cov==7.0.0"
```

If you run tests in parallel, also install `pytest-xdist`:

```bash
python -m pip install pytest-xdist
```

## Basic Usage

Minimal coverage run:

```bash
pytest --cov=src/your_pkg tests
```

Show missing lines in the terminal:

```bash
pytest --cov=src/your_pkg --cov-report=term-missing tests
```

Fail CI if coverage drops below a threshold:

```bash
pytest --cov=src/your_pkg --cov-branch --cov-fail-under=90 tests
```

Append results across multiple runs instead of erasing prior coverage data:

```bash
pytest --cov=src/your_pkg --cov-append tests/unit
pytest --cov=src/your_pkg --cov-append tests/integration
```

## Persistent Configuration

`pytest-cov` can read coverage settings from `.coveragerc`, `tox.ini`, `setup.cfg`, or `pyproject.toml`. If your tests or subprocesses change the working directory, pass `--cov-config` explicitly so coverage uses the right file.

A practical `pyproject.toml` setup:

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

Use bare `--cov` when `source` is already configured under coverage settings. Passing `--cov=...` on the command line overrides coverage's `source` option.

## Reports

`pytest-cov` supports these report types:

- terminal: `term`
- terminal with missing lines: `term-missing`
- HTML: `html[:path]`
- XML: `xml[:path]`
- JSON: `json[:path]`
- LCOV: `lcov[:path]`
- Markdown: `markdown[:path]`
- Markdown append: `markdown-append[:path]`
- Annotated source: `annotate[:path]`

Examples:

```bash
pytest --cov=src/your_pkg \
  --cov-report=term-missing \
  --cov-report=html:htmlcov \
  --cov-report=xml:coverage.xml
```

If you specify any `--cov-report`, the default terminal report is not added automatically. Ask for every report you want.

To generate only file outputs and suppress terminal coverage text:

```bash
pytest --cov=src/your_pkg --cov-report= --cov-report=xml:coverage.xml
```

## Parallel And Distributed Test Runs

For `pytest-xdist`:

```bash
pytest -n auto --cov=src/your_pkg tests
```

`pytest-cov` combines coverage from xdist workers automatically. This is the normal way to collect parallel test coverage in pytest.

Per-test coverage contexts are available with:

```bash
pytest --cov=src/your_pkg --cov-context=test tests
```

Do not combine `--cov-context=test` with distributed runs. Upstream docs call out that distinct test contexts are not supported with xdist.

## Subprocess Coverage In 7.x

`pytest-cov 7.0.0` removed the old subprocess measurement mechanism that depended on a `.pth` file. If you need coverage from spawned Python subprocesses, enable coverage's subprocess patch instead:

```toml
[tool.coverage.run]
patch = ["subprocess"]
```

This requires a recent coverage release with patch support. Upstream documents this as the replacement path for `pytest-cov 7.x`.

## Debugging And Temporary Opt-Out

Coverage tracing can interfere with debuggers. When you need clean breakpoint behavior, disable the plugin for that run:

```bash
pytest --no-cov -k test_name
```

`pytest-cov` also exposes a `no_cover` marker and fixture for targeted opt-outs, but `--no-cov` is the simplest whole-run escape hatch while debugging.

## Auth And Environment

There is no auth layer. The important configuration is local:

- coverage config file location
- pytest `addopts`
- output paths for reports such as `coverage.xml` or `htmlcov/`
- parallel or subprocess behavior when using xdist or spawned Python processes

## Common Pitfalls

- `--cov=path_or_pkg` overrides coverage's `source` setting. If `source` already lives in your coverage config, use bare `--cov`.
- `pytest-cov` overrides coverage's `parallel` option internally. Do not expect a manual `parallel = true` setting to behave the same way as a plain coverage run.
- If you pass one explicit `--cov-report`, you must pass all the others you need too.
- When tests change directories or start subprocesses, an implicit config file lookup can fail. Use `--cov-config=pyproject.toml` or `--cov-config=.coveragerc` when needed.
- `--cov-context=test` is useful for fine-grained analysis, but not with xdist.
- For subprocess coverage on `7.0.0`, old blog posts describing automatic `.pth`-based subprocess support are outdated.

## Version-Sensitive Notes

- Version used here: `7.0.0`
- PyPI current release on March 12, 2026: `7.0.0`
- Upstream docs drift: the `latest` docs include a `7.0.1` changelog entry dated March 2, 2026
- Breaking behavior change in `7.0.0`: subprocess coverage moved away from the old `.pth` mechanism to coverage's patch-based approach

When copying examples from search results, prefer the official docs over older issue comments or blog posts because `pytest-cov 7.x` changed subprocess behavior and the `latest` docs are slightly ahead of the PyPI release.
