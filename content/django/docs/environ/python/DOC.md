---
name: environ
description: "django-environ package guide for Python - environment-driven Django settings and URL-based config parsing"
metadata:
  languages: "python"
  versions: "0.13.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django-environ,django,environment,settings,configuration,secrets,12factor"
---

# django-environ Python Package Guide

## When To Use It

`django-environ` reads Django settings from environment variables and `.env` files. It provides typed accessors such as `bool`, `int`, `list`, and `json`, plus URL parsers for databases, caches, email, and search backends.

This entry is pinned to package version `0.13.0`. The import path is `import environ`, and the docs explicitly note that you do not add `django-environ` to `INSTALLED_APPS`.

Current upstream requirements for `0.13.0`:

- Python `>=3.9`
- Django `>=2.2`

## Install

```bash
python -m pip install django-environ==0.13.0
```

## Initialize In `settings.py`

```python
from pathlib import Path
import environ

BASE_DIR = Path(__file__).resolve().parent.parent

env = environ.Env(
    DEBUG=(bool, False),
    SECRET_KEY=(str, ""),
    ALLOWED_HOSTS=(list, ["localhost", "127.0.0.1"]),
)

# Use an explicit path so startup does not depend on cwd or stack inspection.
environ.Env.read_env(BASE_DIR / ".env")

DEBUG = env("DEBUG")
SECRET_KEY = env("SECRET_KEY")
ALLOWED_HOSTS = env("ALLOWED_HOSTS")

DATABASES = {
    "default": env.db(
        "DATABASE_URL",
        default=f"sqlite:///{BASE_DIR / 'db.sqlite3'}",
    )
}

CACHES = {
    "default": env.cache(
        "CACHE_URL",
        default="locmemcache://",
    )
}
```

Example `.env`:

```dotenv
DEBUG=True
SECRET_KEY=change-me
ALLOWED_HOSTS=localhost,127.0.0.1
DATABASE_URL=postgres://app:secret@127.0.0.1:5432/app
CACHE_URL=redis://127.0.0.1:6379/1
```

The upstream quickstart pattern is: define casts and defaults on `Env(...)`, call `read_env(...)`, then read variables with `env("NAME")` or explicit helpers like `env.int(...)` and `env.db(...)`.

## Core Patterns

### Scalars And Collections

```python
DEBUG = env.bool("DEBUG", default=False)
PORT = env.int("PORT", default=8000)
SITE_NAME = env.str("SITE_NAME", default="example")
ALLOWED_HOSTS = env.list("ALLOWED_HOSTS", default=["localhost"])
FEATURE_FLAGS = env.json("FEATURE_FLAGS", default={})
SAML_ATTRIBUTE_MAPPING = env.dict(
    "SAML_ATTRIBUTE_MAPPING",
    default={},
)
```

Use the typed helpers directly instead of relying on smart-casting from `default=`. The docs say smart-casting is still enabled by default in `0.13.0`, but it can have side effects and is planned to change in the next major release.

### URL-Based Django Settings

```python
DATABASES = {
    "default": env.db(
        "DATABASE_URL",
        default=f"sqlite:///{BASE_DIR / 'db.sqlite3'}",
    )
}
```

Example:

```dotenv
DATABASE_URL=postgres://app:secret@localhost:5432/app
```

`env.db()` is an alias for `env.db_url()`. Use the same pattern for caches, email, search, and channels:

```python
CACHES = {
    "default": env.cache("CACHE_URL", default="locmemcache://")
}

EMAIL_CONFIG = env.email_url(
    "EMAIL_URL",
    default="smtp://user:password@localhost:25",
)
vars().update(EMAIL_CONFIG)

SEARCH_CONFIG = env.search_url("SEARCH_URL", default="simple://")
CHANNEL_LAYERS = {
    "default": env.channels("CHANNELS_URL", default="redis://127.0.0.1:6379/0"),
}
```

Example `.env` values:

```dotenv
CACHE_URL=rediscache://127.0.0.1:6379/1
EMAIL_URL=smtp://user:password@smtp.example.com:587
SEARCH_URL=elasticsearch://127.0.0.1:9200/index-name
CHANNELS_URL=rediss://127.0.0.1:6379/0
```

`0.13.0` adds support for `valkey://` and `valkeys://` cache URLs, and `rediss://` for channels URL parsing.

### Prefixes, Choices, And Defaults

