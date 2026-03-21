---
name: package
description: "dirty-equals package guide for Python projects using declarative equality matchers in tests"
metadata:
  languages: "python"
  versions: "0.11"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "dirty-equals,python,testing,pytest,assertions,matchers"
---

# dirty-equals Python Package Guide

## Golden Rule

Use `dirty-equals` when you need readable assertions against nested or fuzzy data in tests. Import from `dirty_equals`, keep the matcher on the right-hand side of `==`, and prefer the built-in matcher types over custom ad hoc comparison helpers.

As of March 12, 2026, PyPI lists the current release as `0.11`, while the upstream docs brand the same release line as `v0.11.0`.

## Install

```bash
python -m pip install "dirty-equals==0.11"
```

Common alternatives:

```bash
uv add "dirty-equals==0.11"
poetry add "dirty-equals==0.11"
```

If you need the optional Pydantic-backed helpers such as richer URL validation, install the extra:

```bash
python -m pip install "dirty-equals[pydantic]==0.11"
```

## Setup And Test Runner Use

`dirty-equals` is a pure Python assertion helper library. There is no client object, service auth, or runtime initialization step. Install it into the same environment as your test runner and import matchers directly where you write assertions.

Basic pytest usage:

```python
from datetime import datetime, timezone

from dirty_equals import IsNow, IsPartialDict, IsPositiveInt, IsStr

def test_user_payload() -> None:
    payload = {
        "id": 42,
        "email": "alex@example.com",
        "created_at": datetime.now(timezone.utc),
        "debug": {"trace_id": "abc123", "raw": "..."},
    }

    assert payload == IsPartialDict(
        id=IsPositiveInt,
        email=IsStr(regex=r".+@example\.com$"),
        created_at=IsNow(delta=3, tz=timezone.utc),
    )
```

Use this style when exact full-object equality would make tests brittle.

## Core Matcher Patterns

### Partial and strict dictionaries

Use `IsPartialDict(...)` when the payload contains extra keys you do not care about. Use `IsStrictDict(...)` when key set and nested values must match exactly.

```python
from dirty_equals import IsPartialDict, IsStrictDict

assert {"id": 1, "name": "Ada", "active": True} == IsPartialDict(name="Ada")

assert {"name": "Ada", "active": True} == IsStrictDict(
    name="Ada",
    active=True,
)
```

### Lists, tuples, membership, and length

Use the sequence helpers instead of writing multiple index-by-index assertions.

```python
from dirty_equals import Contains, HasLen, IsList

assert ["alpha", "beta", "gamma"] == Contains("beta")
assert ["alpha", "beta", "gamma"] == HasLen(3)
assert [3, 2, 1] == IsList(1, 2, 3, check_order=False)
```

### Strings, JSON, and regex checks

`IsStr(...)` covers substring and regex checks. `IsJson(...)` is for asserting against JSON text or bytes and comparing the decoded value.

```python
from dirty_equals import IsJson, IsStr

assert "job-2026-03-12.log" == IsStr(regex=r"^job-\d{4}-\d{2}-\d{2}\.log$")
assert '{"status":"ok","count":2}' == IsJson({"status": "ok", "count": 2})
```

If you already have a parsed Python dict, compare it directly with `IsPartialDict(...)` or `IsStrictDict(...)` instead of wrapping it in `IsJson(...)`.

### Numeric, date/time, and type checks

Use the built-in fuzzy or constrained matchers instead of hand-written predicates.

```python
from datetime import datetime, timezone

from dirty_equals import IsApprox, IsInstance, IsNow, IsPositiveInt

assert 10.01 == IsApprox(10, delta=0.1)
assert 5 == IsPositiveInt
assert RuntimeError("boom") == IsInstance(RuntimeError)
assert datetime.now(timezone.utc) == IsNow(tz=timezone.utc, delta=2)
```

### Compose matchers with boolean logic

Matchers can be combined with `|`, `&`, and unary `~` for more precise assertions.

