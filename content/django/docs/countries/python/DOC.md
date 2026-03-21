---
name: countries
description: "django-countries package guide for Django country fields, forms, admin filters, and API serialization"
metadata:
  languages: "python"
  versions: "8.2.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django-countries,django,countries,forms,flags,drf,django-filter,localization"
---

# django-countries Python Package Guide

## What It Does

`django-countries` adds an ISO 3166-1 country field for Django models, translated country names, flag assets, form widgets, admin filters, template tags, and serializer helpers.

- Package: `django-countries`
- Django app: `django_countries`
- Main model import: `from django_countries.fields import CountryField`
- Version covered here: `8.2.0`
- Docs root: `https://smileychris.github.io/django-countries/`
- Registry: `https://pypi.org/project/django-countries/`

As of March 12, 2026, PyPI lists `8.2.0` as the latest release, published on November 24, 2025. The docs site is a latest-version docs site, so check the changelog before assuming newer features exist in older pins.

## Install And Compatibility

```bash
python -m pip install "django-countries==8.2.0"
```

Common alternatives:

```bash
uv add "django-countries==8.2.0"
poetry add "django-countries==8.2.0"
```

For better sorting of translated country names:

```bash
python -m pip install "django-countries[pyuca]==8.2.0"
```

Current upstream support signals:

- Python: `3.8` to `3.13`
- Django docs support table: `3.2`, `4.2`, `5.0`, `5.1`, `5.2`
- Django REST Framework: `3.11+`

PyPI classifiers still list Django `4.0` and `4.1`, but the docs support table does not. If your project is pinned to Django `4.0` or `4.1`, verify in CI instead of assuming active support.

## Minimal Django Setup

Add the app:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "django_countries",
]
```

Create a model field:

```python
from django.db import models
from django_countries.fields import CountryField

class Person(models.Model):
    name = models.CharField(max_length=100)
    country = CountryField()
```

Apply migrations:

```bash
python manage.py makemigrations
python manage.py migrate
```

Quick verification:

```python
from django_countries import countries

assert countries["NZ"] == "New Zealand"
```

## Core Model Usage

`CountryField` is a `CharField` underneath, with a default `max_length` of `2`. The database stores country codes, but model instances expose a richer `Country` object.

```python
person = Person.objects.create(name="Chris", country="NZ")

assert person.country.code == "NZ"
assert person.country.name == "New Zealand"
assert person.country.alpha3 == "NZL"
assert person.country.numeric == 554
assert person.country.flag.endswith("nz.gif")
```

Useful properties on the `Country` object include:

- `.code`
- `.name`
- `.flag`
- `.flag_css`
- `.unicode_flag`
- `.alpha3`
- `.numeric`
- `.ioc_code`

If you need the country list outside models:

```python
from django_countries import countries

country_map = dict(countries)
country_name = country_map["BR"]
```

## Querying

Store and compare ISO codes directly when you can:

```python
Person.objects.filter(country="NZ")
```

For country-name lookup, use the dedicated lookups:

```python
Person.objects.filter(country__name="New Zealand")
Person.objects.filter(country__iname="new zealand")
Person.objects.filter(country__icontains="zealand")
```

Do not rely on `country="New Zealand"` exact matches. Current upstream guidance is to use `__name` or `__iname` for exact name filters.

## Forms, Templates, And Admin

For custom forms, use the package form field so translated names stay lazy until render time:

```python
from django import forms
from django_countries.fields import CountryField

class ProfileForm(forms.Form):
    country = CountryField().formfield()
```

For blank values:

```python
country = CountryField(blank=True, blank_label="(Select country)").formfield()
```

To show a flag beside the select:

```python
from django import forms
from django_countries.widgets import CountrySelectWidget

class PersonForm(forms.ModelForm):
    class Meta:
        model = Person
        fields = ("name", "country")
        widgets = {"country": CountrySelectWidget()}
```

In templates:

```django
{% load countries %}
{% get_country "BR" as country %}
{{ country.name }}
<img src="{{ country.flag }}" alt="{{ country.name }} flag">
```

Or build your own select:

```django
{% load countries %}
{% get_countries as countries %}
<select name="country">
  {% for country in countries %}
    <option value="{{ country.code }}">{{ country.name }}</option>
  {% endfor %}
</select>
```

Admin support works out of the box:

```python
from django.contrib import admin