```python
env = environ.Env()
env.prefix = "DJANGO_"
env.warn_on_default = True

DEBUG = env.bool("DEBUG", default=False)  # reads DJANGO_DEBUG
SECRET_KEY = env("SECRET_KEY")            # reads DJANGO_SECRET_KEY
ENVIRONMENT = env.str(
    "ENVIRONMENT",
    default="development",
    choices=("development", "staging", "production"),
)
```

In `0.13.0`, `Env.str(..., choices=...)` is available for basic string validation, and `warn_on_default` can emit `DefaultValueWarning` when missing variables fall back to explicit defaults.

## Secrets And Deployment Config

### `_FILE` Secrets With `FileAwareEnv`

`FileAwareEnv` checks `VAR_FILE` before `VAR`, which is useful for Docker and orchestration systems that mount secrets as files.

```python
from pathlib import Path
import environ

BASE_DIR = Path(__file__).resolve().parent.parent

env = environ.FileAwareEnv()
environ.Env.read_env(BASE_DIR / ".env")

SECRET_KEY = env("SECRET_KEY")
DATABASES = {"default": env.db("DATABASE_URL")}
```

Example:

```dotenv
SECRET_KEY_FILE=/run/secrets/django_secret_key
DATABASE_URL_FILE=/run/secrets/database_url
```

`_FILE` values take precedence. If the referenced file is missing, the read fails, so treat these paths as part of deployment configuration.

### Read Specific Env Files Deliberately

```python
ENV_PATH = env.str("ENV_PATH", default=str(BASE_DIR / ".env"))
environ.Env.read_env(ENV_PATH)
```

The docs also show layering multiple files and controlling precedence with `overwrite=True`.

```python
environ.Env.read_env(BASE_DIR / ".env")
environ.Env.read_env(BASE_DIR / ".env.local", overwrite=True)
```

Use this only when you want later files to override earlier values.

## Common Pitfalls

- `read_env()` will try to infer the project root if you omit the path, but the FAQ explicitly says that is not the recommended pattern. Pass an explicit path.
- Real environment variables win over `.env` values unless you opt into `overwrite=True`.
- Missing required variables fail during settings import. Define defaults where that is acceptable and let critical secrets fail fast.
- `env.list()` is only a comma splitter. For nested or structured data, prefer `env.json()`, `env.dict()`, or a custom parser.
- `env.db()`, `env.cache()`, `env.email_url()`, and `env.search_url()` all expect URL-shaped strings. Validate schemes and credentials early.
- URL passwords with `#`, `@`, `/`, or other unsafe characters must be percent-encoded before they go into `DATABASE_URL` or similar settings.
- `parse_comments=True` can silently truncate values containing `#`. Leave it off unless you really want inline comments.
- Proxy interpolation is opt-in via `environ.Env(interpolate=True)`. If you need a literal leading `$`, enable `env.escape_proxy = True` and escape the value.
- Multiline secrets require `env.str(..., multiline=True)`; do not pre-escape newlines if you want the original formatting preserved.

## Version-Sensitive Notes For `0.13.0`

- `0.13.0` is the current docs release and the current PyPI latest release, published on `2026-02-18`.
- `0.13.0` adds `warn_on_default`, `Env.str(..., choices=...)`, Valkey cache URL schemes (`valkey://`, `valkeys://`), `rediss://` support in channels URL parsing, and extra django-prometheus DB aliases.
- Smart-casting is still enabled by default in `0.13.0`, but the docs say the next major release will disable it by default. Write explicit casts now so future upgrades are boring.
- Older examples that use `Env.unicode()` are outdated. The deprecations page says to use `Env.str()` instead.
- The API reference and some PyPI project links still expose the legacy `readthedocs.org` hostname, but the canonical docs site is the `readthedocs.io` `latest` tree used below.

## Official Sources

- Docs root: https://django-environ.readthedocs.io/en/latest/
- Installation: https://django-environ.readthedocs.io/en/latest/install.html
- Quick start: https://django-environ.readthedocs.io/en/latest/quickstart.html
- Tips: https://django-environ.readthedocs.io/en/latest/tips.html
- Supported types: https://django-environ.readthedocs.io/en/latest/types.html
- API reference: https://django-environ.readthedocs.io/en/latest/api.html
- FAQ: https://django-environ.readthedocs.io/en/latest/faq.html
- Deprecations: https://django-environ.readthedocs.io/en/latest/deprecations.html
- Changelog: https://django-environ.readthedocs.io/en/latest/changelog.html
- PyPI release page: https://pypi.org/project/django-environ/0.13.0/
- Package registry page: https://pypi.org/project/django-environ/
