---
name: phonenumber-field
description: "django-phonenumber-field package guide for Django model, form, and DRF phone number handling"
metadata:
  languages: "python"
  versions: "8.4.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django-phonenumber-field,django,phone,phonenumbers,validation,forms,drf"
---

# django-phonenumber-field 8.4.0 Python Package Guide

## What It Does

`django-phonenumber-field` adds Django-aware phone number fields for models, forms, and Django REST Framework serializers on top of Google's libphonenumber data through either `phonenumbers` or `phonenumberslite`.

Use it when a Django project needs validated, normalized phone numbers instead of raw strings. The main things to get right are:

- install a parsing backend extra
- set a default region if users enter national numbers
- choose database and display formats deliberately
- use explicit output formats for APIs instead of relying on `str()`
- preserve extensions only with extension-friendly formats

## Version Covered

- Package: `django-phonenumber-field`
- Django app label: `phonenumber_field`
- Import paths:
  - `phonenumber_field.modelfields.PhoneNumberField`
  - `phonenumber_field.formfields.PhoneNumberField`
  - `phonenumber_field.serializerfields.PhoneNumberField`
- Version in this doc: `8.4.0`
- Supported Python versions on the `8.4.0` PyPI release: `3.10` to `3.14`
- Supported Django versions on the `8.4.0` PyPI release: `4.2`, `5.1`, `5.2`, `6.0`

Version note: as of March 12, 2026, PyPI lists `8.4.0` as the latest release, so the floating docs URL appears aligned with this package version.

## Install

Install the package with one of its phone-number parsing backends:

```bash
python -m pip install "django-phonenumber-field[phonenumberslite]==8.4.0"
```

Use full `phonenumbers` instead of `phonenumberslite` if you need geocoding, carrier, or timezone metadata:

```bash
python -m pip install "django-phonenumber-field[phonenumbers]==8.4.0"
```

Equivalent commands:

```bash
uv add "django-phonenumber-field[phonenumberslite]==8.4.0"
```

```bash
poetry add "django-phonenumber-field[phonenumberslite]==8.4.0"
```

If you use `SplitPhoneNumberField`, install Babel too:

```bash
python -m pip install Babel
```

## Minimal Setup

Add the app to `INSTALLED_APPS`:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "phonenumber_field",
]
```

If users enter local numbers such as `"604 401 1234"` instead of international `+`-prefixed numbers, set a default region:

```python
# settings.py
PHONENUMBER_DEFAULT_REGION = "CA"
```

Without a region, parsing national numbers is ambiguous and validation may fail.

## No Auth

This package has no authentication flow. Runtime behavior is controlled by Django settings, the selected `phonenumbers` extra, and the options you pass to fields and widgets.

## Core Usage

### Model field

```python
from django.db import models
from phonenumber_field.modelfields import PhoneNumberField

class Contact(models.Model):
    name = models.CharField(max_length=100)
    phone_number = PhoneNumberField(blank=True)
```

Prefer saving international input when possible:

```python
contact = Contact.objects.create(
    name="Alice",
    phone_number="+16044011234",
)
```

The stored value becomes a `PhoneNumber` wrapper when loaded from Django:

```python
contact = Contact.objects.get(pk=1)

assert contact.phone_number.is_valid()
assert contact.phone_number.as_e164 == "+16044011234"
assert contact.phone_number.as_national == "(604) 401-1234"
```

### Direct parsing

If you need parsing outside a model or form, use the wrapper directly:

```python
from phonenumber_field.phonenumber import PhoneNumber

number = PhoneNumber.from_string("(604) 401 1234", region="CA")

assert number.is_valid()
assert number.as_e164 == "+16044011234"
```

### Form field

```python
from django import forms
from phonenumber_field.formfields import PhoneNumberField

class ContactForm(forms.Form):
    phone_number = PhoneNumberField(region="CA")
```

The form field validates input and returns a `PhoneNumber` object in `cleaned_data`.

### Django REST Framework serializer field

If the project uses DRF, use the package serializer field instead of a plain string field:

```python
from rest_framework import serializers
from phonenumber_field.serializerfields import PhoneNumberField

class ContactSerializer(serializers.Serializer):
    phone_number = PhoneNumberField()
