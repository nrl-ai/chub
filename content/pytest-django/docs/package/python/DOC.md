---
name: package
description: "pytest-django package guide for running Django tests with pytest"
metadata:
  languages: "python"
  versions: "4.12.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest-django,pytest,django,testing,fixtures,database"
---

# pytest-django Python Package Guide

## What This Package Does

Use `pytest-django` to run Django tests with `pytest`, load Django settings before test collection, and opt into database access only for tests that need it. There is no separate service client or auth layer; the main setup step is telling pytest which Django settings module to use.

## Install

Install `pytest-django` alongside `pytest` in your development dependencies:

```bash
python -m pip install "pytest>=7,<9" "pytest-django==4.12.0"
```

Common alternatives:

```bash
uv add --dev "pytest>=7,<9" "pytest-django==4.12.0"
poetry add --group dev "pytest>=7,<9" "pytest-django==4.12.0"
```

## Minimal Setup

The plugin must know your Django settings module before tests import models, apps, or anything else that touches Django setup.

Use `pytest.ini`:

```ini
[pytest]
DJANGO_SETTINGS_MODULE = myproject.settings
python_files = tests.py test_*.py *_tests.py
addopts = --reuse-db
```

Or the equivalent `pyproject.toml` config:

```toml
[tool.pytest.ini_options]
DJANGO_SETTINGS_MODULE = "myproject.settings"
python_files = ["tests.py", "test_*.py", "*_tests.py"]
addopts = "--reuse-db"
```

The settings resolution order is:

1. `--ds=myproject.settings`
2. `DJANGO_SETTINGS_MODULE` environment variable
3. The value in `pytest.ini`, `tox.ini`, or `pyproject.toml`

Examples:

```bash
pytest --ds=myproject.settings
DJANGO_SETTINGS_MODULE=myproject.settings pytest
```

If your project uses `django-configurations`, also provide the configuration class:

```bash
pytest --ds=myproject.settings --dc=Dev
DJANGO_SETTINGS_MODULE=myproject.settings DJANGO_CONFIGURATION=Dev pytest
```

## Import Path And Project Discovery

By default, `pytest-django` looks for `manage.py` and adds that project directory to `sys.path`. That is convenient for the standard Django layout, but it can be wrong in monorepos or `src/` layouts.

Disable project auto-discovery when you want imports to stay explicit:

```ini
[pytest]
django_find_project = false
DJANGO_SETTINGS_MODULE = myproject.settings
```

Then make the project importable yourself, usually with an editable install:

```bash
python -m pip install -e .
```

Use this mode if pytest is picking the wrong `manage.py` or your Django package is not rooted at the repository top level.

## Common Test Workflows

### Pure Python test with no database

```python
def test_plain_python_logic():
    assert 2 + 2 == 4
```

### Model test with database access

`pytest-django` blocks database access by default. Add the marker when a test needs ORM access:

```python
import pytest

from blog.models import Post


@pytest.mark.django_db
def test_post_str():
    post = Post.objects.create(title="Hello")
    assert str(post) == "Hello"
```

Fixture equivalents:

- `db`: standard database access with rollback between tests
- `transactional_db`: slower, but uses real transaction behavior

### Transaction-sensitive test

Use transactional access when the code under test depends on commits, rollbacks, or database locking behavior:

```python
import pytest


@pytest.mark.django_db(transaction=True)
def test_select_for_update_path():
    ...
```

### Multi-database test

If the project uses more than one Django database alias, declare the aliases explicitly:

```python
import pytest


@pytest.mark.django_db(databases=["default", "other"])
def test_reads_from_replica():
    ...
```

### Client-based view test

Use the built-in Django test client fixture for request, session, and auth flows:

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

### Temporary settings override

Use the `settings` fixture for per-test overrides:

```python
def test_feature_flag(settings):
    settings.MY_FEATURE_ENABLED = True
    assert settings.MY_FEATURE_ENABLED is True
```

## High-Value Fixtures

- `client`: Django test client
- `async_client`: async test client for async views
- `rf`: `django.test.RequestFactory`
- `async_rf`: async request factory
- `settings`: temporary settings overrides
- `django_user_model`: the active user model
- `admin_user`, `admin_client`: ready-made admin fixtures
- `live_server`: test server for browser or external HTTP tests

Use `rf` only when you want to call a view directly without middleware. If your code depends on sessions, authentication, messages, or CSRF behavior, prefer `client`.

## Database Lifecycle And Performance

The test database is created on first use and then reused inside the current pytest run. For repeated local runs, `--reuse-db` is the standard speed optimization:

```bash
pytest --reuse-db
```

Important companion flags:

- `--create-db`: rebuild the test database even if reuse metadata exists
- `--no-migrations`: build the test database from models instead of running migrations

Use `--create-db` after schema changes. `--reuse-db` does not detect new migrations automatically.

Use `--no-migrations` only as a local speed optimization. It can hide migration bugs that still matter in CI and production.

For custom global database setup, override the session-scoped `django_db_setup` fixture in `conftest.py`.

## Async And Live Server Notes

Use `async_client` or `async_rf` for async views, and keep the test function compatible with the async pytest plugin your project already uses, typically `pytest-asyncio`.

`live_server` depends on transactional database access. Do not assume data created in one live-server test will persist into another test.

## Common Pitfalls

- `ImproperlyConfigured` during collection usually means `DJANGO_SETTINGS_MODULE` or `--ds` was not set early enough.
- If pytest discovers the wrong project root, set `django_find_project = false` and make imports explicit.
- If database access is denied, add `@pytest.mark.django_db`, `db`, or `transactional_db`.
- After changing models or migrations, rerun with `pytest --create-db` if you are reusing the test database.
- `rf` does not run middleware or login flows; use `client` for end-to-end request behavior.
- `--no-migrations` is useful for local speed, but it is not a replacement for testing real migrations.
- `pytest-django` does not replace your async pytest plugin.

## Version Notes

- This guide targets `pytest-django 4.12.0`.
- If your project is pinned to older Django or Python versions, check the package classifiers and changelog before copying this setup unchanged.

## Official Sources

- Docs root: `https://pytest-django.readthedocs.io/en/latest/`
- Tutorial: `https://pytest-django.readthedocs.io/en/latest/tutorial.html`
- Django configuration: `https://pytest-django.readthedocs.io/en/latest/configuring_django.html`
- Python path and project discovery: `https://pytest-django.readthedocs.io/en/latest/managing_python_path.html`
- Database usage: `https://pytest-django.readthedocs.io/en/latest/database.html`
- Fixtures and helpers: `https://pytest-django.readthedocs.io/en/latest/helpers.html`
- Changelog: `https://pytest-django.readthedocs.io/en/latest/changelog.html`
- PyPI package page: `https://pypi.org/project/pytest-django/`
