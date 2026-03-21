---
name: package
description: "multidict package guide for Python projects that need ordered multi-value mappings and case-insensitive header containers"
metadata:
  languages: "python"
  versions: "6.7.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "multidict,python,mapping,http,headers,querystring,aio-libs"
---

# multidict Python Package Guide

## Golden Rule

Use `MultiDict` or `CIMultiDict` when a key may appear more than once and insertion order matters, such as HTTP headers or URL query parameters. `md[key]`, `get()`, and `getone()` return only the first matching value; use `getall()` when duplicate keys are expected.

## Install

`multidict 6.7.1` requires Python `>=3.9`.

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "multidict==6.7.1"
```

The project publishes wheels for Linux, Windows, and macOS. On unsupported platforms, or on Alpine Linux where a source build is used, installation needs a C compiler and Python headers.

If you need to skip the optional C extensions during install, set the documented environment variable before running `pip`:

```bash
export MULTIDICT_NO_EXTENSIONS=1
python -m pip install "multidict==6.7.1"
```

The maintainer docs note that the pure-Python build is much slower than the compiled extension, so prefer the default wheel install when performance matters.

## Choose The Right Type

- `MultiDict`: mutable mapping with duplicate keys and preserved insertion order.
- `CIMultiDict`: like `MultiDict`, but key lookup is case-insensitive.
- `MultiDictProxy`: read-only dynamic view over a `MultiDict`.
- `CIMultiDictProxy`: read-only dynamic view over a `CIMultiDict`.
- `istr`: string subclass used for case-insensitive key handling.

Keys must be `str` instances or subclasses of `str` such as `istr`.

## Initialize And Read Values

Use a sequence of `(key, value)` pairs when duplicates matter.

```python
from multidict import MultiDict

params = MultiDict([
    ("tag", "python"),
    ("tag", "asyncio"),
    ("page", "1"),
])

first_tag = params["tag"]
all_tags = params.getall("tag")
page = params.getone("page")
sort = params.get("sort", "created")

print(first_tag)
print(all_tags)
print(page)
print(sort)
print(list(params.items()))
```

`items()`, `keys()`, and `values()` include duplicate entries in insertion order.

## Add Values Without Replacing Existing Ones

Use `add()` for one value at a time, or `extend()` for multiple incoming pairs.

```python
from multidict import MultiDict

headers = MultiDict([("Accept", "application/json")])

headers.add("Accept", "text/plain")
headers.extend([
    ("X-Trace-Id", "req-1"),
    ("X-Trace-Id", "req-2"),
])

print(list(headers.items()))
print(headers.getall("Accept"))
print(headers.getall("X-Trace-Id"))
```

Use `extend()` when you want to keep existing values and append additional entries for the same key.

## Replace Existing Values For A Key

Use assignment when a key should end up with one value, and `update()` when incoming data should replace the current values for matching keys.

```python
from multidict import MultiDict

headers = MultiDict([
    ("Accept", "application/json"),
    ("Accept", "text/plain"),
    ("Authorization", "Bearer old-token"),
])

headers["Accept"] = "application/xml"
headers.update([
    ("Authorization", "Bearer new-token"),
    ("Accept", "application/json"),
    ("Accept", "text/csv"),
])

print(list(headers.items()))
print(headers.getall("Accept"))
```

Practical behavior:

- `headers[key] = value` removes other existing values for that key and keeps one value.
- `update(...)` replaces the current values for keys present in the incoming data.
- If the incoming `update(...)` payload contains the same key multiple times, those incoming values are preserved.

## Merge Only Missing Keys

Use `merge()` when you want to add values only for keys that are not already present.

```python
from multidict import MultiDict

headers = MultiDict([
    ("Accept", "application/json"),
    ("Authorization", "Bearer token"),
])

headers.merge([
    ("Accept", "text/plain"),
    ("User-Agent", "example-client/1.0"),
])

print(list(headers.items()))
```

In this example, `Accept` is left unchanged because the key already exists, while `User-Agent` is added.

## Case-Insensitive Headers With `CIMultiDict`

Use `CIMultiDict` for HTTP-style headers where lookup should ignore key casing.

```python
from multidict import CIMultiDict, istr

headers = CIMultiDict()
headers["Content-Type"] = "application/json"
headers.add("Set-Cookie", "a=1")
headers.add(istr("set-cookie"), "b=2")

content_type = headers["content-type"]
cookies = headers.getall("SET-COOKIE")

print(content_type)
print(cookies)
print("content-type" in headers)
print(list(headers.items()))
```

Lookup is case-insensitive, but the inserted key strings are preserved when you iterate items.

## Read-Only Views With Proxies

Use a proxy when callers should be able to read a multidict without mutating it.

```python
from multidict import MultiDict, MultiDictProxy

base = MultiDict([("tag", "python")])
view = MultiDictProxy(base)

base.add("tag", "asyncio")
snapshot = view.copy()
base.add("tag", "http")

print(view.getall("tag"))
print(snapshot.getall("tag"))
```

`MultiDictProxy` is a live view over the original object. Call `copy()` when you need a detached mutable `MultiDict` snapshot.

## Remove Values

Use `popone()` to remove the first matching value, `popall()` to remove all values for a key, and `del md[key]` to delete every occurrence of the key.

```python
from multidict import MultiDict

params = MultiDict([
    ("tag", "python"),
    ("tag", "asyncio"),
    ("page", "1"),
])

first_tag = params.popone("tag")
remaining_tags = params.popall("tag", [])
del params["page"]

print(first_tag)
print(remaining_tags)
print(list(params.items()))
```

## Runtime Configuration

`multidict` has no service authentication, API keys, or runtime client configuration.

The only documented environment variables are install-time switches:

- `MULTIDICT_NO_EXTENSIONS=1` disables the optional C extensions.
- `MULTIDICT_DEBUG_BUILD=1` builds extensions in debug mode for extension development.

## Common Pitfalls

- Use `getall()` for repeated keys. `md[key]`, `get()`, and `getone()` return only the first value.
- Use a list of pairs when duplicates matter. A plain `dict` or keyword arguments cannot represent repeated keys.
- Use `add()` or `extend()` to preserve existing values. Assignment and `update()` replace the current values for matching keys.
- Use `CIMultiDict` for HTTP headers. Plain `MultiDict` is case-sensitive.
- Treat `MultiDictProxy` as a live view, not as an immutable snapshot.
- Pass only `str` keys or subclasses of `str`; other key types raise `TypeError`.
- Prefer the compiled extension build for production performance unless you explicitly need the pure-Python fallback.

## Version-Sensitive Notes

- `6.7.1` requires Python `>=3.9`.
- PyPI classifiers for `6.7.1` include Python `3.9` through `3.14`.
- Binary wheels are published for Linux, Windows, and macOS; unsupported targets may need a source build.

## Practical Guidance For Agents

1. Use `MultiDict` for query strings, form fields, or any protocol surface where the same key may repeat.
2. Use `CIMultiDict` for HTTP headers so lookup is case-insensitive.
3. Use `add()` or `extend()` when preserving repeated values is intentional.
4. Use assignment or `update()` when the goal is to replace a key's existing values.
5. Use `getall()` before serializing or forwarding repeated keys to another API.

## Official Sources

- Maintainer docs: `https://multidict.aio-libs.org/en/stable/`
- Changelog: `https://multidict.aio-libs.org/en/latest/changes/`
- Source repository: `https://github.com/aio-libs/multidict`
- PyPI package page: `https://pypi.org/project/multidict/`
