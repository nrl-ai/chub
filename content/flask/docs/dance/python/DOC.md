---
name: dance
description: "Flask-Dance 7.1.0 guide for OAuth and social login flows in Flask applications"
metadata:
  languages: "python"
  versions: "7.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,flask-dance,oauth,oauth2,oauth1,authentication,social-login"
---

# Flask-Dance for Flask Apps

## Install

Pin the package version your app expects:

```bash
python -m pip install "Flask-Dance==7.1.0"
```

If you want the built-in SQLAlchemy token storage helper, install the extra:

```bash
python -m pip install "Flask-Dance[sqla]==7.1.0"
```

Common alternatives:

```bash
uv add "Flask-Dance==7.1.0"
poetry add "Flask-Dance==7.1.0"
```

## Minimal Setup With A Built-In Provider

Use an app factory, load secrets before initializing the extension, then register the provider blueprint.

```python
import os

from flask import Flask, redirect, url_for
from flask_dance.contrib.github import github, make_github_blueprint

def create_app() -> Flask:
    app = Flask(__name__)
    app.config["SECRET_KEY"] = os.environ["FLASK_SECRET_KEY"]

    github_bp = make_github_blueprint(
        client_id=os.environ["GITHUB_OAUTH_CLIENT_ID"],
        client_secret=os.environ["GITHUB_OAUTH_CLIENT_SECRET"],
        redirect_to="github_done",
    )
    app.register_blueprint(github_bp, url_prefix="/login")

    @app.get("/")
    def index():
        return '<a href="/login/github">Sign in with GitHub</a>'

    @app.get("/github")
    def github_done():
        if not github.authorized:
            return redirect(url_for("github.login"))

        response = github.get("/user")
        response.raise_for_status()
        profile = response.json()
        return {"login": profile["login"], "id": profile["id"]}

    return app
```

Important setup rules:

- Set Flask `SECRET_KEY`; Flask-Dance uses the Flask session unless you provide another token storage backend.
- Register the blueprint on the app before the first request.
- The login route comes from the blueprint name and `url_prefix`. With the GitHub provider above, the login URL is `/login/github`.
- The callback route must be reachable at the exact URL you registered with the provider.

## Provider Configuration

Built-in `make_*_blueprint()` helpers usually accept:

- `client_id`
- `client_secret`
- `scope`
- `redirect_to`
- `login_url`
- `authorized_url`
- `storage`

Several built-in providers also support reading credentials from Flask config when you omit them explicitly. Common keys follow the provider naming pattern, for example:

- `GITHUB_OAUTH_CLIENT_ID`
- `GITHUB_OAUTH_CLIENT_SECRET`
- `GOOGLE_OAUTH_CLIENT_ID`
- `GOOGLE_OAUTH_CLIENT_SECRET`

That makes config-driven factories straightforward:

```python
from flask import Flask
from flask_dance.contrib.google import make_google_blueprint

def create_app() -> Flask:
    app = Flask(__name__)
    app.config.from_mapping(
        SECRET_KEY="replace-me",
        GOOGLE_OAUTH_CLIENT_ID="...",
        GOOGLE_OAUTH_CLIENT_SECRET="...",
    )

    google_bp = make_google_blueprint(
        scope=[
            "openid",
            "https://www.googleapis.com/auth/userinfo.email",
        ],
        redirect_to="auth.google_done",
    )
    app.register_blueprint(google_bp, url_prefix="/login")
    return app
```

Provider modules live under `flask_dance.contrib`. Prefer the built-in helper for a supported provider before writing a custom OAuth flow by hand.

## Custom Provider Blueprint

For a provider without a built-in helper, use `OAuth2ConsumerBlueprint` directly:

```python
from flask import Flask
from flask_dance.consumer import OAuth2ConsumerBlueprint

def create_app() -> Flask:
    app = Flask(__name__)
    app.config["SECRET_KEY"] = "replace-me"

    example_bp = OAuth2ConsumerBlueprint(
        "example",
        __name__,
        client_id="client-id",
        client_secret="client-secret",
        base_url="https://api.example.com/",
        authorization_url="https://example.com/oauth/authorize",
        token_url="https://example.com/oauth/token",
    )
    app.register_blueprint(example_bp, url_prefix="/login")
    return app
```

