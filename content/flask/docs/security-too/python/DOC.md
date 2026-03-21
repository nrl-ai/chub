---
name: security-too
description: "Flask-Security-Too package guide for Flask authentication, authorization, registration, and MFA"
metadata:
  languages: "python"
  versions: "5.7.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,authentication,authorization,security,roles,mfa"
---

# Flask-Security-Too Python Package Guide

## What It Is

`Flask-Security-Too` is the maintainer docs and PyPI line for the Flask extension that handles:

- session login/logout
- JSON and token-based authentication
- role and permission checks
- registration, email confirmation, password reset, password change, and email change
- optional unified sign-in, MFA, WebAuthn/passkeys, and social login flows

Naming matters:

- Upstream docs now install with `Flask-Security`
- Import root stays `flask_security`

As of **March 12, 2026**, the stable docs and PyPI both point at `5.7.1`.

## Version And Source Notes

- Package covered: `flask-security-too==5.7.1`
- Docs root: `https://flask-security-too.readthedocs.io/en/stable/`
- Registry URL: `https://pypi.org/project/flask-security-too/`
- Python requirement on PyPI: `>=3.10`
- The official quickstart still centers the SQLAlchemy integration and the `fsqla_v3` model mixins.

## Install

The upstream docs now show the preferred package name:

```bash
pip install "Flask-Security[fsqla,common]==5.7.1"
```

If your lockfile or dependency list still uses the older package name, keep the import path the same:

```bash
pip install "Flask-Security-Too[fsqla,common]==5.7.1"
```

Common companion packages:

```bash
pip install "Flask-SQLAlchemy>=3,<4"
pip install "Flask-WTF>=1.1"
```

Notes:

- `fsqla` enables the Flask-SQLAlchemy datastore helpers and model mixins.
- `common` pulls in the password-hashing support used by the default setup.
- `5.7.1` is on the argon2-default line, so do not assume older bcrypt-only deployments will behave the same without migration planning.

## Minimal Setup

This is the fastest correct starting point for a server-rendered Flask app with API endpoints.

```python
import os

from flask import Flask, jsonify
from flask_sqlalchemy import SQLAlchemy
from flask_security import (
    Security,
    SQLAlchemyUserDatastore,
    auth_required,
    current_user,
    hash_password,
    roles_required,
)
from flask_security.models import fsqla_v3 as fsqla

app = Flask(__name__)
app.config.update(
    SECRET_KEY=os.environ["SECRET_KEY"],
    SECURITY_PASSWORD_SALT=os.environ["SECURITY_PASSWORD_SALT"],
    SQLALCHEMY_DATABASE_URI="sqlite:///app.db",
    SQLALCHEMY_TRACK_MODIFICATIONS=False,
    SESSION_COOKIE_HTTPONLY=True,
    SESSION_COOKIE_SAMESITE="Lax",
    REMEMBER_COOKIE_HTTPONLY=True,
    REMEMBER_COOKIE_SAMESITE="Lax",
)

db = SQLAlchemy(app)
fsqla.FsModels.set_db_info(db)

class Role(db.Model, fsqla.FsRoleMixin):
    pass

class User(db.Model, fsqla.FsUserMixin):
    pass

user_datastore = SQLAlchemyUserDatastore(db, User, Role)
security = Security(app, user_datastore)

@app.get("/api/me")
@auth_required("session", "token")
def me():
    return jsonify(
        id=current_user.id,
        email=current_user.email,
        active=current_user.active,
    )

@app.get("/admin")
@auth_required("session", "token")
@roles_required("admin")
def admin():
    return {"ok": True}

with app.app_context():
    db.create_all()
    if not user_datastore.find_user(email="admin@example.com"):
        user_datastore.create_user(
            email="admin@example.com",
            password=hash_password("change-me-now"),
        )
        db.session.commit()
```

Rules to keep:

- Set `SECRET_KEY` and `SECURITY_PASSWORD_SALT` from environment or secret storage.
- Call `fsqla.FsModels.set_db_info(db)` before declaring `User` and `Role`.
- Use `hash_password()` when creating users.
- Prefer `@auth_required(...)` over raw Flask-Login decorators on protected routes.
- Turn on `SESSION_COOKIE_SECURE` and `REMEMBER_COOKIE_SECURE` in production behind HTTPS.

## Required Model Contract

If you use the built-in SQLAlchemy mixins, most fields come for free. If you customize models, keep these requirements.

User model:

- primary key
- `email` for the default identity flows
- `password`
- `active`
- `fs_uniquifier`
- relationship to roles

Role model:

- primary key
- `name`
- `description`

If you use permission decorators, roles also need a `permissions` field.

`fs_uniquifier` is not optional boilerplate. It is used for session identity and token invalidation. Rotating it logs users out. If you want auth tokens to survive password changes, add `fs_token_uniquifier` and let Flask-Security use that for token generation.

## Core Auth Patterns

### Protect routes with Flask-Security decorators

```python
from flask_security import auth_required, current_user, permissions_required

@app.get("/reports")
@auth_required("token", "session")
@permissions_required("report-read")
def list_reports():
    return {"email": current_user.email}
```

Use these decorators first:

- `auth_required()`
- `auth_token_required()`
- `http_auth_required()`
- `roles_required()` / `roles_accepted()`
- `permissions_required()` / `permissions_accepted()`

