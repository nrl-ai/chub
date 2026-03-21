---
name: model-utils
description: "django-model-utils package guide for Django projects using abstract models, custom fields, managers, and tracking utilities"
metadata:
  languages: "python"
  versions: "5.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,python,models,orm,managers,tracking"
---

# django-model-utils Python Package Guide

## Golden Rule

Use `django-model-utils` as a thin utility layer on top of standard Django models and managers. It is not a standalone Django app, so you usually import the helpers directly and then run normal Django migrations.

As of March 12, 2026, PyPI still lists `5.0.0` as the current release. The official docs under `latest/` are not version-stable, so use the version-pinned `5.0.0` docs for behavior and API details.

## Install

Pin the package to the version your project expects:

```bash
python -m pip install "django-model-utils==5.0.0"
```

Common alternatives:

```bash
uv add "django-model-utils==5.0.0"
poetry add "django-model-utils==5.0.0"
```

This package does not need to be added to `INSTALLED_APPS`.

## Setup Pattern

There is no separate initialization step. Add the fields, mixins, or managers you need to your models, then create and apply migrations:

```bash
python manage.py makemigrations
python manage.py migrate
```

Typical imports:

```python
from django.db import models
from model_utils import Choices, FieldTracker
from model_utils.fields import MonitorField, SplitField, StatusField
from model_utils.managers import InheritanceManager, QueryManager
from model_utils.models import SoftDeletableModel, StatusModel, TimeStampedModel
```

## Core Usage

### Common model pattern

This package is most useful when you combine a few small pieces instead of adopting everything:

```python
from django.db import models
from model_utils import Choices, FieldTracker
from model_utils.fields import MonitorField, SplitField
from model_utils.models import SoftDeletableModel, StatusModel, TimeStampedModel

class Article(TimeStampedModel, StatusModel, SoftDeletableModel):
    STATUS = Choices("draft", "published", "archived")

    title = models.CharField(max_length=200)
    body = SplitField()
    tracker = FieldTracker(fields=["status", "title"])
    published_at = MonitorField(
        monitor="status",
        when=["published"],
        null=True,
        default=None,
    )

    class Meta:
        ordering = ["-modified"]
```

What this gives you:

- `TimeStampedModel`: `created` and `modified` fields that update automatically
- `StatusModel`: a `status` field plus status-specific managers such as `Article.published`
- `SoftDeletableModel`: soft delete support with explicit managers for active vs deleted rows
- `SplitField`: a content field plus generated excerpt support
- `FieldTracker`: in-memory change tracking between loads and saves
- `MonitorField`: timestamp updates when another field changes to selected values

### `Choices`

`Choices` builds readable constants and choice tuples from a single declaration:

```python
from model_utils import Choices

STATUS = Choices(
    ("draft", "Draft"),
    ("published", "Published"),
    ("archived", "Archived"),
)

STATUS.published
STATUS.draft
STATUS
```

Use `subset()` when a field or form should only expose part of a larger choice set:

```python
VISIBLE_STATUS = STATUS.subset("draft", "published")
```

If your codebase already standardizes on Django `TextChoices` or `IntegerChoices`, keep that standard. `django-model-utils` does not require `Choices` everywhere.

### `StatusModel` and `StatusField`

`StatusModel` expects a `STATUS` iterable on the model class and adds:

- a `status` field via `StatusField`
- a `status_changed` timestamp field
- per-status managers, for example `Article.published.all()`

Example:

```python
class Article(StatusModel):
    STATUS = Choices("draft", "published")
    title = models.CharField(max_length=200)
```

Practical notes:

- `StatusField` defaults to the first choice if you do not set `default=...`
- the automatic managers are convenient, but they are ordinary Django managers and still respect queryset chaining rules
- add your own database indexes if you filter heavily on `status`; `StatusField` is not a magic performance layer

### `TimeStampedModel`

Use `TimeStampedModel` when you want `created` and `modified` fields without re-implementing save hooks:

```python
class Note(TimeStampedModel):
    title = models.CharField(max_length=100)
```

`modified` updates on every save. That is useful for audit-style timestamps but not a replacement for domain-specific workflow fields.

### `SoftDeletableModel`

Use `SoftDeletableModel` when rows should stay in the database but disappear from active queries:

```python
class Customer(SoftDeletableModel):
    email = models.EmailField(unique=True)
```

Use the explicit managers:

- `Customer.available_objects`: rows that are not soft-deleted
- `Customer.all_objects`: all rows, including deleted ones

Avoid building new code against `Customer.objects` for soft-delete filtering. The manager behavior on `objects` is in a deprecation path and future releases are expected to stop filtering deleted rows there.

To hard-delete, pass the keyword argument explicitly:

```python
customer.delete(soft=False)
```

### `FieldTracker`

`FieldTracker` helps when code needs to know whether a field changed since the instance was loaded:

```python
class Order(models.Model):
    status = models.CharField(max_length=20)
    total = models.DecimalField(max_digits=10, decimal_places=2)
    tracker = FieldTracker(fields=["status", "total"])

order = Order.objects.get(pk=1)
order.status = "paid"

order.tracker.has_changed("status")
order.tracker.previous("status")
order.tracker.changed()
```

