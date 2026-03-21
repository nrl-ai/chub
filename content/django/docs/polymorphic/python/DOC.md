---
name: polymorphic
description: "django-polymorphic package guide for Django multi-table inheritance, polymorphic querysets, and admin integration"
metadata:
  languages: "python"
  versions: "4.11.2"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,orm,polymorphism,model-inheritance,admin,querysets,typing"
---

# django-polymorphic Python Package Guide

## What It Does

`django-polymorphic` makes Django multi-table inheritance return real subclass instances when you query the base model. Use it when you have one conceptual model with subtype-specific fields and you want the Django ORM and admin to preserve the concrete type automatically.

Use plain Django models instead when:

- you only need abstract base classes or shared mixins
- subtype-specific data can live in separate related models
- the extra joins from multi-table inheritance are not worth the convenience

## Install

```bash
python -m pip install "django-polymorphic==4.11.2"
```

If you use static typing, upstream `4.11` also ships first-party type hints. For type checking, install `django-stubs` and optionally `django-stubs-ext` in your dev environment.

## Django Setup

Add the package and the contenttypes framework to `INSTALLED_APPS`:

```python
INSTALLED_APPS = [
    # ...
    "django.contrib.contenttypes",
    "polymorphic",
]
```

Then run migrations as usual:

```bash
python manage.py migrate
```

There is no package-level network authentication or external service configuration. Setup is just Django app registration plus normal model/admin wiring.

## First Working Model

Use `PolymorphicModel` for the shared base model. Child models should inherit from that base as normal Django multi-table subclasses.

```python
from django.db import models
from polymorphic.models import PolymorphicModel

class Project(PolymorphicModel):
    topic = models.CharField(max_length=200)

class ArtProject(Project):
    artist = models.CharField(max_length=200)

class ResearchProject(Project):
    supervisor = models.CharField(max_length=200)
```

The base model stores an internal `polymorphic_ctype` foreign key to `ContentType`. That field is managed for you on save.

## Querying

A base-model queryset returns real subclasses automatically:

```python
projects = Project.objects.all()
for project in projects:
    print(type(project).__name__, project.topic)
```

Filter by subtype with `instance_of()` and `not_instance_of()`:

```python
art_projects = Project.objects.instance_of(ArtProject)
not_research = Project.objects.not_instance_of(ResearchProject)
```

Filter or order on child-only fields with the documented `ModelName___field` syntax:

```python
from django.db.models import Q

turner_projects = Project.objects.filter(
    Q(ArtProject___artist="T. Turner")
    | Q(ResearchProject___supervisor="T. Turner")
)
```

When you intentionally want base objects only, turn off polymorphic casting for that queryset:

```python
base_rows = Project.objects.non_polymorphic().only("id", "topic")
```

`non_polymorphic()` is useful for narrow read paths, but do not fall back to calling `get_real_instance()` on every row in a loop for large querysets. The package already batches subtype fetching by model type.

## Custom Managers

If you add a custom manager, inherit from `PolymorphicManager`, not `models.Manager`. Keep the first/default manager unfiltered and use plain `PolymorphicManager` for it.

```python
from polymorphic.managers import PolymorphicManager

class TimeOrderedManager(PolymorphicManager):
    def get_queryset(self):
        return super().get_queryset().order_by("-start_date")

    def most_recent(self):
        return self.get_queryset()[:10]

class Project(PolymorphicModel):
    objects = PolymorphicManager()
    objects_ordered = TimeOrderedManager()
```

The first manager defined is used by Django for related-object access and internal behavior. Do not make that manager filtered or non-polymorphic.

## Admin Integration

For one unified admin surface:

- Base admin class: inherit from `PolymorphicParentModelAdmin`
- Child admin classes: inherit from `PolymorphicChildModelAdmin`
- Set `base_model` on both sides
- Set `child_models` on the parent admin or override `get_child_models()`
- Register the child admins too, even though they stay hidden from the index unless `show_in_index = True`

Typical shape:

```python
from django.contrib import admin
from polymorphic.admin import (
    PolymorphicChildModelAdmin,
    PolymorphicParentModelAdmin,
)

@admin.register(Project)
class ProjectAdmin(PolymorphicParentModelAdmin):
    base_model = Project
    child_models = (ArtProject, ResearchProject)

@admin.register(ArtProject)
class ArtProjectAdmin(PolymorphicChildModelAdmin):
    base_model = Project

@admin.register(ResearchProject)
class ResearchProjectAdmin(PolymorphicChildModelAdmin):
    base_model = Project
```

Two rules matter in practice:

