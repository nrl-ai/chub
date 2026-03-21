---
name: csp
description: "django-csp 4.0 for Django projects: dict-based CSP settings, nonce handling, decorators, report-only policies, and 3.8 migration notes"
metadata:
  languages: "python"
  versions: "4.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,django-csp,csp,security,http-headers,xss"
---

# django-csp 4.0 Python Package Guide

## What It Does

`django-csp` adds `Content-Security-Policy` headers to Django responses. In `4.0`, the package uses dict-based settings, supports enforced and report-only policies side by side, exposes constants such as `SELF` and `NONCE`, and changes the decorator API compared with `3.8`.

## Install

```bash
pip install django-csp==4.0
```

Optional Jinja template helper support:

```bash
pip install "django-csp[jinja2]==4.0"
```

## Initialize Django

`4.0` setup requires both the Django app and the middleware:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "csp",
    # ...
]

MIDDLEWARE = [
    # ...
    "csp.middleware.CSPMiddleware",
    # ...
]
```

Middleware order usually does not matter unless other middleware mutates CSP headers or needs to force nonce generation before the response is processed.

## Core Configuration

`4.0` no longer uses `CSP_*` settings. Put policy dictionaries in `CONTENT_SECURITY_POLICY` and optionally `CONTENT_SECURITY_POLICY_REPORT_ONLY`.

```python
# settings.py
from csp.constants import NONE, SELF

CONTENT_SECURITY_POLICY = {
    "DIRECTIVES": {
        "default-src": [SELF],
        "script-src": [SELF, "cdn.example.com"],
        "style-src": [SELF, "fonts.googleapis.com"],
        "font-src": [SELF, "fonts.gstatic.com"],
        "img-src": [SELF, "data:"],
        "connect-src": [SELF, "api.example.com"],
        "object-src": [NONE],
        "base-uri": [SELF],
        "frame-ancestors": [NONE],
        "form-action": [SELF],
    },
}
```

If you are tightening an existing site, add a report-only policy first:

```python
from csp.constants import NONE, SELF

CONTENT_SECURITY_POLICY_REPORT_ONLY = {
    "DIRECTIVES": {
        "default-src": [NONE],
        "script-src": [SELF],
        "style-src": [SELF],
        "img-src": [SELF],
        "connect-src": [SELF],
        "frame-ancestors": [SELF],
        "form-action": [SELF],
        "upgrade-insecure-requests": True,
        "report-uri": ["/csp-report/"],
    },
}
```

Practical notes:

- Directive names use lowercase CSP names with dashes, such as `"default-src"` and `"frame-ancestors"`.
- Use `csp.constants` for keywords like `SELF`, `NONE`, and `NONCE` to avoid quoting mistakes.
- `NONE` is the CSP keyword `"'none'"`; Python `None` means remove that directive from the built header.
- Use lists or tuples for directive values. A plain string where a sequence is expected can produce invalid or surprising policies.

## Nonces

In `4.0`, nonce inclusion moved into the directive itself. Add `NONCE` to `script-src` or `style-src`, then access `request.csp_nonce` before the middleware writes the response header.

```python
# settings.py
from csp.constants import NONCE, SELF

CONTENT_SECURITY_POLICY = {
    "DIRECTIVES": {
        "default-src": [SELF],
        "script-src": [SELF, NONCE],
    },
}
```

In a Django template:

```django
<script nonce="{{ request.csp_nonce }}">
  window.appConfig = {{ config_json|safe }};
</script>
```

Important `4.0` nonce behavior:

- The nonce is only added if `NONCE` is present in `script-src` or `style-src` and `request.csp_nonce` is actually accessed before response processing finishes.
- `bool(request.csp_nonce)` does not generate the nonce in `4.0`; use `str(request.csp_nonce)` or render it in a template.
- Reading an ungenerated nonce after response processing raises `csp.exceptions.CSPNonceError`.
- Nonces are not exposed in browser devtools; verify them with page source.

If custom middleware needs the nonce, place that middleware after `csp.middleware.CSPMiddleware` and force generation during request handling.

## Optional Template Helpers

If you use many nonced scripts, expose `CSP_NONCE` globally:

```python
# settings.py
TEMPLATES = [
    {
        "BACKEND": "django.template.backends.django.DjangoTemplates",
        "OPTIONS": {
            "context_processors": [
                # ...
                "csp.context_processors.nonce",
            ],
            "libraries": {
                "csp": "csp.templatetags.csp",
            },
        },
    },
]
```

Then use the provided template tag:

```django
{% load csp %}
{% script type="application/javascript" async=False %}
  <script>
    console.log("nonced");
  </script>
{% endscript %}
```

For Jinja, install the `jinja2` extra or provide `jinja2>=2.9.6` yourself, then add `csp.extensions.NoncedScript` to the template backend.

## Per-View Overrides

All `4.0` decorators now take a directive dictionary. `REPORT_ONLY=True` targets the report-only policy instead of the enforced policy.

```python
from django.shortcuts import render
from csp.constants import SELF
from csp.decorators import csp, csp_exempt, csp_replace, csp_update