Use this path when the provider is standards-compliant OAuth but Flask-Dance does not already ship a `contrib` module for it.

## Token Storage And Persistence

The default storage keeps tokens in the signed Flask session cookie. That is acceptable for simple demos, but it is usually the wrong choice for production apps because:

- tokens disappear when the session is cleared
- one browser session does not map cleanly to multiple linked OAuth accounts
- storing access tokens in client cookies makes rotation and revocation harder

The docs provide multiple storage backends:

- `SessionStorage`: Flask session-backed storage
- `SQLAlchemyStorage`: database-backed storage
- `MemoryStorage`: in-memory storage, useful in tests
- `NullStorage`: disables persistence

For production apps, prefer `SQLAlchemyStorage`:

```python
from flask import Flask
from flask_dance.consumer.storage.sqla import OAuthConsumerMixin, SQLAlchemyStorage
from flask_dance.contrib.github import make_github_blueprint
from flask_login import current_user
from flask_sqlalchemy import SQLAlchemy

db = SQLAlchemy()

class OAuth(OAuthConsumerMixin, db.Model):
    __tablename__ = "oauth"

    user_id = db.Column(db.Integer, db.ForeignKey("user.id"))
    user = db.relationship("User")

def create_app() -> Flask:
    app = Flask(__name__)
    app.config.from_mapping(
        SECRET_KEY="replace-me",
        SQLALCHEMY_DATABASE_URI="sqlite:///app.db",
    )
    db.init_app(app)

    github_bp = make_github_blueprint(
        storage=SQLAlchemyStorage(OAuth, db.session, user=current_user),
    )
    app.register_blueprint(github_bp, url_prefix="/login")
    return app
```

If you use a database-backed storage, create the tables before starting auth flows or the callback exchange will fail when Flask-Dance tries to save the token.

## Multi-User Account Linking

Flask-Dance has specific guidance for apps where users log into your app first and then link external providers. Combine it with Flask-Login and a persistent token store.

Useful pattern:

- require a local logged-in user before starting the provider flow
- store tokens per application user, not just per browser session
- use `flask_dance.consumer.storage.sqla` with `current_user`
- use the `@oauth_authorized` signal to attach provider identity metadata to the local user record

If your app supports connecting multiple GitHub or Google accounts, model that explicitly in your own database instead of assuming one token per provider name.

## Signals And Custom Auth Handling

Flask-Dance emits signals around the OAuth flow. The important ones are:

- `oauth_before_login`
- `oauth_authorized`
- `oauth_error`

Typical use cases:

- enrich or normalize provider profile data after authorization
- create or link a local user
- reject or redirect when provider auth fails
- audit provider authorization events

Example:

```python
from flask import redirect, url_for
from flask_dance.consumer import oauth_authorized, oauth_error

@oauth_authorized.connect
def github_logged_in(blueprint, token):
    if blueprint.name != "github":
        return None

    response = blueprint.session.get("/user")
    response.raise_for_status()
    profile = response.json()
    link_user_account(profile, token)
    return False

@oauth_error.connect
def github_error(blueprint, message, response, error, error_description, error_uri):
    if blueprint.name == "github":
        return redirect(url_for("login_failed"))
    return None
```

In `7.1.0`, signal handlers can return a Flask response from `oauth_error`, and Flask-Dance will respect it. If you previously relied on side effects only, review those handlers before upgrading.

Returning `False` from `oauth_authorized` tells Flask-Dance not to save the token automatically. Use that when you want your signal handler to decide whether and how the token should be stored.

## Logout And Revocation

Logging out of your Flask app is not the same as revoking the provider token. Clear both pieces when appropriate:

```python
from flask import current_app, redirect, url_for
from flask_dance.contrib.github import github
from flask_login import logout_user

@app.post("/logout")
def logout():
    if github.authorized:
        github.post(
            f"/applications/{current_app.config['GITHUB_OAUTH_CLIENT_ID']}/token",
            auth=(
                current_app.config["GITHUB_OAUTH_CLIENT_ID"],
                current_app.config["GITHUB_OAUTH_CLIENT_SECRET"],
            ),
            json={"access_token": github.token["access_token"]},
        )

    del github.token
    logout_user()
    return redirect(url_for("index"))
```