```

That keeps serializer validation aligned with model and form behavior. Serializer output follows `PHONENUMBER_DEFAULT_FORMAT`, so set that deliberately if your API contract expects `E164`, `RFC3966`, or another format.

## Configuration That Matters

### `PHONENUMBER_DEFAULT_REGION`

Use this when user input is commonly national rather than international:

```python
PHONENUMBER_DEFAULT_REGION = "US"
```

This setting affects parsing. It is the first thing to check if valid local numbers are being rejected.

### `PHONENUMBER_DB_FORMAT`

Choose how values are stored in the database:

```python
PHONENUMBER_DB_FORMAT = "E164"
```

Common choices are `E164`, `INTERNATIONAL`, `NATIONAL`, and `RFC3966`.

Practical rule:

- use `E164` for most APIs, SMS systems, and deduplication logic
- use `RFC3966` or `INTERNATIONAL` if you need extension support preserved in storage
- avoid `NATIONAL` unless the project is truly single-region and you control every reader

Changing `PHONENUMBER_DB_FORMAT` after data already exists can lose extension information, and changing the default region while using `"NATIONAL"` can reinterpret stored values incorrectly.

### `PHONENUMBER_DEFAULT_FORMAT`

Choose how values render when coerced to strings:

```python
PHONENUMBER_DEFAULT_FORMAT = "E164"
```

Keep this aligned with how your app serializes or displays phone numbers. If you need a specific wire format, prefer explicit properties such as `.as_e164` rather than relying on implicit string conversion.

## Widgets And Split Inputs

For a region-aware widget in forms, use `RegionalPhoneNumberWidget`:

```python
from django import forms
from phonenumber_field.formfields import PhoneNumberField
from phonenumber_field.widgets import RegionalPhoneNumberWidget

class ContactForm(forms.Form):
    phone_number = PhoneNumberField(
        widget=RegionalPhoneNumberWidget(region="US"),
    )
```

If you want separate country-prefix and number inputs, use `SplitPhoneNumberField`. This requires Babel and produces a composed `PhoneNumber` value after validation.

```python
from django import forms
from phonenumber_field.formfields import SplitPhoneNumberField

class SignupForm(forms.Form):
    phone_number = SplitPhoneNumberField(region="US")
```

If you need to limit country choices or customize widget attributes, subclass `SplitPhoneNumberField` and override `prefix_field()` or `number_field()`.

## Common Pitfalls

- Installing `django-phonenumber-field` without either the `phonenumbers` or `phonenumberslite` extra.
- Forgetting `"phonenumber_field"` in `INSTALLED_APPS`.
- Accepting national numbers but not setting `PHONENUMBER_DEFAULT_REGION`.
- Relying on `str(phone_number)` in API payloads instead of explicit formats such as `.as_e164`.
- Choosing `PHONENUMBER_DB_FORMAT = "NATIONAL"` and later moving data across regions or changing the default region.
- Expecting phone-number extensions to round-trip cleanly while using the default `E164` DB and display formats.
- Forgetting that `SplitPhoneNumberField` requires `Babel`.
- Treating any parseable number as valid. Use `.is_valid()` or field validation because not all well-formed numbers are valid.

## Version-Sensitive Notes

- This entry targets `django-phonenumber-field==8.4.0`.
- The `8.4.0` release updated supported Python and Django versions. Prefer the support matrix on the `8.4.0` PyPI page over older blog posts or old snippets.
- The `8.4.0` release also fixed handling of empty values in `SplitPhoneNumberField` when `max_length` is set. Re-check any local workaround around split fields after upgrading.
- The `8.0.0` release moved validation behavior from widgets into form fields and removed `PhoneNumberInternationalFallbackWidget`. If you are upgrading older code, do not try to restore that widget in `8.4.0`.
- As of March 12, 2026, the docs URL `https://django-phonenumber-field.readthedocs.io/en/latest/` and the package version are aligned at the latest `8.4.0` release, but install commands in this doc stay version-pinned on purpose.

## Official Context

- Docs root: https://django-phonenumber-field.readthedocs.io/en/latest/
- Reference docs: https://django-phonenumber-field.readthedocs.io/en/stable/reference.html
- Handling extensions: https://django-phonenumber-field.readthedocs.io/en/stable/phonenumbers.html
- `8.4.0` release notes: https://github.com/stefanfoulis/django-phonenumber-field/releases/tag/8.4.0
- PyPI release page: https://pypi.org/project/django-phonenumber-field/8.4.0/
- Repository: https://github.com/stefanfoulis/django-phonenumber-field
