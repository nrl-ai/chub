---
name: simple-history
description: "django-simple-history package guide for Django model auditing, user attribution, and rollback workflows"
metadata:
  languages: "python"
  versions: "3.11.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django-simple-history,django,history,audit,admin,middleware,models"
---

# django-simple-history Python Package Guide

## What It Does

`django-simple-history` adds audit history to Django models by creating historical models and attaching a `history` manager. Use it when you need:

- a per-row change log
- point-in-time reads
- user attribution for changes
- admin-visible history and rollback flows

For package version `3.11.0`, the standard workflow is:

1. Install the package and add `simple_history` to `INSTALLED_APPS`.
2. Add `HistoricalRecords()` to tracked models, or call `register()` for models you cannot edit directly.
3. Run migrations so the historical tables exist.
4. Add `HistoryRequestMiddleware` if you want `history_user` populated from the current request.

## Version Covered

- Package: `django-simple-history`
- Import namespace: `simple_history`
- Version in this doc: `3.11.0`
- Python requirement: `>=3.10`
- PyPI classifiers for `3.11.0`: Django `4.2`, `5.1`, `5.2`, `6.0`; Python `3.10` through `3.14`

Version-sensitive note: upstream `3.11.0` adds Django `6.0` and Python `3.14` support, and drops Django `5.0` and Python `3.9`.

## Install

```bash
python -m pip install django-simple-history==3.11.0
```

```bash
uv add django-simple-history==3.11.0
```

```bash
poetry add django-simple-history==3.11.0
```

## Initialize And Migrate

Add the app:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "simple_history",
]
```

Track a model:

```python
# models.py
from django.db import models
from simple_history.models import HistoricalRecords

class Poll(models.Model):
    question = models.CharField(max_length=200)
    pub_date = models.DateTimeField("date published")
    history = HistoricalRecords()
```

Create and apply migrations:

```bash
python manage.py makemigrations
python manage.py migrate
```

If the model already has rows, backfill initial snapshots:

```bash
python manage.py populate_history --auto
```

If you cannot modify the model class directly, register it instead:

```python
from simple_history import register
from third_party_app.models import ExternalModel

register(ExternalModel)
```

## Core Usage

### Read historical rows

Each tracked model gets a `history` manager ordered newest first:

```python
poll = Poll.objects.get(pk=1)

latest_entry = poll.history.first()
full_audit_trail = poll.history.all()
```

You can also query history from the model class:

```python
history_qs = Poll.history.filter(question__icontains="django")
```

### Create changes and inspect history

```python
from django.utils import timezone

poll = Poll.objects.create(question="Initial", pub_date=timezone.now())
poll.question = "Updated"
poll.save()

for entry in poll.history.all():
    print(entry.history_date, entry.history_type, entry.question)
```

`history_type` is:

- `"+"` for create
- `"~"` for update
- `"-"` for delete

### Read model state at a point in time

Use `as_of()` when you need object state at a past timestamp:

```python
snapshot_qs = Poll.history.as_of(some_dt)
old_poll = Poll.history.as_of(some_dt).get(pk=1)
```

For a single object, `poll.history.most_recent()` is the common convenience path.

### Diff two historical records

```python
new_record, old_record = poll.history.all()[:2]
delta = new_record.diff_against(old_record)

for change in delta.changes:
    print(change.field, change.old, change.new)
