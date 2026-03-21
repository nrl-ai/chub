---
name: package
description: "Mako template library for Python projects using Template and TemplateLookup for file-based templating, inheritance, and escaping"
metadata:
  languages: "python"
  versions: "1.3.10"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "mako,python,templates,templating,html,web"
---

# Mako Python Package Guide

## Golden Rule

Use `Mako` when the project already depends on Mako or you need Python-powered templating with inheritance and defs. Import `Template` or `TemplateLookup` from `mako`, keep template source trusted, and enable explicit escaping instead of assuming HTML autoescape is on by default.

## Install

Pin the package version your project expects:

```bash
python -m pip install "Mako==1.3.10"
```

Common alternatives:

```bash
uv add "Mako==1.3.10"
poetry add "Mako==1.3.10"
```

If you only need the runtime, no extra initialization step is required after installation.

## Core Usage

### Render a string template

Use `Template` for inline or one-off templates:

```python
from mako.template import Template

template = Template("Hello ${name}!")
result = template.render(name="Ada")

print(result)
```

By default, `render()` returns a Python string unless you configure `output_encoding`. If you set an output encoding, `render()` returns `bytes`; use `render_unicode()` when you want text output consistently.

### Load templates from files

Use `TemplateLookup` for any real application with multiple template files, includes, or inheritance:

```python
from mako.lookup import TemplateLookup

lookup = TemplateLookup(
    directories=["templates"],
    module_directory=".mako_modules",
    filesystem_checks=True,
    strict_undefined=True,
)

template = lookup.get_template("users/detail.html")
html = template.render(user={"name": "Ada"})
```

Key options:

- `directories`: search roots for template URIs
- `module_directory`: compiled Python module cache; use a writable location
- `filesystem_checks`: `True` for development reloads, often `False` in production for less stat overhead
- `strict_undefined`: raise immediately for missing variables instead of letting `UNDEFINED` fail later during stringification

### Use Mako syntax correctly

Common syntax you will need:

```mako
<%page expression_filter="h"/>

% if user:
  <h1>${user["name"]}</h1>
% else:
  <h1>Anonymous</h1>
% endif

<%def name="badge(label)">
  <span class="badge">${label}</span>
</%def>

${badge("admin")}
```

Important constructs:

- `${expr}`: evaluate and emit an expression
- `% ...`: control lines such as `if`, `for`, and `while`
- `<% ... %>`: embedded Python block
- `<%def>`: reusable template function
- `<%block>` and `<%inherit file="...">`: layout composition and template inheritance

### Template inheritance

Use inheritance for layouts and `<%block>` for replaceable sections:

```mako
## base.html
<html>
  <body>
    <%block name="body"/>
  </body>
</html>
```

```mako
## child.html
<%inherit file="base.html"/>

<%block name="body">
  Hello ${name}
</%block>
```

```python
from mako.lookup import TemplateLookup

lookup = TemplateLookup(directories=["templates"])
html = lookup.get_template("child.html").render(name="Ada")
```

## Escaping And Output Configuration

Mako does not behave like Jinja's autoescape-by-default setups. If you output HTML, opt into escaping deliberately.

### Escape individual expressions

```mako
${user_input | h}
```

`h` applies HTML escaping. This is the safest default for untrusted values inserted into HTML text nodes.

### Set escaping for the whole template

Use page-level or lookup-level filters so you do not have to remember `| h` everywhere:

```mako
<%page expression_filter="h"/>
```

Or configure the lookup:

```python
from mako.lookup import TemplateLookup

lookup = TemplateLookup(
    directories=["templates"],
    default_filters=["str", "h"],
)
```

Useful filter notes:

- `expression_filter="h"` or `default_filters=["str", "h"]` is a practical HTML default
- `n` disables the default filter chain for a specific expression when you intentionally need raw output
- `str` is part of the default filter behavior in Python 3 and is why many old Python 2 era examples no longer match current behavior

