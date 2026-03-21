---
name: package
description: "MarkupSafe package guide for Python safe HTML escaping and markup handling"
metadata:
  languages: "python"
  versions: "3.0.3"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "markupsafe,python,html,escaping,security,templating"
---

# MarkupSafe Python Package Guide

## What It Is

`markupsafe` escapes untrusted text for HTML and XML and tracks whether a value is already safe. Its core types are:

- `escape()` for turning untrusted input into escaped HTML-safe text
- `Markup` for values that are already safe to insert into HTML/XML
- `escape_silent()` for optional values that may be `None`
- `soft_str()` for preserving an existing `Markup` value when converting to text

Use it when generating HTML fragments directly, writing framework integrations that expose HTML, or preserving safe template output without double-escaping it.

## Install

`3.0.3` requires Python 3.9 or newer.

```bash
pip install markupsafe==3.0.3
```

Common imports:

```python
from markupsafe import Markup, escape, escape_silent, soft_str
```

## Setup Model

There is no service initialization, config file, or authentication layer.

The main setup decision is whether a value is:

- untrusted text that must be escaped with `escape()`
- trusted markup you intentionally wrap with `Markup(...)`
- an object that provides safe HTML via `__html__()` or `__html_format__()`

If that distinction is unclear, default to `escape()`.

## Core Usage

### Escape untrusted input

```python
from markupsafe import escape

user_input = '<script>alert("xss")</script>'
safe = escape(user_input)

html = f"<p>{safe}</p>"
print(html)
# <p>&lt;script&gt;alert(&#34;xss&#34;)&lt;/script&gt;</p>
```

`escape()` returns a `Markup` instance, not a plain `str`. Once a value is `Markup`, combining it with more text will escape the new text automatically.

### Build HTML fragments safely

```python
from markupsafe import Markup

name = 'Alice <Admin>'
html = Markup("<strong>%s</strong>") % name

print(html)
# <strong>Alice &lt;Admin&gt;</strong>
```

`Markup` is a `str` subclass. Formatting and concatenation return `Markup` and escape inserted values.

### Handle optional values

```python
from markupsafe import Markup, escape_silent

nickname = None
html = Markup("<span>%s</span>") % escape_silent(nickname)

print(html)
# <span></span>
```

Use `escape_silent()` when `None` should render as an empty string instead of the literal text `"None"`.

### Preserve safe strings when normalizing values

```python
from markupsafe import escape, soft_str

value = escape("<User 1>")

double_escaped = escape(str(value))
correct = escape(soft_str(value))

print(double_escaped)  # &amp;lt;User 1&amp;gt;
print(correct)         # &lt;User 1&gt;
```

Use `soft_str()` instead of `str()` if the value may already be `Markup`.

### Implement HTML on your own objects

```python
from markupsafe import Markup, escape

class UserLink:
    def __init__(self, user_id: int, display_name: str) -> None:
        self.user_id = user_id
        self.display_name = display_name

    def __html__(self) -> str:
        return f'<a href="/users/{self.user_id}">{escape(self.display_name)}</a>'

html = Markup("%s") % UserLink(7, "<Admin>")
print(html)
# <a href="/users/7">&lt;Admin&gt;</a>
```

If an object defines `__html__()`, MarkupSafe treats that result as already safe and will not escape it again.

### Use `format()` when HTML-specific formatting matters

```python
from markupsafe import Markup

template = Markup("<p>User: {name}</p>")
html = template.format(name='"World"')

print(html)
# <p>User: &#34;World&#34;</p>
```

`Markup.format()` escapes inserted values. If you need format-spec behavior for safe HTML objects, implement `__html_format__()`.

### Convert markup back to plain text

```python
from markupsafe import Markup

value = Markup("Main &raquo; <em>About</em>")

print(value.unescape())   # Main » <em>About</em>
print(value.striptags())  # Main » About
```

Use:

- `unescape()` when you want text with HTML entities resolved
- `striptags()` when you want display text without tags

## Config And Auth

There are no package-specific environment variables, auth flows, credentials, or runtime config objects.

The only meaningful policy decision is your trust boundary:

- call `escape()` for untrusted or unknown text
- only create `Markup(...)` from literals or already-sanitized HTML you trust
- keep `__html__()` and `__html_format__()` implementations strict and reviewable

## Common Pitfalls

### `Markup(...)` does not escape input

This is the most important footgun.

```python
from markupsafe import Markup, escape

raw = "<script>alert(1)</script>"

unsafe = Markup(raw)
safe = escape(raw)
```

`Markup(raw)` marks the text safe as-is. It is only correct if `raw` is already trusted HTML.

### Plain string interpolation is still unsafe

This is wrong if `user_input` is untrusted:

```python
html = f"<p>{user_input}</p>"
```

Escape first, or format through `Markup`.

### `__html__()` bypasses normal escaping

If your object returns HTML from `__html__()`, any user-controlled fields inside that output must be escaped manually.

### `str()` can lose the safe-string marker

Converting `Markup` to plain `str` and then escaping again can double-escape entities. Prefer `soft_str()`.

### Do not assume all `Markup` string methods still escape arguments

In `3.0.0`, some `str`-style methods stopped escaping their search/removal arguments:

- `strip`, `lstrip`, `rstrip`
- `removeprefix`, `removesuffix`
- `partition`, `rpartition`
- `replace` only escapes its `new` argument

Do not rely on these methods to sanitize input. Escape explicitly before mixing in user data.

### Prefer `importlib.metadata.version()` over `markupsafe.__version__`

`__version__` is deprecated in `3.0.x`. For version checks:

```python
from importlib.metadata import version

markupsafe_version = version("markupsafe")
```

## Version Notes For 3.0.3

- `3.0.3` is the package version covered by this doc.
- PyPI metadata for `3.0.3` requires Python `>=3.9`.
- `3.0.3` changes include a `DeprecationWarning` for `__version__` instead of `UserWarning`, plus packaging and wheel updates.
- `3.0.2` fixed compatibility when `__str__` returns a `str` subclass.
- `3.0.1` fixed compatibility with proxy objects.
- `3.0.0` dropped Python 3.7 and 3.8 support and changed argument-escaping behavior for some `Markup` string methods.

If code was written against `2.x`, review `3.0.0` changes before copying old examples.

## Official Sources

- Docs root: https://markupsafe.palletsprojects.com/en/stable/
- Working with safe text: https://markupsafe.palletsprojects.com/en/stable/escaping/
- HTML representations: https://markupsafe.palletsprojects.com/en/stable/html/
- String formatting: https://markupsafe.palletsprojects.com/en/stable/formatting/
- Changelog: https://markupsafe.palletsprojects.com/en/stable/changes/
- PyPI: https://pypi.org/project/markupsafe/
