---
name: bdd
description: "pytest-bdd package guide for Python projects using Gherkin feature files with pytest"
metadata:
  languages: "python"
  versions: "8.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest-bdd,pytest,bdd,gherkin,testing,python"
---

# pytest-bdd Python Package Guide

## Golden Rule

Use `pytest-bdd` as a thin layer on top of normal `pytest`: keep test state in fixtures, keep feature files readable, and bind scenarios explicitly from Python. Do not design around a separate BDD runner or a shared mutable "context" object.

## Install

Install `pytest-bdd` alongside `pytest` as a test dependency:

```bash
python -m pip install "pytest>=8" "pytest-bdd==8.1.0"
```

Common alternatives:

```bash
uv add --dev "pytest>=8" "pytest-bdd==8.1.0"
poetry add --group dev "pytest>=8" "pytest-bdd==8.1.0"
```

Run tests with plain `pytest`:

```bash
pytest
```

## Minimal Setup

Recommended layout:

```text
tests/
  features/
    publish_article.feature
  test_publish_article.py
pytest.ini
```

Set a base directory for feature files so step modules can use short relative paths:

```ini
# pytest.ini
[pytest]
bdd_features_base_dir = tests/features
markers =
    publishing: BDD tag from feature files
```

If you use `pytest --strict-markers`, register the markers you expect feature tags to create.

## Write A Feature File

Keep one Gherkin feature per `.feature` file:

```gherkin
Feature: Blog publishing
  Scenario: Publishing an article
    Given a draft article titled "BDD with pytest"
    When the article is published
    Then the article status is "published"
```

## Bind Scenarios And Implement Steps

```python
from pytest_bdd import given, parsers, scenario, then, when

@scenario("publish_article.feature", "Publishing an article")
def test_publishing_article():
    pass

@given(parsers.parse('a draft article titled "{title}"'), target_fixture="article")
def draft_article(title: str):
    return {"title": title, "status": "draft"}

@when("the article is published")
def publish_article(article):
    article["status"] = "published"

@then(parsers.parse('the article status is "{status}"'))
def assert_status(article, status: str):
    assert article["status"] == status
```

Key points:

- `@scenario(...)` binds a Python test function to a scenario in a `.feature` file.
- `target_fixture` lets a step provide or override a pytest fixture value.
- Step functions can depend on normal fixtures exactly like other pytest tests.

For many scenarios at once, use `scenarios(...)`:

```python
from pytest_bdd import scenarios

scenarios("features")
```

Use this for bulk binding, but keep any manual `@scenario(...)` declarations earlier in the file.

## Step Parsers And Parameters

`pytest-bdd` supports plain strings and parser helpers:

- `parsers.parse(...)` for typed placeholders such as `{count:d}`
- `parsers.cfparse(...)` for parse expressions with more converters
- `parsers.re(...)` for regular expressions when you need exact control

Example with typed parsing:

```python
from pytest_bdd import given, parsers

@given(parsers.parse("there are {count:d} users"), target_fixture="users")
def users(count: int):
    return list(range(count))
```

Use parser-based steps when values come from the feature file. Keep the parsing logic in the decorator instead of manually splitting strings inside the step function.

## Scenario Outlines

Use Gherkin `Scenario Outline` plus `Examples` for data-driven behavior:

```gherkin
Scenario Outline: Pricing
  Given a cart total of <total>
  Then the discount is <discount>

  Examples:
    | total | discount |
    | 20    | 0        |
    | 120   | 10       |
```

Match placeholders with parser-based steps:

```python
from pytest_bdd import given, parsers, then

@given(parsers.parse("a cart total of {total:d}"), target_fixture="cart_total")
def cart_total(total: int):
    return total

@then(parsers.parse("the discount is {discount:d}"))
def discount(cart_total: int, discount: int):
    actual = 10 if cart_total >= 100 else 0
    assert actual == discount
```

Prefer scenario outlines over stacking `@pytest.mark.parametrize` on top of BDD scenarios.

## Datatables, Docstrings, And Tags

`pytest-bdd` can pass richer Gherkin inputs into step functions:

- Docstrings are exposed as a `docstring` argument.
- Data tables are exposed as a `datatable` argument.
- Tags become pytest markers, so you can select with normal pytest filtering.

Example tag selection:

```bash
pytest -m publishing
```

If you need custom tag behavior, implement hooks such as `pytest_bdd_apply_tag`. Other useful hooks include scenario and step hooks for reporting or custom logging.

## Config And Test Initialization

There is no service authentication model in `pytest-bdd`; configuration is local to pytest and your test tree.

Use these setup rules:

- Put shared fixtures in `conftest.py`.
- Set `bdd_features_base_dir` in `pytest.ini` when feature files live outside the test module directory.
- Keep step text stable and human-readable; move conditional logic into fixtures and helpers.
- Use normal pytest fixture scopes (`function`, `module`, `session`) for expensive setup.

Example shared fixture:

```python
# tests/conftest.py
import pytest

@pytest.fixture
def app_client():
    return build_test_client()
```

## Common Pitfalls

- Only one feature is allowed per `.feature` file.
- Feature tags become pytest markers. With `--strict-markers`, undeclared tags will fail collection.
- `bdd_features_base_dir` is resolved from the pytest root directory, not from whatever shell directory you happen to be in.
- `parsers.re(...)` uses full matching in modern `pytest-bdd`; partial regex matches that used to pass can now fail.
- Parsed step arguments are regular function arguments, not pytest fixtures. Use `target_fixture` when a step should create or override fixture data.
- Do not mix scenario outlines and `@pytest.mark.parametrize` for the same behavior unless you have a very specific reason and have verified collection behavior.
- If you call `scenarios(...)`, keep any manual `@scenario(...)` bindings earlier in the module.
- In `8.1.0`, `datatable` and `docstring` are reserved step argument names for parser-based steps.

## Version-Sensitive Notes For 8.1.0

- `8.1.0` reserves `datatable` and `docstring` as parser argument names because those names are now used for Gherkin table and docstring injection.
- `8.1.0` also adds template variable rendering inside docstrings and datatables, which matters if your step logic reads multiline or tabular scenario data.
- The `8.0` line adds support for `Rule`, localized Gherkin with `# language: ...`, multiple example tables, and tags on individual examples.
- Since the `7.x` line, `parsers.re(...)` behaves as a full match, so older regex-based steps may need anchors or updated patterns during migration.

## Official Sources

- Stable docs: `https://pytest-bdd.readthedocs.io/en/stable/`
- Changelog and migration notes: `https://pytest-bdd.readthedocs.io/en/stable/#changelog`
- PyPI package metadata: `https://pypi.org/project/pytest-bdd/`
