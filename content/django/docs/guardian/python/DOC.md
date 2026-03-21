---
name: guardian
description: "django-guardian object-level permissions for Django projects"
metadata:
  languages: "python"
  versions: "3.3.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,django-guardian,permissions,auth,acl,object-permissions,admin"
---

# django-guardian 3.3.0 Python Package Guide

## Golden Rule

Use `django-guardian` only as an extension to Django's normal auth system, not a replacement for it.

Keep Django's default backend enabled and add Guardian's object-permission backend alongside it:

```python
AUTHENTICATION_BACKENDS = (
    "django.contrib.auth.backends.ModelBackend",
    "guardian.backends.ObjectPermissionBackend",
)
```

If you remove `ModelBackend`, normal Django auth and admin permission behavior will break.

## Install

Pin the package when you need behavior that matches this entry exactly:

```bash
pip install django-guardian==3.3.0
```

If you use `uv`:

```bash
uv add django-guardian==3.3.0
```

Upstream install docs say Guardian requires Django `3.2+`. The docs landing page says the current docs support Python `3.9+` and Django `4.2+`, while current PyPI metadata for `3.3.0` says Python `>=3.10`. Treat Python/Django compatibility as version-sensitive and verify against your deployed stack before pinning.

## Minimal Setup

Add Guardian to installed apps, keep the default backend, and run migrations:

```python
# settings.py
INSTALLED_APPS = [
    # Django apps
    "django.contrib.admin",
    "django.contrib.auth",
    "django.contrib.contenttypes",
    "django.contrib.sessions",
    # Third-party apps
    "guardian",
]

AUTHENTICATION_BACKENDS = (
    "django.contrib.auth.backends.ModelBackend",
    "guardian.backends.ObjectPermissionBackend",
)
```

```bash
python manage.py migrate
```

Guardian creates its object-permission tables and also creates a database-backed anonymous Guardian user unless you disable that behavior.

If your app does not use anonymous object permissions, disable them up front:

```python
ANONYMOUS_USER_NAME = None
```

If you do use anonymous object permissions but want a different username for the database row:

```python
ANONYMOUS_USER_NAME = "public"
```

## Core Usage

Assume a model like:

```python
from django.db import models

class Project(models.Model):
    name = models.CharField(max_length=200)
```

### Assign a permission to a user

```python
from guardian.shortcuts import assign_perm

project = Project.objects.get(pk=1)
assign_perm("projects.change_project", request.user, project)
```

Using only the codename is also valid when Guardian can infer the model:

```python
assign_perm("change_project", request.user, project)
```

### Check a permission

```python
if request.user.has_perm("projects.change_project", project):
    ...
```

Without the object argument, Django performs a global permission check instead:

```python
request.user.has_perm("projects.change_project")
```

### Remove a permission

```python
from guardian.shortcuts import remove_perm

remove_perm("projects.change_project", request.user, project)
```

### Assign permissions to a group

```python
from django.contrib.auth.models import Group
from guardian.shortcuts import assign_perm

editors = Group.objects.get(name="editors")
assign_perm("projects.view_project", editors, project)
```

### Query only objects a user can access

Use `get_objects_for_user` instead of iterating a queryset and calling `has_perm()` repeatedly:

```python
from guardian.shortcuts import get_objects_for_user

projects = get_objects_for_user(
    request.user,
    "projects.view_project",
    klass=Project,
)
```

You can also pass multiple permissions and control whether group-derived permissions are included.

### Inspect effective permissions

```python
from guardian.shortcuts import get_perms

perms = get_perms(request.user, project)
if "change_project" in perms:
    ...
```

## View and API Enforcement

For one-off checks, plain `has_perm(..., obj)` is usually the cleanest option.

For repeated checks in class-based views, use Guardian's mixin:

```python
from django.views.generic import DetailView
from guardian.mixins import PermissionRequiredMixin

class ProjectDetailView(PermissionRequiredMixin, DetailView):
    model = Project
    permission_required = "projects.view_project"
    raise_exception = True
```

`PermissionRequiredMixin` checks against `self.object` or `get_object()` when available. If `raise_exception = True`, it raises `PermissionDenied`; otherwise it follows its normal forbidden handling flow.

Guardian also ships decorators such as `permission_required_or_403`, but upstream warns that decorator-based object lookup can add extra database queries if the view does similar lookups again.

## Admin Integration

