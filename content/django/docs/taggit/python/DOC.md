---
name: taggit
description: "django-taggit tagging library for Django models, forms, admin, and Django REST Framework"
metadata:
  languages: "python"
  versions: "6.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,taggit,tags,orm,admin,drf"
---

# django-taggit Python Package Guide

## Golden Rule

Use `django-taggit` for model tagging in Django projects instead of inventing a custom comma-separated field or ad hoc many-to-many schema. Import from `taggit`, add `taggit` to `INSTALLED_APPS`, run migrations, and use `TaggableManager` on saved model instances.

## Install

Pin the package version your project expects:

```bash
python -m pip install "django-taggit==6.1.0"
```

Common alternatives:

```bash
uv add "django-taggit==6.1.0"
poetry add "django-taggit==6.1.0"
```

Add the app and migrate:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "taggit",
]
```

```bash
python manage.py migrate
```

## Initialize In A Model

`TaggableManager` behaves like a many-to-many field with tag-aware helpers:

```python
from django.db import models
from taggit.managers import TaggableManager

class Article(models.Model):
    title = models.CharField(max_length=200)
    body = models.TextField()
    tags = TaggableManager(blank=True)
```

Create and tag an object:

```python
article = Article.objects.create(title="Intro", body="...")
article.tags.add("django", "orm")
article.tags.add("python", tag_kwargs={"slug": "python"})
```

Read tags:

```python
names = list(article.tags.names())
slugs = list(article.tags.slugs())
all_tags = list(article.tags.all())
```

Replace or clear tags:

```python
article.tags.set(["django", "tutorial"], clear=True)
article.tags.remove("tutorial")
article.tags.clear()
```

## Querying And Core ORM Usage

Filter by a single tag:

```python
Article.objects.filter(tags__name__in=["django"])
Article.objects.filter(tags__slug__in=["django"])
```

When you filter across tag joins, add `distinct()` unless you explicitly want duplicates:

```python
Article.objects.filter(tags__name__in=["django", "python"]).distinct()
```

Useful helpers on the manager:

```python
article.tags.similar_objects()
Article.tags.most_common()  # via the model-level manager
```

Avoid N+1 queries when rendering tag lists:

```python
Article.objects.prefetch_related("tags")
```

## Forms, Admin, And API Serialization

Model forms work with `TaggableManager`, but if you save with `commit=False`, you still need `save_m2m()`:

```python
form = ArticleForm(request.POST)
article = form.save(commit=False)
article.author = request.user
article.save()
form.save_m2m()
```

The admin integration supports tag display and tag merge operations:

```python
from django.contrib import admin
from taggit.admin import TaggitListFilter

from .models import Article

@admin.register(Article)
class ArticleAdmin(admin.ModelAdmin):
    list_display = ["title"]
    list_filter = [TaggitListFilter]
```

For Django REST Framework, use the serializer helpers from `taggit.serializers`:

```python
from rest_framework import serializers
from taggit.serializers import TaggitSerializer, TagListSerializerField

from .models import Article

class ArticleSerializer(TaggitSerializer, serializers.ModelSerializer):
    tags = TagListSerializerField()

    class Meta:
        model = Article
        fields = ["id", "title", "tags"]
```

## Configuration And Customization

There is no package-specific auth layer. Configuration is standard Django setup plus optional taggit settings and model customization.

### Custom parser and renderer

Override how tag strings are parsed from forms and rendered back to strings:

```python
# settings.py
TAGGIT_TAGS_FROM_STRING = "path.to.tags_from_string"
TAGGIT_STRING_FROM_TAGS = "path.to.string_from_tags"
```

This is useful if you need stricter quoting rules or a non-comma input format.

### Unicode slug behavior

`django-taggit` now keeps Unicode characters during slugification by default. If you must preserve the older stripping behavior:

```python
TAGGIT_STRIP_UNICODE_WHEN_SLUGIFYING = True
```

Prefer leaving the default unless you are maintaining older slug compatibility.

### Custom tag or through models

Use a custom tag model when you need extra fields on tags, non-integer primary keys, or a non-generic relation:

```python
from django.db import models
from taggit.managers import TaggableManager
from taggit.models import GenericUUIDTaggedItemBase, TagBase

class MyTag(TagBase):
    description = models.TextField(blank=True)

class TaggedArticle(GenericUUIDTaggedItemBase):
    tag = models.ForeignKey(
        MyTag,
        on_delete=models.CASCADE,
        related_name="tagged_articles",
    )

class Article(models.Model):
    id = models.UUIDField(primary_key=True)
    title = models.CharField(max_length=200)
    tags = TaggableManager(through=TaggedArticle)
```

If you switch to your own tag and through models from the start, follow the upstream guidance and remove `"taggit"` from `INSTALLED_APPS` before your first migration so Django does not create the default taggit tables you will not use.

## Operations And Maintenance

Natural keys are supported for `dumpdata` and `loaddata`, which helps when fixtures move across databases.

`6.1.0` also adds management commands to clean tag data:

```bash
python manage.py deduplicate_tags
python manage.py remove_orphaned_tags
```

The orphan cleanup command accepts `--noinput` for unattended runs.

## Common Pitfalls

- Save the model instance before calling `add()`, `set()`, or `remove()` on `TaggableManager`. Like other many-to-many relations, unsaved instances cannot write tag relations.
- `set()` expects an iterable such as `["django", "python"]`; do not pass multiple positional tag arguments.
- Add `distinct()` when filtering through `tags__...` joins or you may get duplicate rows.
- `TaggableManager(blank=True)` controls form validation only. It does not bypass migrations or model-save ordering constraints.
- If you use `form.save(commit=False)`, call `save_m2m()` after saving the instance or the tags will not persist.
- If you use custom tag models, align `through=` and `tag = ForeignKey(...)` exactly with the upstream base classes or migrations will get messy quickly.

## Version-Sensitive Notes

- The official docs currently describe `django-taggit 6.1.0` as supporting Django `4.1+` and Python `3.9+`.
- The PyPI project metadata for `6.1.0` still says `Requires: Python >=3.8`, and its long description still mentions Django `3.2 or greater`. Treat that as upstream metadata drift, not a reliable support floor.
- `6.1.0` adds `deduplicate_tags` and `remove_orphaned_tags` management commands.
- `6.0.0` changed default ordering for `TaggableManager` to use the tagged item's primary key. Set `ordering=[]` on the manager if you need the pre-6.0 unordered behavior.
- `5.0.0` removed Django `3.2` support and vendored the DRF serializer helpers directly into `taggit.serializers`.
- `2.0.0` changed `TaggableManager.set()` to require an iterable instead of varargs.
- `3.0.0` changed the default slugification behavior to preserve Unicode characters.

## Official Sources

- Docs root: `https://django-taggit.readthedocs.io/en/latest/`
- Getting started: `https://django-taggit.readthedocs.io/en/latest/getting_started.html`
- API reference: `https://django-taggit.readthedocs.io/en/latest/api.html`
- Forms: `https://django-taggit.readthedocs.io/en/latest/forms.html`
- Admin: `https://django-taggit.readthedocs.io/en/latest/admin.html`
- Serializers: `https://django-taggit.readthedocs.io/en/latest/serializers.html`
- Custom tagging: `https://django-taggit.readthedocs.io/en/latest/custom_tagging.html`
- Changelog: `https://django-taggit.readthedocs.io/en/latest/changelog.html`
- PyPI: `https://pypi.org/project/django-taggit/`
