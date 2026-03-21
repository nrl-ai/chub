---
name: package
description: "Jinja2 package guide for Python templating, environment setup, and safe rendering"
metadata:
  languages: "python"
  versions: "3.1.6"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "jinja2,templating,html,python,sandbox"
---

# Jinja2 Python Package Guide

## What It Is

`Jinja2` is the Pallets template engine for Python. Use it when you need reusable text or HTML templates, template inheritance, macros, filters, tests, or sandboxed rendering.

- Package: `Jinja2`
- Import name: `jinja2`
- Version covered: `3.1.6`
- Docs family: `3.1.x`

## Installation

Install the pinned package version when you need behavior aligned with this guide:

```bash
pip install Jinja2==3.1.6
```

`MarkupSafe` is installed automatically. `Babel` is optional if you need template translation support.

## Initialization And Setup

For real applications, create one `Environment` during app startup and reuse it. Avoid creating ad hoc `Template(...)` instances for application templates because that uses a shared anonymous environment.

### Package-based templates

Use `PackageLoader` when templates live inside an importable Python package:

```python
from jinja2 import Environment, PackageLoader, StrictUndefined, select_autoescape

env = Environment(
    loader=PackageLoader("yourapp"),
    autoescape=select_autoescape(["html", "htm", "xml"]),
    undefined=StrictUndefined,
    trim_blocks=True,
    lstrip_blocks=True,
)
```

### Filesystem-based templates

Use `FileSystemLoader` when templates live in a project directory:

```python
from jinja2 import Environment, FileSystemLoader, StrictUndefined, select_autoescape

env = Environment(
    loader=FileSystemLoader("templates"),
    autoescape=select_autoescape(["html", "htm", "xml"]),
    undefined=StrictUndefined,
)
```

### String templates

Use `from_string()` for short dynamic templates, tests, or generated content:

```python
from jinja2 import Environment, StrictUndefined

env = Environment(undefined=StrictUndefined)
template = env.from_string("Hello {{ name }}!")
print(template.render(name="Ada"))
```

## Core Usage

### Render a template file

```python
template = env.get_template("emails/welcome.html")
html = template.render(user={"name": "Ada"}, project_name="Sample Portal")
```

### Inheritance and blocks

Jinja is strongest when you use a base template plus child templates:

```jinja
{# templates/base.html #}
<!doctype html>
<html>
  <body>
    {% block content %}{% endblock %}
  </body>
</html>
```

```jinja
{# templates/home.html #}
{% extends "base.html" %}

{% block content %}
  <h1>{{ title }}</h1>
{% endblock %}
```

```python
html = env.get_template("home.html").render(title="Dashboard")
```

### Includes, imports, and macros

Use `include` for partial templates and `import` / `from ... import` for macro libraries:

```jinja
{# templates/forms.html #}
{% macro input(name, value="", type="text") -%}
  <input type="{{ type }}" name="{{ name }}" value="{{ value|e }}">
{%- endmacro %}
```

```jinja
{% import "forms.html" as forms %}
{{ forms.input("email") }}
```

Important context rule:

- `include` gets the current context by default.
- Imported templates do not get the current context by default because imports are cached.
- Use `with context` explicitly if imported macros need access to current template variables.

### Custom filters, tests, and globals

Register these on the environment before loading templates:

```python
from markupsafe import Markup

def initials(value: str) -> str:
    return "".join(part[0].upper() for part in value.split() if part)

def is_even(value: int) -> bool:
    return value % 2 == 0

env.filters["initials"] = initials
env.tests["even"] = is_even
env.globals["cdn_host"] = "https://cdn.example.com"

template = env.from_string(
    "{{ user.name|initials }} {% if count is even %}even{% endif %}"
)
print(template.render(user={"name": "Ada Lovelace"}, count=4))
```

If a filter or test needs environment or context data, use `pass_environment`, `pass_eval_context`, or `pass_context`.

### Async rendering

Enable async mode only if your template calls async functions or iterates async values:

```python
from jinja2 import Environment

env = Environment(enable_async=True)
template = env.from_string("Hello {{ awaitable_name() }}")

async def awaitable_name():
    return "Ada"

result = await template.render_async(awaitable_name=awaitable_name)
```

With async enabled, Jinja compiles different code paths and `render_async()` / `generate_async()` become available.

### Native Python types

If you want template output to be a Python value instead of always a string, use `NativeEnvironment`:

```python
from jinja2.nativetypes import NativeEnvironment

env = NativeEnvironment()
template = env.from_string("{{ x + y }}")
result = template.render(x=4, y=2)

assert result == 6
assert isinstance(result, int)
```

