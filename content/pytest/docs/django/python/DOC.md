---
name: django
description: "pytest-django package guide for running Django tests with pytest"
metadata:
  languages: "python"
  versions: "4.12.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "pytest-django,pytest,django,testing,fixtures,database"
---

# pytest-django Python Package Guide

## Golden Rule

Use `pytest` as the test runner, point `pytest-django` at your Django settings before collection starts, and request database access explicitly with `@pytest.mark.django_db` or a database fixture. As of March 11, 2026, PyPI has `pytest-django 4.12.0`, but the Read the Docs changelog page still labels `4.12.0` as "Not released yet", so treat PyPI as the release-of-record for the package version.

## Install

Pin the plugin version to the project series you expect:

```bash
python -m pip install "pytest-django==4.12.0"
```

In most projects you also pin `pytest` itself:

```bash
python -m pip install "pytest>=7,<9" "pytest-django==4.12.0"
```

Common alternatives:

```bash
uv add "pytest>=7,<9" "pytest-django==4.12.0"
poetry add --group dev "pytest>=7,<9" "pytest-django==4.12.0"
```

## Minimal Setup

The plugin needs Django settings before tests import Django models or apps. The common stable setup is `pytest.ini`:

```ini
[pytest]
DJANGO_SETTINGS_MODULE = myproject.settings
python_files = tests.py test_*.py *_tests.py
addopts = --reuse-db
```

Equivalent `pyproject.toml`:

```toml
[tool.pytest.ini_options]
DJANGO_SETTINGS_MODULE = "myproject.settings"
python_files = ["tests.py", "test_*.py", "*_tests.py"]
addopts = "--reuse-db"
```

The settings resolution order is:

1. `--ds=<settings.module>`
2. `DJANGO_SETTINGS_MODULE` environment variable
3. config file value in `pytest.ini`, `tox.ini`, or `pyproject.toml`

Examples:

```bash
pytest --ds=myproject.settings
DJANGO_SETTINGS_MODULE=myproject.settings pytest
```

If the project uses `django-configurations`, also pass `--dc=ConfigName` or set `DJANGO_CONFIGURATION`.

## Project Discovery And Import Path

By default, `pytest-django` searches for `manage.py` and adds that project directory to `sys.path`. That is convenient for standard layouts, but it can be wrong in monorepos or `src/` layouts.

Disable auto-discovery when you want explicit imports:

```ini
[pytest]
django_find_project = false
DJANGO_SETTINGS_MODULE = myproject.settings
```

Then ensure the Django project is importable yourself, for example with an editable install:

```bash
python -m pip install -e .
```

Use this explicit mode if pytest is picking the wrong `manage.py` or the project is not rooted at the repository top level.

## Core Usage

### Fast unit test with no database

```python
def test_plain_python_logic():
    assert 2 + 2 == 4
```

### Model test that needs the database

```python
import pytest

from blog.models import Post

@pytest.mark.django_db
def test_post_str():
    post = Post.objects.create(title="Hello")
    assert str(post) == "Hello"
```

`pytest-django` blocks database access by default. That guard is intentional and catches accidental slow integration tests.

### Transactional test

Use transactional access when the code under test depends on real transaction behavior:

```python
import pytest

@pytest.mark.django_db(transaction=True)
def test_select_for_update_path():
    ...
```

Fixture equivalents:

- `db`: normal test database access with transaction rollback
- `transactional_db`: slower, but closer to real transaction behavior

### Multi-database access

`4.12.0` documents non-experimental support for multiple databases:

```python
import pytest

@pytest.mark.django_db(databases=["default", "other"])
def test_reads_from_replica():
    ...
```

### Client-based view test

```python
import pytest
from django.urls import reverse

@pytest.mark.django_db
def test_dashboard_requires_login(client, django_user_model):
    user = django_user_model.objects.create_user(
        username="alice",
        password="secret",
    )
    client.force_login(user)

    response = client.get(reverse("dashboard"))

    assert response.status_code == 200
```

