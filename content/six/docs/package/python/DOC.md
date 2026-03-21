---
name: package
description: "six package guide for Python 2 and 3 compatibility helpers"
metadata:
  languages: "python"
  versions: "1.17.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "python,six,compatibility,python2,python3,stdlib"
---

# six Python Package Guide

## Golden Rule

Use `six` only when the code must stay compatible across Python 2.7 and Python 3, or when an older dependency surface still expects `six` helpers. For Python 3-only code, prefer native Python 3 syntax and the standard library instead of adding new `six` usage.

## Installation

```bash
pip install six==1.17.0
```

```bash
uv add six==1.17.0
```

```bash
poetry add six==1.17.0
```

## Setup And Initialization

`six` has no service initialization, credentials, or config files.

```python
import six
from six.moves import urllib_parse
```

Notes:

- The package name and import name are both `six`.
- Upstream documents `six` as a single Python file, so vendoring is possible if a project deliberately embeds it. Keep the license notice if you do that.

## Core Usage

### Runtime Compatibility Checks And Shared Types

Use the built-in type tuples and flags instead of spelling out Python 2 and 3 branches yourself.

```python
import six

def normalize_value(value):
    if isinstance(value, six.string_types):
        return six.ensure_text(value).strip()
    if isinstance(value, six.integer_types):
        return value
    raise TypeError("unsupported value type")

if six.PY3:
    mode = "python3"
else:
    mode = "python2"
```

Useful helpers:

- `six.string_types`, `six.text_type`, `six.binary_type`
- `six.integer_types`, `six.class_types`
- `six.PY2`, `six.PY3`

### Text And Binary Normalization

When code must behave the same across Python 2 and 3, normalize at the boundary.

```python
import six

def to_wire_bytes(value):
    return six.ensure_binary(value, encoding="utf-8")

def to_text(value):
    return six.ensure_text(value, encoding="utf-8")

def to_native_str(value):
    return six.ensure_str(value, encoding="utf-8")
```

Practical guidance:

- `ensure_binary()` returns `six.binary_type`
- `ensure_text()` returns `six.text_type`
- `ensure_str()` returns native `str`
- Prefer explicit encoding arguments at I/O boundaries

### Dictionary Iteration

For code that still has to run on Python 2 and Python 3, use the iterator helpers instead of version-specific methods.

```python
import six

def serialize_headers(headers):
    return {six.ensure_text(k): six.ensure_text(v) for k, v in six.iteritems(headers)}
```

Common helpers:

- `six.iteritems()`
- `six.iterkeys()`
- `six.itervalues()`
- `six.viewitems()`, `six.viewkeys()`, `six.viewvalues()`

### Standard-Library Renames With `six.moves`

`six.moves` gives one import surface for stdlib modules that moved between Python 2 and 3.

```python
from six.moves import configparser, reload_module
from six.moves.urllib.parse import urlencode
from six.moves.urllib.request import urlopen

def fetch(url, params):
    query = urlencode(params)
    with urlopen(f"{url}?{query}") as response:
        return response.read()
```

Reach for `six.moves` for cases like:

- `configparser`
- `copyreg`
- `html_parser`
- `queue`
- `urllib.parse`, `urllib.request`, `urllib.error`, `urllib.response`
- `reload_module`

### Metaclasses And Decorators

If a shared library must support both runtimes and still needs metaclasses, use the helpers instead of branching class syntax.

```python
import six

class Meta(type):
    pass

class Base(object):
    pass

class Example(six.with_metaclass(Meta, Base)):
    pass
```

Also useful:

- `six.add_metaclass()`
- `six.wraps()`
- `six.raise_from()`
- `six.reraise()`

## Config And Auth

`six` does not define auth, environment variables, or global runtime configuration.

Agent guidance:

- There is nothing to initialize beyond importing the module.
- Any configuration comes from the application code that uses `six`, not from `six` itself.

## Common Pitfalls

- Do not add new `six` dependencies to Python 3-only code unless you specifically need to preserve a shared compatibility layer. Most helpers only matter when maintaining cross-version code.
- The Read the Docs site is still published as `six 1.15.0` even though PyPI's latest release is `1.17.0`. Treat the docs as the API reference, then check the changelog for deltas introduced after `1.15.0`.
- `six.moves` uses lazy proxy modules in `sys.modules`. Code that blindly inspects every module object can trip over those proxies.
- `six.u()` is only safe with ASCII literals on Python 2 because it does not know the source literal encoding.
- Prefer importing the exact thing you need from `six.moves` instead of mixing direct Python-version-specific stdlib imports with `six` shims in the same module.

## Version-Sensitive Notes For 1.17.0

- `1.17.0` removed `URLopener` and `FancyURLopener` from `six.moves.urllib.request` when running on Python `3.14+`. Do not write new code against those classes.
- `1.17.0` changed `six.moves.UserDict` on Python 2 to point at `UserDict.IterableUserDict` instead of `UserDict.UserDict`.
- `1.16.0` previously updated `_SixMetaPathImporter` for Python `3.10`, so older blog posts about import-hook breakage may no longer apply.
- `1.15.0` optimized `ensure_str()` and `ensure_binary()`, and the published docs site still reflects that doc build generation.

## When To Reach For `six`

Use `six` when:

- a library still supports Python 2.7 and Python 3
- a dependency or vendored codebase already uses `six`
- you need `six.moves` imports instead of hand-written Python 2/3 import branches

Avoid new `six` usage when:

- the project is Python 3-only
- standard library names already match your supported interpreter range
- compatibility branches can be deleted rather than preserved

## Official Sources

- Docs: https://six.readthedocs.io/
- PyPI: https://pypi.org/project/six/
- Repository: https://github.com/benjaminp/six
- Changelog: https://raw.githubusercontent.com/benjaminp/six/master/CHANGES