Do not use Flask-Login's `@login_required` as your primary protection layer here. You bypass Flask-Security's auth-method handling, freshness checks, and CSRF behavior.

### JSON login and token auth

`5.7.1` still supports mixed browser/API apps. By default, `SECURITY_API_ENABLED_METHODS` includes `session`, `token`, and `basic`.

Request a token from the login endpoint:

```bash
curl -X POST "http://localhost:5000/login?include_auth_token" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{"email":"admin@example.com","password":"change-me-now"}'
```

Use the returned token on protected endpoints:

```bash
curl "http://localhost:5000/api/me" \
  -H "Authentication-Token: <token>"
```

## Configuration That Usually Matters First

```python
app.config.update(
    SECRET_KEY=os.environ["SECRET_KEY"],
    SECURITY_PASSWORD_SALT=os.environ["SECURITY_PASSWORD_SALT"],
    SECURITY_PASSWORD_HASH="argon2",
    SECURITY_API_ENABLED_METHODS=["session", "token", "basic"],
    SECURITY_TOKEN_MAX_AGE=3600,
    SECURITY_RETURN_GENERIC_RESPONSES=True,
    SECURITY_EMAIL_VALIDATOR_ARGS={"check_deliverability": False},
    SECURITY_REGISTERABLE=True,
    SECURITY_CONFIRMABLE=True,
    SECURITY_RECOVERABLE=True,
    SECURITY_CHANGEABLE=True,
    SECURITY_CHANGE_EMAIL=True,
)
```

Why these are the high-value knobs:

- `SECURITY_PASSWORD_SALT` is required for the default password/token behavior.
- `SECURITY_PASSWORD_HASH="argon2"` makes migrations explicit in code and tests.
- `SECURITY_TOKEN_MAX_AGE` prevents indefinitely valid auth tokens.
- `SECURITY_RETURN_GENERIC_RESPONSES=True` reduces user-enumeration leakage on auth endpoints.
- `SECURITY_REGISTERABLE`, `SECURITY_CONFIRMABLE`, `SECURITY_RECOVERABLE`, `SECURITY_CHANGEABLE`, and `SECURITY_CHANGE_EMAIL` control the common account-management flows.
- `SECURITY_EMAIL_VALIDATOR_ARGS={"check_deliverability": False}` is useful in tests or seeded local environments.

If you enable mail-driven flows, wire up your Flask mail extension and sender settings before testing signup, confirmation, or recovery.

## CSRF, Sessions, And API Clients

The official mixed-app pattern is:

```python
import flask_wtf

app.config["WTF_CSRF_CHECK_DEFAULT"] = False
app.config["SECURITY_CSRF_PROTECT_MECHANISMS"] = ["session", "basic"]
flask_wtf.CSRFProtect(app)
```

Practical rules:

- Keep CSRF enabled for browser session flows.
- Send the CSRF token header on JSON requests that still rely on the session cookie.
- Do not expect token-auth requests to need the same CSRF treatment as session-cookie requests.
- Put `@auth_required(...)` on endpoints that need Flask-Security to enforce the right mechanism.

## Optional Feature Areas

Once the base login flow works, the upstream docs support these higher-level features:

- registration and confirmation flows
- email and username identity flows
- password reset and password change
- unified sign-in and newer register-form behavior
- two-factor auth, passkeys/WebAuthn, and recovery codes
- OAuth/social login integrations

Enable one feature family at a time and test the generated views or JSON responses before combining several of them.

## Common Pitfalls

- Confusing install name and import name. Install may be `Flask-Security` or `Flask-Security-Too`; the import stays `flask_security`.
- Forgetting `SECRET_KEY` or `SECURITY_PASSWORD_SALT`.
- Protecting routes with Flask-Login decorators instead of Flask-Security decorators.
- Creating custom models without `fs_uniquifier`.
- Adding permission checks without a `permissions` field on roles.
- Copying old bcrypt-era setup notes into an argon2-default deployment.
- Overriding `Security.render_json` and assuming it returns a string; `5.7.1` changed that contract to return a `dict`.

## Version-Sensitive Notes For `5.7.1`

- `5.7.1` requires Python `>=3.10` according to PyPI metadata.
- `5.7.1` changed `Security.render_json` to return a `dict` instead of a serialized string. Update custom subclasses and tests that assert the old behavior.
- The maintainer quickstart now shows `Flask-Security[fsqla,common]` as the preferred install name, even though this guide tracks the `flask-security-too` PyPI package.
- If you are upgrading from older `5.4.x` or earlier guidance, review the argon2 default, token freshness behavior, and newer registration/account-flow settings before reusing old snippets.

## Official Sources Used

- `https://flask-security-too.readthedocs.io/en/stable/`
- `https://flask-security-too.readthedocs.io/en/stable/quickstart.html`
- `https://flask-security-too.readthedocs.io/en/stable/configuration.html`
- `https://flask-security-too.readthedocs.io/en/stable/patterns.html`
- `https://flask-security-too.readthedocs.io/en/stable/models.html`
- `https://flask-security-too.readthedocs.io/en/stable/features.html`
- `https://flask-security-too.readthedocs.io/en/stable/changelog.html`
- `https://pypi.org/project/flask-security-too/`
