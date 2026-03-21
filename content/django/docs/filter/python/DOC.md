---
name: filter
description: "django-filter package guide for QuerySet filtering in Django and Django REST Framework"
metadata:
  languages: "python"
  versions: "25.2"
  revision: 2
  updated-on: "2026-03-11"
  source: maintainer
  tags: "django-filter,django,drf,queryset,filters,forms,api"
---

# django-filter Python Package Guide

## What It Is

`django-filter` adds declarative filtering for Django `QuerySet`s. The main abstraction is a `FilterSet`: define filter fields once, bind request query params to it, and read the filtered queryset from `.qs`.

Use it for:

- server-rendered Django list pages
- reusable filtering logic outside view code
- Django REST Framework list endpoints via `DjangoFilterBackend`

Package identity:

- PyPI package: `django-filter`
- Import namespace and app label: `django_filters`
- Version covered: `25.2`
- Release date: `2025-10-05`
- Python requirement: `>=3.10`
- `25.2` support floor: Django `5.2+`

## Installation

Install the version your project is pinned to:

```bash
pip install django-filter==25.2
```

```bash
uv add django-filter==25.2
```

```bash
poetry add django-filter==25.2
```

## Setup And Initialization

Add `django_filters` to `INSTALLED_APPS`:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "django_filters",
]
```

When using Django REST Framework, enable the filter backend:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "rest_framework",
    "django_filters",
]

REST_FRAMEWORK = {
    "DEFAULT_FILTER_BACKENDS": [
        "django_filters.rest_framework.DjangoFilterBackend",
    ],
}
```

Core workflow:

1. Define a `FilterSet`.
2. Bind it to `request.GET` plus a base queryset.
3. Use `.qs` as the filtered queryset.

## Core Usage

### Define A `FilterSet`

```python
import django_filters

from .models import Product

class ProductFilter(django_filters.FilterSet):
    min_price = django_filters.NumberFilter(field_name="price", lookup_expr="gte")
    max_price = django_filters.NumberFilter(field_name="price", lookup_expr="lte")
    released_after = django_filters.DateFilter(
        field_name="release_date",
        lookup_expr="gte",
    )
    uncategorized = django_filters.BooleanFilter(
        field_name="category",
        lookup_expr="isnull",
    )

    class Meta:
        model = Product
        fields = {
            "name": ["exact", "icontains"],
            "category": ["exact"],
            "in_stock": ["exact"],
        }
```

`Meta.fields` can be either:

- a list of model field names for simple exact filters
- a dict of model field names to allowed lookup expressions

### Bind It In A Django View

```python
from django.shortcuts import render

from .filters import ProductFilter
from .models import Product

def product_list(request):
    product_filter = ProductFilter(
        request.GET,
        queryset=Product.objects.all(),
        request=request,
    )
    return render(
        request,
        "products/list.html",
        {
            "filter": product_filter,
            "object_list": product_filter.qs,
        },
    )
```

Use `product_filter.form` to render the bound form in templates, and always query through `product_filter.qs`.

### Use `FilterView` For Generic List Pages

```python
from django_filters.views import FilterView

from .filters import ProductFilter
from .models import Product

class ProductListView(FilterView):
    model = Product
    filterset_class = ProductFilter
    template_name = "products/list.html"
```

### Integrate With Django REST Framework

Use the DRF integration module, not the base package imports:

```python
from django_filters import rest_framework as filters
from rest_framework import generics

from .models import Product
from .serializers import ProductSerializer

class ProductAPIFilter(filters.FilterSet):
    min_price = filters.NumberFilter(field_name="price", lookup_expr="gte")
    max_price = filters.NumberFilter(field_name="price", lookup_expr="lte")

    class Meta:
        model = Product
        fields = ["category", "in_stock"]

class ProductListAPI(generics.ListAPIView):
    queryset = Product.objects.all()
    serializer_class = ProductSerializer
    filter_backends = [filters.DjangoFilterBackend]
    filterset_class = ProductAPIFilter
```

For simple equality filters, `filterset_fields` is enough:

```python
from django_filters.rest_framework import DjangoFilterBackend
from rest_framework import generics

class ProductListAPI(generics.ListAPIView):
    queryset = Product.objects.all()
    serializer_class = ProductSerializer
    filter_backends = [DjangoFilterBackend]
    filterset_fields = ["category", "in_stock"]
```

DRF-specific behavior to remember:

- `BooleanFilter` uses an API-friendly `BooleanWidget` that accepts lowercase `true` and `false`
- datetime model fields generate `IsoDateTimeFilter`
- raised `django.core.exceptions.ValidationError` values are reraised as DRF validation errors

## Configuration

### FilterSet Options

Use `Meta` for generation policy:

- `model` and `fields` for autogenerated filters
- `exclude` to block autogenerated filters for specific model fields
- `form` to supply a custom base form class
- `filter_overrides` to change default filter generation for field types
- `unknown_field_behavior` to control what happens when unknown model fields are referenced

