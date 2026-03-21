---
name: q2
description: "django-q2 package guide for background tasks, worker clusters, and schedules in Django projects"
metadata:
  languages: "python"
  versions: "1.9.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,task-queue,scheduler,workers,async,jobs,background"
---

# django-q2 Python Package Guide

`django-q2` is a Django-native task queue and scheduler. Use it when a Django app needs background jobs, recurring schedules, and a worker cluster without introducing Celery. The package is installed as `django-q2`, but the Django app label and import namespace are `django_q`.

## Install

Install the version used here explicitly if the project is pinned:

```bash
python -m pip install "django-q2==1.9.0"
```

Useful optional dependencies from the official docs and package metadata:

- `redis` for the Redis broker
- `boto3` for the SQS broker
- `pymongo` for the MongoDB broker
- `croniter` for cron schedules
- `blessed` for `qmonitor` and `qmemory`
- `psutil` if you use `cpu_affinity`

Example:

```bash
python -m pip install "django-q2==1.9.0" redis croniter blessed
```

## Minimal Django Setup

Add `django_q` to `INSTALLED_APPS` and run migrations:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "django_q",
]
```

```bash
python manage.py migrate
```

Choose a broker and define `Q_CLUSTER`. Redis is the best default for most non-trivial deployments:

```python
# settings.py
import os

Q_CLUSTER = {
    "name": "default",
    "workers": 4,
    "timeout": 90,
    "retry": 120,
    "recycle": 500,
    "queue_limit": 50,
    "save_limit": 250,
    "label": "Django Q2",
    "redis": os.environ["REDIS_URL"],
}
```

For local development or small installations, the ORM broker is simpler:

```python
Q_CLUSTER = {
    "name": "default",
    "workers": 2,
    "timeout": 90,
    "retry": 120,
    "queue_limit": 50,
    "bulk": 10,
    "orm": "default",
}
```

`django-q2` does not execute queued tasks inside the web process. Start a cluster in a separate process:

```bash
python manage.py qcluster
```

## Running And Routing Clusters

The cluster process manages a worker pool with Django Q2's internal sentinel. For one queue, start one cluster process and scale that cluster with settings like `workers`, `timeout`, and `recycle`.

```bash
python manage.py qcluster
```

If you use multiple queues, route them by cluster name:

```bash
Q_CLUSTER_NAME=long python manage.py qcluster
```

```python
from django_q.tasks import async_task

async_task("myapp.tasks.generate_report", report_id, cluster="long")
```

Schedules can also be restrained to a named cluster.

## Core Task Usage

Queue a normal importable callable:

```python
from django_q.tasks import async_task

task_id = async_task(
    "myapp.tasks.send_welcome_email",
    user_id,
    group="email",
    timeout=60,
    hook="myapp.hooks.on_welcome_email_complete",
)
```

Define the task as normal Django code:

```python
# myapp/tasks.py
from django.core.mail import send_mail

def send_welcome_email(user_id: int) -> None:
    send_mail(
        subject="Welcome",
        message=f"User {user_id} signed up.",
        from_email="noreply@example.com",
        recipient_list=["ops@example.com"],
    )
```

Read results later:

```python
from django_q.tasks import fetch, result

value = result(task_id, wait=200)
task = fetch(task_id, wait=200)
```

Task APIs agents usually need first:

- `async_task()` to enqueue work
- `result()` to get the saved result
- `fetch()` to fetch the full task record
- `result_group()` and `fetch_group()` for grouped work
- `async_iter()` for bulk fan-out work without manual loops

If your task function needs keyword names like `hook`, `group`, or `timeout`, pass queue options inside `q_options` so they do not collide with the function signature:

```python
from django_q.tasks import async_task

opts = {
    "group": "reports",
    "timeout": 30,
    "hook": "myapp.hooks.on_report_complete",
}

async_task("myapp.tasks.build_report", report_id, q_options=opts)
```

For repeated enqueue operations, reuse a broker connection:

```python
from django_q.brokers import get_broker
from django_q.tasks import async_task

broker = get_broker()
for report_id in report_ids:
    async_task("myapp.tasks.build_report", report_id, broker=broker)
```

## Scheduling Jobs

Create schedules from code or the admin. Cron schedules require `croniter`.

```python
from django_q.models import Schedule
from django_q.tasks import schedule

schedule(
    "myapp.tasks.send_digest",
    schedule_type=Schedule.CRON,
    cron="0 8 * * 1-5",
    repeats=-1,
    cluster="default",
)
```

Run a job every five minutes for two hours:

```python
from datetime import datetime

from django_q.models import Schedule
from django_q.tasks import schedule

