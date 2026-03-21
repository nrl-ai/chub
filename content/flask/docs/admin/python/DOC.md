---
name: admin
description: "Flask-Admin package guide for building authenticated Flask admin interfaces and CRUD views"
metadata:
  languages: "python"
  versions: "2.0.2"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask-admin,flask,admin,sqlalchemy,wtforms,crud"
---

# Flask-Admin Python Package Guide

## What It Is

`Flask-Admin` adds an admin UI layer to Flask applications. The common case is CRUD screens for SQLAlchemy models, but the package also supports custom `BaseView` pages, file administration, and menu/navigation helpers.

This entry is for `Flask-Admin==2.0.2`. Use `2.x` setup and config patterns here. Do not copy older `1.6.x` snippets that rely on `template_mode=` or unprefixed `ADMIN_*` config names.

## Install

Install the base package plus the backend extra you need. For SQLAlchemy-backed admin views, the official `2.x` docs use the `sqlalchemy` extra.

```bash
python -m venv .venv
source .venv/bin/activate
pip install "flask-admin[sqlalchemy]==2.0.2" "Flask-SQLAlchemy"
```

Other useful extras are available for MongoEngine, Peewee, Redis, and translation support. Install only the integration you actually use.

## Minimal SQLAlchemy Setup

In `2.x`, initialize `Admin` with a theme object instead of `template_mode=...`.

```python
from flask import Flask
from flask_admin import Admin
from flask_admin.contrib.sqla import ModelView
from flask_admin.theme import Bootstrap4Theme
from flask_sqlalchemy import SQLAlchemy

app = Flask(__name__)
app.config.update(
    SECRET_KEY="dev-only-change-me",
    SQLALCHEMY_DATABASE_URI="sqlite:///app.db",
)

db = SQLAlchemy(app)

class User(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    email = db.Column(db.String(255), unique=True, nullable=False)
    is_active = db.Column(db.Boolean, default=True, nullable=False)

admin = Admin(
    app,
    name="Example Admin",
    theme=Bootstrap4Theme(),
)
admin.add_view(ModelView(User, db.session))

with app.app_context():
    db.create_all()
```

Practical rules:

- Keep `SECRET_KEY` set before you use admin forms, sessions, or CSRF features.
- Pass `db.session` to `ModelView` for SQLAlchemy-backed views.
- Prefer an explicit theme object so the code is obviously `2.x` and not an older template-mode example.

## App Factory Setup

Use `init_app()` when your project follows the Flask extension pattern.

```python
from flask import Flask
from flask_admin import Admin
from flask_admin.contrib.sqla import ModelView
from flask_admin.theme import Bootstrap4Theme
from flask_sqlalchemy import SQLAlchemy

db = SQLAlchemy()
admin = Admin(name="Example Admin", theme=Bootstrap4Theme())

class UserAdmin(ModelView):
    can_view_details = True
    column_searchable_list = ["email"]
    column_filters = ["is_active"]

def create_app() -> Flask:
    app = Flask(__name__)
    app.config.from_mapping(
        SECRET_KEY="dev-only-change-me",
        SQLALCHEMY_DATABASE_URI="sqlite:///app.db",
    )

    db.init_app(app)
    admin.init_app(app)

    from .models import User

    admin.add_view(UserAdmin(User, db.session))
    return app
```

If the factory can run multiple times in tests, avoid blindly calling `add_view()` on every import path. Register views once per process or guard duplicate registration.

## Core `ModelView` Configuration

These attributes and hooks cover most real projects:

- `can_create`, `can_edit`, `can_delete` to disable CRUD actions
- `can_view_details` to expose a read-only details page
- `column_list`, `column_exclude_list`, `column_searchable_list`, and `column_filters` to shape list views
- `column_editable_list` for inline editing from the list page
- `form_excluded_columns`, `form_args`, and `form_widget_args` for form customization
- `form_ajax_refs` for foreign-key lookups backed by AJAX
- `inline_models` to edit related rows inline
- `can_export` and `export_types` for CSV-style exports
- `page_size`, `can_set_page_size`, and `page_size_options` for pagination behavior

Example:

```python
from flask_admin.contrib.sqla import ModelView
from flask_admin.form import SecureForm

class UserAdmin(ModelView):
    form_base_class = SecureForm

    can_view_details = True
    can_export = True
    column_list = ["email", "is_active"]
    column_searchable_list = ["email"]
    column_filters = ["is_active"]
    page_size = 50
    can_set_page_size = True
    page_size_options = [25, 50, 100]
```

Use one subclass per model once behavior starts to diverge. A single shared `ModelView` across unrelated models becomes hard to reason about quickly.

## Access Control And Authentication

`Flask-Admin` does not provide a complete authentication system. Protect every view by overriding `is_accessible()` and usually `inaccessible_callback()`.

```python
from flask import redirect, request, url_for
from flask_login import current_user
from flask_admin.contrib.sqla import ModelView

class StaffModelView(ModelView):
    def is_accessible(self) -> bool:
        return current_user.is_authenticated and current_user.is_admin

    def inaccessible_callback(self, name, **kwargs):
        return redirect(url_for("login", next=request.url))
```

