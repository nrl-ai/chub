---
name: package
description: "Hypothesis property-based testing library for Python projects"
metadata:
  languages: "python"
  versions: "6.151.9"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "hypothesis,property-based-testing,testing,pytest,fuzzing,python"
---

# Hypothesis Python Package Guide

## Golden Rule

Use Hypothesis to generate inputs and state transitions that exercise properties of your code, not to hard-code a long list of example test cases. Start with `pytest`, add `@given(...)`, and only tune settings when the default search becomes too slow or too strict for your test.

## Install

Basic install:

```bash
python -m pip install "hypothesis==6.151.9"
```

Common alternatives:

```bash
uv add "hypothesis==6.151.9"
poetry add "hypothesis==6.151.9"
```

Useful extras from the official docs:

```bash
python -m pip install "hypothesis[pytest]==6.151.9"
python -m pip install "hypothesis[cli]==6.151.9"
python -m pip install "hypothesis[numpy,pandas]==6.151.9"
python -m pip install "hypothesis[django]==6.151.9"
```

- `pytest` extra: install the pytest integration
- `cli` extra: install `hypothesis write`
- domain extras such as `numpy`, `pandas`, and `django`: install first-party extension support

## Minimal Setup

Hypothesis works with both `pytest` and `unittest`, but the common path is `pytest`.

```python
from hypothesis import given, strategies as st

@given(st.lists(st.integers()))
def test_reverse_round_trip(xs: list[int]) -> None:
    assert list(reversed(list(reversed(xs)))) == xs
```

Run the test with:

```bash
pytest
```

What Hypothesis does for you:

- generates many valid inputs from the strategy
- shrinks failures to a smaller counterexample
- stores failing examples in its local example database so they are retried first later

## Core Usage Patterns

### Basic strategies

Use `hypothesis.strategies` (`st`) to describe the space of inputs you want:

```python
from hypothesis import given, strategies as st

@given(
    st.dictionaries(
        keys=st.text(min_size=1, max_size=20),
        values=st.integers(min_value=0, max_value=100),
        max_size=20,
    )
)
def test_total_is_never_negative(counts: dict[str, int]) -> None:
    assert sum(counts.values()) >= 0
```

Common strategy building blocks:

- `st.integers()`, `st.floats()`, `st.text()`, `st.binary()`, `st.booleans()`
- `st.lists(...)`, `st.sets(...)`, `st.dictionaries(...)`, `st.tuples(...)`
- `st.none() | st.text()` or `st.one_of(...)` for alternatives
- `st.sampled_from(...)` for finite enums

### Build domain objects

Use `st.builds(...)` when your test targets constructors, dataclasses, or value objects:

```python
from dataclasses import dataclass

from hypothesis import given, strategies as st

@dataclass
class User:
    id: int
    name: str
    is_admin: bool = False

user_strategy = st.builds(
    User,
    id=st.integers(min_value=1),
    name=st.text(min_size=1, max_size=50),
)

@given(user_strategy)
def test_user_name_is_never_empty(user: User) -> None:
    assert user.name
```

### Use `from_type()` and `register_type_strategy()` for typed APIs

If your code already uses type annotations, `st.from_type(...)` can infer strategies for many annotated types. For project-specific classes, register a strategy once and reuse it:

```python
from hypothesis.strategies import register_type_strategy

register_type_strategy(User, user_strategy)
```

That lets later tests use `st.from_type(User)` without repeating the constructor logic.

### Use `st.data()` when later draws depend on earlier values

When you need dependent generation, draw values inside the test instead of encoding everything into one large strategy:

```python
from hypothesis import given, strategies as st

@given(st.data())
def test_pop_reduces_length(data) -> None:
    items = data.draw(st.lists(st.integers(), min_size=1, max_size=20))
    index = data.draw(st.integers(min_value=0, max_value=len(items) - 1))
    before = len(items)
    items.pop(index)
    assert len(items) == before - 1
```

## Configuration And Test Runner Integration

Hypothesis has no API credentials or service authentication. The main configuration surface is settings and runner integration.

Per-test settings:

```python
from hypothesis import given, settings, strategies as st

@settings(max_examples=200, deadline=None)
@given(st.lists(st.integers(), max_size=200))
def test_sort_is_idempotent(xs: list[int]) -> None:
    assert sorted(sorted(xs)) == sorted(xs)
```

Settings you will tune most often:

- `max_examples`: how many generated examples to try
- `deadline`: per-example execution deadline in milliseconds, or `None`
- `derandomize`: deterministic generation, useful for CI profiles
- `database`: where Hypothesis stores examples, or `None` to disable persistence
- `suppress_health_check`: narrow opt-out for specific health checks when justified

Project-wide profiles:

