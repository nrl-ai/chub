---
name: mptt
description: "django-mptt package guide for Python tree models, admin, forms, and template usage"
metadata:
  languages: "python"
  versions: "0.18.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,django-mptt,mptt,trees,hierarchy,orm"
---

# django-mptt Python Package Guide

## What It Is

`django-mptt` adds tree support to Django models using Modified Preorder Tree Traversal. Install it as `django-mptt`, but import from `mptt`.

Use it when a model needs parent/child hierarchy and you need efficient ancestor, descendant, sibling, and tree-order queries.

Current package facts for this entry:

- PyPI version covered: `0.18.0`
- Python requirement: `>=3.9`
- Current supported Django line from PyPI classifiers: `4.2`, `5.0`, `5.1`, `5.2`
- Upstream maintenance state: PyPI marks the project as currently unmaintained

## Install

```bash
pip install django-mptt==0.18.0
```

Add `mptt` to `INSTALLED_APPS`:

```python
INSTALLED_APPS = [
    # ...
    "mptt",
]
```

Then create and apply migrations:

```bash
python manage.py makemigrations
python manage.py migrate
```

## Basic Model Setup

Subclass `MPTTModel` and use `TreeForeignKey` for the parent link:

```python
from django.db import models
from mptt.models import MPTTModel, TreeForeignKey

class Category(MPTTModel):
    name = models.CharField(max_length=100, unique=True)
    parent = TreeForeignKey(
        "self",
        on_delete=models.CASCADE,
        null=True,
        blank=True,
        related_name="children",
    )

    class MPTTMeta:
        order_insertion_by = ["name"]

    class Meta:
        verbose_name_plural = "categories"

    def __str__(self) -> str:
        return self.name
```

Important setup rules:

- In multiple inheritance, keep `MPTTModel` first to avoid Django model validation errors.
- `TreeForeignKey` is recommended for the parent field because forms and admin render tree-aware choices.
- `MPTTModel` manages `level`, `lft`, `rght`, and `tree_id` automatically. Do not use those fields as business identifiers.

## Core Usage

Create nodes with normal ORM calls:

```python
root = Category.objects.create(name="Books")
fiction = Category.objects.create(name="Fiction", parent=root)
history = Category.objects.create(name="History", parent=root)
```

Common tree operations:

```python
root.get_children()
fiction.get_ancestors(include_self=True)
root.get_descendants()
root.get_descendant_count()
fiction.get_siblings(include_self=True)
fiction.get_root()
fiction.is_leaf_node()
fiction.is_root_node()
```

`TreeManager` returns querysets in tree order by default, so `Category.objects.all()` is usually suitable for navigation trees and nested rendering.

`get_children()` is preferable to the reverse relation when you want immediate children because it can avoid a query for leaf nodes. `get_descendant_count()` uses tree fields and does not need a database query.

## Rendering Trees In Templates

Load `mptt_tags` and render with `recursetree`:

```django
{% load mptt_tags %}

<ul class="root">
  {% recursetree nodes %}
    <li>
      {{ node.name }}
      {% if not node.is_leaf_node %}
        <ul class="children">
          {{ children }}
        </ul>
      {% endif %}
    </li>
  {% endrecursetree %}
</ul>
```

For preloaded querysets, use `get_cached_trees()` or its template alias `cache_tree_children` to cache parent/child links on nodes and avoid follow-up queries during traversal.

## Admin Integration

For a simple tree-aware changelist:

```python
from django.contrib import admin
from mptt.admin import MPTTModelAdmin

from .models import Category

@admin.register(Category)
class CategoryAdmin(MPTTModelAdmin):
    pass
```

For drag and drop:

```python
from django.contrib import admin
from mptt.admin import DraggableMPTTAdmin

from .models import Category

@admin.register(Category)
class CategoryAdmin(DraggableMPTTAdmin):
    list_display = ("tree_actions", "indented_title")
    list_display_links = ("indented_title",)
```

Practical admin notes:

- `DraggableMPTTAdmin` does not work well on big trees; upstream warns about trees with more than a few hundred nodes or deeper than 10 levels.
- `tree_actions` should be first in `list_display` for draggable admin.
- Global indentation can be set with `MPTT_ADMIN_LEVEL_INDENT`; per-admin indentation uses `mptt_level_indent`.

## Forms And Controlled Moves

`TreeForeignKey` already uses a tree-aware form field. Use `TreeNodeChoiceField` directly when building custom forms:

```python
from django import forms
from mptt.forms import TreeNodeChoiceField

from .models import Category

class CategoryForm(forms.Form):
    parent = TreeNodeChoiceField(
        queryset=Category.objects.all(),
        required=False,
        level_indicator="--",
    )
```

For manual repositioning flows, `MoveNodeForm` validates target and position choices:

```python
from mptt.forms import MoveNodeForm

form = MoveNodeForm(category, request.POST or None)
if form.is_valid():
    category = form.save()
```

## Bulk Updates And Imports

For localized bulk changes inside one tree or a small part of the forest, use `delay_mptt_updates()` inside a transaction:

```python
from django.db import transaction

with transaction.atomic():
    with Category.objects.delay_mptt_updates():
        # bulk inserts or reparents within a small number of trees
        ...
```

For changes across much of the table, use `disable_mptt_updates()` and rebuild afterward:

```python
from django.db import transaction

with transaction.atomic():
    with Category.objects.disable_mptt_updates():
        # large bulk changes
        ...
    Category.objects.rebuild()
```

Use `disable_mptt_updates()` carefully: upstream documents that it leaves the tree inconsistent until `rebuild()` runs. `delay_mptt_updates()` is safer for localized work because it performs partial rebuilds of modified trees.

If you have nested input data, the manager also exposes bulk-tree helpers such as `build_tree_nodes()` for efficient imports before `bulk_create()`.

## Configuration Notes

There is no auth or credential setup. Configuration is model and admin driven.

Useful `MPTTMeta` options:

- `order_insertion_by` for sibling ordering
- `parent_attr` if your parent field is not named `parent`
- `left_attr`, `right_attr`, `tree_id_attr`, `level_attr` if you need alternate structural field names

Useful admin settings:

- `MPTT_ADMIN_LEVEL_INDENT` for global admin indentation
- `mptt_level_indent` for per-admin indentation
- `expand_tree_by_default = True` on `DraggableMPTTAdmin` if you want the tree expanded on first load

Testing setting:

- `MPTT_ALLOW_TESTING_GENERATORS = True` only when you intentionally use generators such as `model_bakery` in tests and you know how to set tree fields safely

## Common Pitfalls

- Install name and import name differ: `pip install django-mptt`, then `import mptt`.
- If you use `order_insertion_by`, saved instances can become stale after inserts or reparents. Call `refresh_from_db()` before relying on tree fields from older in-memory objects.
- `move_to()` assumes both the current node and target node reflect current database state. Stale tree fields can corrupt the move.
- `move_to()` is explicit positioning logic. If you want automatic sibling ordering from `order_insertion_by`, create or reparent normally and let MPTT place the node.
- `disable_mptt_updates()` and `delay_mptt_updates()` both create temporary inconsistencies during the block. Wrap them in `transaction.atomic()`.
- `DraggableMPTTAdmin` is not a good default for very large or very deep trees.
- `MPTTModel` manages structural fields itself. Manual edits to `lft`, `rght`, `tree_id`, or `level` are a corruption risk.

## Version-Sensitive Notes For 0.18.0

This entry tracks the version used here `0.18.0`, which is also the current PyPI release as of March 12, 2026.

Relevant upstream changes around this release:

- `0.18` fixed how indexes are defined for Django 5 and newer.
- `0.17` added support for Python `3.13` and Django `5.1` and `5.2`.
- `0.17` dropped Django versions earlier than `4.2`.
- `0.16` fixed `get_cached_trees()` for multi-tree querysets and fixed `rebuild()` for custom managers not named `objects`.

If you are upgrading from `0.16.x` or older, recheck your Django support floor and any custom index expectations on Django 5+ projects.

## Official Sources

- Docs root: https://django-mptt.readthedocs.io/en/latest/
- Installation: https://django-mptt.readthedocs.io/en/latest/install.html
- Tutorial: https://django-mptt.readthedocs.io/en/latest/tutorial.html
- Models and managers: https://django-mptt.readthedocs.io/en/latest/models.html
- Forms: https://django-mptt.readthedocs.io/en/latest/forms.html
- Admin: https://django-mptt.readthedocs.io/en/latest/admin.html
- Templates: https://django-mptt.readthedocs.io/en/latest/templates.html
- Utilities: https://django-mptt.readthedocs.io/en/latest/utilities.html
- Testing: https://django-mptt.readthedocs.io/en/latest/testing.html
- Changelog: https://django-mptt.readthedocs.io/en/latest/changelog.html
- PyPI: https://pypi.org/project/django-mptt/
- Repository: https://github.com/django-mptt/django-mptt