Important:

- Menu visibility is not authorization. Always enforce access checks in the view class.
- Protect custom `BaseView` pages the same way you protect `ModelView` subclasses.
- If you integrate Flask-Security-Too or another auth library, reuse its login route in `inaccessible_callback()` instead of creating a parallel admin-only auth flow.

## Custom Pages, Templates, And Routing

Use `BaseView` for pages that are not directly tied to one model.

```python
from flask_admin import BaseView, expose

class AnalyticsView(BaseView):
    @expose("/")
    def index(self):
        return self.render("admin/analytics.html")
```

```python
admin.add_view(AnalyticsView(name="Analytics", endpoint="analytics"))
```

Template rules that matter:

- Extend `admin/master.html` for custom admin pages.
- Extend `admin/model/list.html`, `admin/model/create.html`, or `admin/model/edit.html` when you only need to customize built-in CRUD screens.
- Prefer extending upstream templates over copying the entire admin template tree.

`2.x` also supports host-aware routing on `Admin(...)` and `@expose(...)` when your Flask app uses host matching:

```python
from flask import Flask
from flask_admin import BaseView, Admin, expose

app = Flask(__name__, host_matching=True, static_host="cdn.example.com")

class ReportsView(BaseView):
    @expose("/", host="admin.example.com")
    def index(self):
        return self.render("admin/reports.html")

admin = Admin(app, host="admin.example.com")
admin.add_view(ReportsView(name="Reports", endpoint="reports"))
```

Only use `host=` when your Flask routing setup already enables host matching. Otherwise keep admin routes path-based.

## Config, CSRF, And CSP

The `2.x` line moves project config to `FLASK_ADMIN_*` names. Use the namespaced settings, not the older `ADMIN_*` variants.

Useful config:

- `FLASK_ADMIN_SWATCH` for the Bootswatch theme name
- `FLASK_ADMIN_FLUID_LAYOUT` to use full-width layout
- `FLASK_ADMIN_RAISE_ON_INTEGRITY_ERROR` to bubble integrity errors during writes
- `FLASK_ADMIN_RAISE_ON_VIEW_EXCEPTION` to raise view exceptions instead of swallowing them

For CSRF protection on model forms:

```python
from flask_admin.contrib.sqla import ModelView
from flask_admin.form import SecureForm

class ProtectedModelView(ModelView):
    form_base_class = SecureForm
```

For Content Security Policy nonces, `2.x` adds `csp_nonce_generator=` on `Admin(...)`:

```python
from flask import Flask, g
from flask_admin import Admin
from flask_admin.theme import Bootstrap4Theme

def csp_nonce() -> str:
    return g.csp_nonce

app = Flask(__name__)
admin = Admin(app, theme=Bootstrap4Theme(), csp_nonce_generator=csp_nonce)
```

Practical notes:

- `SecureForm` still depends on Flask sessions working correctly, so test create and edit flows after enabling it.
- If your app already enforces CSP, wire `csp_nonce_generator` up early instead of patching templates later.
- Keep config names consistent. Mixing `ADMIN_*` and `FLASK_ADMIN_*` creates hard-to-see drift during upgrades.

## Common Pitfalls

### Copying `1.x` examples into a `2.0.2` project

Avoid these old patterns in `2.0.2` code:

- `Admin(..., template_mode="bootstrap3")`
- unprefixed `ADMIN_RAISE_ON_*` config names
- Flask-BabelEx integration examples
- legacy S3 file admin code that does not use `boto3.client("s3")`

### Forgetting the backend dependency

`Flask-Admin` does not make SQLAlchemy support available unless you install the right extra and ORM dependencies. If imports from `flask_admin.contrib.sqla` fail, check your installed extras first.

### Leaving custom views unprotected

Developers often secure `ModelView` subclasses and forget `BaseView` pages. Every admin-exposed page needs explicit access control.

### Registering views repeatedly in app factories

Calling `admin.add_view(...)` on every factory invocation can produce duplicate menu items or test pollution. Make initialization idempotent in test-heavy apps.

### Replacing upstream templates wholesale

Full template copies make minor upgrades brittle. Extend a specific upstream template unless you truly need to own the full markup.

## Version-Sensitive Notes For `2.0.2`

- `2.0.0` introduced the new `theme=` API and deprecates `template_mode`.
- `2.0.0` renames `ADMIN_RAISE_ON_INTEGRITY_ERROR` and `ADMIN_RAISE_ON_VIEW_EXCEPTION` to `FLASK_ADMIN_RAISE_ON_INTEGRITY_ERROR` and `FLASK_ADMIN_RAISE_ON_VIEW_EXCEPTION`.
- `2.0.0` drops Flask-BabelEx support in favor of Flask-Babel.
- `2.0.0` changes S3 file admin to use `boto3.client("s3")` rather than legacy boto-style credential arguments.
- `2.0.2` is the current package version on PyPI as of March 12, 2026.
- The `latest` docs root is the correct canonical doc root for `2.0.2`, but some subpages still render `2.0.0` or `2.0.1` in their page headers. Treat the changelog and PyPI version as the source of truth for the current release number.