```python
from hypothesis import HealthCheck, settings

settings.register_profile(
    "ci",
    max_examples=500,
    derandomize=True,
    suppress_health_check=[HealthCheck.too_slow],
)

settings.load_profile("ci")
```

You can also select a profile with the environment variable:

```bash
export HYPOTHESIS_PROFILE=ci
pytest
```

Useful pytest flags from the official integration docs:

```bash
pytest --hypothesis-profile=ci
pytest --hypothesis-show-statistics
pytest --hypothesis-verbosity=verbose
pytest --hypothesis-seed=12345
```

## Examples, Seeds, And Failure Reproduction

Use the built-in reproduction tools before inventing your own debug harness.

### Add explicit examples

`@example(...)` forces specific inputs to run before generated cases:

```python
from hypothesis import example, given, strategies as st

@example([])
@example([0])
@given(st.lists(st.integers()))
def test_reversed_twice_is_original(xs: list[int]) -> None:
    assert list(reversed(list(reversed(xs)))) == xs
```

### Pin a deterministic seed

`@seed(...)` or `--hypothesis-seed=...` is useful when you want a repeatable run while investigating a flaky or performance-sensitive test:

```python
from hypothesis import given, seed, strategies as st

@seed(20260311)
@given(st.text())
def test_text_round_trip(s: str) -> None:
    assert s.encode("utf-8").decode("utf-8") == s
```

### Replay an exact failure

Hypothesis can emit `@reproduce_failure(...)` for a failing example. This is for short-lived local debugging only.

- It is tied to the exact Hypothesis version that generated it.
- It can stop working after an upgrade, downgrade, or even if internal encoding changes.
- Do not keep it in long-lived committed tests.

## Stateful Testing

For APIs with sequences of operations, use `RuleBasedStateMachine` instead of squeezing the whole workflow into one example:

```python
from hypothesis import strategies as st
from hypothesis.stateful import RuleBasedStateMachine, invariant, rule

class SetMachine(RuleBasedStateMachine):
    def __init__(self) -> None:
        super().__init__()
        self.items: set[int] = set()

    @rule(x=st.integers())
    def add_item(self, x: int) -> None:
        self.items.add(x)

    @invariant()
    def contents_match_python_set_rules(self) -> None:
        assert len(self.items) == len(set(self.items))

TestSetMachine = SetMachine.TestCase
```

Use this for caches, queues, parsers, protocol clients, and CRUD APIs where bugs only appear after a sequence of actions.

## CLI Ghostwriter

If you have the `cli` extra installed, Hypothesis can generate starter tests:

```bash
hypothesis write your_module.your_function
```

Use ghostwriter as scaffolding, not as the final test design. You still need to decide the actual invariants and edge cases that matter for your code.

## Common Pitfalls

- Avoid heavy use of `assume(...)` or `.filter(...)`. They can make generation inefficient and trigger health checks or unsatisfiable tests.
- Do not hide real bugs by broadly suppressing health checks. Suppress only the specific check you understand.
- Keep tests deterministic apart from Hypothesis input generation. Time, global state, randomness, filesystem races, and network calls are common sources of flaky failures.
- Prefer properties over implementation snapshots. Hypothesis is strongest when you assert invariants, round-trips, ordering rules, idempotence, or equivalence between implementations.
- Remember that the example database persists local failures. If behavior changes unexpectedly between runs, check whether `.hypothesis/` is replaying a previously found case.
- Install the right extra for framework-specific helpers such as NumPy, pandas, or Django support.

## Version-Sensitive Notes For 6.151.9

- The official PyPI release and the docs currently align on `6.151.9`.
- `hypothesis` now requires Python `>=3.10`; older project interpreters need an older Hypothesis release.
- `@reproduce_failure(...)` is intentionally version-specific and should not be treated as a stable fixture format.
- The built-in profile mechanism and `HYPOTHESIS_PROFILE` environment variable are the preferred way to keep local and CI settings aligned without hard-coding settings in every test.

## Official Sources

- Docs root: https://hypothesis.readthedocs.io/en/latest/
- Quickstart: https://hypothesis.readthedocs.io/en/latest/quickstart.html
- API reference: https://hypothesis.readthedocs.io/en/latest/reference/api.html
- Strategies reference: https://hypothesis.readthedocs.io/en/latest/reference/strategies.html
- Pytest integration and CLI: https://hypothesis.readthedocs.io/en/latest/integrations.html
- First-party extensions: https://hypothesis.readthedocs.io/en/latest/extensions.html
- Settings tutorial: https://hypothesis.readthedocs.io/en/latest/tutorial/settings.html
- Flaky failures: https://hypothesis.readthedocs.io/en/latest/tutorial/flaky.html
- Package registry: https://pypi.org/project/hypothesis/
