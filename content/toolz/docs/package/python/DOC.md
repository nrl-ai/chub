---
name: package
description: "toolz Python package guide for functional utilities, iterable pipelines, and dictionary transforms"
metadata:
  languages: "python"
  versions: "1.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "toolz,python,functional,iterators,pipelines,dictionaries"
---

# toolz Python Package Guide

## Golden Rule

Use `toolz` when you need small, composable functional helpers over normal Python iterables, callables, and dictionaries. Favor `pipe()` for readable left-to-right pipelines, use `toolz.curried` when you want partial application inside a pipeline, and remember that many iterable helpers are lazy iterators that need to be consumed explicitly.

## Install

Pin the package version your project expects:

```bash
python -m pip install "toolz==1.1.0"
```

Common alternatives:

```bash
uv add "toolz==1.1.0"
poetry add "toolz==1.1.0"
```

`toolz` is pure Python. If a codebase explicitly wants the compiled sibling package for speed, that is usually `cytoolz`, but do not swap dependencies silently if the project is pinned to `toolz`.

## Setup And Import Patterns

`toolz` is a local utility library. There is no auth, service config, or runtime initialization step beyond installation and imports.

Common import styles:

```python
from toolz import compose, pipe
from toolz.dicttoolz import assoc_in, update_in, valmap
from toolz.itertoolz import groupby, pluck, reduceby
```

For pipeline-heavy code, the curried namespace is often cleaner:

```python
from toolz.curried import filter, map, partition_all, pipe
```

Use narrow imports in application code when you want clearer ownership of helpers. Use `toolz.curried` when the code is intentionally pipeline-oriented.

## Core Usage

### Build readable pipelines with `pipe`

`pipe(value, f, g, h)` applies functions left to right and is usually easier to read than nested calls.

```python
from toolz import pipe
from toolz.curried import filter, map, take

result = pipe(
    [1, 2, 3, 4, 5, 6],
    filter(lambda x: x % 2 == 0),
    map(lambda x: x * 10),
    take(2),
    list,
)

print(result)  # [20, 40]
```

### Reuse logic with `compose` and `curry`

Use `compose()` when you need a reusable function object. Composition runs right to left.

```python
from toolz import compose, curry

@curry
def clamp(low, high, value):
    return max(low, min(high, value))

normalize_score = compose(clamp(0, 100), int)

print(normalize_score("104"))  # 100
```

Use `pipe()` for one-off flows and `compose()` for reusable transformations.

### Prefer `reduceby` over `groupby` for large streams

`groupby()` collects every matching item into lists. That is convenient for small inputs, but it materializes the whole grouping in memory.

```python
from toolz.itertoolz import reduceby

rows = [
    {"team": "red", "points": 3},
    {"team": "blue", "points": 5},
    {"team": "red", "points": 7},
]

totals = reduceby(
    key=lambda row: row["team"],
    binop=lambda acc, row: acc + row["points"],
    seq=rows,
    init=0,
)

print(totals)  # {'red': 10, 'blue': 5}
```

For log-style or iterator-driven workloads, `reduceby()` is usually the safer default.

### Update nested dictionaries without manual copying

`dicttoolz` is useful when you want immutable-style updates to nested structures.

```python
from toolz.dicttoolz import assoc_in, update_in

config = {
    "db": {"host": "localhost", "port": 5432},
    "features": {"beta": False},
}

next_config = assoc_in(config, ["db", "host"], "db.internal")
next_config = update_in(next_config, ["db", "port"], lambda port: port + 1)

print(next_config)
```

Useful dictionary helpers:

- `assoc()` and `assoc_in()` return updated copies
- `update_in()` transforms an existing nested value
- `merge()` performs a shallow merge
- `valmap()` and `itemmap()` transform dictionary values or items

### Work with iterables lazily

Many `toolz.itertoolz` and `toolz.curried` helpers return iterators. Consume them with `list()`, `tuple()`, `set()`, `dict()`, `next()`, or a loop when you need actual values.

```python
from toolz.curried import map, partition_all, pipe

batches = pipe(
    range(10),
    map(lambda x: x + 1),
    partition_all(4),
    list,
)

print(batches)  # [(1, 2, 3, 4), (5, 6, 7, 8), (9, 10)]
```

Common iterable helpers agents reach for:

- `pluck("field", rows)` for extracting a key from each mapping
- `partition_all(n, seq)` for batching
- `concat(seqs)` and `concatv(*seqs)` for flattening
- `unique(seq)` for preserving first-seen order
- `frequencies(seq)` for counting values

## Configuration Notes

- There is no global runtime configuration file.
- The important setup decision is import style: plain namespace imports for explicitness, or `toolz.curried` for partial-application-heavy pipelines.
- `toolz` is synchronous. It works fine inside async code for local data transforms, but it does not provide async iterators, IO helpers, or scheduler integration.

## Common Pitfalls

- `compose()` runs functions right to left. If the code needs left-to-right readability, use `pipe()`.
- Helpers imported from `toolz.curried` often return lazy iterators. Materialize them before indexing, serializing, or reusing them twice.
- `groupby()` can consume large amounts of memory because it stores full lists per key. Use `reduceby()` or explicit streaming reductions for large inputs.
- `merge()` is shallow. For nested updates, use `assoc_in()` or `update_in()` instead of expecting a deep merge.
- Generators are one-shot. If a pipeline consumes an iterator once, later stages cannot replay it unless you re-create or cache it.
- `join()` is not a database engine. Be deliberate about which side is smaller and whether materialization is acceptable.
- `curry` changes call style. That is useful, but it can make debugging signatures less obvious if overused across a large codebase.

## Version-Sensitive Notes For `1.1.0`

- The version used here `1.1.0` matches the live PyPI release as of March 12, 2026.
- The repository metadata for `1.1.0` requires Python `>=3.9`, so older Python 3.8 environments need an older `toolz` release.
- `toolz` remains a lightweight utility package with stable core namespaces (`functoolz`, `itertoolz`, `dicttoolz`, and `curried`). For most code generation tasks, the bigger risk is copying examples that accidentally mix eager Python built-ins with lazy `toolz` iterators.

## Official Sources

- Docs root: https://toolz.readthedocs.io/en/latest/
- Installation docs: https://toolz.readthedocs.io/en/latest/install.html
- API index: https://toolz.readthedocs.io/en/latest/api.html
- Curry and curried namespace docs: https://toolz.readthedocs.io/en/latest/curry.html
- Streaming analytics notes: https://toolz.readthedocs.io/en/latest/streaming-analytics.html
- PyPI package page: https://pypi.org/project/toolz/
- Repository metadata: https://github.com/pytoolz/toolz/blob/master/pyproject.toml
