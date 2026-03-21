---
name: benchmark
description: "pytest-benchmark package guide for Python projects using pytest performance benchmarks"
metadata:
  languages: "python"
  versions: "5.2.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest-benchmark,pytest,benchmark,performance,testing"
---

# pytest-benchmark Python Package Guide

## Golden Rule

Use `pytest-benchmark` only for code paths that are already covered by normal correctness tests, and benchmark the callable directly instead of wrapping it in extra Python layers. The plugin is designed for `pytest` suites and auto-calibrates rounds for microbenchmarks unless you explicitly switch to pedantic mode.

## Install

Pin the plugin version you expect in CI:

```bash
python -m pip install "pytest-benchmark==5.2.3"
```

Common variants:

```bash
uv add --dev "pytest-benchmark==5.2.3"
poetry add --group dev "pytest-benchmark==5.2.3"
```

Optional extras from PyPI:

```bash
python -m pip install "pytest-benchmark[histogram]==5.2.3"
python -m pip install "pytest-benchmark[aspect]==5.2.3"
python -m pip install "pytest-benchmark[elasticsearch]==5.2.3"
```

- `histogram`: plotting support for `--benchmark-histogram`
- `aspect`: experimental `benchmark.weave(...)` patch utilities via `aspectlib`
- `elasticsearch`: storing benchmark runs in Elasticsearch

## Initialize In A Pytest Suite

Once installed, the plugin registers the `benchmark` fixture automatically.

Basic benchmark:

```python
import json

def parse_payload(payload: str) -> dict:
    return json.loads(payload)

def test_parse_payload(benchmark):
    payload = '{"name": "context-hub", "rank": 458}'
    result = benchmark(parse_payload, payload)
    assert result["name"] == "context-hub"
```

Run only benchmark tests:

```bash
pytest --benchmark-only
```

Run the full suite but disable stats collection while still executing the benchmarked code once:

```bash
pytest --benchmark-disable
```

Skip benchmark tests entirely:

```bash
pytest --benchmark-skip
```

## Project-Level Configuration

There is no auth setup. Configuration is local to pytest through CLI flags, markers, and your pytest config.

Typical `pyproject.toml` setup:

```toml
[tool.pytest.ini_options]
addopts = [
  "--benchmark-autosave",
  "--benchmark-sort=mean",
  "--benchmark-columns=min,max,mean,stddev,median,iqr,ops,rounds,iterations",
]
```

Useful CLI options from the official docs:

- `--benchmark-min-time=SECONDS`: minimum time per round
- `--benchmark-max-time=SECONDS`: max total time per test, default `1.0`
- `--benchmark-min-rounds=NUM`: lower bound on round count, default `5`
- `--benchmark-disable-gc`: disable GC during measurement
- `--benchmark-warmup[=auto|on|off]`: enable warmup, default `auto`
- `--benchmark-warmup-iterations=NUM`: cap warmup iterations, default `100000`
- `--benchmark-time-unit=ns|us|ms|s`: force display unit

Per-test tuning uses the `benchmark` marker:

```python
import pytest

def parse_payload(payload: str) -> dict:
    return {"raw": payload}

@pytest.mark.benchmark(
    group="json",
    min_time=0.05,
    max_time=0.5,
    min_rounds=10,
    disable_gc=True,
    warmup=False,
)
def test_parse_payload(benchmark):
    benchmark(parse_payload, '{"name": "context-hub"}')
```

## Core Usage Patterns

### Benchmark the final callable directly

This is the preferred microbenchmark pattern because extra wrapper calls distort very fast measurements:

```python
def test_tokenize(benchmark):
    text = "alpha beta gamma"
    tokens = benchmark(str.split, text)
    assert tokens == ["alpha", "beta", "gamma"]
```

### Keep correctness assertions outside the measured section

The fixture returns the callable result, so assert afterward:

```python
import re

def test_compile_regex(benchmark):
    compiled = benchmark(re.compile, r"pytest-benchmark")
    assert compiled.pattern == r"pytest-benchmark"
```

### Save and compare benchmark history

Store runs in the default `.benchmarks/` directory:

```bash
pytest --benchmark-autosave
pytest --benchmark-save=baseline
pytest --benchmark-compare=0001
pytest --benchmark-compare-fail=mean:5%
```

For saved runs outside pytest, use the companion CLI:

```bash
pytest-benchmark compare 0001 0002
```

### Export JSON or histograms

```bash
pytest --benchmark-json=artifacts/benchmark.json
pytest --benchmark-histogram=artifacts/bench
```

If you want histogram output, install the `histogram` extra first.

### Attach extra metadata to saved JSON

