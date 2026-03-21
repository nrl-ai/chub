---
name: celery-results
description: "Django result backends for Celery task states, return values, and group results in Python projects"
metadata:
  languages: "python"
  versions: "2.6.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,celery,results,result-backend,orm,cache,python"
---

# `django-celery-results` for Python

`django-celery-results` adds Celery result backends that store task state, return values, and group metadata through Django's ORM or cache framework.

Use it when a Django project already has Celery and you need one or more of these:

- `AsyncResult.get()` or task result polling
- persistent task state rows in the Django database
- chord or group result storage
- Django admin visibility into task outcomes

It is only a result backend package. You still need a normal Celery app, a broker, and workers.

## Official Context

- Docs root: https://django-celery-results.readthedocs.io/en/latest/
- Getting started: https://django-celery-results.readthedocs.io/en/latest/getting_started.html
- Models reference: https://django-celery-results.readthedocs.io/en/latest/reference/django_celery_results.models.html
- PyPI: https://pypi.org/project/django-celery-results/
- Repository: https://github.com/celery/django-celery-results
- Releases: https://github.com/celery/django-celery-results/releases
- Celery Django integration: https://docs.celeryq.dev/en/stable/django/first-steps-with-django.html
- Celery configuration reference: https://docs.celeryq.dev/en/stable/userguide/configuration.html

## Install And Compatibility

Pin the target version when you need this exact package version:

```bash
pip install "django-celery-results==2.6.0"
```

The package metadata for `2.6.0` requires:

- `celery>=5.2.7,<6.0`
- `Django>=3.2.25`

A typical install is:

```bash
pip install "celery>=5.2.7,<6" "django>=3.2.25" "django-celery-results==2.6.0"
```

PyPI classifiers for `2.6.0` cover Python `3.8` through `3.13` and Django `3.2`, `4.1`, `4.2`, `5.0`, `5.1`, and `5.2`.

## Minimal Django Setup

Add the app and run migrations:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "django_celery_results",
]
```

```bash
python manage.py migrate django_celery_results
```

Create the Celery app with the standard Django integration pattern:

```python
# proj/celery.py
import os

from celery import Celery

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "proj.settings")

app = Celery("proj")
app.config_from_object("django.conf:settings", namespace="CELERY")
app.autodiscover_tasks()
```

```python
# proj/__init__.py
from .celery import app as celery_app

__all__ = ("celery_app",)
```

If tasks never need stored results, do not add this package just to mirror a standard Celery setup.

## Configure The Result Backend

### Option 1: Django database backend

This is the normal choice when callers depend on durable result rows.

```python
# settings.py
import os

CELERY_BROKER_URL = os.environ["CELERY_BROKER_URL"]
CELERY_RESULT_BACKEND = "django-db"
CELERY_ACCEPT_CONTENT = ["json"]
CELERY_TASK_SERIALIZER = "json"
CELERY_RESULT_SERIALIZER = "json"
CELERY_TASK_TRACK_STARTED = True
CELERY_RESULT_EXTENDED = True
CELERY_RESULT_EXPIRES = 86400
```

Notes:

- `CELERY_TASK_TRACK_STARTED = True` exposes a visible `STARTED` state instead of jumping from `PENDING` to the terminal state.
- `CELERY_RESULT_EXTENDED = True` stores extra task metadata when Celery provides it.
- `CELERY_RESULT_EXPIRES` only sets expiration policy. Schedule Celery's `backend_cleanup` task with beat if you want old rows removed automatically.

### Option 2: Django cache backend

Use this only if your project already has a cache configured and ephemeral result storage is acceptable.

```python
# settings.py
import os

CELERY_RESULT_BACKEND = "django-cache"
CELERY_CACHE_BACKEND = "results"

CACHES = {
    "results": {
        "BACKEND": "django.core.cache.backends.redis.RedisCache",
        "LOCATION": os.environ["REDIS_URL"],
    }
}
```

`CELERY_CACHE_BACKEND` is the Django cache alias to use. The backend type is still selected with `CELERY_RESULT_BACKEND = "django-cache"`.

For durable results, prefer `django-db`.

## Core Usage

Define a task, send it, and inspect its stored state:

```python
# myapp/tasks.py
from celery import shared_task

