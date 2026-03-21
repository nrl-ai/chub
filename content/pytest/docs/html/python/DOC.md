---
name: html
description: "pytest-html package guide for Python projects generating HTML test reports from pytest runs"
metadata:
  languages: "python"
  versions: "4.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest-html,pytest,testing,html,reporting"
---

# pytest-html Python Package Guide

## Golden Rule

Use `pytest-html` as a pytest plugin, not as a standalone reporting tool. Install the package into the same environment as `pytest`, generate reports with `--html=...`, and prefer the official user guide for hook behavior and report customization.

As of March 12, 2026, the package metadata and maintainer release notes are current for `4.2.0`, but parts of the Read the Docs site still lag that release. PyPI and the maintainer release notes are the safer source for version and compatibility facts.

## Install

Install `pytest-html` into the same environment as your test suite:

```bash
python -m pip install "pytest-html==4.2.0"
```

Common alternatives:

```bash
uv add --dev "pytest-html==4.2.0"
poetry add --group dev "pytest-html==4.2.0"
```

Verify the plugin is available:

```bash
pytest --help | rg -- --html
```

## Basic Usage

Generate an HTML report from a normal pytest run:

```bash
pytest --html=report.html
```

Generate a self-contained report that inlines CSS and JavaScript:

```bash
pytest --html=report.html --self-contained-html
```

The plugin is auto-discovered by pytest after installation. You do not need to import anything for the basic CLI flow.

## Project Setup

For repeatable local and CI runs, put the default HTML report options in pytest config instead of repeating CLI flags.

`pytest.ini`:

```ini
[pytest]
addopts = --html=reports/pytest.html --self-contained-html
generate_report_on_test = True
render_collapsed = all
initial_sort = result
environment_table_redact_list =
    (?i).*token.*
    (?i).*secret.*
```

Equivalent `pyproject.toml`:

```toml
[tool.pytest.ini_options]
addopts = "--html=reports/pytest.html --self-contained-html"
generate_report_on_test = true
render_collapsed = "all"
initial_sort = "result"
environment_table_redact_list = [
  "(?i).*token.*",
  "(?i).*secret.*",
]
```

Useful config knobs from the official user guide:

- `generate_report_on_test = True`: stream report updates during the run instead of writing only once at the end
- `render_collapsed = all`: collapse all result rows initially
- `initial_sort = result`: sort the report by test outcome on initial load
- `environment_table_redact_list`: regex list for masking sensitive environment values in the Environment section

## Core Customization

Put report hooks in `conftest.py` or a dedicated pytest plugin.

### Set a custom report title

```python
def pytest_html_report_title(report):
    report.title = "My Project Test Report"
```

### Add environment metadata

The Environment section is driven by `pytest-metadata`. Update it before `pytest-html` finalizes the report:

```python
import os

import pytest
from pytest_metadata.plugin import metadata_key

@pytest.hookimpl(tryfirst=True)
def pytest_sessionfinish(session, exitstatus):
    session.config.stash[metadata_key]["git_sha"] = os.getenv("GIT_SHA", "unknown")
    session.config.stash[metadata_key]["build_id"] = os.getenv("BUILD_ID", "local")
```

### Attach extras on failures

Use `report.extras` and helpers from `pytest_html.extras`:

```python
import pytest
from pytest_html import extras

@pytest.hookimpl(hookwrapper=True)
def pytest_runtest_makereport(item, call):
    outcome = yield
    report = outcome.get_result()

    if report.when != "call":
        return

    report.extras = getattr(report, "extras", [])

    if report.failed:
        report.extras.append(extras.text("request payload here", name="payload"))
        report.extras.append(extras.url("https://ci.example.test/job/123", name="CI job"))
```

Available extra types in the official guide include raw HTML, JSON, text, URLs, and images.

### Edit summary or results-table content

Common hooks:

- `pytest_html_results_summary(prefix, summary, postfix)`
- `pytest_html_results_table_header(cells)`
- `pytest_html_results_table_row(report, cells)`

Use these when you need to add columns, remove columns, or append custom summary content.

## CI And Artifact Workflow

Typical CI command:

```bash
pytest --html=artifacts/pytest.html --self-contained-html
```

Recommended practice:

- write the report into a stable artifacts directory
- publish the HTML file as a CI artifact
- prefer `--self-contained-html` when the artifact will be viewed outside the original workspace
- keep attachments small if you add extras for every failure

## Config And Security Notes

`pytest-html` has no remote authentication model. The main security and configuration concerns are about what runtime data you expose in the report.

- Treat the report as potentially sensitive test output.
- Redact secrets in the Environment table with `environment_table_redact_list`.
- Be careful with custom extras; they can embed raw HTML, URLs, logs, and payloads.
- `--self-contained-html` does not safely inline every external asset you might attach. File and image references can still point outside the report.

## Common Pitfalls

- PyPI for `4.2.0` requires Python `>=3.9`, but the Read the Docs installation page still says Python `3.6+`. Use PyPI for compatibility facts.
- The Read the Docs changelog page currently stops at `4.1.1`; use the maintainer `4.2.0` GitHub release page for the current release notes.
- If you mutate environment metadata in `pytest_sessionfinish`, mark the hook with `@pytest.hookimpl(tryfirst=True)` or `pytest-html` may render the old values.
- `--self-contained-html` is convenient for artifacts, but attached images and linked files can still trigger warnings or broken references.
- For modern versions, use `report.extras`, not the deprecated `report.extra`.
- The old `extra` fixture is deprecated; use the `extras` fixture or `pytest_html.extras`.
- If you customize results-table columns, insert or remove entries from `cells`; older direct assignment patterns are deprecated.
- `py.xml` is deprecated in the plugin ecosystem. Prefer plain strings or supported helpers in custom hooks.

## Version-Sensitive Notes For 4.2.0

- The maintainer `4.2.0` release adds support for Python `3.13`.
- The maintainer `4.2.0` release adds support for `pytest 8.4`.
- The maintainer `4.2.0` release drops support for Python `3.8`.
- If you are upgrading older hooks, watch for deprecated patterns documented in the official deprecations page: `report.extra`, `extra` fixture naming, direct `cells` reassignment, `py.xml`, and older collapse-format values.

## Official Sources Used

- Docs root: https://pytest-html.readthedocs.io/en/latest/
- User guide: https://pytest-html.readthedocs.io/en/latest/user_guide.html
- Installation page: https://pytest-html.readthedocs.io/en/latest/installing.html
- Deprecations: https://pytest-html.readthedocs.io/en/latest/deprecations.html
- Changelog page: https://pytest-html.readthedocs.io/en/latest/changelog.html
- PyPI: https://pypi.org/project/pytest-html/
- Maintainer release notes: https://github.com/pytest-dev/pytest-html/releases/tag/4.2.0
