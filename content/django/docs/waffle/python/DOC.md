---
name: waffle
description: "django-waffle package guide for Django feature flags, switches, and percentage rollouts"
metadata:
  languages: "python"
  versions: "5.0.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django-waffle,django,feature-flags,rollouts,switches,experiments,python"
---

# django-waffle 5.0.0 Python Package Guide

## What It Does

`django-waffle` adds feature gating to Django with three primitives:

- `Flag`: request-aware feature flags with user, group, staff, authenticated, language, percentage, testing, and rollout controls
- `Switch`: global on or off toggles
- `Sample`: random percentage sampling when you do not have or need a request object

Use it for staged rollouts, kill switches, and limited experiments. Do not treat it as an authorization system or a replacement for Django permissions.

## Install

```bash
python -m pip install "django-waffle==5.0.0"
```

Common equivalents:

```bash
uv add "django-waffle==5.0.0"
poetry add "django-waffle==5.0.0"
```

## Minimal Setup

Add the app and middleware, then run migrations:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "waffle",
]

MIDDLEWARE = [
    # ...
    "waffle.middleware.WaffleMiddleware",
    # ...
]
```

```bash
python manage.py migrate
```

Waffle stores flags, switches, and samples in Django models, so migrations have to run before you call the helper APIs.

## Choose The Right Primitive

### Flag

Use `Flag` when activation depends on the current request or current user.

Important controls from upstream:

- `everyone` for a global tri-state override
- testing mode through query string plus cookies
- user and group targeting
- superuser, staff, and authenticated checks
- language matching
- percentage rollout
- rollout mode for sticky-on behavior as percentages increase

### Switch

Use `Switch` for global maintenance gates and simple kill switches:

```python
import waffle

if waffle.switch_is_active("billing-maintenance"):
    enable_read_only_mode()
```

### Sample

Use `Sample` for random percentage experiments when no request object is involved:

```python
import waffle

search_v2_on = waffle.sample_is_active("search-v2")
if search_v2_on:
    use_search_v2()
```

Important: `sample_is_active()` is random on each evaluation. If you need a stable answer for the rest of the code path, store the result in a local variable and reuse it.

## Core Usage

### Gate logic inside a view

```python
import waffle
from django.http import HttpResponse

def checkout(request):
    if waffle.flag_is_active(request, "new-checkout"):
        return HttpResponse("new flow")
    return HttpResponse("old flow")
```

Use `flag_is_active(request, ...)` for request-aware checks, `switch_is_active(...)` for global toggles, and `sample_is_active(...)` for random sampling.

### Gate an entire view with a decorator

```python
from waffle.decorators import waffle_flag

@waffle_flag("new-checkout")
def checkout(request):
    ...
```

When the named flag is inactive, the wrapped view returns `404` by default. You can also pass a redirect target. Waffle supports inverted decorators such as `@waffle_flag("!new-checkout")` and `@waffle_switch("!billing-maintenance")`.

There is no sample decorator because sample evaluation is intentionally random.

### Gate class-based views

```python
from django.views.generic import TemplateView
from waffle.mixins import WaffleFlagMixin

class NewCheckoutView(WaffleFlagMixin, TemplateView):
    waffle_flag = "new-checkout"
    template_name = "checkout/new.html"
```

Waffle also provides `WaffleSwitchMixin` and `WaffleSampleMixin`.

### Gate templates

```django
{% load waffle_tags %}

