---
name: celery-beat
description: "Database-backed periodic task scheduling for Celery in Django projects, with admin-managed interval, crontab, solar, and clocked schedules."
metadata:
  languages: "python"
  versions: "2.9.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,celery,beat,scheduler,periodic-tasks,cron,task-queue,python"
---

# `django-celery-beat` for Python

Use `django-celery-beat` when a Django project needs Celery beat schedules in the database instead of a local `celerybeat-schedule` file. It adds Django models and admin screens for interval, crontab, solar, and clocked schedules, and Celery beat reloads changes from the database.

This package is not a full Celery setup by itself. You still need:

- a Django project
- a Celery app
- a broker such as Redis or RabbitMQ
- one or more Celery workers
- one Celery beat process

## Install

Pin the target version if you need exact reproducibility:

```bash
pip install "django-celery-beat==2.9.0"
```

Typical installs also include Celery itself:

```bash
pip install "celery>=5.3" "django-celery-beat==2.9.0"
```

PyPI metadata for `2.9.0` declares:

- Python `>=3.8`
- Django classifiers through `6.0`

## Minimal Django and Celery setup

Add the app and run its migrations:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "django_celery_beat",
]
```

```bash
python manage.py migrate
```

Create the Celery app with the standard Django integration:

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

Point beat at the database scheduler:

```python
# settings.py
CELERY_BROKER_URL = os.environ["CELERY_BROKER_URL"]
CELERY_RESULT_BACKEND = os.getenv("CELERY_RESULT_BACKEND", "rpc://")
CELERY_BEAT_SCHEDULER = "django_celery_beat.schedulers:DatabaseScheduler"
CELERY_TIMEZONE = TIME_ZONE
```

`django-celery-beat` has no package-specific auth. Credentials come from:

- `CELERY_BROKER_URL` and broker-specific auth
- your result backend config if used
- Django database access for schedule tables

## Define tasks

Keep tasks in `tasks.py` so `app.autodiscover_tasks()` can find them:

```python
# myapp/tasks.py
from celery import shared_task

@shared_task
def import_contacts(source_id: int, force: bool = False) -> None:
    print(f"importing contacts from source {source_id}, force={force}")
```

## Create schedules and periodic tasks

`args`, `kwargs`, and `headers` are stored as JSON strings in the database.

### Interval schedule

```python
import json

from django_celery_beat.models import IntervalSchedule, PeriodicTask

schedule, _ = IntervalSchedule.objects.get_or_create(
    every=10,
    period=IntervalSchedule.SECONDS,
)

PeriodicTask.objects.update_or_create(
    name="import-contacts-every-10s",
    defaults={
        "interval": schedule,
        "task": "myapp.tasks.import_contacts",
        "args": json.dumps([42]),
        "kwargs": json.dumps({"force": False}),
        "enabled": True,
    },
)
```

Reuse shared `IntervalSchedule` rows instead of creating duplicates for the same cadence.

### Crontab schedule

```python
import json
from zoneinfo import ZoneInfo

from django_celery_beat.models import CrontabSchedule, PeriodicTask

schedule, _ = CrontabSchedule.objects.get_or_create(
    minute="0",
    hour="2",
    day_of_week="1-5",
    day_of_month="*",
    month_of_year="*",
    timezone=ZoneInfo("UTC"),
)

PeriodicTask.objects.update_or_create(
    name="weekday-nightly-import",
    defaults={
        "crontab": schedule,
        "task": "myapp.tasks.import_contacts",
        "kwargs": json.dumps({"source_id": 42, "force": True}),
        "queue": "imports",
        "enabled": True,
    },
)
```

### One-off clocked task

Clocked schedules are for one-time execution and require `one_off=True`:

```python
import json
from datetime import timedelta

from django.utils import timezone
from django_celery_beat.models import ClockedSchedule, PeriodicTask

clocked, _ = ClockedSchedule.objects.get_or_create(
    clocked_time=timezone.now() + timedelta(minutes=5),
)

