---
name: testing
description: "Flask-Testing 0.8.1 guide for unittest-style Flask application tests and live server tests"
metadata:
  languages: "python"
  versions: "0.8.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,flask-testing,testing,unittest,test-client,live-server"
---

# Flask-Testing for Flask Apps

## Install

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "Flask-Testing==0.8.1"
```

If you use template/context/flashed-message assertions, make sure Flask signals are available. Those helpers require `blinker` to be importable.

## Minimal `TestCase` Setup

`TestCase` expects you to implement `create_app()` and return a configured Flask app.

```python
from flask import Flask, flash, jsonify, redirect, render_template, url_for
from flask_testing import TestCase

class ViewTests(TestCase):
    def create_app(self) -> Flask:
        app = Flask(__name__)
        app.config.update(
            TESTING=True,
            SECRET_KEY="test-secret",
            SERVER_NAME="localhost",
        )

        @app.get("/")
        def index():
            flash("ready")
            return render_template("index.html", name="Ada")

        @app.get("/api")
        def api():
            return jsonify(ok=True)

        @app.get("/go")
        def go():
            return redirect(url_for("index"))

        return app

    def test_index(self) -> None:
        response = self.client.get("/")

        self.assert200(response)
        self.assert_template_used("index.html")
        self.assert_context("name", "Ada")
        self.assert_message_flashed("ready")

    def test_json(self) -> None:
        response = self.client.get("/api")

        self.assert200(response)
        assert response.json == {"ok": True}

    def test_redirect(self) -> None:
        response = self.client.get("/go")

        self.assert_redirects(response, url_for("index"))
```

What `TestCase` gives you:

- `self.app`: the Flask app returned by `create_app()`
- `self.client`: the Flask test client
- a pushed request context around each test method
- a custom response class with a `response.json` convenience property
- Flask-aware assertions such as `assert200`, `assert_template_used`, `assert_context`, and `assert_redirects`

In normal use, define `setUp()` and `tearDown()` like plain `unittest.TestCase` methods. You do not call `super().setUp()` or `super().tearDown()` for Flask-Testing's internal setup.

## Core Assertions And Helpers

Common helpers from `TestCase`:

- `assert200`, `assert404`, `assertStatus`
- `assertRedirects` / `assert_redirects`
- `assertTemplateUsed` / `assert_template_used`
- `assertContext` / `assert_context`
- `getContextVariable` / `get_context_variable`
- `assertMessageFlashed` / `assert_message_flashed`

The package keeps both camelCase and snake_case aliases. Match the style already used in the codebase you are editing.

### Template and context assertions

These helpers rely on Flask's template-rendered signal. If signals are unavailable, Flask-Testing raises a runtime error instead of silently passing.

If you want template assertions without paying template-render cost, set:

```python
class FastTemplateTests(TestCase):
    render_templates = False
```

With `render_templates = False`, template rendering is skipped for the test case, so the response body from `render_template(...)` is not useful for HTML assertions. Use template/context assertions instead.

### Redirect assertions

`assert_redirects(response, location)` compares both the status code and the target URL. For `0.8.1`, the maintained source accepts these redirect status codes:

- `301`
- `302`
- `303`
- `305`
- `307`

Pitfall:

- `308` is not accepted by `assertRedirects()` in `0.8.1`, even though newer Flask and Werkzeug code may emit it.

If your app uses relative redirect locations, set `SERVER_NAME` in the test config so URL normalization stays predictable.

## App Factory And Config Patterns

Keep the app factory explicit and set testing config inside `create_app()`:

```python
from flask import Flask
from flask_testing import TestCase

class MyTests(TestCase):
    def create_app(self) -> Flask:
        app = Flask(__name__)
        app.config.update(
            TESTING=True,
            SECRET_KEY="test-secret",
            SQLALCHEMY_DATABASE_URI="sqlite://",
            WTF_CSRF_ENABLED=False,
        )
        return app
```

Important config that frequently matters in tests:

- `TESTING=True`: enables Flask testing behavior and better error propagation
- `SECRET_KEY`: required for sessions and flashed messages
- `SERVER_NAME`: helps with `url_for(..., _external=True)` and redirect assertions
- extension-specific test settings such as `SQLALCHEMY_DATABASE_URI`, `MAIL_SUPPRESS_SEND`, or `WTF_CSRF_ENABLED`

Pitfall:

- Older docs show patterns like `TESTING = True` or `SQLALCHEMY_DATABASE_URI = "sqlite://"` as class attributes. Flask-Testing does not automatically merge those into `app.config`; your `create_app()` implementation must do that itself, or your app factory must explicitly read from the test case object.

## Database Setup Pattern

The official docs include a Flask-SQLAlchemy example. The practical pattern is:

```python
from flask_testing import TestCase

from myapp import create_app, db

class DatabaseTests(TestCase):
    def create_app(self):
        app = create_app()
        app.config.update(
            TESTING=True,
            SQLALCHEMY_DATABASE_URI="sqlite://",
        )
        return app

    def setUp(self):
        db.create_all()

    def tearDown(self):
        db.session.remove()
        db.drop_all()
```

Use a disposable database per test class or per test run. `self.client` requests execute real Flask request teardown behavior, so extension sessions may be removed between requests just like in the app itself.

## Live Server Tests

Use `LiveServerTestCase` when you need a real HTTP server instead of Flask's in-process test client. This is the path for browser-driven tests, external callbacks, or anything that must talk to an actual socket.

```python
from flask import Flask
from flask_testing import LiveServerTestCase
from urllib.request import urlopen

class LiveTests(LiveServerTestCase):
    LIVESERVER_PORT = 0
    LIVESERVER_TIMEOUT = 10

    def create_app(self) -> Flask:
        app = Flask(__name__)
        app.config["TESTING"] = True

        @app.get("/")
        def index():
            return "ok"

        return app

    def test_server_is_reachable(self) -> None:
        response = urlopen(self.get_server_url())
        assert response.status == 200
```

Key behavior:

- `LIVESERVER_PORT = 0` asks the OS for a free port, which is safer for parallel runs
- `get_server_url()` returns the actual base URL to call
- `LIVESERVER_TIMEOUT` controls how long startup waits before failing
- the live server runs in a separate process, so share test state through a real database or other externalized state, not in-memory globals

## Common Pitfalls

- Use `from flask_testing import TestCase`, not the legacy `flask.ext.testing` import path.
- The docs site is stale; always cross-check version-sensitive behavior against PyPI and the maintainer repository.
- Signal-based assertions require Flask signals. If they fail with a runtime error, check whether `blinker` is available in the environment.
- `assertRedirects()` in `0.8.1` does not treat HTTP `308` as valid.
- `render_templates = False` is good for speed but bad for asserting rendered HTML output.
- `LiveServerTestCase` is not the same as Flask's test client; requests go through a real socket and process boundary.
- Twill-related helpers in the old docs are effectively legacy paths. Prefer modern browser or HTTP tooling instead.

## Version-Sensitive Notes

- Public docs and changelog pages lag behind the `0.8.1` package published on PyPI.
- The GitHub release UI is also behind PyPI for this package, so use PyPI for the latest published package version and GitHub source for exact helper behavior.
- Flask-Testing is still centered on `unittest`. If the project is already strongly pytest-native, plain Flask fixtures or a pytest-focused plugin may fit better than wrapping everything in `TestCase`.