Use this for conditional side effects, audit logging, and transition checks. It does not replace database constraints or transaction boundaries.

For foreign keys, tracker values are based on the raw `field_id` value rather than automatically loading the related object.

### `InheritanceManager`

Use `InheritanceManager` for model inheritance trees when you need subclass instances back from the base queryset:

```python
from model_utils.managers import InheritanceManager

class Place(models.Model):
    objects = InheritanceManager()
    name = models.CharField(max_length=100)

class Restaurant(Place):
    serves_pizza = models.BooleanField(default=False)

places = Place.objects.select_subclasses()
```

Without `select_subclasses()`, you get base-model instances. With it, rows are cast back to their concrete subclasses.

### `QueryManager`

Use `QueryManager` for simple filtered managers when a full custom manager class would be overkill:

```python
class Post(models.Model):
    published = models.BooleanField(default=False)

    public = QueryManager(published=True)
```

This is a convenience wrapper around a fixed queryset filter. Do not stretch it into complex business logic.

## Fields And Utilities

### `MonitorField`

`MonitorField` updates a timestamp when another field changes:

```python
class Invoice(models.Model):
    status = models.CharField(max_length=20)
    paid_at = MonitorField(
        monitor="status",
        when=["paid"],
        null=True,
        default=None,
    )
```

Set an explicit `default` when the field is nullable. Do not rely on historical implicit default behavior across older examples.

### `SplitField`

`SplitField` stores a content body and an excerpt companion field:

```python
class BlogPost(models.Model):
    body = SplitField()
```

Useful behaviors:

- access the rendered excerpt via `instance.body.excerpt`
- insert a split marker in the content to control where the excerpt ends
- otherwise the excerpt is generated from the opening paragraphs

The package documents these Django settings for excerpt behavior:

- `SPLIT_MARKER`
- `SPLIT_DEFAULT_PARAGRAPHS`

### Other useful model helpers

The package also includes abstract models such as:

- `TimeFramedModel` for start/end publication windows
- `UUIDModel` for UUID primary-style identifiers

Use them only when they match your domain model cleanly. They are conveniences, not required architecture.

## Configuration

This package has no authentication layer and almost no global configuration.

Configuration that matters in practice:

- you do not add the package to `INSTALLED_APPS`
- excerpt behavior for `SplitField` can be adjusted with `SPLIT_MARKER` and `SPLIT_DEFAULT_PARAGRAPHS`
- model behavior still depends on normal Django settings such as database engine, timezone handling, and installed middleware

Example Django settings for `SplitField`:

```python
SPLIT_MARKER = "<!-- more -->"
SPLIT_DEFAULT_PARAGRAPHS = 3
```

## Common Pitfalls

- Do not point agents at `https://django-model-utils.readthedocs.io/en/latest/` and assume it matches `5.0.0`. Use the version-pinned docs for this package version.
- Do not add `django-model-utils` to `INSTALLED_APPS`. It provides model helpers, not a Django app with migrations or app config you must register.
- Always run `makemigrations` and `migrate` after adding package fields or abstract-model inheritance to a model.
- `SoftDeletableModel.objects` is not the safe long-term manager for active rows. Use `available_objects` or `all_objects` explicitly.
- Hard delete requires `delete(soft=False)`. Passing a positional argument is no longer the intended pattern.
- `FieldTracker` tracks local instance state, not concurrent updates from other transactions.
- `InheritanceManager` only returns subclass instances when you call `select_subclasses()`.
- `SplitField` excerpt output updates when the model is saved. If you inspect excerpt behavior before saving, you can misread the result.

## Version-Sensitive Notes For 5.0.0

- `django-model-utils 5.0.0` dropped the deprecated `JoinManager` and `JoinQueryset`. If older code imports them, rewrite that code instead of pinning to ancient examples.
- `SaveSignalHandlingModel` was removed before `5.0.0`; do not use blog posts that still recommend it.
- `SplitField` no longer accepts the old `no_excerpt_field` argument.
- The `SoftDeletableModel` API now documents the `soft` keyword argument on `delete()`.
- The upstream changelog documents a transition in `MonitorField` nullable default behavior across recent releases. For stable behavior, set `default` explicitly when using `null=True`.
- The setup page for `5.0.0` still says Python `3.7+`, but the PyPI metadata for the same release requires Python `>=3.8`. Treat the PyPI requirement as authoritative for packaging and environment resolution.
- The published `5.0.0` package metadata does not claim Django 6 support. Validate compatibility before using it in Django 6 projects.

## Official Sources

- Docs landing page: https://django-model-utils.readthedocs.io/en/latest/
- Version-pinned docs: https://django-model-utils.readthedocs.io/en/5.0.0/
- Setup: https://django-model-utils.readthedocs.io/en/5.0.0/setup.html
- Fields: https://django-model-utils.readthedocs.io/en/5.0.0/fields.html
- Models: https://django-model-utils.readthedocs.io/en/5.0.0/models.html
- Managers: https://django-model-utils.readthedocs.io/en/5.0.0/managers.html
- Utilities: https://django-model-utils.readthedocs.io/en/5.0.0/utilities.html
- Changelog: https://django-model-utils.readthedocs.io/en/5.0.0/changelog.html
- PyPI: https://pypi.org/project/django-model-utils/