@csp_update({"img-src": "imgsrv.example.com"})
def dashboard(request):
    return render(request, "dashboard.html")

@csp_replace({"frame-ancestors": [SELF]})
def embeddable(request):
    return render(request, "embed.html")

@csp({"default-src": [SELF], "script-src": [SELF, "js.example.com"]})
def strict_view(request):
    return render(request, "strict.html")

@csp_exempt()
def legacy_callback(request):
    return render(request, "callback.html")
```

Use these sparingly. `@csp_exempt()` now requires parentheses in `4.0`, even with no arguments.

## Violation Reports

`django-csp` does not process reports for you. You need your own endpoint or a third-party reporting service.

If you want to sample reports, switch middleware classes and set the percentage:

```python
# settings.py
from csp.constants import SELF

MIDDLEWARE = [
    # ...
    "csp.contrib.rate_limiting.RateLimitedCSPMiddleware",
    # ...
]

CONTENT_SECURITY_POLICY_REPORT_ONLY = {
    "REPORT_PERCENTAGE": 10.0,
    "DIRECTIVES": {
        "default-src": [SELF],
        "report-uri": ["/csp-report/"],
    },
}
```

In `4.0`, `REPORT_PERCENTAGE` is a float between `0.0` and `100.0`, so `10.0` means 10% of requests include `report-uri`.

## Trusted Types

Trusted Types help with DOM XSS, not just server-rendered inline scripts. The upstream guide recommends enabling them in report-only mode first:

```python
CONTENT_SECURITY_POLICY_REPORT_ONLY = {
    "DIRECTIVES": {
        "require-trusted-types-for": ["'script'"],
        "trusted-types": ["default"],
        "report-uri": ["/csp-report/"],
    },
}
```

Then fix violating sinks such as `innerHTML`, `outerHTML`, `insertAdjacentHTML`, `document.write`, and `DOMParser.parseFromString` before enforcing the policy. The docs prefer rewriting code or using a library like DOMPurify over hand-rolled sanitizers.

## Authentication And Runtime Model

`django-csp` has no authentication layer, credentials, or external service handshake. Its runtime model is purely Django settings, middleware, decorators, and template integration. Most production mistakes come from policy shape and middleware behavior, not auth.

## Common Pitfalls

- `4.0` is not backward-compatible with `3.8`. Old `CSP_*` settings are not supported.
- Add `"csp"` to `INSTALLED_APPS`; this is now part of the official install path.
- Decorators now take a single directive dictionary, and `@csp_exempt()` needs parentheses.
- `default-src` does not automatically cover a directive once that directive is explicitly set. If you add `img-src`, include every image source you need there.
- `REPORT_PERCENTAGE` requires `csp.contrib.rate_limiting.RateLimitedCSPMiddleware`; it is ignored with the default middleware.
- `REPORT_PERCENTAGE` units changed from the `3.8` style fraction to a `0.0` to `100.0` percentage.
- `NONCE` moved from `CSP_INCLUDE_NONCE_IN` to directive values, and `request.csp_nonce` is lazy in a different way than `3.8`.
- Python `None` removes a directive; `csp.constants.NONE` emits the CSP keyword `'none'`.
- `csp` is the Django integration namespace even though the PyPI package name is `django-csp`.

## Version-Sensitive Notes

- `4.0` was released on April 2, 2025.
- The `v4.0` release notes call out several breaking changes:
  - configuration moved from `CSP_*` settings to `CONTENT_SECURITY_POLICY` and `CONTENT_SECURITY_POLICY_REPORT_ONLY`
  - nonce configuration moved to the `NONCE` sentinel inside directive values
  - `request.csp_nonce` is falsy until read as a string
  - Django `<=3.2` support was dropped
  - Python `3.8` support was dropped
- `python manage.py check` can help find legacy `CSP_*` settings and generate a migration starting point.
- If you subclass `CSPMiddleware`, `build_policy()` and `build_policy_ro()` are deprecated in `4.0` and scheduled for removal in `4.1`; migrate custom middleware to `build_policy_parts()`.

If the target project is pinned to `django-csp==3.8` or older, do not copy this file's `4.0` configuration and decorator examples into that codebase.

## Official Sources

- Docs root: https://django-csp.readthedocs.io/en/latest/
- Installation: https://django-csp.readthedocs.io/en/latest/installation.html
- Configuration: https://django-csp.readthedocs.io/en/latest/configuration.html
- Migration guide: https://django-csp.readthedocs.io/en/latest/migration-guide.html
- Decorators: https://django-csp.readthedocs.io/en/latest/decorators.html
- Nonces: https://django-csp.readthedocs.io/en/latest/nonce.html
- Violation reports: https://django-csp.readthedocs.io/en/latest/reports.html
- Trusted Types: https://django-csp.readthedocs.io/en/latest/trusted_types.html
- PyPI: https://pypi.org/project/django-csp/
- Release notes: https://github.com/mozilla/django-csp/releases
