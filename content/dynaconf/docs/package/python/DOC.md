---
name: package
description: "Dynaconf package guide for Python configuration management with layered settings files, environment overrides, validation, and framework integrations"
metadata:
  languages: "python"
  versions: "3.2.12"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "dynaconf,python,configuration,settings,envvars,django,flask"
---

# Dynaconf Python Package Guide

## Golden Rule

Use `dynaconf` as a layered settings system: define explicit `settings_files`, keep secrets out of tracked config, and let environment variables or dedicated loaders override file defaults. Do not treat Dynaconf as "magic global state"; most integration bugs come from unclear file order, wrong environment selection, or env var names that do not match Dynaconf's parsing rules.

## Install

Pin the package version your project expects:

```bash
python -m pip install "dynaconf==3.2.12"
```

Common alternatives:

```bash
uv add "dynaconf==3.2.12"
poetry add "dynaconf==3.2.12"
```

Useful extras from the upstream docs:

```bash
python -m pip install "dynaconf[yaml]==3.2.12"
python -m pip install "dynaconf[redis]==3.2.12"
python -m pip install "dynaconf[vault]==3.2.12"
python -m pip install "dynaconf[all]==3.2.12"
```

Use extras only for the loaders or integrations you actually need.

## Initialize A Project

The CLI scaffolder is the fastest path when starting fresh:

```bash
dynaconf init -f toml
```

That creates a typical layout such as:

```text
config.py
settings.toml
.secrets.toml
```

If you initialize manually, keep `settings_files` explicit. The configuration reference notes that this is required for file-backed loading in Dynaconf `3.x`.

```python
from dynaconf import Dynaconf, Validator

settings = Dynaconf(
    envvar_prefix="DYNACONF",
    settings_files=["settings.toml", ".secrets.toml"],
    environments=True,
    load_dotenv=True,
    validators=[
        Validator("DATABASE_URL", env="production", must_exist=True),
    ],
    validate_only_current_env=True,
)
```

Minimal file-backed config:

```toml
[default]
debug = false
log_level = "INFO"

[development]
debug = true
database_url = "sqlite:///dev.db"

[production]
database_url = "@format postgresql://{env[PGUSER]}:{env[PGPASSWORD]}@{env[PGHOST]}/{env[PGDATABASE]}"
```

Keep local-only credentials in `.secrets.toml` and keep that file out of source control.

## Core Usage

### Read settings

Dynaconf exposes keys as attributes and mapping lookups:

```python
from config import settings

if settings.DEBUG:
    print("debug mode")

dsn = settings.get("DATABASE_URL")
current_env = settings.current_env
```

### Switch environments

Dynaconf can keep multiple environment sections in the same file. The common switcher is `ENV_FOR_DYNACONF`.

```bash
export ENV_FOR_DYNACONF=production
python app.py
```

You can also get a scoped settings object in code:

```python
from config import settings

production_settings = settings.from_env("production")
```

### Override with environment variables

Environment variables override file values. By default Dynaconf looks for the `DYNACONF_` prefix unless you change `envvar_prefix`.

```bash
export DYNACONF_DEBUG=true
export DYNACONF_DATABASE_URL="postgresql://app:secret@db/app"
export DYNACONF_DATABASES__default__HOST=db.internal
export DYNACONF_FEATURE_FLAGS='@json ["search","billing"]'
```

Practical notes:

- Use double underscores for nested keys such as `DATABASES__default__HOST`.
- Dynaconf parses booleans, numbers, lists, dicts, and other structured values from env vars, but only when the string is in a supported format.
- Use `@json`, `@format`, `@merge`, and similar tokens when you need explicit parsing behavior.

### Validate required settings

Use validators to fail early instead of discovering missing settings at runtime:

```python
from dynaconf import Dynaconf, Validator

settings = Dynaconf(
    settings_files=["settings.toml", ".secrets.toml"],
    validators=[
        Validator("DEBUG", must_exist=True, is_type_of=bool),
        Validator("DATABASE_URL", env="production", must_exist=True),
    ],
    validate_only_current_env=True,
)

settings.validators.validate()
```

`validate_only_current_env=True` matters when some keys are intentionally defined only for selected environments.

### Use the CLI against an existing instance

Except for `dynaconf init`, CLI commands need to know which settings instance to load.

```bash
dynaconf -i config.settings list
dynaconf -i config.settings validate
```