@shared_task(bind=True, autoretry_for=(ConnectionError,), retry_backoff=True)
def fetch_report(self, report_id: int) -> dict:
    self.update_state(state="PROGRESS", meta={"report_id": report_id, "step": "fetching"})
    return {"report_id": report_id, "status": "ready"}
```

Start a worker:

```bash
celery -A proj worker --loglevel=INFO
```

Queue work and later read the stored state:

```python
from celery.result import AsyncResult

from myapp.tasks import fetch_report

queued = fetch_report.delay(42)

result = AsyncResult(queued.id)
print(result.state)

if result.ready():
    print(result.get(timeout=1))
```

In the `2.6.0` line, `TaskResult.status` accepts custom states cleanly, so states such as `PROGRESS` can be persisted without relying on a fixed built-in status list.

## Query Stored Results Through Django

`django-celery-results` exposes models for direct inspection:

```python
from django_celery_results.models import GroupResult, TaskResult

latest_failures = (
    TaskResult.objects
    .filter(status="FAILURE")
    .order_by("-date_done")[:20]
)

for row in latest_failures:
    print(row.task_id, row.task_name, row.status, row.date_done)

group = GroupResult.objects.get(group_id="my-group-id")
print(group.result)
```

With `django-celery-beat`, task result rows can also record `periodic_task_name`, which helps trace a stored result back to the scheduled job that launched it.

## Testing And Local Development

If tests run Celery eagerly and still expect result rows, store eager results explicitly:

```python
# settings.py
CELERY_TASK_ALWAYS_EAGER = True
CELERY_TASK_STORE_EAGER_RESULT = True
```

Without `CELERY_TASK_STORE_EAGER_RESULT = True`, eager tasks execute inline but their results are typically not persisted to the configured backend.

## Auth And Configuration Model

This package has no package-specific auth layer. Access control comes from the systems around it:

- Celery broker credentials for RabbitMQ, Redis, SQS, or another transport
- Django database credentials if you use `django-db`
- Django cache credentials if you use `django-cache`
- Django admin permissions if you expose result rows in admin

Treat task results as application data. If tasks can return secrets, tokens, or PII, do not store those values directly in task results.

## Common Patterns

### Ignore results for fire-and-forget tasks

If a task does not need a stored return value, turn off results for that task instead of writing unnecessary rows:

```python
from celery import shared_task

@shared_task(ignore_result=True)
def send_webhook(payload: dict) -> None:
    ...
```

This matters even when `django-db` is configured globally.

### Clean up expired results

If your project stores many results, schedule backend cleanup so the `django_celery_results_taskresult` table does not grow without bound.

## Common Pitfalls

### Install name and app name differ

Package name:

```text
django-celery-results
```

Django app and import path:

```text
django_celery_results
```

### Migrations are required before the backend works

If `CELERY_RESULT_BACKEND = "django-db"` is set before `python manage.py migrate django_celery_results`, result writes fail at runtime.

### `django-cache` is not durable by default

If your cache is memory-only or has aggressive eviction, task results may disappear before callers fetch them. Use `django-db` if callers depend on stable result retrieval.

### `CELERY_CACHE_BACKEND` is a cache alias, not the backend type

For the cache backend, set `CELERY_RESULT_BACKEND = "django-cache"` and then point `CELERY_CACHE_BACKEND` at a Django cache alias such as `"default"` or `"results"`.

### `ignore_result=True` overrides your backend expectations

If a task or global setting ignores results, you will not get useful rows in `TaskResult` even though the backend is configured.

### `STARTED` state requires tracking to be enabled

Seeing only `PENDING` and terminal states usually means `CELERY_TASK_TRACK_STARTED` or task-level `track_started=True` is not enabled.

### Large or sensitive results become database baggage

Returned objects are serialized into backend storage. Keep task return payloads small and non-sensitive; persist larger business data in your own tables or object storage and return identifiers instead.

## Version-Sensitive Notes For `2.6.0`

- The Read the Docs `latest` site is still useful for setup and model reference, but it currently renders documentation labeled `2.4.0`. For `2.6.0` compatibility, cross-check PyPI metadata and the maintainer release notes.
- The `2.6.0` release line adds Django `5.2` and Python `3.13` support in upstream packaging metadata.
- The `2.6.0` release line allows custom task states in `TaskResult.status` and stores a start timestamp when started tracking is enabled.