- register the child admins too, even though they stay hidden from the index unless `show_in_index = True`
- put fieldset customizations on child admins; if you subclass child admins, prefer `base_form` and `base_fieldsets` over `form` and `fieldsets`

If you need polymorphic inlines, use the package's polymorphic inline helpers instead of plain Django inline classes alone.

## Configuration And Auth

`django-polymorphic` does not talk to an external API and has no package-specific auth model. The configuration that actually matters is:

- `INSTALLED_APPS` includes both `"polymorphic"` and `"django.contrib.contenttypes"`
- your base models inherit from `PolymorphicModel`
- your default manager stays a plain `PolymorphicManager`
- your admin registration uses the polymorphic admin base classes if you want a unified admin UI

If your project also uses Django auth, DRF, or custom permissions, those stay configured the normal Django way. `django-polymorphic` only changes model/query behavior.

## Migrations And Existing Data

When converting an existing model tree to `django-polymorphic`:

1. Change the base model to inherit from `PolymorphicModel`.
2. Create the migration that adds `polymorphic_ctype_id`.
3. Backfill `polymorphic_ctype` for existing rows before relying on polymorphic queries.

Minimal backfill pattern:

```python
from django.contrib.contenttypes.models import ContentType

from myapp.models import Project

ct = ContentType.objects.get_for_model(Project)
Project.objects.filter(polymorphic_ctype__isnull=True).update(polymorphic_ctype=ct)
```

Do this in a migration or controlled data-fix step, not ad hoc at request time.

## Performance Notes

- A normal polymorphic queryset does one query for the base model plus one extra query per concrete subtype present in the result set.
- This is much better than row-by-row upcasting, which can turn into one extra query per object.
- Each subclass still means extra joins for child fields, so do not use polymorphic inheritance for very hot paths unless the type-aware model shape is worth it.
- Large iterations fetch a maximum of `2000` objects per chunk by default. Use `.iterator(chunk_size=...)` when memory or query shape matters.

## Type Checking

Starting in `4.11`, the package ships type hints for its public APIs and documents how to annotate `PolymorphicManager` and related descriptors.

Practical guidance:

- there are no extra runtime dependencies for typing support
- `django-stubs` is required in the type-checking environment
- `django-stubs-ext` is optional
- Django cannot infer precise polymorphic return unions automatically, so annotate custom managers and relationship descriptors explicitly if you want strong editor and CI feedback

## Common Pitfalls

- Install the Django app as `"polymorphic"`, not `"django_polymorphic"`.
- Keep `"django.contrib.contenttypes"` enabled; `polymorphic_ctype` depends on it.
- Child-field filters use three underscores between model and field, for example `ArtProject___artist`.
- Do not make a filtered custom manager the first manager on the model; Django uses the first manager for related-object access and internal behavior.
- Register the child admin classes too. The parent admin alone is not enough.
- If you bypass polymorphic behavior with `.non_polymorphic()`, any later per-row upcasting can create an N+1 query pattern.
- Manual raw-SQL deletes can leave stale `polymorphic_ctype` values behind and make polymorphic queries skip or fail on affected rows.
- This package is for Django model inheritance, not for abstract model mixins or single-table inheritance.

## Version-Sensitive Notes

As of `2026-03-12`, PyPI lists `4.11.2` as the current release, published on `2026-03-07`.

- `4.11` added first-party type hints and a dedicated typing guide.
- PyPI marks `4.10.3` and `4.10.4` as yanked because of a base manager bug. Do not pin those releases in new work.
- The `4.x` line targets modern Python and Django versions. If a project is still pinned below Python `3.10` or needs older Django compatibility, verify whether it belongs on the `3.x` line instead.
- The official stable docs are slightly inconsistent today: the home, quickstart, admin, migrating, and typing pages render as `4.11.2`, the performance page renders as `4.11.1`, and the changelog page currently renders `4.9.0` headings. Use PyPI release history for exact patch-level version selection.

## Official Sources

- Docs root: https://django-polymorphic.readthedocs.io/en/stable/
- Quickstart: https://django-polymorphic.readthedocs.io/en/stable/quickstart.html
- Managers and querysets: https://django-polymorphic.readthedocs.io/en/stable/managers.html
- Admin integration: https://django-polymorphic.readthedocs.io/en/stable/admin.html
- Performance: https://django-polymorphic.readthedocs.io/en/stable/performance.html
- Migrating existing models: https://django-polymorphic.readthedocs.io/en/stable/migrating.html
- Type hints: https://django-polymorphic.readthedocs.io/en/stable/typing.html
- Changelog: https://django-polymorphic.readthedocs.io/en/stable/changelog.html
- PyPI: https://pypi.org/project/django-polymorphic/
- Repository: https://github.com/jazzband/django-polymorphic
