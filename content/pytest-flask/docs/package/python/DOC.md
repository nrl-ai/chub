---
name: package
description: "pytest-flask package guide for testing Flask apps with pytest fixtures, clients, live servers, and config markers"
metadata:
  languages: "python"
  versions: "1.3.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest-flask,pytest,flask,testing,pytest-plugin,fixtures"
---

# pytest-flask Python Package Guide

## Golden Rule

Use `pytest-flask` as a pytest plugin layered on top of your Flask app's own test configuration. Define a real `app` fixture, enable `TESTING`, and target released `1.3.0` behavior instead of assuming the Read the Docs `latest` build is identical to the PyPI release.

As of the current upstream docs and package metadata:

- PyPI lists `pytest-flask 1.3.0`
- Read the Docs `stable` still shows `1.2.0`
- Read the Docs `latest` includes unreleased development changes

That version drift matters for fixture behavior and compatibility notes.

## Install

Install the plugin into the same environment as your Flask app and `pytest`:

```bash
python -m pip install "pytest-flask==1.3.0"
```

For a fresh test environment:

```bash
python -m pip install "Flask>=2,<4" "pytest>=7" "pytest-flask==1.3.0"
```

If your project already depends on Flask, you usually only add the plugin as a test dependency:

```bash
uv add --dev "pytest-flask==1.3.0"
poetry add --group test "pytest-flask==1.3.0"
```

## Environment And Prerequisites

`pytest-flask` has no package-specific authentication flow and no required package-specific environment variables.

The important prerequisites are in your Flask app and test config:

- `pytest` installed
- a Flask application available to tests
- `TESTING = True` in the test app config
- `SECRET_KEY` set if tests use `session`
- `SERVER_NAME` set when you need `url_for(..., _external=True)`

## Minimal Setup

`pytest-flask` does not create your Flask app for you. The core requirement is an `app` fixture in `conftest.py`.

```python
# tests/conftest.py
import pytest
from flask import Flask


@pytest.fixture
def app():
    app = Flask(__name__)
    app.config.update(
        TESTING=True,
        SECRET_KEY="test-secret",
        SERVER_NAME="example.test",
    )

    @app.get("/ping")
    def ping():
        return {"ok": True}

    return app
```

With that fixture in place, the plugin provides Flask-aware helpers like `client`, `config`, `client_class`, and `live_server`.

## Basic Request Tests With `client`

Use `client` for request-level tests. The plugin automatically manages request context around the test client so Flask globals such as `url_for`, `request`, and `session` are available without manually opening `app.test_request_context()`.

```python
# tests/test_ping.py
from flask import session, url_for


def test_ping(client):
    response = client.get(url_for("ping"))

    assert response.status_code == 200
    assert response.json == {"ok": True}
    assert session is not None
```

Common calls use the standard Flask test client API:

```python
def test_post_json(client):
    response = client.post("/items", json={"name": "widget"})
    assert response.status_code in {200, 201}
```

## Class-Based Tests With `client_class`

For class-based tests, use `client_class` to attach the client to `self.client`.

```python
class TestViews:
    def test_ping(self, client_class):
        response = self.client.get("/ping")
        assert response.json["ok"] is True
```

## Per-Test App Config With `config` And `pytest.mark.options`

Use `config` when you want to mutate `app.config` inside a test:

```python
def test_feature_flag(config, client):
    config["FEATURE_X_ENABLED"] = True

    response = client.get("/ping")

    assert response.status_code == 200
```

Use the `options` marker for per-test config overrides declared at the test level:

```python
import pytest


@pytest.mark.options(DEBUG=False, SERVER_NAME="example.test")
def test_options_marker(client):
    response = client.get("/ping")
    assert response.status_code == 200
```

This is the upstream-supported pattern for configuration overrides instead of mutating module-level globals.

## Accept Header Fixtures

The plugin includes header fixtures for common Accept values:

- `accept_any`
- `accept_json`
- `accept_jsonp`
- `accept_mimetype`

Example:

```python
def test_json_response(client, accept_json):
    response = client.get("/ping", headers=accept_json)
    assert response.status_code == 200
    assert response.mimetype == "application/json"
```

## App Factory Pattern

If your project uses an application factory, keep the fixture thin and create the app inside it:

```python
# tests/conftest.py
import pytest

from myapp import create_app


@pytest.fixture
def app():
    app = create_app()
    app.config.update(
        TESTING=True,
        SECRET_KEY="test-secret",
    )
    return app
```

This lets the rest of your tests keep using the same plugin fixtures.

## Live Server Tests

Use `live_server` when you need a real HTTP server instead of Flask's in-process test client, for example with browser tests or external callbacks.

```python
import json
from urllib.request import urlopen

from flask import url_for


def test_live_server(live_server):
    @live_server.app.get("/health")
    def health():
        return {"status": "ok"}

    live_server.start()

    with urlopen(url_for("health", _external=True)) as response:
        payload = json.load(response)

    assert payload == {"status": "ok"}
```

Relevant plugin controls documented upstream:

- `--start-live-server`
- `--no-start-live-server`
- `--live-server-port=PORT`
- `--live-server-wait=SECONDS`
- `live_server_scope = session|function` in `pytest.ini`

Recommended `pytest.ini` when tests need route registration before startup and better isolation:

```ini
[pytest]
addopts = --no-start-live-server
live_server_scope = function
```

When you need an external URL, prefer `url_for(..., _external=True)` or the plugin's live-server URL helpers instead of hard-coding `localhost`.

## Request Context And Sessions

One of the main reasons to use `pytest-flask` is automatic request-context handling around the test client. That lets you use Flask globals directly in tests after a request:

```python
from flask import session


def test_session_access(client):
    client.get("/ping")
    session["seen"] = True
    assert session["seen"] is True
```

For session-dependent tests, keep `SECRET_KEY` explicit in the test app config.

## Common Pitfalls

- `pytest-flask` requires an `app` fixture. Installing the plugin alone does not create an application or register routes.
- Do not assume Read the Docs `latest` matches the released `1.3.0` package. The docs site currently mixes released and unreleased content.
- The deprecated `request_ctx` fixture was removed in `1.3.0` for Flask `3.0` compatibility. Rewrite older examples around `client`, Flask globals, or explicit Flask context managers.
- `live_server` is session-scoped by default. Set `live_server_scope = function` if tests should not share routes or config.
- If you add routes inside a `live_server` test, use `--no-start-live-server` and call `live_server.start()` only after route registration.
- Fixed live-server ports are convenient for browser tools but can collide in CI. Prefer the default random port unless another tool requires a fixed port.
- If your app uses a custom response class, test against its actual response contract instead of assuming behavior from older plugin examples.

## Version Notes For `1.3.0`

- PyPI's latest released version is `1.3.0`.
- The `1.3.0` release removed the deprecated `request_ctx` fixture.
- The `1.3.0` release includes Flask `3.0.0` compatibility fixes.
- The Read the Docs `stable` site can miss `1.3.0` changes because it still renders `1.2.0`.

## Official Sources

- Docs index: `https://pytest-flask.readthedocs.io/en/latest/`
- Tutorial: `https://pytest-flask.readthedocs.io/en/latest/tutorial.html`
- Features and fixtures: `https://pytest-flask.readthedocs.io/en/latest/features.html`
- Changelog: `https://pytest-flask.readthedocs.io/en/latest/changelog.html`
- PyPI package page: `https://pypi.org/project/pytest-flask/`
