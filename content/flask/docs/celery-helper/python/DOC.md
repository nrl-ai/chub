---
name: celery-helper
description: "Flask-Celery-Helper 1.1.0 guide for maintaining legacy Flask app-factory integrations with Celery"
metadata:
  languages: "python"
  versions: "1.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,celery,background-jobs,legacy,redis,worker"
---

# Flask-Celery-Helper for Flask Apps

## Compatibility And Scope

The released `1.1.0` PyPI page lists these tested versions:

- Python `2.6`, `2.7`, `3.3`, `3.4`
- Flask `0.10.1`
- Celery `3.1.11`
- Redis `2.9.1`

Treat that as the real support envelope for this doc. Do not assume compatibility with Flask 2.x or 3.x, Celery 5.x, or modern Python.

If you are starting a new Flask project, prefer plain Celery setup from current Celery docs instead of adding this extension.

## Install

Pin the package and keep the rest of the stack consistent with the old support window:

```bash
python -m pip install "Flask-Celery-Helper==1.1.0"
```

For a legacy environment that follows the package's documented test matrix, pin Celery and Flask explicitly as well:

```bash
python -m pip install \
  "Flask-Celery-Helper==1.1.0" \
  "Flask==0.10.1" \
  "celery==3.1.11"
```

If you use the Redis-backed `single_instance` flow from the release examples, install the matching Redis helper too:

```bash
python -m pip install "Flask-Redis-Helper"
```

## Minimal Setup

The basic release example configures the broker on the Flask app, constructs the extension, and exposes the Celery app as `example.celery` for the worker process:

```python
from flask import Flask
from flask.ext.celery import Celery

app = Flask("example")
app.config["CELERY_BROKER_URL"] = "redis://localhost"
app.config["CELERY_RESULT_BACKEND"] = "redis://localhost"

celery = Celery(app)

@celery.task()
def add_together(a, b):
    return a + b

if __name__ == "__main__":
    result = add_together.delay(23, 42)
    print(result.get())
```

Run the worker against the Celery object, not just the Flask app module:

```bash
celery -A example.celery worker
python example.py
```

## App Factory Setup

The main reason to keep this package is `init_app()` support for older factory layouts.

`extensions.py`

```python
from flask.ext.celery import Celery

celery = Celery()
```

`application.py`

```python
from flask import Flask

from extensions import celery

def create_app():
    app = Flask(__name__)
    app.config["CELERY_IMPORTS"] = ("tasks.add_together",)
    app.config["CELERY_BROKER_URL"] = "redis://localhost"
    app.config["CELERY_RESULT_BACKEND"] = "redis://localhost"
    celery.init_app(app)
    return app
```

`tasks.py`

```python
from extensions import celery

@celery.task()
def add_together(a, b):
    return a + b
```

`manage.py`

```python
from application import create_app

app = create_app()
```

Worker startup still needs a module path that resolves to the initialized Celery object in your project. The PyPI example shows the Celery app living on `example.celery`; adapt that pattern to your own module layout and verify it before automating worker commands.

## Configuration And Secrets

This extension reads Celery settings from Flask config. The release examples use:

- `CELERY_BROKER_URL`
- `CELERY_RESULT_BACKEND`
- `CELERY_IMPORTS`

Example:

```python
app.config.update(
    CELERY_BROKER_URL="redis://localhost/0",
    CELERY_RESULT_BACKEND="redis://localhost/0",
    CELERY_IMPORTS=("tasks.email", "tasks.reports"),
)
```

Auth is delegated to the broker and result-backend URLs, not to `Flask-Celery-Helper` itself. Put credentials in connection URLs and load them from environment or config files outside source control:

```python
import os

app.config["CELERY_BROKER_URL"] = os.environ["CELERY_BROKER_URL"]
app.config["CELERY_RESULT_BACKEND"] = os.environ.get("CELERY_RESULT_BACKEND", "")
```

Typical legacy connection strings:

- Redis: `redis://:password@host:6379/0`
- RabbitMQ / AMQP: `amqp://user:password@host:5672//`

Version note:

- `1.1.0` changed the package so `CELERY_RESULT_BACKEND` is no longer mandatory.
- In practice, if your code calls `AsyncResult.get()` or expects stored results, you still need a working result backend.

## Core Usage

### Queue a task

```python
task = add_together.delay(23, 42)
```

### Wait for a result

```python
value = task.get(timeout=10)
```

### Register task modules in factory apps

If your project creates Celery before importing task modules, set `CELERY_IMPORTS` so the worker sees those tasks when it starts:

```python
app.config["CELERY_IMPORTS"] = (
    "tasks.add_together",
    "tasks.cleanup",
)
```

Without that, factory-style apps often start a worker that does not know about your tasks yet.

## `single_instance` Locking

The package ships a `single_instance` decorator for deduplicating overlapping task runs.

PyPI's release example uses Redis plus `Flask-Redis-Helper`:

```python
import time
from flask import Flask
from flask.ext.celery import Celery, single_instance
from flask.ext.redis import Redis

app = Flask("example")
app.config["REDIS_URL"] = "redis://localhost"
app.config["CELERY_BROKER_URL"] = "redis://localhost"
app.config["CELERY_RESULT_BACKEND"] = "redis://localhost"

celery = Celery(app)
Redis(app)

@celery.task(bind=True)
@single_instance
def sleep_one_second(a, b):
    time.sleep(1)
    return a + b
```

The `1.1.0` changelog says `single_instance` gained SQLite, MySQL, and PostgreSQL support in addition to Redis. The release page does not include a full non-Redis example, so validate locking behavior in your exact backend before relying on it for correctness.

## Common Pitfalls

- This package is from 2014. Treat it as legacy maintenance tooling, not a current Flask recommendation.
- The released examples use the old `flask.ext.celery` namespace. That is an old Flask extension import style and is a strong signal that the package targets old Flask versions.
- Do not mix this guide with modern Celery 5.x or modern Flask 3.x tutorials. The API assumptions are different enough that copy-paste code is likely to fail.
- In factory apps, initialize the Flask app before starting the worker and make sure the worker imports the actual Celery object.
- `CELERY_RESULT_BACKEND` may be optional for fire-and-forget tasks, but result retrieval with `.get()` still needs a backend.
- `CELERY_IMPORTS` matters for worker startup in older factory layouts. Missing task imports usually show up as unregistered-task failures at runtime.
- `single_instance` is not a substitute for broader distributed-systems guarantees. Use it as a task dedupe helper, not as your only concurrency control.

## Version-Sensitive Notes

- `1.1.0` is the last published PyPI release shown on the project page.
- The `1.1.0` changelog includes:
  - Windows support
  - `CELERY_RESULT_BACKEND` no longer mandatory
  - `single_instance` support for SQLite, MySQL, and PostgreSQL in addition to Redis
  - a breaking move of `CELERY_LOCK`
- The docs URL is the repository root on GitHub, which is not version-locked to `1.1.0`. For package-matched behavior, anchor your code decisions to the `1.1.0` PyPI release page first.
- Current Celery documentation still says Flask integration packages are "not needed". That is the modern direction; use this package only when you have an inherited codebase that already depends on it.

## Source URLs

- PyPI project page: `https://pypi.org/project/Flask-Celery-Helper/`
- Official repository: `https://github.com/Robpol86/Flask-Celery-Helper`
- Current Celery docs: `https://docs.celeryq.dev/en/latest/getting-started/introduction.html`