If object permissions should be manageable in Django admin, replace `ModelAdmin` with `GuardedModelAdmin`:

```python
from django.contrib import admin
from guardian.admin import GuardedModelAdmin

from .models import Project

@admin.register(Project)
class ProjectAdmin(GuardedModelAdmin):
    pass
```

This adds Guardian's object-permission UI to the model's admin change page.

## Performance and Scaling

Guardian's default models use generic foreign keys. Upstream docs say that is flexible and usually good enough, but it can become a bottleneck on large datasets or hot permission paths.

### Use cached permission checks in loops

If you need many checks for the same user across many objects, use `ObjectPermissionChecker` and prefetch:

```python
from guardian.core import ObjectPermissionChecker

projects = Project.objects.all()
checker = ObjectPermissionChecker(request.user)
checker.prefetch_perms(projects)

for project in projects:
    if checker.has_perm("change_project", project):
        ...
```

### Consider direct foreign-key permission models for hot paths

For high-volume permission tables, Guardian supports direct FK models:

```python
from django.db import models
from guardian.models import GroupObjectPermissionBase, UserObjectPermissionBase

class ProjectUserObjectPermission(UserObjectPermissionBase):
    content_object = models.ForeignKey(Project, on_delete=models.CASCADE)

class ProjectGroupObjectPermission(GroupObjectPermissionBase):
    content_object = models.ForeignKey(Project, on_delete=models.CASCADE)
```

The field name must be exactly `content_object` or Guardian's direct-relation queries will not detect it correctly.

If you introduce these models into an existing project, plan the migration carefully. Upstream docs note that you can temporarily disable direct-model detection with `enabled = False` while creating tables and migrating data.

## Custom User and Group Models

Guardian depends on the user-to-group relationship shape that Django auth expects. If you use a custom user model and Guardian imports create problems in the same app's `models.py`, upstream recommends:

```python
GUARDIAN_MONKEY_PATCH_USER = False
```

and subclassing `guardian.mixins.GuardianUserMixin` in the custom user model.

If your custom user model cannot create Guardian's anonymous user with the default initializer, point `GUARDIAN_GET_INIT_ANONYMOUS_USER` at your own factory.

For custom group models, Guardian provides corresponding group mixins and a custom object-permission model path. Do not directly import Guardian's default group object-permission model in that setup; use the helper accessors from `guardian.utils`.

## Common Pitfalls

- Forgetting `"guardian"` in `INSTALLED_APPS` or forgetting `python manage.py migrate`.
- Replacing Django's `ModelBackend` instead of adding Guardian alongside it.
- Assigning object permissions and then checking them without passing the object to `has_perm()`.
- Looping over objects with `has_perm()` instead of using `get_objects_for_user()` or `ObjectPermissionChecker.prefetch_perms()`.
- Expecting Django's anonymous user object to be the same as Guardian's database-backed anonymous user.
- Enabling direct-FK permission models but naming the relation field something other than `content_object`.
- Introducing custom user/group permission models after project startup without planning migrations; upstream settings for custom permission models are meant to be set at the start of a project.

## Version-Sensitive Notes

- This entry targets `django-guardian==3.3.0`.
- PyPI lists `3.3.0` as the current release, published on `2026-02-24`.
- The docs URL points to `https://django-guardian.readthedocs.io/en/stable/`, which was appropriate on `2026-03-12` because it reflects the current release series. It is still a floating alias, so re-check it before assuming it still matches `3.3.0` later.
- Upstream compatibility signals are not fully aligned across surfaces:
  - docs home says Python `3.9+` and Django `4.2+`
  - install page says Django `3.2+`
  - PyPI metadata for `3.3.0` says Python `>=3.10` and classifiers include Django `3.2` through `5.2`
- For pinned production work, trust the package metadata and your tested project matrix over older third-party examples.

## Official Context

- Documentation root: https://django-guardian.readthedocs.io/en/stable/
- Installation: https://django-guardian.readthedocs.io/en/stable/installation/
- Configuration: https://django-guardian.readthedocs.io/en/latest/configuration/
- Permission checks: https://django-guardian.readthedocs.io/en/stable/userguide/checks/
- Admin integration: https://django-guardian.readthedocs.io/en/stable/api/admin/
- Custom user model: https://django-guardian.readthedocs.io/en/3.0.6/userguide/custom-user-model/
- PyPI: https://pypi.org/project/django-guardian/
- Repository: https://github.com/django-guardian/django-guardian