{% flag "new-checkout" %}
  <a href="/checkout/new/">Try the new checkout</a>
{% else %}
  <a href="/checkout/">Checkout</a>
{% endflag %}
```

Template tags are available for `flag`, `switch`, and `sample`. For Jinja2 templates, Waffle exposes a `waffle` object so you can write checks such as `waffle.flag("new-checkout")`.

## Managing Flags, Switches, And Samples

Use Django admin for manual control or the built-in management commands for scripted updates.

Create or update a flag:

```bash
python manage.py waffle_flag new-checkout --create --percent=10 --rollout
```

Turn a switch on:

```bash
python manage.py waffle_switch billing-maintenance on --create
```

Set a sample percentage:

```bash
python manage.py waffle_sample search-v2 25.0 --create
```

Useful list and cleanup commands:

```bash
python manage.py waffle_flag -l
python manage.py waffle_switch -l
python manage.py waffle_sample -l
python manage.py waffle_delete --flags new-checkout --switches billing-maintenance
```

## Configuration And Auth Notes

Waffle integrates with Django request and auth state. Flags can target authenticated users, staff, superusers, groups, specific users, and request languages.

The settings most likely to matter in production are:

- `WAFFLE_OVERRIDE`: query-string overrides for manual or end-to-end testing
- `WAFFLE_COOKIE` and `WAFFLE_TEST_COOKIE`: cookie naming
- `WAFFLE_MAX_AGE`: cookie lifetime
- `WAFFLE_SECURE`: whether Waffle cookies are marked secure
- `WAFFLE_CACHE_NAME` and `WAFFLE_CACHE_PREFIX`: cache selection and namespacing
- `WAFFLE_READ_FROM_WRITE_DB`: read from the write database if replica lag would make checks stale
- `WAFFLE_CREATE_MISSING_FLAGS`, `WAFFLE_CREATE_MISSING_SWITCHES`, `WAFFLE_CREATE_MISSING_SAMPLES`: auto-create missing objects instead of falling back silently
- `WAFFLE_LOG_MISSING_FLAGS`, `WAFFLE_LOG_MISSING_SWITCHES`, `WAFFLE_LOG_MISSING_SAMPLES`: log missing definitions so typos surface quickly
- `WAFFLE_ENABLE_ADMIN_PAGES`: disable built-in admin pages if you are replacing them

Practical defaults:

- keep `WAFFLE_OVERRIDE = False` unless you have a controlled testing need
- keep auto-create settings off in production
- turn on missing-item logging outside local development
- use a dedicated cache prefix if you are changing Waffle versions or custom models across deployments

## Custom Models

Waffle supports swappable models:

- `WAFFLE_FLAG_MODEL`
- `WAFFLE_SWITCH_MODEL`
- `WAFFLE_SAMPLE_MODEL`

Set these only at project start. Upstream warns that changing them later will not yield workable migrations. If you use a custom model, define it before the first `migrate` and run `makemigrations` first.

## Testing

For automated tests, prefer Waffle's test helpers instead of mutating database state manually:

```python
from waffle.testutils import override_flag

def test_checkout_uses_new_flow(client):
    with override_flag("new-checkout", active=True):
        response = client.get("/checkout/")
        assert response.status_code == 200
```

Waffle also provides `override_switch` and `override_sample`. These restore previous state automatically and delete temporary objects they created.

For external browser-style tests that talk to a running server in a separate process, `WAFFLE_OVERRIDE` allows per-request control through the query string, for example `GET /checkout/?new-checkout=1`.

## Common Pitfalls

- Using `Sample` for logic that must stay stable across multiple checks in one request or task.
- Leaving `WAFFLE_OVERRIDE` enabled outside controlled testing.
- Turning on `WAFFLE_CREATE_MISSING_*` in production and then hiding typos in flag names.
- Changing `WAFFLE_FLAG_MODEL`, `WAFFLE_SWITCH_MODEL`, or `WAFFLE_SAMPLE_MODEL` after the first migration.
- Assuming Waffle is an authorization system.
- Forgetting that rollout mode changes how "off" cookies behave as the rollout percentage increases.

## Version-Sensitive Notes

- `5.0.0` drops support for Django `3.2`, `4.0`, and `4.1`, and drops Python `3.8`.
- `5.0.0` adds Django `5.2` support.
- Current package metadata requires Python `>=3.9` and Django `>=4.2`.
- `5.0.0` includes a breaking fix for `flag.everyone`: custom code that depends on `Flag.is_active_for_user()` should be rechecked after upgrading from older releases.
- The docs URL, `https://waffle.readthedocs.io/en/latest/`, displayed `5.0.0` on `2026-03-12`, but it is a floating docs root. For behavior-sensitive debugging, verify against the versioned `v5.0.0` page plus the `v5.0.0` release and changelog.

## Official Sources

- Docs index: https://waffle.readthedocs.io/en/stable/index.html
- Installation: https://waffle.readthedocs.io/en/stable/starting/installation.html
- Configuration: https://waffle.readthedocs.io/en/stable/starting/configuring.html
- Flag reference: https://waffle.readthedocs.io/en/stable/types/flag.html
- Switch reference: https://waffle.readthedocs.io/en/stable/types/switch.html
- Sample reference: https://waffle.readthedocs.io/en/stable/types/sample.html
- Views: https://waffle.readthedocs.io/en/stable/usage/views.html
- Decorators: https://waffle.readthedocs.io/en/stable/usage/decorators.html
- Templates: https://waffle.readthedocs.io/en/stable/usage/templates.html
- CLI: https://waffle.readthedocs.io/en/stable/usage/cli.html
- Automated testing: https://waffle.readthedocs.io/en/stable/testing/automated.html
- Versioned 5.0.0 flag page: https://waffle.readthedocs.io/en/v5.0.0/types/flag.html
- PyPI: https://pypi.org/project/django-waffle/
- Repository: https://github.com/django-waffle/django-waffle
- Changelog: https://github.com/django-waffle/django-waffle/blob/master/CHANGES
- Release: https://github.com/django-waffle/django-waffle/releases/tag/v5.0.0