The CLI docs also allow setting `INSTANCE_FOR_DYNACONF=config.settings` in the environment.

## Django And Flask Integration

### Django

Upstream guidance for Django uses the extension at the bottom of `settings.py`:

```python
from dynaconf import DjangoDynaconf

settings = DjangoDynaconf(__name__)
```

Important Django-specific notes from the maintainer docs:

- Put the Dynaconf hook at the end of `settings.py`.
- Do not add more settings code below that hook.
- Django settings files are layered, and Dynaconf overlays on top of them rather than replacing Django's startup model.
- Environment variable names for Django use the `DJANGO_` prefix by default in the documented integration.

The Django docs also call out `dynaconf[yaml]` during setup. Follow the integration guide instead of mixing plain-Dynaconf and Django-specific examples.

### Flask

Flask integration uses `FlaskDynaconf`:

```python
from flask import Flask
from dynaconf import FlaskDynaconf

app = Flask(__name__)
FlaskDynaconf(app, settings_files=["settings.toml", ".secrets.toml"])
```

Use Flask integration when you want Dynaconf to populate `app.config` cleanly instead of manually syncing values yourself.

## Secrets And External Loaders

Dynaconf supports layered secrets files plus external stores such as Redis and HashiCorp Vault.

### Local secrets file

For local development, keep secrets in `.secrets.toml`, `.secrets.yaml`, or the matching configured format and add them to `.gitignore`.

### Vault

Install the extra and enable the loader:

```bash
python -m pip install "dynaconf[vault]==3.2.12"
```

Typical environment variables from the maintainer docs:

```bash
export VAULT_ENABLED_FOR_DYNACONF=true
export VAULT_URL_FOR_DYNACONF="http://localhost:8200"
export VAULT_TOKEN_FOR_DYNACONF="..."
```

Use Vault for runtime secrets in deployed environments instead of committing `.secrets.*` files.

### Redis

Install the Redis extra and enable the loader:

```bash
python -m pip install "dynaconf[redis]==3.2.12"
```

Typical environment variables:

```bash
export REDIS_ENABLED_FOR_DYNACONF=true
export REDIS_HOST_FOR_DYNACONF=localhost
export REDIS_PORT_FOR_DYNACONF=6379
```

Redis and Vault settings still participate in Dynaconf's environment layering, so keep your environment names and prefixes consistent.

## Common Pitfalls

- `settings_files` is not optional for manual file-backed setup in Dynaconf `3.x`. If you omit it, your files may never be loaded.
- Docs site examples currently render as `3.2.11`, while PyPI ships `3.2.12`. Pin the package version separately from the docs page title.
- First-level keys are more forgiving than nested keys. Nested env var paths are case-sensitive in practice, and Windows uppercases env vars, which can break mixed-case nested lookups unless you account for it.
- Do not turn on broad `merge_enabled` behavior unless you want global merging semantics everywhere. Prefer targeted `@merge` or `dynaconf_merge` markers where you actually need it.
- Validators can fail because Dynaconf checks all declared environments by default. Use `validate_only_current_env=True` when only the active environment should be enforced.
- `dynaconf` CLI commands other than `init` need `-i config.settings` or `INSTANCE_FOR_DYNACONF`; they do not automatically discover your instance.
- `.secrets.*` is a local convenience, not a production secret-management strategy.
- For Django, keep the Dynaconf hook at the bottom of `settings.py`. Putting more settings code below it leads to confusing override order.

## Version-Sensitive Notes

- PyPI lists `3.2.12` as the current package version covered here.
- The upstream docs root still renders `3.2.11`, so treat release-specific bugfix details from the docs site carefully when troubleshooting.
- PyPI marks `3.2.8` and `3.2.9` as yanked; avoid pinning those versions in reproductions or templates.
- The CLI reference documents `dynaconf inspect` as a tech-preview command. Do not build automation that assumes its output is stable.

## Official Sources

- Docs: `https://www.dynaconf.com/`
- Configuration reference: `https://www.dynaconf.com/configuration/`
- Environment variables: `https://www.dynaconf.com/envvars/`
- Validation: `https://www.dynaconf.com/validation/`
- Django integration: `https://www.dynaconf.com/django/`
- Flask integration: `https://www.dynaconf.com/flask/`
- Secrets and loaders: `https://www.dynaconf.com/secrets/`
- CLI: `https://www.dynaconf.com/cli/`
- PyPI: `https://pypi.org/project/dynaconf/`