PeriodicTask.objects.create(
    name="run-import-once",
    task="myapp.tasks.import_contacts",
    clocked=clocked,
    one_off=True,
    args=json.dumps([42]),
)
```

### Admin-managed schedules

The Django admin UI is the main reason to use this package:

- create `IntervalSchedule`, `CrontabSchedule`, `SolarSchedule`, or `ClockedSchedule`
- create one `PeriodicTask` that points to exactly one schedule row
- toggle `enabled` to pause or resume a task without deleting it

## Run workers and beat

Use separate processes in real deployments:

```bash
celery -A proj worker --loglevel=info
celery -A proj beat --loglevel=info --scheduler django_celery_beat.schedulers:DatabaseScheduler
```

The documented shortcut also works:

```bash
celery -A proj beat -l info -S django
```

Only run one scheduler for a given schedule set. Multiple beat instances against the same database-backed schedule will duplicate dispatches.

Celery's configuration docs note that the `django-celery-beat` scheduler checks for externally changed schedules every `5` seconds by default, so admin edits are not necessarily visible immediately.

## Configuration notes

### Time zones

If your project uses timezone-aware Django settings, keep:

```python
CELERY_TIMEZONE = TIME_ZONE
```

If you intentionally run a timezone-naive legacy project, the package exposes:

```python
DJANGO_CELERY_BEAT_TZ_AWARE = False
```

If you change Django timezone settings after tasks already ran, reset task state and bump the schedule change marker:

```python
from django_celery_beat.models import PeriodicTask, PeriodicTasks

PeriodicTask.objects.update(last_run_at=None)
PeriodicTasks.update_changed()
```

### Routing and expiry

`PeriodicTask` can store execution-routing fields such as:

- `queue`
- `exchange`
- `routing_key`
- `priority`
- `expires` or `expire_seconds`

Use these when the scheduled task should land on a specific worker queue. Do not set both `expires` and `expire_seconds` on the same task.

## Common pitfalls

### Package name and import name differ

Install name:

```text
django-celery-beat
```

Django app and import path:

```text
django_celery_beat
```

### Each `PeriodicTask` must point to exactly one schedule type

Choose exactly one of:

- `interval`
- `crontab`
- `solar`
- `clocked`

If more than one is set, model validation fails.

### Bulk updates need an explicit schedule-change bump

Normal model saves trigger schedule change tracking. If you use `QuerySet.update()`, raw SQL, or bulk operations, call:

```python
from django_celery_beat.models import PeriodicTasks

PeriodicTasks.update_changed()
```

Older snippets sometimes reference `PeriodicTasks.changed()`. Prefer `update_changed()` in current source and docs.

### JSON fields must be serialized strings

These model fields are text fields, not Python containers:

- `args`
- `kwargs`
- `headers`

Use `json.dumps(...)` before saving them.

### Clocked schedules need `one_off=True`

If you attach a `ClockedSchedule` without `one_off=True`, the model validation path rejects it.

## Version-sensitive notes for `2.9.0`

- The target version `2.9.0` matches the PyPI release published on `2026-02-28`.
- Read the Docs `latest` is currently stale and still renders `2.5.0` in the page chrome, so use it for concepts and examples, but prefer PyPI metadata and the `v2.9.0` GitHub release for compatibility facts.
- The PyPI long description currently still includes older `2.8.1` text in some places; the structured PyPI metadata and release tag are the safer version anchors.
- The `v2.9.0` release notes add Django `6.0` support and remove the Django upper version limit.

## Official sources used

- Docs root: https://django-celery-beat.readthedocs.io/en/latest/
- PyPI project: https://pypi.org/project/django-celery-beat/
- PyPI version JSON: https://pypi.org/pypi/django-celery-beat/2.9.0/json
- Repository: https://github.com/celery/django-celery-beat
- Release notes: https://github.com/celery/django-celery-beat/releases/tag/v2.9.0
- Celery Django integration: https://docs.celeryq.dev/en/latest/django/first-steps-with-django.html
- Celery configuration reference: https://docs.celeryq.dev/en/main/userguide/configuration.html