```python
def build_index(path: str) -> dict:
    return {"path": path}

def test_search_index(benchmark):
    benchmark.extra_info["dataset"] = "fixtures/search-v2.json"
    benchmark.extra_info["scenario"] = "cold-cache"
    benchmark(build_index, "fixtures/search-v2.json")
```

## Pedantic Mode

Use `benchmark.pedantic(...)` when you need fixed rounds and iterations instead of auto calibration.

```python
def encode_record(value: bytes) -> str:
    return value.hex()

def test_encode_record(benchmark):
    payload = b"context-hub"
    benchmark.pedantic(
        encode_record,
        args=(payload,),
        iterations=100,
        rounds=50,
        warmup_rounds=5,
    )
```

Use `setup` when each round needs fresh inputs:

```python
def sort_items(items: list[int]) -> list[int]:
    return sorted(items)

def test_sort_items(benchmark):
    def setup():
        return ([5, 1, 4, 2, 3],), {}

    benchmark.pedantic(sort_items, setup=setup, rounds=100)
```

Use `teardown` when each round leaves side effects that need cleanup:

```python
def append_item(target: list[int], value: int) -> None:
    target.append(value)

def test_append_item(benchmark):
    values = []

    def teardown(target, value):
        target.clear()

    benchmark.pedantic(
        append_item,
        args=(values, 1),
        teardown=teardown,
        rounds=100,
    )
```

## Storage And Comparison

Saved runs go to `file://./.benchmarks` by default. You can override that with `--benchmark-storage`.

Examples:

```bash
pytest --benchmark-storage=file://./artifacts/benchmarks --benchmark-autosave
pytest --benchmark-storage=elasticsearch+https://host1,host2/index/doctype?project_name=ContextHub --benchmark-save=main
```

Use file storage unless you already have a reason to centralize benchmark history.

## Common Pitfalls

- One benchmark test should measure one callable. If you want to compare implementations, parametrize separate tests instead of benchmarking multiple functions in one test.
- Avoid benchmarking I/O-heavy or non-deterministic code if you want stable regressions. The official FAQ calls out VMs, background services, external resources, GC, and JIT effects as common sources of high `StdDev`.
- `benchmark(...)` is usually better than the decorator form for microbenchmarks because wrapper call overhead can skew very fast timings.
- `pytest-xdist` and benchmarks are a bad combination. `pytest-benchmark` auto-disables benchmarks when xdist is enabled by design; run benchmarks in a dedicated non-xdist job.
- In pedantic mode, the default `iterations=1` is unsafe for functions faster than roughly `100us`. Increase `iterations` or prefer normal auto-calibrated mode.
- If you supply a pedantic `setup` function, do not also pass `args`, `kwargs`, or `iterations`.
- `--benchmark-disable` still executes the benchmarked callable once. Use `--benchmark-skip` if you need to avoid the run entirely.
- If you must benchmark on a noisy machine or VM, the official FAQ suggests trying a custom timer such as `time.process_time`, because it excludes sleeping and waiting for I/O.
- When `StdDev` is noisy, compare `median` and `iqr` instead of treating `min` or `stddev` as the only signal.

## Version-Sensitive Notes For 5.2.3

- `5.2.3` adds support for `pytest 9.0`.
- `5.2.2` fixes benchmark auto-disable behavior with newer `pytest-xdist`.
- `5.2.1` adds markers so pytest does not try to assert-rewrite plugin internals, addressing `PytestAssertRewriteWarning` noise.
- `5.2.0` adds pedantic `teardown`, the `--benchmark-time-unit` option, and minimal typing support.
- `5.1.0` raises the minimum supported `pytest` version to `8.1`.
- `5.0.0` drops Python `3.8`; current PyPI metadata for `5.2.3` requires Python `>=3.9`.

## Official Sources

- Docs root: `https://pytest-benchmark.readthedocs.io/en/stable/`
- Installation: `https://pytest-benchmark.readthedocs.io/en/latest/installation.html`
- Usage: `https://pytest-benchmark.readthedocs.io/en/v5.2.3/usage.html`
- Calibration: `https://pytest-benchmark.readthedocs.io/en/stable/calibration.html`
- Pedantic mode: `https://pytest-benchmark.readthedocs.io/en/latest/pedantic.html`
- Comparing past runs: `https://pytest-benchmark.readthedocs.io/en/stable/comparing.html`
- FAQ: `https://pytest-benchmark.readthedocs.io/en/stable/faq.html`
- Changelog: `https://pytest-benchmark.readthedocs.io/en/latest/changelog.html`
- PyPI: `https://pypi.org/project/pytest-benchmark/`
