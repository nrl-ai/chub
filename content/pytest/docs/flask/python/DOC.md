---
name: flask
description: "pytest-flask plugin guide for testing Flask apps with pytest fixtures, clients, live servers, and config markers"
metadata:
  languages: "python"
  versions: "1.3.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest,flask,testing,pytest-plugin,fixtures"
---

# pytest-flask Python Package Guide

## Golden Rule

Use `pytest-flask` as a pytest plugin layered on top of your Flask app's own test configuration. Define a real `app` fixture, enable `TESTING`, and target released `1.3.0` behavior instead of assuming the Read the Docs `latest` build is stable.

As of March 12, 2026:

- PyPI lists `pytest-flask 1.3.0`
- Read the Docs `stable` is still `1.2.0`
- Read the Docs `latest` is a dev build (`1.3.1.dev...`) with unreleased changes

That version drift matters for fixture behavior and supported Python versions.

## Install

Install the plugin into the same environment as your Flask app and `pytest`:

```bash
python -m pip install "pytest-flask==1.3.0"
```

For a fresh test environment:

```bash
python -m pip install "Flask>=2,<4" "pytest>=7" "pytest-flask==1.3.0"
```

If your project already depends on Flask, you usually only add the plugin:

```bash
uv add --dev "pytest-flask==1.3.0"
poetry add --group test "pytest-flask==1.3.0"
```

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
    )

    @app.get("/ping")
    def ping():
        return {"ok": True}

    return app
```

With that fixture in place, the plugin provides Flask-aware test helpers.

## Core Usage

### `client` fixture

Use `client` for request-level tests. `pytest-flask` automatically pushes a request context around the test client, so Flask globals like `url_for`, `request`, and `session` are available without wrapping each call in `app.test_request_context()`.

```python
from flask import session, url_for

def test_ping(client):
    response = client.get(url_for("ping"))

    assert response.status_code == 200
    assert response.json == {"ok": True}
    assert session is not None
```

### `client_class` fixture

For class-based tests, use `client_class` to attach the test client to `self.client`.

```python
class TestViews:
    def test_ping(self, client_class):
        response = self.client.get("/ping")
        assert response.json["ok"] is True
```

### `config` fixture

Use `config` to mutate `app.config` for a specific test.

```python
def test_feature_flag(config, client):
    config["FEATURE_X_ENABLED"] = True

    response = client.get("/ping")

    assert response.status_code == 200
```

### Content-negotiation fixtures

The plugin includes request-header fixtures for common Accept values:

- `accept_any`
- `accept_json`
- `accept_jsonp`
- `accept_mimetype`

Example:

```python
def test_json_response(client, accept_json):
    response = client.get("/ping", headers=accept_json)
    assert response.is_json
```

### `pytest.mark.options`

Use the `options` marker for per-test config overrides instead of mutating global config in module scope.

```python
import pytest

@pytest.mark.options(DEBUG=False, SERVER_NAME="example.test")
def test_options_marker(client):
    response = client.get("/ping")
    assert response.status_code == 200
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

This keeps the rest of your tests using the same `client`, `config`, and `live_server` fixtures.

## Live Server Tests

Use `live_server` when you need a real HTTP server instead of Flask's in-process test client, for example with browser tests or external callbacks.

Basic pattern:

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

Relevant CLI and config controls from the upstream docs:

- `--start-live-server`: start automatically when the fixture is requested
- `--no-start-live-server`: require explicit `live_server.start()`
- `--live-server-port=PORT`: bind a fixed port instead of a random one
- `--live-server-wait=SECONDS`: wait time before considering startup failed
- `live_server_scope = session|function`: configure fixture scope in `pytest.ini`

Recommended `pytest.ini` when tests need route registration before startup and better isolation:

```ini
[pytest]
addopts = --no-start-live-server
live_server_scope = function
```

When you need the server's base URL in requests or browser automation, use `url_for(..., _external=True)` or the `live_server` URL helpers from the plugin rather than hard-coding `localhost`.

## Request Context And Sessions

One of the main reasons to use `pytest-flask` instead of raw Flask fixtures is automatic request-context handling. That lets you write tests like this:

```python
from flask import session

def test_session_access(client):
    client.get("/ping")
    session["seen"] = True
    assert session["seen"] is True
```

For session-dependent tests, make sure the fixture config sets a deterministic `SECRET_KEY`.

## Configuration Notes

There is no package-level authentication model in `pytest-flask`. Configuration is about test process behavior and your Flask app settings.

Keep these settings explicit in tests:

- `TESTING = True`
- `SECRET_KEY` set for session usage
- `SERVER_NAME` if you rely on `url_for(..., _external=True)`
- database or extension settings overridden to point at test resources

A practical pattern is to centralize app config in the `app` fixture and use `pytest.mark.options` or the `config` fixture for test-specific overrides.

## Common Pitfalls

- `pytest-flask` requires an `app` fixture. Installing the plugin alone does not create a Flask application or register routes for you.
- Do not treat Read the Docs `latest` as equivalent to released `1.3.0`. The official docs site currently mixes a stale `stable` build and a newer unreleased `latest` build.
- `request_ctx` was removed in `1.3.0` for Flask 3.0 compatibility. If older tests or blog posts rely on it, rewrite them around `client`, `session`, `request`, or explicit Flask context managers.
- `live_server` is session-scoped by default. That can leak routes or config between tests unless you set `live_server_scope = function`.
- Fixed live-server ports are convenient for browser tools but can collide in parallel CI. Prefer the default random port unless an external dependency requires a known port.
- If you add routes inside a `live_server` test, use `--no-start-live-server` and call `live_server.start()` after route registration.
- The plugin will not overwrite a custom response class that already defines `json`. If your app customizes response behavior, test against your own response contract, not assumptions from older examples.

## Version-Sensitive Notes

- PyPI's latest released version is `1.3.0`, published on October 23, 2023.
- The `1.3.0` release removed the deprecated `request_ctx` fixture and added compatibility fixes for Flask `3.0.0`.
- The Read the Docs `stable` site still renders `1.2.0`, so it can miss released `1.3.0` changes.
- The Read the Docs `latest` site shows a `1.3.1.dev...` build. Its changelog includes unreleased work such as Python `3.10`-`3.12` support, dropping `3.7`, and improved typing. Treat those items as upcoming or branch-head behavior until a newer PyPI release exists.
- PyPI metadata still declares `Python >=3.7`. If your project is on modern Python, that is probably fine in practice, but for strict compatibility claims prefer the released metadata over the unreleased docs.

## Official Sources

- Docs index: `https://pytest-flask.readthedocs.io/en/latest/`
- Tutorial: `https://pytest-flask.readthedocs.io/en/latest/tutorial.html`
- Features and fixtures: `https://pytest-flask.readthedocs.io/en/latest/features.html`
- Changelog: `https://pytest-flask.readthedocs.io/en/latest/changelog.html`
- PyPI package page: `https://pypi.org/project/pytest-flask/`
