---
name: extensions
description: "django-extensions package guide for Python projects using Django management commands, model mixins, and developer tooling"
metadata:
  languages: "python"
  versions: "4.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,django-extensions,management-commands,shell,debugging,orm"
---

# django-extensions Python Package Guide

## Golden Rule

Use `django-extensions` as Django project tooling, not as a standalone runtime API.

- Install the package version that matches your Django support window.
- Add `"django_extensions"` to `INSTALLED_APPS`.
- Reach for its `manage.py` commands first: `shell_plus`, `runserver_plus`, `graph_models`, `runscript`, `show_urls`, and `show_permissions`.
- Treat command-specific dependencies as optional add-ons, not as globally required.

This guide covers `django-extensions==4.1`.

## What django-extensions Adds

`django-extensions` is a grab bag of developer-facing Django helpers:

- management commands for local debugging, schema inspection, data export, and repetitive project tasks
- model and field mixins such as `TimeStampedModel` and `TitleSlugDescriptionModel`
- class-based-view permission mixins
- template debugging helpers and other project utilities

For most teams, the package is mainly about faster local development and maintenance workflows.

## Version-Sensitive Notes

- The package version covered here and the current latest PyPI release are both `4.1`.
- The upstream docs URL you were given, `https://django-extensions.readthedocs.io/en/latest/`, is already rendering `4.2` docs. For `4.1`, prefer `https://django-extensions.readthedocs.io/en/stable/`.
- The upstream README and PyPI metadata for the `4.1` line require `Django 4.2 or later`.
- The `4.1` release added the `show_permissions` command, added per-app styling support for `graph_models`, and included a fix around `JSONField` and `bulk_update()`.
- If you copy examples from `latest/`, verify they still exist in `stable/` before assuming they are valid for `4.1`.

## Installation

### pip

```bash
pip install django-extensions==4.1
```

### uv

```bash
uv add django-extensions==4.1
```

### Poetry

```bash
poetry add django-extensions==4.1
```

### Base Requirements

For the `4.1` line:

- Django: `4.2+`
- Python/Django support policy: upstream says it follows Django-supported Python and Django versions

Common optional dependencies:

- `werkzeug` for `runserver_plus`
- `pyOpenSSL` if you want HTTPS cert generation with `runserver_plus --cert-file`
- Graphviz plus either `pygraphviz` or `pydot` for rendered `graph_models` output
- `ipython`, `bpython`, or `ptpython` for richer `shell_plus` sessions

## Setup

Add the app to your Django settings:

```python
INSTALLED_APPS = [
    # ...
    "django_extensions",
]
```

Then confirm the extra commands are available:

```bash
python manage.py help
```

If a command needs an optional dependency and it is missing, the command raises an error when you invoke it.

## Core Usage

### `shell_plus`

`shell_plus` starts a Django-aware shell and auto-imports models. It is usually the fastest way to inspect data or prototype ORM queries.

```bash
python manage.py shell_plus
python manage.py shell_plus --ipython
python manage.py shell_plus --plain
python manage.py shell_plus --print-sql
python manage.py shell_plus --lab
```

Useful settings:

```python
SHELL_PLUS = "ipython"
SHELL_PLUS_PRINT_SQL = True
SHELL_PLUS_PRINT_SQL_TRUNCATE = 1000
SHELL_PLUS_DONT_LOAD = ["sites", "blog.pictures"]
SHELL_PLUS_MODEL_ALIASES = {
    "blog": {"Messages": "blog_messages"},
}
SHELL_PLUS_IMPORTS = [
    "from myapp.services import build_report",
]
```

Use `SHELL_PLUS_DONT_LOAD`, aliases, or a collision resolver when models from different apps share names.

### `runserver_plus`

`runserver_plus` wraps Django's development server with the Werkzeug debugger.

```bash
pip install werkzeug
python manage.py runserver_plus
python manage.py runserver_plus 0.0.0.0:8000
python manage.py runserver_plus --cert-file /tmp/dev-cert.crt
```

Useful settings:

```python
RUNSERVER_PLUS_SERVER_ADDRESS_PORT = "0.0.0.0:8000"
RUNSERVER_PLUS_PRINT_SQL = True
RUNSERVER_PLUS_PRINT_SQL_TRUNCATE = 1000
RUNSERVER_PLUS_EXTRA_FILES = []
RUNSERVER_PLUS_TRUSTED_HOSTS = [".localhost", "127.0.0.1"]
```

Important behavior:

- this debugger can execute code inside traceback frames
- Werkzeug 3.0.3+ restricts debugger hosts unless `RUNSERVER_PLUS_TRUSTED_HOSTS` allows them
- this command is for local development only

### `graph_models`

`graph_models` is the quickest way to visualize model relationships or export a diagram during a refactor.

```bash
python manage.py graph_models -a > my_project.dot
python manage.py graph_models -a -g -o my_project.png
python manage.py graph_models users billing auth > domain.dot
python manage.py graph_models -a --app-style path/to/style.json -o styled.png
```

Backends:

```bash
pip install pygraphviz
# or
pip install pyparsing pydot
```

Useful defaults:

```python
GRAPH_MODELS = {
    "all_applications": True,
    "group_models": True,
    "app_labels": ["users", "billing", "auth"],
}
```