Practical rule:

- revoke provider tokens when the provider supports it and your app owns that lifecycle
- always clear local session state
- if you use database-backed storage, remove the stored token row as part of logout or unlink

## Reverse Proxy And Callback URLs

OAuth callbacks break easily when Flask sees the wrong scheme or host behind a reverse proxy. If your app runs behind Nginx, a load balancer, or a platform proxy, configure Flask/Werkzeug to trust the forwarded headers you actually use.

Common production fix:

```python
from werkzeug.middleware.proxy_fix import ProxyFix

app.wsgi_app = ProxyFix(app.wsgi_app, x_proto=1, x_host=1)
```

Without that, Flask-Dance may generate `http://...` callback URLs while your provider is configured for `https://...`, which causes redirect URI mismatch errors.

## Testing OAuth Flows

The testing docs cover several useful patterns:

- `MemoryStorage` for simple in-memory token tests
- `NullStorage` when you want auth behavior without persistence
- `requests_mock` to stub provider HTTP calls
- a provided pytest fixture for isolating OAuth state

Example:

```python
import requests_mock
from flask import Flask
from flask_dance.contrib.github import github, make_github_blueprint
from flask_dance.consumer.storage import MemoryStorage

def create_app() -> Flask:
    app = Flask(__name__)
    app.config["SECRET_KEY"] = "testing"

    github_bp = make_github_blueprint(
        client_id="test-client-id",
        client_secret="test-client-secret",
        storage=MemoryStorage({"access_token": "fake-token"}),
    )
    app.register_blueprint(github_bp, url_prefix="/login")

    @app.get("/me")
    def me():
        response = github.get("/user")
        response.raise_for_status()
        return response.json()

    return app

def test_me(client):
    with requests_mock.Mocker() as mocker:
        mocker.get(
            "https://api.github.com/user",
            json={"login": "octocat", "id": 1},
        )

        response = client.get("/me")
        assert response.status_code == 200
        assert response.json["login"] == "octocat"
```

When testing, avoid making the real OAuth redirect round-trip unless you are doing an end-to-end browser test. Most application logic can be covered by seeding token storage and mocking the provider API response.

## Common Pitfalls

- Do not hard-code provider secrets in source files. Load them from environment or a secret manager.
- Package name and import path differ: `Flask-Dance` vs `flask_dance`.
- Register the callback URL with the provider exactly as Flask-Dance generates it, including scheme, host, prefix, and trailing path pieces.
- Session-backed token storage is easy to start with but weak for multi-user apps and account-linking flows.
- If you connect `oauth_authorized`, remember that returning `False` disables automatic token persistence.
- If you use Flask-Login, require a current user before account-linking routes or you may attach the provider token to the wrong local session.
- If the provider requires extra scopes, request them up front. Missing scopes often only surface later when the first API request fails.
- Old blog posts may still show providers or flows that changed in `7.x`, especially around Twitter support and PKCE-related behavior.

## Version-Sensitive Notes For 7.1.0

- `7.1.0` changes signal handling so a response returned from an `oauth_error` signal handler is used directly.
- `7.1.0` also fixes Azure auto-refresh token behavior for short-lived access tokens.
- In `7.0.0`, the built-in Twitter provider was removed because the API is no longer broadly accessible. Do not copy older `make_twitter_blueprint()` examples from pre-7.0 articles.
- In `7.0.0`, PKCE support was added. If you are integrating with a provider that expects PKCE, prefer current `7.x` docs and examples over older snippets.

## Official Sources Used

- Docs root: `https://flask-dance.readthedocs.io/en/latest/`
- Providers: `https://flask-dance.readthedocs.io/en/latest/providers.html`
- Storages: `https://flask-dance.readthedocs.io/en/latest/storages.html`
- Multi-user: `https://flask-dance.readthedocs.io/en/latest/multi-user.html`
- Signals: `https://flask-dance.readthedocs.io/en/latest/signals.html`
- Logout: `https://flask-dance.readthedocs.io/en/latest/logout.html`
- Testing: `https://flask-dance.readthedocs.io/en/latest/testing.html`
- PyPI: `https://pypi.org/project/flask-dance/`
- Changelog: `https://github.com/singingwolfboy/flask-dance/blob/main/CHANGELOG.rst`