```python
from dirty_equals import IsInt, IsPositiveInt, IsStr

assert 7 == (IsInt & IsPositiveInt)
assert -1 == ~IsPositiveInt
assert "ok" == (IsStr(regex="^ok$") | IsInt)
```

## Configuration And Customization

There is no auth configuration. The main configuration surface is matcher construction and matcher settings.

### Class-style vs instance-style matchers

Many simple matchers can be used either as a class-like singleton or as an initialized matcher:

```python
from dirty_equals import IsPositiveInt, IsStr

assert 1 == IsPositiveInt
assert 1 == IsPositiveInt()
assert "abc" == IsStr
assert "abc" == IsStr(min_length=1)
```

Use an initialized matcher when you need arguments such as `regex=`, `delta=`, `tz=`, `check_order=False`, or `strict=False`.

### Per-matcher settings

Several matcher families expose a `.settings(...)` helper so you can flip defaults without rewriting the full matcher definition each time.

Common examples from the docs:

- `IsList.settings(check_order=False)` for order-insensitive list checks
- `IsDatetime.settings(enforce_tz=False)` when you explicitly want naive datetimes to pass

### Custom matchers

When the built-ins are not enough, subclass `DirtyEquals` and implement `equals`.

```python
from dirty_equals import DirtyEquals

class IsEven(DirtyEquals):
    def equals(self, other: object) -> bool:
        return isinstance(other, int) and other % 2 == 0

assert 4 == IsEven()
```

Keep custom matchers narrow and test-focused. Prefer composing existing matchers first.

## Common Pitfalls

- The package name uses a hyphen, but the import path uses an underscore: `pip install dirty-equals`, then `from dirty_equals import ...`.
- Put the matcher on the right side of `==`. The library overloads equality to make `actual == matcher` expressive and readable.
- Do not treat matcher classes that require arguments as zero-config singletons. `IsApprox(10)` and `IsNow(...)` must be initialized.
- `IsJson(...)` is for JSON text or bytes. If your code already parsed JSON into Python objects, compare those objects directly.
- `IsNow` and the datetime matchers are timezone-aware by default unless you explicitly relax that behavior. Be deliberate about `tz=` and `enforce_tz=`.
- `IsNumeric` cannot combine `approx` with comparison bounds like `gt`, `ge`, `lt`, or `le` in the same matcher.
- The docs note that some repr-rich error output is best in pytest and that instantiated types can display more clearly in complex failures.
- Upstream notes the project is not currently tested on PyPy.

## Version-Sensitive Notes For 0.11

- PyPI publishes the release as `0.11`; the docs header and GitHub release use `v0.11.0`. Treat them as the same release line.
- `0.11` requires Python `>=3.9` and publishes classifiers through Python `3.14`.
- The upstream docs were updated for the `v0.11.0` line on November 17, 2025. Prefer those docs over older blog posts that may predate the latest matcher APIs and Python version support.
- The optional `pydantic` extra in `0.11` targets Pydantic v2-era dependencies. If you use URL helpers that lean on Pydantic types, keep the extra and your Pydantic version aligned.

## Official Sources

- Docs root: https://dirty-equals.helpmanual.io/latest/
- Usage guide: https://dirty-equals.helpmanual.io/latest/
- Dictionary matchers: https://dirty-equals.helpmanual.io/latest/types/dict/
- Sequence matchers: https://dirty-equals.helpmanual.io/latest/types/other-sequences/
- String matchers: https://dirty-equals.helpmanual.io/latest/types/string/
- Inspection matchers: https://dirty-equals.helpmanual.io/latest/types/type-inspection/
- Other matchers: https://dirty-equals.helpmanual.io/latest/types/other/
- Numeric matchers: https://dirty-equals.helpmanual.io/latest/types/numeric/
- Datetime matchers: https://dirty-equals.helpmanual.io/latest/types/dates-times/
- PyPI metadata: https://pypi.org/project/dirty-equals/
- Repository metadata: https://github.com/samuelcolvin/dirty-equals/blob/main/pyproject.toml
