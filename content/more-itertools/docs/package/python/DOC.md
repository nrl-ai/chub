---
name: package
description: "more-itertools package guide for Python iterable helpers, recipes, and advanced iterator utilities"
metadata:
  languages: "python"
  versions: "10.8.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "more-itertools,itertools,iterators,iterables,python,utilities"
---

# more-itertools Python Package Guide

## Golden Rule

Use `more-itertools` when the standard `itertools` module is close but not quite enough. Install the package as `more-itertools`, import it as `more_itertools`, and assume most helpers either consume an iterator or cache part of it unless the docs say otherwise.

## Install

Pin the version your project expects:

```bash
python -m pip install "more-itertools==10.8.0"
```

Common alternatives:

```bash
uv add "more-itertools==10.8.0"
poetry add "more-itertools==10.8.0"
```

## Setup

There is no client initialization, auth, or environment configuration. Import the helpers you need directly:

```python
from more_itertools import chunked, windowed

rows = list(chunked(range(7), 3))
windows = list(windowed([1, 2, 3, 4], 2))
```

If you use many helpers in one file, the module alias is convenient:

```python
import more_itertools as mit

print(list(mit.chunked("ABCDEFG", 3)))
print(list(mit.sliding_window(range(5), 3)))
```

## Core Usage

### Group items into batches

Use `chunked()` for generic iterables when you want lists:

```python
from more_itertools import chunked

for batch in chunked(range(8), 3):
    print(batch)

# [0, 1, 2]
# [3, 4, 5]
# [6, 7]
```

Set `strict=True` when partial final batches should be rejected:

```python
from more_itertools import chunked

list(chunked(range(8), 3, strict=True))
# raises ValueError because the last chunk is short
```

Use `sliced()` for sliceable sequences such as tuples, strings, and lists when you want tuple slices instead of list batches:

```python
from more_itertools import sliced

print(list(sliced((1, 2, 3, 4, 5), 2)))
# [(1, 2), (3, 4), (5,)]
```

### Build sliding windows

Use `windowed()` when you want padded windows or a `step` argument:

```python
from more_itertools import windowed

print(list(windowed([1, 2, 3, 4, 5], 3)))
# [(1, 2, 3), (2, 3, 4), (3, 4, 5)]

print(list(windowed([1, 2, 3], 4)))
# [(1, 2, 3, None)]
```

Use `sliding_window()` when incomplete windows should be dropped instead of padded:

```python
from more_itertools import sliding_window

print(list(sliding_window(range(6), 4)))
# [(0, 1, 2, 3), (1, 2, 3, 4), (2, 3, 4, 5)]
```

### Peek ahead without consuming values

`peekable()` wraps an iterator so you can look ahead, prepend items, and check truthiness for exhaustion:

```python
from more_itertools import peekable

tokens = peekable(iter(["SELECT", "*", "FROM"]))

if tokens and tokens.peek() == "SELECT":
    print(next(tokens))

tokens.prepend("WITH")
print(list(tokens))
# ['WITH', '*', 'FROM']
```

For one-time inspection without changing the iterable contract, `spy()` is often simpler:

```python
from more_itertools import spy

head, it = spy(range(5), 2)
print(head)
print(list(it))
```

### Partition an iterable by key

`bucket()` groups a source iterable into child iterables keyed by a function:

```python
from more_itertools import bucket

items = ["a1", "b1", "a2", "c1", "b2"]
grouped = bucket(items, key=lambda item: item[0])

print(list(grouped["a"]))
print(list(grouped["b"]))
```

Use a `validator` when probing keys that might not exist, especially for long or infinite iterables:

```python
from itertools import count, islice
from more_itertools import bucket

odds = bucket(
    count(1, 2),
    key=lambda n: n % 10,
    validator=lambda key: key in {1, 3, 5, 7, 9},
)

print(list(islice(odds[3], 3)))
print(list(odds[2]))
```

### Rewind an iterator

Use `seekable()` when you need to revisit already-consumed values:

```python
from more_itertools import seekable

it = seekable(str(n) for n in range(5))

print(next(it), next(it))
it.seek(0)
print(list(it))
```

This is useful when a parser needs a second pass over a streamed token source and you cannot materialize the whole input up front.

## Configuration And Environment

- No API keys, credentials, or environment variables are required.
- The package is pure utility code; behavior is controlled entirely by function arguments.
- If reproducibility matters, pin the package version explicitly because helper semantics do change across major releases.

## Common Pitfalls

- The package name uses a hyphen, but the import uses an underscore: `pip install more-itertools`, then `import more_itertools`.
- Iterator helpers usually consume input. Do not expect to reuse a generator after passing it to `chunked()`, `windowed()`, `bucket()`, or similar helpers.
- `peekable`, `seekable`, `bucket`, `ichunked`, and `distribute` can cache data internally. On large or infinite streams, this can become a memory problem.
- `sliced()` only works with objects that support slicing. For generic iterators, use `chunked()` instead.
- `divide()` preserves order but exhausts the input before returning; `distribute()` starts streaming sooner but uses `itertools.tee()` and changes the grouping order.
- `windowed()` pads short inputs with `fillvalue`; `sliding_window()` yields nothing when the input is shorter than the window size.
- `chunked()` returns lists. If you need tuple batches or want parity with newer stdlib behavior, check whether `batched()` or `itertools.batched()` is the better fit.

## Version-Sensitive Notes

- `10.8.0` adds `derangements()`, `argmin()`, `argmax()`, `running_median()`, and `extract()`. Prefer these over custom helpers when you see them in upstream examples.
- `10.6.0` dropped official Python 3.8 support; current package metadata requires Python `>=3.9`.
- `10.0.0` changed `batched()` and `matmul()` to yield tuples instead of lists. If older code expects list batches, it will break.
- `10.0.0` also removed the `DeprecationWarning` from `batched()`. On newer Python versions, stdlib overlap is increasingly intentional, so prefer the stdlib when it already covers your use case.

## Official Sources

- Stable docs: `https://more-itertools.readthedocs.io/en/stable/`
- API reference: `https://more-itertools.readthedocs.io/en/stable/api.html`
- Version history: `https://more-itertools.readthedocs.io/en/stable/versions.html`
- PyPI: `https://pypi.org/project/more-itertools/`