schedule(
    "myapp.tasks.refresh_metrics",
    schedule_type=Schedule.MINUTES,
    minutes=5,
    repeats=24,
    next_run=datetime.utcnow().replace(hour=18, minute=0),
)
```

If you want the task to know the scheduled timestamp it was originally supposed to run at, use `intended_date_kwarg`:

```python
schedule(
    "myapp.tasks.run_billing_cycle",
    schedule_type=Schedule.DAILY,
    intended_date_kwarg="scheduled_for",
)
```

Scheduling fields that matter most in practice:

- `schedule_type`: once, minutes, hourly, daily, weekly, monthly, quarterly, yearly, or cron
- `minutes`: interval for `Schedule.MINUTES`
- `cron`: cron expression for `Schedule.CRON`
- `repeats`: `-1` means forever
- `next_run`: first execution time
- `cluster`: route the scheduled task to one named cluster
- `q_options`: per-schedule task options like `timeout` or `broker_name`

## Configuration And Broker/Auth Model

`django-q2` has no package-level authentication system. Access control comes from the broker or backend you configure in `Q_CLUSTER`.

Redis using a URI:

```python
Q_CLUSTER = {
    "name": "default",
    "workers": 4,
    "timeout": 90,
    "retry": 120,
    "redis": "redis://user:password@redis.example.com:6379/0",
}
```

Redis using `django-redis`:

```python
Q_CLUSTER = {
    "name": "default",
    "workers": 4,
    "timeout": 90,
    "django_redis": "default",
}
```

Amazon SQS:

```python
Q_CLUSTER = {
    "name": "sqs",
    "workers": 4,
    "timeout": 60,
    "retry": 90,
    "queue_limit": 100,
    "bulk": 5,
    "sqs": {
        "aws_region": "us-east-1",
        "aws_access_key_id": os.environ["AWS_ACCESS_KEY_ID"],
        "aws_secret_access_key": os.environ["AWS_SECRET_ACCESS_KEY"],
    },
}
```

Operational settings that most agent-written setups need to choose deliberately:

- `workers`: process count for the cluster
- `timeout`: maximum runtime for a task
- `retry`: redelivery delay for receipt-capable brokers
- `recycle`: restart workers after N tasks
- `save_limit`: how many successful task results to keep
- `queue_limit`: queued task cap in memory
- `sync`: force synchronous execution, mainly for tests
- `catch_up`: whether missed schedules replay after downtime
- `cpu_affinity`: optional performance tuning, requires `psutil`

All cluster processes that should work on the same queue need:

- the same broker
- the same cluster name
- the same Django `SECRET_KEY`

The docs also state that Django Q2 signs task packages with `SECRET_KEY`, so keep it consistent across worker instances.

## Management Commands

```bash
python manage.py migrate
python manage.py qcluster
python manage.py qmonitor
python manage.py qmemory
python manage.py qinfo
```

`qmonitor` and `qmemory` require `blessed`.

## Common Pitfalls

- The pip package is `django-q2`, but `INSTALLED_APPS` and imports use `django_q`.
- No background task runs unless a `qcluster` process is running or you explicitly force sync mode.
- Keep `retry` greater than `timeout` and greater than the longest real task duration, or brokers with delivery receipts can run the same task multiple times.
- `sync=True` is for tests and debugging. It bypasses the broker and does not validate production worker behavior.
- Cron schedules need `croniter`. Monitoring commands need `blessed`. Redis, SQS, MongoDB, and CPU-affinity features each need their own optional dependency.
- The ORM broker is convenient for local development, but Redis or another external broker is the safer default for production throughput and failure handling.
- `catch_up` defaults to replaying missed schedules. If a backlog burst after downtime is unacceptable, set `"catch_up": False`.
- Use `q_options` when task option names would otherwise collide with your function kwargs.
- Do not start multiple identical cluster supervisor processes in one environment to "scale up" by accident. The official cluster docs recommend tuning `workers`, `recycle`, and `timeout` first.

## Version-Sensitive Notes For `1.9.0`

- The version used here for this session is `1.9.0`.
- PyPI shows `django-q2 1.9.0` as the latest official release, published on December 4, 2025.
- The Read the Docs stream still serves pages branded `Django Q2 1.6.0`, so treat the docs site as conceptually accurate for APIs and configuration, but not as a precise version snapshot for `1.9.0`.
- The official `v1.8.0` release says support for Python `3.8` was dropped. Do not assume `1.9.0` works on Python `3.8`.
- The official `v1.9.0` release notes call out two changes that matter operationally: a fix for compatibility with `redis-py > 5`, and Django `6.0` support.
- If project behavior disagrees with the docs site, trust the installed package version, current PyPI metadata, and the GitHub release notes for the exact series you are on.

## Recommended Agent Workflow

1. Confirm that the project actually wants a Django-native queue and not a more distributed system.
2. Add `django_q`, run migrations, and choose the broker before writing task code.
3. Start one real `qcluster` process early and verify one simple `async_task()` path end to end.
4. Set `timeout`, `retry`, and `catch_up` explicitly instead of keeping defaults by accident.
5. Add schedules only after basic background execution is working.
6. When debugging version-specific behavior, compare the installed package against the official GitHub releases because the docs stream is behind.

## Official Sources Used

- Docs root: `https://django-q2.readthedocs.io/en/latest/`
- Configuration docs: `https://django-q2.readthedocs.io/en/master/configure.html`
- Tasks docs: `https://django-q2.readthedocs.io/en/master/tasks.html`
- Schedules docs: `https://django-q2.readthedocs.io/en/master/schedules.html`
- Cluster docs: `https://django-q2.readthedocs.io/en/master/cluster.html`
- Package page: `https://pypi.org/project/django-q2/`
- Releases: `https://github.com/django-q2/django-q2/releases`