@admin.register(Person)
class PersonAdmin(admin.ModelAdmin):
    list_display = ["name", "country"]
    search_fields = ["name", "country"]
```

For admin list filtering, use the package filter:

```python
from django.contrib import admin
from django_countries.filters import CountryFilter

@admin.register(Person)
class PersonAdmin(admin.ModelAdmin):
    list_filter = [("country", CountryFilter)]
```

In `8.2.0`, `CountryFilter` also supports filtering through relations such as `("contact__country", CountryFilter)`.

## Configuration

Common global settings:

```python
from django.utils.translation import gettext_lazy as _

COUNTRIES_COMMON_NAMES = True
COUNTRIES_ONLY = ["US", "CA", "MX"]
COUNTRIES_FIRST = ["US", "CA"]
COUNTRIES_FIRST_BREAK = "----------"
COUNTRIES_FIRST_BY_LANGUAGE = {
    "fr": ["FR", "CH", "BE", "LU"],
    "de": ["DE", "AT", "CH", "LI"],
}
COUNTRIES_FIRST_AUTO_DETECT = True
COUNTRIES_OVERRIDE = {
    "NZ": _("Middle Earth"),
    "AU": None,
    "IND": {
        "names": [_("Indonesia")],
        "ioc_code": "INA",
        "flag_url": "flags/id.gif",
    },
}
COUNTRIES_FLAG_URL = "flags/16x10/{code_upper}.png"
```

What each setting is for:

- `COUNTRIES_COMMON_NAMES`: use friendlier common names instead of strict ISO names
- `COUNTRIES_ONLY`: restrict the available list
- `COUNTRIES_FIRST`: pin specific countries to the top
- `COUNTRIES_FIRST_BREAK`: add a separator after pinned countries
- `COUNTRIES_FIRST_BY_LANGUAGE`: reorder top countries by active language
- `COUNTRIES_FIRST_AUTO_DETECT`: prepend the locale-detected country when possible
- `COUNTRIES_OVERRIDE`: rename, exclude, or override metadata for specific countries
- `COUNTRIES_FLAG_URL`: change where `country.flag` points

Set `COUNTRIES_COMMON_NAMES = False` if you need official ISO naming instead of the package's common-name defaults.

## Per-Request Or Per-Field Customization

If a single field needs its own country list, pass a custom `Countries` subclass:

```python
from django.db import models
from django.utils.translation import gettext_lazy as _

from django_countries import Countries
from django_countries.fields import CountryField

class G8Countries(Countries):
    only = [
        "CA",
        "FR",
        "DE",
        "IT",
        "JP",
        "GB",
        ("EU", _("European Union")),
    ]

class Vote(models.Model):
    country = CountryField(countries=G8Countries)
```

In `8.2.0`, you can also apply temporary thread-local overrides with `countries_context()`:

```python
from django.shortcuts import render
from django_countries import countries_context

def checkout_view(request):
    with countries_context(first=["US", "CA"], first_by_language={}):
        form = ProfileForm()
    return render(request, "checkout.html", {"form": form})
```

`countries_context()` is useful when the country order or allowed list depends on request state, locale, user preference, or geolocation.

## Django REST Framework

For model serializers, use `CountryFieldMixin`:

```python
from django_countries.serializers import CountryFieldMixin
from rest_framework import serializers

class PersonSerializer(CountryFieldMixin, serializers.ModelSerializer):
    class Meta:
        model = Person
        fields = ("name", "country")
        extra_kwargs = {
            "country": {"country_dict": ("name", "alpha3")},
        }
```

In `8.2.0`, the mixin accepts `name_only`, `country_dict`, and related output options through `Meta.extra_kwargs`, so you do not need to redefine the field for common serializer customization.

For explicit serializer fields:

```python
from django_countries.serializer_fields import CountryField
from rest_framework import serializers

class CountrySerializer(serializers.Serializer):
    country = CountryField(country_dict=("code", "name", "alpha3"))
```

Supported `country_dict` keys documented upstream:

- `code`
- `name`
- `alpha3`
- `numeric`
- `unicode_flag`
- `ioc_code`

`name_only=True` returns only the country name. The serializer accepts either a code string or a country dictionary as input.

## django-filter Integration

`8.2.0` adds a package helper for `django-filter`:

```python
import django_filters
from django_countries.django_filters import CountryFilter

