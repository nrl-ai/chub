---
name: jwt-extended
description: "Flask-JWT-Extended 4.7.1 guide for JWT authentication in Flask APIs and browser apps"
metadata:
  languages: "python"
  versions: "4.7.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,jwt,authentication,authorization,cookies,api"
---

# Flask-JWT-Extended Python Guide

## Install

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "flask-jwt-extended==4.7.1"
```

Common alternatives:

```bash
uv add "flask-jwt-extended==4.7.1"
poetry add "flask-jwt-extended==4.7.1"
```

You also need Flask itself:

```bash
python -m pip install "Flask>=3,<4"
```

## Minimal Header-Based API Auth

For API clients, start with bearer tokens in the `Authorization` header. Set `JWT_SECRET_KEY`, initialize `JWTManager`, issue an access token only after real credential verification, and protect routes with `@jwt_required()`.

```python
from datetime import timedelta
import os

from flask import Flask, jsonify, request
from flask_jwt_extended import (
    JWTManager,
    create_access_token,
    get_jwt_identity,
    jwt_required,
)

app = Flask(__name__)
app.config["JWT_SECRET_KEY"] = os.environ["JWT_SECRET_KEY"]
app.config["JWT_ACCESS_TOKEN_EXPIRES"] = timedelta(hours=1)

jwt = JWTManager(app)

@app.post("/login")
def login():
    data = request.get_json() or {}
    username = data.get("username")
    password = data.get("password")

    if not username or password != "correct-password":
        return jsonify(msg="Bad username or password"), 401

    access_token = create_access_token(identity=username)
    return jsonify(access_token=access_token)

@app.get("/me")
@jwt_required()
def me():
    return jsonify(user=get_jwt_identity())
```

Clients call the protected route with:

```http
Authorization: Bearer <access_token>
```

## App Factory Setup

For real projects, keep the extension unbound until app creation:

```python
import os

from flask import Flask
from flask_jwt_extended import JWTManager

jwt = JWTManager()

def create_app() -> Flask:
    app = Flask(__name__)
    app.config["JWT_SECRET_KEY"] = os.environ["JWT_SECRET_KEY"]
    jwt.init_app(app)
    return app
```

This avoids binding extension state at import time and makes testing easier.

## Refresh Tokens

Use refresh tokens for long-lived sessions. In 4.x, the refresh endpoint should use `@jwt_required(refresh=True)` rather than old 3.x decorators.

```python
from flask_jwt_extended import create_refresh_token

@app.post("/login")
def login():
    data = request.get_json() or {}
    username = data.get("username")

    access_token = create_access_token(identity=username)
    refresh_token = create_refresh_token(identity=username)
    return jsonify(
        access_token=access_token,
        refresh_token=refresh_token,
    )

@app.post("/refresh")
@jwt_required(refresh=True)
def refresh():
    identity = get_jwt_identity()
    return jsonify(access_token=create_access_token(identity=identity))
```

Keep access tokens short-lived. Refresh tokens should have tighter storage rules and a clear revocation story.

## Cookie-Based Auth For Browser Apps

If your frontend is a browser app instead of an API client, store JWTs in cookies and keep CSRF protection enabled.

```python
from flask_jwt_extended import set_access_cookies, unset_jwt_cookies

app.config.update(
    JWT_TOKEN_LOCATION=["cookies"],
    JWT_COOKIE_SECURE=True,
    JWT_COOKIE_CSRF_PROTECT=True,
    JWT_COOKIE_SAMESITE="Lax",
)

@app.post("/token/auth")
def cookie_login():
    access_token = create_access_token(identity="user-123")
    response = jsonify(msg="login successful")
    set_access_cookies(response, access_token)
    return response

@app.post("/logout")
def logout():
    response = jsonify(msg="logout successful")
    unset_jwt_cookies(response)
    return response
