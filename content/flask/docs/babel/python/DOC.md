---
name: babel
description: "Flask-Babel 4.0.0 guide for translations, locale selection, and locale-aware formatting in Flask apps"
metadata:
  languages: "python"
  versions: "4.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,flask-babel,i18n,l10n,translations,gettext,localization"
---

# Flask-Babel for Flask Apps

## Install

Pin the package version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "Flask-Babel==4.0.0"
```

Common alternatives:

```bash
uv add "Flask-Babel==4.0.0"
poetry add "Flask-Babel==4.0.0"
```

## Initialize In A Flask App

Use an application factory and pass selector functions to `Babel(...)` or `init_app(...)`.

```python
from flask import Flask, g, request
from flask_babel import Babel

babel = Babel()

def select_locale() -> str:
    user = getattr(g, "user", None)
    if user and user.locale:
        return user.locale
    return request.accept_languages.best_match(["en", "de", "fr"]) or "en"

def select_timezone() -> str:
    user = getattr(g, "user", None)
    if user and user.timezone:
        return user.timezone
    return "UTC"

def create_app() -> Flask:
    app = Flask(__name__)
    app.config.from_mapping(
        BABEL_DEFAULT_LOCALE="en",
        BABEL_DEFAULT_TIMEZONE="UTC",
        BABEL_TRANSLATION_DIRECTORIES="translations",
        BABEL_DOMAIN="messages",
    )

    babel.init_app(
        app,
        locale_selector=select_locale,
        timezone_selector=select_timezone,
    )

    return app
```

Important:

- Pass selectors during initialization. In current versions, `Babel.locale_selector` and `Babel.timezone_selector` are not runtime attributes you set later.
- Locale and timezone selectors are called once per request and cached.
- Flask-Babel expects your app to store internal datetimes in UTC and localize for display.

## Core Translation Usage

### Translate request-time strings

```python
from flask import Blueprint, flash, redirect, url_for
from flask_babel import gettext, ngettext

bp = Blueprint("main", __name__)

@bp.post("/invite")
def invite():
    sent = 3
    flash(gettext("Invitation sent"))
    flash(ngettext("%(count)d email queued", "%(count)d emails queued", sent, count=sent))
    return redirect(url_for("main.index"))
```

### Use lazy translations for class attributes and form labels

```python
from flask_babel import lazy_gettext as _
from wtforms import StringField

class ProfileForm:
    display_name = StringField(_("Display name"))
```

Use `lazy_gettext()` when the string is defined at import time. Plain `gettext()` there will resolve too early.

### Template translations

After `init_app()`, Flask-Babel registers gettext support and common Jinja filters by default.

```jinja2
<h1>{{ gettext("Welcome back") }}</h1>
<p>{{ invoice_total|currencyformat("USD") }}</p>
<p>{{ created_at|datetimeformat("medium") }}</p>
```

If you do not want Flask-Babel to modify the Jinja environment, initialize it with `configure_jinja=False`.

## Locale-Aware Date, Time, And Number Formatting

Use Flask-Babel helpers instead of manual `strftime()` calls for user-facing output:

```python
from datetime import datetime, timezone

from flask_babel import format_currency, format_datetime, format_decimal

now = datetime.now(timezone.utc)

formatted_date = format_datetime(now, "long")
formatted_price = format_currency(19.99, "USD")
formatted_ratio = format_decimal(12345.678)
```

Common helpers:

- `format_datetime()`
- `format_date()`
- `format_time()`
- `format_timedelta()`
- `format_number()`
- `format_decimal()`
- `format_currency()`
- `format_percent()`

## Translation Catalog Workflow

Flask-Babel uses gettext catalogs under the configured translation directories.

Typical layout:

```text
yourapp/
  translations/
    de/
      LC_MESSAGES/
        messages.po
        messages.mo
```

Minimal extraction config:

```ini
[python: **.py]
[jinja2: **/templates/**.html]
```

Extract, initialize, compile, and update catalogs with `pybabel`:

```bash
pybabel extract -F babel.cfg -k lazy_gettext -o messages.pot .
pybabel init -i messages.pot -d translations -l de
pybabel compile -d translations
pybabel update -i messages.pot -d translations
```

Notes:

- Add `-k lazy_gettext` if you use lazy strings.
- `pybabel compile` is required before the app can use updated translations.
- During development, you can have `flask run` watch compiled `.mo` files with `--extra-files` if you want reloads after recompiling.

## Configuration You Will Actually Use

Key config values in `4.0.0`:

- `BABEL_DEFAULT_LOCALE`: fallback locale, default `en`
- `BABEL_DEFAULT_TIMEZONE`: fallback timezone, default `UTC`
- `BABEL_TRANSLATION_DIRECTORIES`: semicolon-separated translation roots
- `BABEL_DOMAIN`: gettext domain, default `messages`

If you have multiple translation roots, `BABEL_DOMAIN` can also be semicolon-separated to match them positionally:

```python
app.config["BABEL_TRANSLATION_DIRECTORIES"] = "translations;plugins/payments/translations"
app.config["BABEL_DOMAIN"] = "messages;payments"
```

## Runtime Helpers

If the user changes locale or timezone during a request, refresh the cached values before rendering more translated output:

```python
from flask import flash, request
from flask_babel import gettext, refresh

def update_preferences(user):
    user.locale = request.form["locale"]
    user.timezone = request.form["timezone"]
    refresh()
    flash(gettext("Language was changed"))
```

Use `force_locale()` for one-off work in a different language, such as sending email:

```python
from flask_babel import force_locale, gettext

with force_locale("en_US"):
    subject = gettext("Your receipt")
```

`babel.list_translations()` is useful for locale pickers. In current versions it always includes the default locale, even if no compiled catalog exists for it.

## Common Pitfalls

- The docs landing page still says Flask-Babel only needs `Jinja >= 2.5`. For `4.0.0`, maintainer metadata requires `Jinja2 >= 3.1`.
- Do not mutate `babel.locale_selector` or `babel.timezone_selector` after startup. Pass selectors into `Babel(...)` or `init_app(...)`.
- Selectors are cached per request. If you update the active user's locale or timezone mid-request, call `refresh()` before generating more translated text.
- Forgetting `pybabel compile` leaves translated strings unchanged because Flask-Babel reads compiled `.mo` files, not raw `.po` files.
- `BABEL_TRANSLATION_DIRECTORIES` and `BABEL_DOMAIN` are semicolon-separated lists. If you provide multiple directories, keep the domain order aligned.
- `lazy_gettext()` is the safe choice for form labels, dataclass defaults, and other import-time constants.
- If you disable Jinja integration with `configure_jinja=False`, template gettext helpers and formatting filters will not be auto-registered.

## Version-Sensitive Notes For 4.0.0

- The version used here `4.0.0` still matches the current published package metadata and the docs site title as of March 12, 2026.
- The important selector API for modern Flask-Babel is `locale_selector=` and `timezone_selector=` during initialization. Older attribute-based patterns were removed in the 3.x line and should not be copied into 4.0.0 code.
- Current maintainer metadata requires Python 3.8+, Flask 2.0+, Babel 2.12+, Jinja2 3.1+, and pytz 2022.7+.