Example:

```python
from django.db import models
from django_filters import CharFilter, FilterSet, UnknownFieldBehavior

class ProductFilter(FilterSet):
    class Meta:
        model = Product
        fields = ["name", "release_date"]
        unknown_field_behavior = UnknownFieldBehavior.WARN
        filter_overrides = {
            models.CharField: {
                "filter_class": CharFilter,
                "extra": lambda field: {"lookup_expr": "icontains"},
            },
        }
```

### Package Settings

`django-filter` settings are prefixed with `FILTERS_`. The most useful global settings are:

- `FILTERS_DEFAULT_LOOKUP_EXPR` to change the implicit lookup from `exact`
- `FILTERS_VERBOSE_LOOKUPS` to control human-readable lookup labels
- `FILTERS_DISABLE_HELP_TEXT` to suppress generated help text such as `in` and `range` CSV hints
- `FILTERS_EMPTY_CHOICE_LABEL`, `FILTERS_NULL_CHOICE_LABEL`, and `FILTERS_NULL_CHOICE_VALUE` for form choice behavior

Example:

```python
# settings.py
FILTERS_DEFAULT_LOOKUP_EXPR = "exact"
FILTERS_VERBOSE_LOOKUPS = True
FILTERS_DISABLE_HELP_TEXT = False
```

## Auth And Request Context

`django-filter` does not manage authentication, sessions, or permissions. Those still belong to Django or DRF.

What it does control is queryset narrowing for the current request. If a filter depends on the request object:

- pass `request=request` when instantiating the filterset
- handle `self.request is None`, because upstream documents that `request` is optional

Request-aware patterns supported by the docs include:

- overriding `FilterSet.qs`
- callable querysets for `ModelChoiceFilter` and `ModelMultipleChoiceFilter`
- custom filter methods that inspect request data

## Common Pitfalls

### Package Name And Import Path Differ

Install `django-filter`, but import and configure `django_filters`.

### Always Set `field_name` And `lookup_expr` Explicitly For Custom Filters

This looks right but is wrong:

```python
class ProductFilter(django_filters.FilterSet):
    price__gt = django_filters.NumberFilter()
```

That resolves to `price__gt__exact`. Write it explicitly:

```python
class ProductFilter(django_filters.FilterSet):
    price__gt = django_filters.NumberFilter(
        field_name="price",
        lookup_expr="gt",
    )
```

### Match The Filter Type To The Lookup

`isnull` expects a boolean value even if the model field is an integer foreign key:

```python
class ProductFilter(django_filters.FilterSet):
    uncategorized = django_filters.BooleanFilter(
        field_name="category",
        lookup_expr="isnull",
    )
```

### `Meta.fields` Dict Keys Must Be Model Field Names

Declarative aliases such as `min_price` belong on the class body, not inside the `Meta.fields` dict.

### Empty Strings Are Skipped By Default

An empty query parameter is treated as "no filter". If you need to filter for `""`, implement a custom method or a documented sentinel convention.

### Do Not Mix `filterset_fields` And `filterset_class`

For DRF and generic views, use one or the other. Reach for `filterset_class` as soon as you need non-default lookups, labels, widgets, or request-aware filtering.

### Always Read From `.qs`

Current versions do not proxy the queryset API from the `FilterSet` itself. Use `.qs` explicitly instead of assuming the filterset behaves like a queryset.

## Version-Sensitive Notes For 25.2

- `25.2` adds testing for Django `6.0`.
- `25.2` drops support for Django versions lower than `5.2`.
- `25.2` drops support for Python `3.9`; the package metadata now requires Python `>=3.10`.
- `25.1` removes the deprecated built-in schema-generation hooks for DRF views. If your project still relies on those methods, move schema generation to `drf-spectacular` or another current OpenAPI tool before upgrading.
- The project uses CalVer-style package versions such as `25.2`, not SemVer.

## Official Sources

- Stable docs root: https://django-filter.readthedocs.io/en/stable/
- Installation guide: https://django-filter.readthedocs.io/en/stable/guide/install.html
- Usage guide: https://django-filter.readthedocs.io/en/stable/guide/usage.html
- Tips guide: https://django-filter.readthedocs.io/en/stable/guide/tips.html
- DRF integration guide: https://django-filter.readthedocs.io/en/stable/guide/rest_framework.html
- FilterSet reference: https://django-filter.readthedocs.io/en/stable/ref/filterset.html
- Settings reference: https://django-filter.readthedocs.io/en/stable/ref/settings.html
- Migration guide: https://django-filter.readthedocs.io/en/stable/guide/migration.html
- PyPI package page: https://pypi.org/project/django-filter/25.2/
- Changelog: https://github.com/carltongibson/django-filter/blob/main/CHANGES.rst