```

Use cookie mode when the browser should automatically send the token. When you do that:

- serve over HTTPS in production
- keep CSRF protection on
- make `SameSite`, domain, and secure-cookie settings explicit for your deployment

The docs include both explicit refresh-token routes and implicit cookie-refresh patterns. Prefer explicit refresh endpoints unless you have a strong reason to hide refresh mechanics in response hooks.

## Loading The Current User

If protected routes need a real user object instead of just the JWT identity value, register a user lookup callback and then access `current_user`.

```python
from flask_jwt_extended import current_user

@jwt.user_lookup_loader
def load_user(jwt_header, jwt_payload):
    user_id = jwt_payload["sub"]
    return User.query.get(user_id)

@app.get("/profile")
@jwt_required()
def profile():
    return jsonify(
        id=current_user.id,
        email=current_user.email,
    )
```

`User` is your own model or repository layer. This keeps the token small while still letting route handlers work with application-level user records.

## Revocation, Logout, And Blocklists

For logout, compromised credentials, or administrative invalidation, register a blocklist callback and store revoked token `jti` values in a shared persistent store such as Redis or a database.

```python
from flask_jwt_extended import get_jwt

@jwt.token_in_blocklist_loader
def is_token_revoked(jwt_header, jwt_payload):
    return redis_client.sismember("jwt_blocklist", jwt_payload["jti"])

@app.delete("/logout")
@jwt_required()
def revoke_current_token():
    jti = get_jwt()["jti"]
    redis_client.sadd("jwt_blocklist", jti)
    return jsonify(msg="token revoked")
```

`redis_client` is your own Redis connection or adapter.

Do not use a process-local Python set for this in production. Revocation state must survive restarts and be shared across workers.

## Configuration Checklist

Most apps should review these settings explicitly:

- `JWT_SECRET_KEY`: signing secret for JWTs; keep it out of source control and stable across instances
- `JWT_ACCESS_TOKEN_EXPIRES`: short lifetime for access tokens
- `JWT_REFRESH_TOKEN_EXPIRES`: separate lifetime for refresh tokens
- `JWT_TOKEN_LOCATION`: choose header mode for APIs or cookie mode for browser flows
- `JWT_COOKIE_SECURE`: `True` in production when using cookies
- `JWT_COOKIE_CSRF_PROTECT`: keep enabled for cookie-based auth
- `JWT_IDENTITY_CLAIM`: defaults to `sub` in 4.x

Practical rules:

- Use a dedicated JWT secret instead of hard-coding values in the app module.
- Treat JWT contents as signed, not encrypted. Do not put secrets or unnecessary PII in claims.
- Keep token payloads small; every authenticated request carries them back to the server.

## Common Pitfalls

- Search results still surface the old `3.0.0_release` docs. Those pages are not reliable for 4.7.1 code.
- The package name is `flask-jwt-extended`, but imports are from `flask_jwt_extended`.
- `@jwt_required()` is the 4.x decorator. Old 3.x helpers like `@jwt_refresh_token_required` should not be copied forward.
- `get_jwt()` replaces older raw-JWT helper patterns from pre-4.x examples.
- `current_user` is not automatic; it depends on a configured user lookup callback.
- Cookie mode requires CSRF handling and correct cookie settings. Disabling CSRF to make a browser flow "work" is usually the wrong fix.
- Revocation requires shared persistent storage. In-memory revocation breaks as soon as you run multiple workers or restart the process.
- Rotating `JWT_SECRET_KEY` invalidates outstanding tokens. Plan key changes deliberately.

## Version-Sensitive Notes For 4.7.1

As of March 12, 2026, the stable docs and PyPI both align on `4.7.1`.

Important 4.x behavior changes from the official upgrade guide:

- JWT payload structure changed in 4.x.
- The default identity claim moved to `sub`.
- Custom claims are flattened into the top-level payload instead of living under a nested claim bucket.
- Several callback names and signatures changed between 3.x and 4.x.

If you are updating older code, read the 4.x upgrade guide before reusing decorators, callbacks, claim access patterns, or token-inspection helpers from blog posts and issue comments.
