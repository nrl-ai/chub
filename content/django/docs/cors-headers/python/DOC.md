---
name: cors-headers
description: "django-cors-headers guide for Django projects configuring CORS, credentials, CSRF, and dynamic origin rules"
metadata:
  languages: "python"
  versions: "4.9.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,cors,http,middleware,csrf,security"
---

# django-cors-headers for Django

## Golden Rule

`django-cors-headers` is middleware and settings, not an API client. In Django projects, install the package, add `corsheaders` to `INSTALLED_APPS`, put `corsheaders.middleware.CorsMiddleware` near the top of `MIDDLEWARE`, and prefer explicit allowed origins over permissive wildcard settings.

As of March 12, 2026, PyPI still lists `4.9.0` as the latest release. Use the 4.9.0 package docs and metadata for version-specific behavior, because the GitHub `main` branch has already moved on to unreleased changes.

## Install

Pin the package version your project expects:

```bash
python -m pip install "django-cors-headers==4.9.0"
```

Common alternatives:

```bash
uv add "django-cors-headers==4.9.0"
poetry add "django-cors-headers==4.9.0"
```

## Django Setup

Add the app:

```python
INSTALLED_APPS = [
    # ...
    "corsheaders",
]
```

Then add the middleware as high as possible, before middleware that can generate responses such as `CommonMiddleware` or `WhiteNoiseMiddleware`:

```python
MIDDLEWARE = [
    "corsheaders.middleware.CorsMiddleware",
    "django.middleware.common.CommonMiddleware",
    # ...
]
```

Minimal explicit-origin configuration:

```python
CORS_ALLOWED_ORIGINS = [
    "https://app.example.com",
    "http://localhost:3000",
]
```

At least one of these settings must be configured:

- `CORS_ALLOWED_ORIGINS`
- `CORS_ALLOWED_ORIGIN_REGEXES`
- `CORS_ALLOW_ALL_ORIGINS`

## Core Configuration Patterns

### Explicit allowlist

Use this for most production apps:

```python
CORS_ALLOWED_ORIGINS = [
    "https://app.example.com",
    "https://admin.example.com",
]
```

Origins must include scheme, and port if non-default. Do not use bare hostnames like `"app.example.com"`.

### Regex-based subdomains

Use this when you have many tenant subdomains:

```python
CORS_ALLOWED_ORIGIN_REGEXES = [
    r"^https://\w+\.example\.com$",
]
```

### Allow all origins

Use this only for tightly controlled internal cases or local development:

```python
CORS_ALLOW_ALL_ORIGINS = True
```

### Restrict CORS to part of the site

If only your API needs CORS, scope it:

```python
CORS_URLS_REGEX = r"^/api/.*$"
```

### Extend default methods and headers

Use the package defaults rather than replacing them wholesale:

```python
from corsheaders.defaults import default_headers, default_methods

CORS_ALLOW_HEADERS = (
    *default_headers,
    "x-client-version",
)

CORS_ALLOW_METHODS = (
    *default_methods,
    "PURGE",
)
```

## Credentials, Cookies, and CSRF

`django-cors-headers` does not handle authentication by itself. Its job is to emit the right CORS headers so browser requests can reach your Django app.

For cookie-based auth across origins:

```python
CORS_ALLOW_CREDENTIALS = True
SESSION_COOKIE_SAMESITE = "None"
SESSION_COOKIE_SECURE = True
CSRF_COOKIE_SAMESITE = "None"
CSRF_COOKIE_SECURE = True
```

For cross-origin unsafe requests such as `POST`, `PUT`, `PATCH`, and `DELETE`, configure Django CSRF separately. CORS and CSRF are not the same thing:

```python
CORS_ALLOWED_ORIGINS = [
    "https://readonly.example.com",
    "https://app.example.com",
]

CSRF_TRUSTED_ORIGINS = [
    "https://app.example.com",
]
```

Use `CSRF_TRUSTED_ORIGINS` for origins that need write access over HTTPS. Do not assume a CORS allowlist is enough to satisfy Django CSRF checks.

## Dynamic Origin Rules With Signals

If static settings are not enough, connect handlers to `corsheaders.signals.check_request_enabled`. If any handler returns truthy, the request is allowed.

Example:

```python
# myapp/handlers.py
from corsheaders.signals import check_request_enabled

def cors_allow_public_api(sender, request, **kwargs):
    return request.path.startswith("/public-api/")

check_request_enabled.connect(cors_allow_public_api)
```

Connect handlers from your app config so they load during startup:

```python
# myapp/apps.py
from django.apps import AppConfig

class MyAppConfig(AppConfig):
    name = "myapp"

    def ready(self) -> None:
        from . import handlers  # noqa: F401
```

Then reference the app config in `INSTALLED_APPS`, for example `"myapp.apps.MyAppConfig"`.

## Common Pitfalls

- Keep the trailing comma in `INSTALLED_APPS`. Without it, agents sometimes turn the entry into a string concatenation bug and trigger `ModuleNotFoundError`.
- Put `CorsMiddleware` before `CommonMiddleware`, `WhiteNoiseMiddleware`, or other middleware that may return responses early.
- Include schemes in origins. `https://example.com` is valid; `example.com` is not.
- Use `CORS_ALLOWED_ORIGINS` for explicit sites and `CSRF_TRUSTED_ORIGINS` for cross-origin write requests. They solve different problems.
- Prefer the renamed settings: `CORS_ALLOWED_ORIGINS`, `CORS_ALLOWED_ORIGIN_REGEXES`, and `CORS_ALLOW_ALL_ORIGINS`. Older `*_WHITELIST` and `CORS_ORIGIN_ALLOW_ALL` aliases still work, but the new names take precedence.
- Do not look for the old `CORS_REPLACE_HTTPS_REFERER` setting or `CorsPostCsrfMiddleware`; both were removed in the 4.0.0 release line.
- For browser clients that need cookies, `CORS_ALLOW_CREDENTIALS = True` is not enough by itself. Django cookie `SameSite` and `Secure` settings still matter.

## Version-Sensitive Notes

- `4.9.0` added Django `6.0` support.
- PyPI release metadata for `4.9.0` says Python `>=3.9` and documents Python `3.9` to `3.14` support.
- The current GitHub `main` README now says Python `3.10` to `3.14` supported because a post-4.9.0 changelog entry drops Python `3.9`. That is branch drift, not a contradiction in the 4.9.0 release metadata.
- `4.0.0` introduced `CORS_ALLOW_PRIVATE_NETWORK`, added async middleware support, and removed `CORS_REPLACE_HTTPS_REFERER` plus `CorsPostCsrfMiddleware`.

## Official Sources

- Maintainer docs and setup guide: `https://github.com/adamchainz/django-cors-headers`
- Release-specific docs snapshot and metadata: `https://pypi.org/project/django-cors-headers/4.9.0/`
- Release history and provenance: `https://pypi.org/project/django-cors-headers/`
- Changelog: `https://raw.githubusercontent.com/adamchainz/django-cors-headers/main/CHANGELOG.rst`