```

In current releases, diff output is sorted consistently, which is useful for tests and agent-generated assertions.

### Restore from a historical record

```python
historical = poll.history.earliest()
restored = historical.instance
restored.save()
```

This is an application-level restore. Signals, validation, and related model side effects still apply when you save the reconstructed instance.

## Configuration And User Attribution

`django-simple-history` has no API credentials or network auth. The relevant configuration is Django-side tracking of the acting user and of what fields get historical coverage.

### Track the acting user from requests

Add the middleware:

```python
# settings.py
MIDDLEWARE = [
    # ...
    "simple_history.middleware.HistoryRequestMiddleware",
    # ...
]
```

With Django auth enabled, this populates `history_user` from `request.user`.

### Track the acting user manually

When work happens outside the request cycle, set `_history_user` before saving:

```python
poll._history_user = request.user
poll.save()
```

You can also set `_change_reason` or `_history_date` when you need explicit metadata:

```python
poll._change_reason = "import fix"
poll.save()
```

### Disable history globally

```python
SIMPLE_HISTORY_ENABLED = False
```

This is useful in controlled maintenance flows, imports, or tests where audit rows would be noise.

### Add history to Django admin

```python
from django.contrib import admin
from simple_history.admin import SimpleHistoryAdmin

from .models import Poll

@admin.register(Poll)
class PollAdmin(SimpleHistoryAdmin):
    pass
```

## Customizing What Gets Tracked

Exclude fields that do not belong in history:

```python
history = HistoricalRecords(excluded_fields=["updated_at"])
```

Track many-to-many changes explicitly:

```python
class Poll(models.Model):
    categories = models.ManyToManyField("Category")
    history = HistoricalRecords(m2m_fields=["categories"])
```

If you skip `m2m_fields`, normal M2M edits are not tracked.

## Bulk Operations

Regular `save()` and `delete()` create historical rows. Bulk APIs need explicit helpers.

Use the package utilities when you need history for bulk inserts or updates:

```python
from django.utils import timezone
from simple_history.utils import bulk_create_with_history, bulk_update_with_history

polls = [
    Poll(question="One", pub_date=timezone.now()),
    Poll(question="Two", pub_date=timezone.now()),
]
bulk_create_with_history(polls, Poll)
```

```python
polls[0].question = "Renamed"
bulk_update_with_history(polls, Poll, ["question"])
```

Do not expect `QuerySet.update()` to create history rows. It bypasses `post_save`, which the package relies on.

## Common Pitfalls

- Forgetting `"simple_history"` in `INSTALLED_APPS` before creating migrations.
- Adding `HistoricalRecords()` but never running `makemigrations` and `migrate`.
- Using `QuerySet.update()` and expecting audit rows.
- Using `F()` expressions on tracked models. The docs call this unsupported because the historical insert sees the unresolved expression and raises `ValueError`.
- Assuming M2M changes are tracked automatically. They are only tracked when you opt in with `m2m_fields=[...]`.
- Expecting request middleware to populate `history_user` in Celery tasks, management commands, or scripts. Set `_history_user` manually there.
- Using model field names that conflict with reserved historical model names such as `history_id`, `history_date`, `history_change_reason`, or `history_type`.

## Version-Sensitive Notes

- `3.11.0` is the current package release covered here and aligns with the current stable docs.
- `3.10.0` moved the project from the old Jazzband organization to `django-commons/django-simple-history`. Prefer current repository links when following issues or releases.
- `3.9.0` removed the deprecated `simple_history_admin_list.display_list` template tag. Older admin customizations may need cleanup.
- `3.8.0` changed save-suppression behavior so `skip_history_when_saving` also works for object creation, and it improved M2M history query performance.

## Official Links

- Documentation root: https://django-simple-history.readthedocs.io/en/stable/
- Quick start: https://django-simple-history.readthedocs.io/en/stable/quick_start.html
- Querying history: https://django-simple-history.readthedocs.io/en/stable/querying_history.html
- Historical model customization: https://django-simple-history.readthedocs.io/en/stable/historical_model.html
- User tracking: https://django-simple-history.readthedocs.io/en/stable/user_tracking.html
- Common issues: https://django-simple-history.readthedocs.io/en/stable/common_issues.html
- Admin integration: https://django-simple-history.readthedocs.io/en/stable/admin.html
- Changelog: https://django-simple-history.readthedocs.io/en/stable/changelog.html
- PyPI: https://pypi.org/project/django-simple-history/
- Repository: https://github.com/django-commons/django-simple-history
