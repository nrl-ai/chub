---
name: package
description: "Flask 3.1.3 package guide for building and testing Python web applications"
metadata:
  languages: "python"
  versions: "3.1.3"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "flask,python,web,wsgi,jinja,werkzeug"
---

# Flask Python Package Guide

## Install

Flask 3.1.3 requires Python 3.9+.

```bash
python -m venv .venv
source .venv/bin/activate
pip install "Flask==3.1.3"
```

Useful extras:

```bash
pip install "Flask[dotenv]==3.1.3"
pip install "Flask[async]==3.1.3"
```

- `dotenv` adds `.env` / `.flaskenv` loading support for the `flask` CLI.
- `async` enables `async def` views and other async request hooks.

## Minimal App

```python
from flask import Flask

app = Flask(__name__)

@app.get("/")
def healthcheck():
    return {"ok": True}
```

Run it with the CLI:

```bash
flask --app app run --debug
```

Important:

- Do not name your module `flask.py`; it conflicts with the package import.
- Use `--app` unless your entry file is named `app.py` or `wsgi.py`.
- Use `--debug` at startup rather than trying to flip `DEBUG` later in code.
- The built-in server is for development only, not production.

## Recommended App Factory Layout

For any non-trivial project, prefer an application factory and blueprints. This makes testing easier and avoids binding extension state too early.

`myapp/__init__.py`

```python
from flask import Flask

def create_app(test_config: dict | None = None) -> Flask:
    app = Flask(__name__, instance_relative_config=True)

    app.config.from_mapping(
        SECRET_KEY="dev-only-change-me",
        DATABASE_URL="sqlite:///app.db",
    )

    if test_config:
        app.config.update(test_config)
    else:
        app.config.from_prefixed_env()

    from .views import bp
    app.register_blueprint(bp)

    return app
```

`myapp/views.py`

```python
from flask import Blueprint, current_app, jsonify

bp = Blueprint("main", __name__)

@bp.get("/")
def index():
    return jsonify(
        app_name=current_app.name,
        debug=current_app.debug,
    )
```

Run a factory app:

```bash
flask --app myapp:create_app run --debug
```

Flask will auto-detect a factory named `create_app` or `make_app`.

## Core Request and Response Patterns

### Route methods and path params

```python
from flask import Flask, abort, jsonify, request

app = Flask(__name__)

@app.get("/users/<int:user_id>")
def get_user(user_id: int):
    verbose = request.args.get("verbose") == "1"

    user = {"id": user_id, "name": "Ada"}
    if not user:
        abort(404)

    if verbose:
        user["debug"] = {"remote_addr": request.remote_addr}

    return jsonify(user)

@app.post("/users")
def create_user():
    payload = request.get_json(force=False, silent=False)
    if not payload or "name" not in payload:
        abort(400)

    return jsonify(id=123, name=payload["name"]), 201
```

Use:

- `request.args` for query parameters
- `request.form` for HTML form fields
- `request.files` for uploads
- `request.get_json()` for JSON request bodies
- `jsonify(...)` or dict return values for JSON responses
- `abort(status_code)` for simple HTTP errors

### Templates and static assets

```python
from flask import Flask, render_template

app = Flask(__name__)

@app.get("/hello/<name>")
def hello(name: str):
    return render_template("hello.html", name=name)
```

- Templates live under `templates/`
- Static files live under `static/`
- `__name__` on the `Flask(...)` app tells Flask where to find those resources

### Sessions

```python
from flask import Flask, redirect, session, url_for

app = Flask(__name__)
app.config["SECRET_KEY"] = "replace-in-production"

@app.post("/login")
def login():
    session["user_id"] = 42
    session.permanent = True
    return redirect(url_for("me"))

@app.get("/me")
def me():
    return {"user_id": session.get("user_id")}
```

Flask's built-in session uses a signed cookie, not a server-side session store. Keep session payloads small and non-sensitive.

## Configuration and Secrets

Flask configuration lives on `app.config`, which behaves like a dict.

```python
app.config.update(
    TESTING=False,
    SECRET_KEY="replace-me",
    TRUSTED_HOSTS=["example.com", ".example.com"],
    MAX_CONTENT_LENGTH=16 * 1024 * 1024,
    SESSION_COOKIE_SECURE=True,
    SESSION_COOKIE_SAMESITE="Lax",
)
```

Prefer loading environment-driven config in the factory:

```python
def create_app():
    app = Flask(__name__)
    app.config.from_prefixed_env()
    return app
```

With the default `FLASK_` prefix:

```bash
export FLASK_SECRET_KEY="$(python -c 'import secrets; print(secrets.token_hex())')"
export FLASK_MAIL_ENABLED=false
flask --app myapp:create_app run --debug
```

Key config to know in 3.1.x:

- `SECRET_KEY`: required for sessions, flash messages, and many extensions.
- `SECRET_KEY_FALLBACKS`: lets you rotate signing keys without immediately invalidating active sessions.
- `TRUSTED_HOSTS`: validates the incoming host header during routing. Use this instead of assuming `SERVER_NAME` will restrict hosts.
- `MAX_CONTENT_LENGTH`: caps request body size.
- `MAX_FORM_MEMORY_SIZE` and `MAX_FORM_PARTS`: new in 3.1 for multipart form limits.
- `SESSION_COOKIE_SECURE`: send cookies only over HTTPS.
- `SESSION_COOKIE_SAMESITE`: usually `"Lax"` unless cross-site behavior is required.
- `SESSION_COOKIE_PARTITIONED`: new in 3.1 for embedded third-party cookie scenarios; enabling it also requires secure cookies.
- `TESTING`: enable in tests so exceptions propagate and extensions switch into test-friendly behavior.