class PersonFilterSet(django_filters.FilterSet):
    country = CountryFilter(empty_label="Any country")

    class Meta:
        model = Person
        fields = ["country"]
```

This filter populates its choices from `django_countries.countries`, so you do not have to maintain a duplicated country choice list.

## Multiple-Country Fields

For multi-select storage:

```python
from django.db import models
from django_countries.fields import CountryField

class Incident(models.Model):
    title = models.CharField(max_length=100)
    countries = CountryField(multiple=True)
```

Behavior:

- Stored in the database as a comma-separated string
- Exposed in Python as a list of `Country` objects
- Sorted and deduplicated by default

If you need different behavior:

```python
countries = CountryField(
    multiple=True,
    multiple_sort=False,
    multiple_unique=False,
    null=True,
)
```

`null=True` for `multiple=True` is an `8.1.0` feature. On older versions, do not assume that combination works.

## Common Pitfalls

- Forgetting `"django_countries"` in `INSTALLED_APPS`. The field may import, but templates, widgets, and static flag assets will not be wired correctly.
- Treating `instance.country` as always being a raw string. On model instances it is a `Country` object; use `.code` if you need the stored alpha-2 code.
- Building country choices manually in forms. Prefer `CountryField().formfield()` or `CountrySelectWidget()` so translations, blank labels, and widgets stay aligned with package behavior.
- Forgetting static file handling for flags. If you use `country.flag` or `CountrySelectWidget`, make sure `collectstatic` and `STATIC_URL` or `COUNTRIES_FLAG_URL` match how assets are served.
- Using Django admin `autocomplete_fields`. Upstream docs say `CountryField` does not support it and falls back to a regular select widget.
- Mixing up the two filter helpers. Django admin uses `django_countries.filters.CountryFilter`; `django-filter` integration uses `django_countries.django_filters.CountryFilter`.
- Assuming exact name filters use the plain field. Use `country__name` or `country__iname`, not `country="New Zealand"`.
- Assuming third-party admin filter packages will work with `CountryField`. Upstream recommends the built-in admin `CountryFilter`.

## Version-Sensitive Notes For `8.2.0`

- PyPI lists `8.2.0` as latest on March 12, 2026, with a release date of November 24, 2025. The changelog entry is dated November 25, 2025.
- `8.2.0` adds `COUNTRIES_FIRST_BY_LANGUAGE`, `COUNTRIES_FIRST_AUTO_DETECT`, `countries_context()`, `COUNTRIES_OVERRIDE["..."]["flag_url"]`, `django_countries.django_filters.CountryFilter`, related-field admin filtering, and `CountryFieldMixin` `Meta.extra_kwargs` customization.
- `8.2.0` also fixes `CountryField.formfield(empty_label=...)`, admin filtering for `multiple=True`, related admin filtering, and localized name deserialization in serializer fields.
- `8.1.0` adds `null=True` support for `CountryField(multiple=True)` and improves OpenAPI generation for `country_dict` and `name_only`.
- `8.1.1` fixes selected-option rendering for `CountryField(multiple=True)` in Django forms.
- `8.0.0` rolled in the previously yanked `7.8`, `7.9`, and `7.9.1` changes and dropped Python `3.7`. Do not pin to those yanked `7.x` releases.
- The docs site is not version-frozen. If your project is pinned below `8.2.0`, verify against the changelog before copying examples that use dynamic ordering, `countries_context()`, or the new filter helpers.

## Official Sources Used

- Docs root: `https://smileychris.github.io/django-countries/`
- Installation: `https://smileychris.github.io/django-countries/installation/`
- CountryField reference: `https://smileychris.github.io/django-countries/usage/field/`
- Forms and widgets: `https://smileychris.github.io/django-countries/usage/forms/`
- Settings: `https://smileychris.github.io/django-countries/usage/settings/`
- Django REST Framework integration: `https://smileychris.github.io/django-countries/integrations/drf/`
- django-filter integration: `https://smileychris.github.io/django-countries/integrations/django_filters/`
- Customization and dynamic ordering: `https://smileychris.github.io/django-countries/advanced/customization/`
- Multiple countries: `https://smileychris.github.io/django-countries/advanced/multiple/`
- Changelog: `https://smileychris.github.io/django-countries/changelog/`
- Repository: `https://github.com/SmileyChris/django-countries`
- PyPI package page: `https://pypi.org/project/django-countries/`