High-value built-in fixtures:

- `client`: Django test client
- `async_client`: async test client for async views
- `rf`: `RequestFactory`
- `async_rf`: async request factory
- `settings`: temporary settings override fixture
- `django_user_model`: active user model
- `admin_user`, `admin_client`: ready-made admin-authenticated fixtures
- `live_server`: server fixture for browser or external HTTP tests

### Temporary settings override

```python
def test_feature_flag(settings):
    settings.MY_FEATURE_ENABLED = True
    assert settings.MY_FEATURE_ENABLED is True
```

## Database Lifecycle And Performance

The test database is created on first use and then reused inside the current pytest session. For repeated local runs, `--reuse-db` is the standard speed knob:

```bash
pytest --reuse-db
```

Important companion flags:

- `--create-db`: rebuild the test database even if `--reuse-db` metadata exists
- `--no-migrations`: build the test database from models instead of running migrations

Use `--create-db` after schema changes. `--reuse-db` does not automatically detect that your migrations changed.

Use `--no-migrations` only as a local speed optimization. It can hide migration problems that matter in CI or production.

For custom global database setup, override the session-scoped `django_db_setup` fixture in `conftest.py`.

## Auth And Request Testing

`pytest-django` itself has no external auth configuration layer; auth behavior comes from Django. The fastest stable patterns are:

- create users with `django_user_model`
- authenticate browser-style tests with `client.force_login(user)`
- use `admin_client` for built-in admin tests
- use `rf` only when you want to call a view directly without middleware

`rf` is not the same as `client`: middleware, sessions, and auth are not run automatically. If your code expects `request.user`, session state, or CSRF behavior, prefer `client` unless you are deliberately unit-testing a narrow view function.

## Async And Live Server Notes

Use `async_client` or `async_rf` for async views, and keep the test function compatible with the async pytest stack your project already uses, typically `pytest-asyncio`.

`live_server` depends on `transactional_db`. Because each test starts with a clean database state, do not assume data created in one test will persist into another browser-style or live-server test.

## Common Pitfalls

- Missing settings: `ImproperlyConfigured` during collection usually means `DJANGO_SETTINGS_MODULE` or `--ds` was not set early enough.
- Wrong import path: auto-discovery found the wrong `manage.py`; disable it with `django_find_project = false` and make imports explicit.
- Database access denied: add `@pytest.mark.django_db`, `db`, or `transactional_db`.
- Reused stale schema: after changing migrations, rerun with `pytest --create-db`.
- RequestFactory confusion: `rf` does not execute middleware or login flows.
- Hidden migration bugs: `--no-migrations` is useful for local speed, but it is not a substitute for testing real migrations.
- Async tests hanging or being skipped: install and configure the async pytest plugin your project uses; `pytest-django` does not replace that layer.

## Version-Sensitive Notes

- PyPI lists `pytest-django 4.12.0` as released on February 14, 2026.
- The Read the Docs changelog currently still shows `v4.12.0 (Not released yet)`, so the changelog page is lagging the package registry.
- The `4.12.0` changelog entry documents multiple-database support as no longer experimental.
- If you are maintaining an older project, verify the Django and Python version classifiers on PyPI before copying examples into a project pinned below Django `4.2` or Python `3.10`.

## Official Sources

- Docs root: `https://pytest-django.readthedocs.io/en/latest/`
- Tutorial: `https://pytest-django.readthedocs.io/en/latest/tutorial.html`
- Django configuration: `https://pytest-django.readthedocs.io/en/latest/configuring_django.html`
- Python path and project discovery: `https://pytest-django.readthedocs.io/en/latest/managing_python_path.html`
- Database usage: `https://pytest-django.readthedocs.io/en/latest/database.html`
- Fixtures and helpers: `https://pytest-django.readthedocs.io/en/latest/helpers.html`
- Changelog: `https://pytest-django.readthedocs.io/en/latest/changelog.html`
- PyPI package page: `https://pypi.org/project/pytest-django/`