If image rendering fails, generate plain DOT output first. Most failures come from missing Graphviz system libraries rather than Django itself.

### `runscript`

Use `runscript` for small, repeatable Django-context automation jobs that are too specific for a management command.

Project layout:

```text
scripts/
  __init__.py
  delete_stale_sessions.py
```

Example script:

```python
from django.utils import timezone

from sessions.models import Session

def run(*args):
    cutoff = timezone.now()
    Session.objects.filter(expires_at__lt=cutoff).delete()
```

Run it with:

```bash
python manage.py runscript delete_stale_sessions
python manage.py runscript delete_stale_sessions --script-args dry-run
```

`scripts/__init__.py` is required. Without it, Django will not discover the scripts package.

### Jobs scheduling helpers

`django-extensions` can scaffold and run jobs, but it does not schedule them by itself.

Create the job layout:

```bash
python manage.py create_jobs myapp
```

That creates a `jobs/` package with directories such as `hourly/`, `daily/`, `weekly/`, `monthly/`, and `yearly/`.

Each job lives in its own module and exposes a `Job` class derived from one of the base job classes.

Run jobs:

```bash
python manage.py runjob myjob
python manage.py runjobs hourly
python manage.py runjobs daily
python manage.py runjobs -l
```

Typical cron wiring:

```cron
@hourly /path/to/project/manage.py runjobs hourly
@daily /path/to/project/manage.py runjobs daily
```

### Model and field extensions

If you want convenience base classes instead of more management commands, start with the model extensions.

Example:

```python
from django.db import models
from django_extensions.db.models import TimeStampedModel, TitleSlugDescriptionModel

class Article(TimeStampedModel, TitleSlugDescriptionModel):
    body = models.TextField()
```

Commonly useful pieces:

- `TimeStampedModel` for `created` and `modified`
- `ActivatorModel` for active/inactive scheduling fields and helpers
- `TitleDescriptionModel` and `TitleSlugDescriptionModel` for common content patterns

For new Django `4.2+` projects, prefer Django's built-in `models.JSONField` unless you are maintaining older code that already depends on the extension field.

### `show_permissions`

`show_permissions` is a good `4.1`-specific command to inspect generated model permissions.

```bash
python manage.py show_permissions
python manage.py show_permissions blog
python manage.py show_permissions blog.Post
python manage.py show_permissions --all
python manage.py show_permissions --app-label blog
```

Use it when debugging custom auth flows, role-based access, or unexpected permission rows after migrations.

## Configuration And Auth

- There is no external authentication or API client setup. Configuration is standard Django settings plus optional Python packages for specific commands.
- Keep `django-extensions` in a development dependency group unless you intentionally use its commands in deployed admin or maintenance environments.
- Command-specific settings only affect those commands. For example, `SHELL_PLUS_*` settings do not change normal app imports.
- If you expose anything over the network for local debugging, treat `runserver_plus` as unsafe outside a trusted machine.

## Common Pitfalls

- Package name and Django app name differ: install `django-extensions`, configure `django_extensions`.
- `latest/` docs are version-drifting. For `4.1`, prefer the `stable/` docs root and the `4.1` release page.
- `runserver_plus` is not safe for production or shared environments.
- `graph_models` often fails because Graphviz or the rendering backend is not installed correctly.
- `runscript` and jobs discovery require real Python packages with `__init__.py` files.
- Jobs do not run automatically. `create_jobs` only scaffolds files.
- `shell_plus` can import colliding model names; set aliases or skip modules instead of relying on import order.
- Some commands are destructive or invasive, such as `reset_db`. Do not use them casually on a non-local database.

## Recommended Agent Workflow

1. Confirm the project's Django version before pinning `django-extensions`.
2. Install `django-extensions==4.1` and add `"django_extensions"` to `INSTALLED_APPS`.
3. Use `shell_plus` for ORM exploration, `runserver_plus` for local traceback debugging, and `graph_models` for schema inspection.
4. If a command errors immediately, check for optional dependencies before changing app code.
5. If docs disagree, prefer the `4.1` PyPI/release metadata and `stable/` docs over `latest/`.

## Official Sources

- Stable docs root: https://django-extensions.readthedocs.io/en/stable/
- Installation instructions: https://django-extensions.readthedocs.io/en/stable/installation_instructions.html
- `shell_plus`: https://django-extensions.readthedocs.io/en/latest/shell_plus.html
- `runserver_plus`: https://django-extensions.readthedocs.io/en/latest/runserver_plus.html
- `graph_models`: https://django-extensions.readthedocs.io/en/latest/graph_models.html
- `runscript`: https://django-extensions.readthedocs.io/en/latest/runscript.html
- Jobs scheduling: https://django-extensions.readthedocs.io/en/latest/jobs_scheduling.html
- Model extensions: https://django-extensions.readthedocs.io/en/stable/model_extensions.html
- `show_permissions`: https://django-extensions.readthedocs.io/en/latest/show_permissions.html
- PyPI release page: https://pypi.org/project/django-extensions/4.1/
- Upstream repository: https://github.com/django-extensions/django-extensions
- Upstream releases: https://github.com/django-extensions/django-extensions/releases/tag/4.1