Auth note:

- Flask does not ship with a full authentication system.
- Store auth provider secrets in config or environment, not inline in code.

## Testing

The official docs recommend `pytest` and Flask's built-in test client / CLI runner.

`tests/conftest.py`

```python
import pytest

from myapp import create_app

@pytest.fixture()
def app():
    app = create_app({"TESTING": True, "SECRET_KEY": "test-secret"})
    yield app

@pytest.fixture()
def client(app):
    return app.test_client()

@pytest.fixture()
def runner(app):
    return app.test_cli_runner()
```

`tests/test_app.py`

```python
from flask import session

def test_index(client):
    response = client.get("/")
    assert response.status_code == 200

def test_redirect_chain(client):
    response = client.get("/logout", follow_redirects=True)
    assert response.status_code == 200
    assert response.history

def test_modify_session(client):
    with client.session_transaction() as sess:
        sess["user_id"] = 1

    response = client.get("/me")
    assert response.json["user_id"] == 1

def test_context_access(client):
    with client:
        client.post("/login")
        assert session["user_id"] == 42
```

Useful testing primitives:

- `app.test_client()` for HTTP requests without a live server
- `follow_redirects=True` to assert on final redirected responses
- `client.session_transaction()` to seed or inspect session state
- `app.test_cli_runner()` for `@app.cli.command(...)` tests
- `with app.app_context():` when testing code that needs `current_app`, database bindings, or extension state

## Async Support

Flask 3.1.3 supports async views and async request hooks if installed with the `async` extra.

```python
from flask import Flask, jsonify

app = Flask(__name__)

@app.get("/aggregate")
async def aggregate():
    result = await fetch_remote_data()
    return jsonify(result)
```

Important limitations:

- Flask remains a WSGI framework.
- One worker still handles one request/response cycle.
- Async helps with concurrent IO inside a view, not with serving more requests per worker.
- Do not spawn background tasks with `asyncio.create_task()` from a view; unfinished tasks are cancelled when the async view completes.

If most of your stack is async-first, or you need websockets and long-lived async workloads, an ASGI-native framework may be a better fit.

## Deployment

Do not deploy the development server.

Use a production WSGI server or managed platform, for example:

- Gunicorn
- Waitress
- mod_wsgi
- uWSGI
- gevent-based hosting

Common production concerns:

- terminate TLS before Flask or at the reverse proxy
- set `TRUSTED_HOSTS`
- set secure cookie flags
- configure body/form limits
- if behind a proxy, configure proxy handling correctly before trusting forwarded headers

## Common Pitfalls

- Module named `flask.py`: breaks imports.
- Missing `SECRET_KEY`: sessions and flash messages will not work correctly.
- Late debug changes: `DEBUG` may behave inconsistently if changed after startup; use `flask --debug`.
- Using `SERVER_NAME` as host protection: in 3.1 it no longer restricts requests to that domain; use `TRUSTED_HOSTS`.
- Large uploads with no limits: set `MAX_CONTENT_LENGTH`, and for multipart forms also set `MAX_FORM_MEMORY_SIZE` and `MAX_FORM_PARTS`.
- Assuming Flask sessions are encrypted server-side: they are signed cookies by default, so don't store secrets or large blobs in them.
- Binding extensions directly inside module import paths: prefer `extension = Extension()` and `extension.init_app(app)` in the factory.
- Copying old blog posts that use `FLASK_ENV` or `app.env`: those were removed in Flask 2.3.
- Relying on `flask.__version__`: deprecated since 3.0. Use `importlib.metadata.version("flask")`.

## Version-Sensitive Notes for 3.1.3

- `3.1.3` is the current PyPI release as of 2026-03-11.
- `3.1.3` is a security-fix release and should not otherwise change behavior relative to the latest feature release.
- `3.1.2` fixed async `stream_with_context` behavior and corrected session state when using `follow_redirects` in tests.
- `3.1.1` fixed signing key selection order when `SECRET_KEY_FALLBACKS` is enabled.
- `3.1.0` added `SECRET_KEY_FALLBACKS`, `TRUSTED_HOSTS`, `MAX_FORM_MEMORY_SIZE`, `MAX_FORM_PARTS`, and `SESSION_COOKIE_PARTITIONED`.
- `3.1.0` also changed `SERVER_NAME` behavior so it no longer restricts incoming requests to that domain.

## Official Sources

- Flask installation: `https://flask.palletsprojects.com/en/stable/installation/`
- Flask quickstart: `https://flask.palletsprojects.com/en/stable/quickstart/`
- Flask configuration: `https://flask.palletsprojects.com/en/stable/config/`
- Flask testing: `https://flask.palletsprojects.com/en/stable/testing/`
- Flask async support: `https://flask.palletsprojects.com/en/stable/async-await/`
- Flask application factories: `https://flask.palletsprojects.com/en/stable/patterns/appfactories/`
- Flask deployment: `https://flask.palletsprojects.com/en/stable/deploying/`
- Flask changelog: `https://flask.palletsprojects.com/en/stable/changes/`
- Flask release page: `https://github.com/pallets/flask/releases/tag/3.1.3`
- PyPI package: `https://pypi.org/project/Flask/`