This is useful for config generation or lightweight expression evaluation, not just HTML rendering.

## Configuration And Safety Notes

Jinja2 has no network authentication or service credentials. The main setup work is environment configuration.

### Autoescaping

Do not rely on defaults for HTML. Configure autoescaping explicitly:

```python
from jinja2 import Environment, select_autoescape

env = Environment(
    autoescape=select_autoescape(
        enabled_extensions=("html", "htm", "xml"),
        default_for_string=True,
    )
)
```

The upstream API docs explicitly encourage configuring autoescape now instead of relying on the default behavior.

### Undefined handling

The default undefined behavior can hide bugs because missing values often render as empty strings. Prefer:

```python
from jinja2 import StrictUndefined

env = Environment(undefined=StrictUndefined)
```

### Loader selection

- `PackageLoader`: templates shipped inside a Python package
- `FileSystemLoader`: templates from one or more directories
- `DictLoader`: unit tests and small in-memory fixtures
- `ChoiceLoader`: user override templates before default templates
- `ModuleLoader`: precompiled templates

### Caching and deployment

Useful production options:

- `cache_size`: in-memory template cache size, default `400`
- `auto_reload`: leave enabled in development; disable if source files never change in production
- `bytecode_cache`: add a `FileSystemBytecodeCache` or memcached-backed bytecode cache for repeated template compilation
- `compile_templates()`: precompile templates for deployment-time packaging

### Sandboxed rendering

Use `SandboxedEnvironment` only when you intentionally need to execute untrusted templates, and still keep the threat model narrow:

```python
from jinja2.sandbox import SandboxedEnvironment

env = SandboxedEnvironment(undefined=StrictUndefined)
```

The sandbox is not a complete security boundary. Upstream recommends passing only relevant data, avoiding objects with side-effecting methods, catching rendering errors, and using `ImmutableSandboxedEnvironment` if you must prevent template code from mutating lists and dicts.

## Common Pitfalls

### Autoescape is not universally on by default

If you render HTML without explicit autoescape configuration, you can create XSS problems or inconsistent escaping behavior.

### Do not mutate an environment after templates are loaded

The API docs warn that modifying filters, tests, globals, or configuration after the first template load can cause surprising or undefined behavior.

### `include` and `import` do not share context the same way

This is a frequent source of confusion:

- included templates see current context by default
- imported macro templates do not
- `with context` changes that behavior and disables import caching

### Default undefined values can hide missing data

If you do not set `StrictUndefined`, a typo such as `{{ user.nmae }}` may render as blank output instead of failing loudly.

### Whitespace output can look wrong even when logic is correct

Block tags on their own lines can leave blank lines unless you enable `trim_blocks` and `lstrip_blocks`, or use `-%}` whitespace control in the template.

### Sandbox is not a license to pass full application objects

Even with `SandboxedEnvironment`, do not pass global state, ORM objects with side effects, or broad service objects. Restrict the data shape you expose.

## Version-Sensitive Notes For 3.1.6

- `3.1.6` is a security release. The official release notes say it fixes the `|attr` filter so it no longer bypasses environment attribute lookup, which matters for sandboxed environments.
- `3.1.5` added more sandbox hardening and async-related fixes, including better handling for indirect `str.format` calls in sandboxed rendering and a correct `Environment.overlay(enable_async=...)` behavior.
- `3.1.0` removed older deprecated APIs. For modern code:
  - use `pass_context`, `pass_eval_context`, and `pass_environment` instead of the old `contextfilter` / `environmentfilter` style decorators
  - import `Markup` and `escape` from `markupsafe`, not from `jinja2`
  - treat autoescape and `with` as built-in behavior, not optional extensions
- The current official docs for the `3.1.x` line say Jinja supports Python `3.7` and newer.

## Official Sources

- Docs root: https://jinja.palletsprojects.com/en/stable/
- Introduction: https://jinja.palletsprojects.com/en/stable/intro/
- API: https://jinja.palletsprojects.com/en/stable/api/
- Template Designer Documentation: https://jinja.palletsprojects.com/en/stable/templates/
- Sandbox: https://jinja.palletsprojects.com/en/stable/sandbox/
- Native Python Types: https://jinja.palletsprojects.com/en/stable/nativetypes/
- Changelog: https://jinja.palletsprojects.com/en/stable/changes/
- PyPI package page: https://pypi.org/project/Jinja2/3.1.6/
- Source repository: https://github.com/pallets/jinja
- Release notes: https://github.com/pallets/jinja/releases/tag/3.1.6