### Return bytes when you need encoded output

```python
from mako.template import Template

template = Template(
    "Hello ${name}",
    output_encoding="utf-8",
    encoding_errors="replace",
)

payload = template.render(name="Ada")
assert isinstance(payload, bytes)
```

If the caller expects text, prefer `render_unicode()` or avoid `output_encoding`.

## Error Handling And Debugging

For development, turn on strict variable checking and readable exception formatting:

```python
from mako import exceptions
from mako.lookup import TemplateLookup

lookup = TemplateLookup(
    directories=["templates"],
    strict_undefined=True,
    format_exceptions=True,
)

try:
    html = lookup.get_template("users/detail.html").render(user=None)
except Exception:
    print(exceptions.text_error_template().render())
```

Useful exception helpers:

- `format_exceptions=True`: render HTML traceback output instead of a blank server error page in many web integrations
- `exceptions.text_error_template()`: readable text traceback
- `exceptions.html_error_template()`: HTML traceback page
- `exceptions.RichTraceback()`: structured traceback details when you need custom logging

## Configuration Notes

### Template lookup cache

`TemplateLookup` caches templates in memory and can also write compiled modules to disk. In production:

- keep `module_directory` on persistent writable storage
- disable `filesystem_checks` only when template files are deployed atomically and do not need live reload
- tune `collection_size` only if you create a very large number of distinct templates and need LRU-style eviction

### Imports and shared helpers

If templates need shared imports, configure them on the lookup instead of repeating imports in every file:

```python
from mako.lookup import TemplateLookup

lookup = TemplateLookup(
    directories=["templates"],
    imports=["from myapp.formatting import format_money"],
)
```

### Web framework integration

Most integrations still reduce to "build a `TemplateLookup`, then render a template with a context dict". Keep the lookup object process-wide when possible instead of rebuilding it per request.

## Common Pitfalls

- Do not treat template source as safe user input. Mako templates can contain arbitrary Python code, so template authors are effectively trusted-code authors.
- Missing variables become `UNDEFINED` by default and may fail later in a confusing place. Use `strict_undefined=True` for application templates.
- HTML escaping is not automatic unless you configure it. `Template("${value}")` will emit raw content.
- `render()` can return `bytes` when `output_encoding` is set. That is a common source of framework integration bugs.
- File-based features such as `<%inherit>`, `<%include>`, and URI lookups are much easier to manage through `TemplateLookup` than by instantiating `Template(filename=...)` ad hoc everywhere.
- If you disable `filesystem_checks`, template edits will not be picked up until the process restarts or the cache is invalidated.
- Old blog posts often use Python 2 examples or pre-`strict_undefined` workarounds. Prefer the current docs for filter behavior and undefined-variable handling.

## Version-Sensitive Notes For 1.3.10

- `Mako 1.3.10` is the current version on PyPI as of March 12, 2026.
- `1.3.10` fixes a `strict_undefined` bug involving nested list comprehensions, so projects that rely on strict undefined checking should prefer `1.3.10` over earlier `1.3.x` releases.
- PyPI marks `1.3.7` and `1.3.4` as yanked. Avoid pinning them in new work.
- If you are copying older examples, remember that Python 3 output/filter behavior assumes `str` in the default filter chain; current filtering docs reflect that behavior.

## Official Sources

- Docs: `https://docs.makotemplates.org/en/latest/`
- Usage and API patterns: `https://docs.makotemplates.org/en/latest/usage.html`
- Syntax reference: `https://docs.makotemplates.org/en/latest/syntax.html`
- Filtering and escaping: `https://docs.makotemplates.org/en/latest/filtering.html`
- Runtime behavior: `https://docs.makotemplates.org/en/latest/runtime.html`
- Changelog: `https://docs.makotemplates.org/en/latest/changelog.html`
- PyPI package page: `https://pypi.org/project/Mako/`
