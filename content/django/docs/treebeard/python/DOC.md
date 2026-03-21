---
name: treebeard
description: "django-treebeard tree models for Django, including materialized-path, adjacency-list, nested-set, and experimental PostgreSQL ltree implementations"
metadata:
  languages: "python"
  versions: "5.0.5"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,django-treebeard,trees,orm,models,admin"
---

# django-treebeard 5.0.5 Python Package Guide

## Golden Rule

Use treebeard's node APIs instead of editing tree bookkeeping fields yourself. For new projects, start with `MP_Node` unless you have a clear reason to prefer `AL_Node`, `NS_Node`, or the experimental PostgreSQL `LT_Node`.

This entry targets `django-treebeard==5.0.5`. The official docs root still renders a `4.7` title, but its installation page and changelog reflect the current `5.x` support matrix.

## Install

```bash
pip install "django-treebeard==5.0.5"
```

Current official prerequisites for `5.0.5`:

- Python `3.10+`
- Django `5.2+`
- Supported backends called out on PyPI: PostgreSQL, MySQL, MSSQL, SQLite

If the project is pinned to Django `4.2` or `5.1`, do not upgrade blindly. The `5.0.0` release dropped support for both.

## Minimal Setup

Add `treebeard` to `INSTALLED_APPS`:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "treebeard",
]
```

Define a tree model. `MP_Node` is the safest default:

```python
from django.db import models
from treebeard.mp_tree import MP_Node

class Category(MP_Node):
    name = models.CharField(max_length=100)
    slug = models.SlugField(unique=True)

    node_order_by = ["name"]

    def __str__(self) -> str:
        return self.name
```

Create migrations normally:

```bash
python manage.py makemigrations
python manage.py migrate
```

## Core Usage

Create nodes with treebeard helpers:

```python
root = Category.add_root(name="Books", slug="books")
fiction = root.add_child(name="Fiction", slug="fiction")
history = root.add_child(name="History", slug="history")
scifi = fiction.add_child(name="Sci-Fi", slug="sci-fi")
```

Read relationships from the model API:

```python
root = Category.get_root_nodes().get(slug="books")
descendants = root.get_descendants()
ancestors = scifi.get_ancestors()
subtree = Category.get_tree(parent=root)
depth = scifi.get_depth()
```

Move nodes with the provided API instead of editing parent/path fields:

```python
history.move(fiction, pos="sorted-child")
```

If `node_order_by` is set, ordering is enforced from those fields during insert and move operations. It takes precedence over manual drag-and-drop ordering in admin.

## Admin Integration

Use treebeard's admin classes when you want safe move controls in Django admin:

```python
from django.contrib import admin
from treebeard.admin import TreeAdmin
from treebeard.forms import movenodeform_factory

from .models import Category

@admin.register(Category)
class CategoryAdmin(TreeAdmin):
    form = movenodeform_factory(Category)
```

Practical notes:

- `MP_Node` and `NS_Node` get the richer AJAX admin interface.
- `AL_Node` gets a more basic admin interface.
- `5.0.0` renamed internal `MoveNodeForm` fields from `_position` and `_ref_node_id` to `treebeard_position` and `treebeard_ref_node`. Update custom tests, form posts, or wrappers that referenced the old field names directly.

## Bulk Import And Repair

Use the bulk helpers for seed data or controlled migrations:

```python
payload = Category.dump_bulk()

Category.load_bulk(
    [
        {
            "data": {"name": "Books", "slug": "books"},
            "children": [
                {"data": {"name": "Fiction", "slug": "fiction"}},
            ],
        }
    ]
)
```

For diagnostics and repair:

```python
problems = Category.find_problems()
Category.fix_tree()
Category.fix_tree(fix_paths=True, parent=root)
```

Use `fix_tree()` as a repair tool, not part of normal writes. With `fix_paths=False`, treebeard only repairs safer metadata like `depth` and `numchild`; `fix_paths=True` is slower and intended for deeper cleanup.

## Configuration And Auth

`django-treebeard` is a model library, not a service client:

- No package-level authentication is required.
- Configuration is model-level: choose the node base class, optional `node_order_by`, and any custom admin/forms integration.
- `LT_Node` is PostgreSQL-only and experimental because it depends on the `ltree` extension.

Important materialized-path settings:

- The default `steplen` of `4` allows up to `1,679,615` children per node.
- Increasing `steplen` increases child capacity but reduces maximum depth unless you also increase `path.max_length`.
- Do not change `steplen`, `alphabet`, or `node_order_by` after you have saved the first object unless you are doing a controlled dump/reload migration.

## Common Pitfalls

- Do not edit internal fields like `path`, `depth`, or `numchild` directly.
- If you override the default manager for an `MP_Node` model, subclass `MP_NodeManager`; if you change the queryset handler, subclass `MP_NodeQuerySet` too.
- If you keep sibling or related nodes in memory across add/move operations, reload them with `refresh_from_db()` before trusting their fields.
- `node_order_by` fields should have stable values before insert or move. Auto-populated or null values can produce surprising ordering.
- Older code using `MP_Node.fix_tree(destructive=...)` breaks on `5.x`; use `fix_tree(fix_paths=...)`.
- Wrap multi-step business logic in transactions when consistency matters, even though `5.x` moved internal writes into transactions and added more locking around concurrent inserts.

## Choosing A Tree Type

- `MP_Node`: best default for most Django projects.
- `AL_Node`: simplest schema when you prefer a parent-pointer model and can accept more traversal work.
- `NS_Node`: useful for read-heavy trees when more expensive writes are acceptable.
- `LT_Node`: experimental PostgreSQL-only option when `ltree` is already part of your database strategy.

## Version-Sensitive Notes

- `5.0.5` was released on `2026-02-19` and reverted root-node locking added in `5.0.3` because of performance implications when adding new roots.
- `5.0.4` fixed a `TypeError` when adding MP or LT root nodes with `node_order_by` set.
- `5.0.3` added row locks around concurrent `add_child()` and root-creation paths to reduce race conditions.
- `5.0.2` refreshes moved `MP` and `NS` nodes from the database automatically, but other affected in-memory objects may still need `refresh_from_db()`.
- `5.0.0` was a major release: raw SQL tree operations were rewritten to Django ORM, experimental `LT_Node` support was added, Django `6.0` and Python `3.14` support were added, Django `4.2` and `5.1` support were dropped, `MoveNodeForm` internals were renamed, and `MP_Node.fix_tree()` no longer accepts the old `destructive` argument.
- The official docs site is slightly misleading for version discovery: `/en/latest/` still shows a `4.7 documentation` title even though the installation page and changelog contain current `5.x` guidance.

## Official Sources

- Docs root: https://django-treebeard.readthedocs.io/en/latest/
- Installation: https://django-treebeard.readthedocs.io/en/latest/install.html
- Tutorial: https://django-treebeard.readthedocs.io/en/latest/tutorial.html
- Materialized path API: https://django-treebeard.readthedocs.io/en/latest/mp_tree.html
- Caveats: https://django-treebeard.readthedocs.io/en/latest/caveats.html
- Admin: https://django-treebeard.readthedocs.io/en/latest/admin.html
- Changelog: https://django-treebeard.readthedocs.io/en/latest/changes.html
- PyPI release page: https://pypi.org/project/django-treebeard/5.0.5/
