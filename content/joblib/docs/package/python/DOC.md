---
name: package
description: "Joblib Python package guide for caching, parallel loops, and object persistence"
metadata:
  languages: "python"
  versions: "1.5.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "joblib,python,parallelism,caching,persistence,numpy"
---

# Joblib Python Package Guide

## Golden Rule

Use `joblib` for three things: caching pure function results with `Memory`, running simple parallel loops with `Parallel` and `delayed`, and persisting large Python or NumPy-heavy objects with `dump` and `load`. Keep the default `loky` backend for generic Python work, prefer threads when the workload releases the GIL, and never `joblib.load()` data from an untrusted source.

## Install

Pin the version your project expects:

```bash
python -m pip install "joblib==1.5.3"
```

Common alternatives:

```bash
uv add "joblib==1.5.3"
poetry add "joblib==1.5.3"
```

Useful dependency notes from upstream:

- `joblib` has no mandatory dependencies beyond Python.
- `numpy` is optional but is a common companion and unlocks the main large-array optimizations described in the docs.
- `lz4` is optional and makes `.lz4` compressed persistence available automatically.
- `psutil` is optional and can help mitigate memory leaks in parallel worker processes.

## Initialize And Setup

There is no auth or remote service setup. The main setup decision is where caches and temporary worker files should live.

Typical imports:

```python
from joblib import Memory, Parallel, delayed, dump, load, parallel_config
```

Typical local cache setup:

```python
from pathlib import Path
from joblib import Memory

cache_dir = Path(".cache/joblib")
memory = Memory(cache_dir, verbose=0)
```

Use a cache directory that is safe to delete and not intended for manual editing. `Memory` stores pickled results on disk.

## Core Usage

### Cache expensive pure functions with `Memory`

Decorate a function once and joblib will reuse the stored result for matching inputs:

```python
from pathlib import Path
from joblib import Memory

memory = Memory(Path(".cache/joblib"), verbose=0)

@memory.cache
def build_features(values: list[int]) -> list[int]:
    print("computing")
    return [value * value for value in values]

first = build_features([1, 2, 3])   # computes
second = build_features([1, 2, 3])  # loads from cache
```

Use `ignore=[...]` for arguments that should not affect cache keys:

```python
@memory.cache(ignore=["debug"])
def transform(data_path: str, debug: bool = False) -> str:
    return data_path.upper()
```

Use cache validation when cached results expire based on time or external state:

```python
from joblib import Memory, expires_after

memory = Memory(".cache/joblib", verbose=0)

@memory.cache(cache_validation_callback=expires_after(hours=1))
def fetch_snapshot() -> dict:
    return {"status": "ok"}
```

### Run parallel loops with `Parallel` and `delayed`

Basic process-based parallelism:

```python
from joblib import Parallel, delayed

def score_item(x: int) -> int:
    return x * x

results = Parallel(n_jobs=-1)(
    delayed(score_item)(x) for x in range(10)
)
```

Prefer threads for I/O-heavy or native-code workloads that release the GIL:

```python
from joblib import Parallel, delayed

results = Parallel(n_jobs=8, prefer="threads")(
    delayed(score_item)(x) for x in range(100)
)
```

If you want results incrementally instead of materializing the whole list at once:

```python
from joblib import Parallel, delayed

parallel = Parallel(n_jobs=4, return_as="generator")
for result in parallel(delayed(score_item)(x) for x in range(20)):
    print(result)
```

### Control backend defaults with `parallel_config`

Use `parallel_config()` when the parallel call is buried inside another library or when you want a scoped override without hard-coding backend choices in reusable code:

```python
from joblib import Parallel, delayed, parallel_config

with parallel_config(
    backend="loky",
    n_jobs=4,
    inner_max_num_threads=1,
    temp_folder="/tmp/joblib",
):
    results = Parallel()(delayed(score_item)(x) for x in range(20))
```

`inner_max_num_threads` matters when NumPy, BLAS, or similar native libraries spawn their own thread pools inside each worker.

### Persist Python objects with `dump` and `load`

Use joblib persistence when pickled objects contain large arrays or when compressed storage is convenient:

```python
from joblib import dump, load

payload = {"weights": [1, 2, 3], "name": "model-a"}

dump(payload, "artifacts/model.joblib")
restored = load("artifacts/model.joblib")
```

Compression is built in:

```python
from joblib import dump

dump(payload, "artifacts/model.joblib.gz", compress=("gzip", 3))
```

For large NumPy payloads, the standard library `pickle` protocol 5 can also be a reasonable alternative on Python 3.8+ when you do not need joblib-specific features.

## Configuration And Environment

There is no authentication layer. The configuration knobs that matter are local execution and filesystem behavior.

Important options and environment variables:

- `n_jobs`: `-1` means "use all CPUs"; `1` disables parallelism and is the easiest debug mode.
- `prefer="threads"`: use this when the hot path mostly runs in native code or is I/O-bound.
- `backend="loky"`: default process backend; robust, but serialization overhead can dominate small tasks.
- `temp_folder`: worker temp directory for memmapping with process-based backends.
- `max_nbytes` and `mmap_mode`: control when large arrays are memory-mapped for worker sharing.
- `JOBLIB_TEMP_FOLDER`: first-choice environment variable for process-worker temp files.
- `TMPDIR`, `TMP`, `TEMP`: fallback temp directory controls used by the system temp folder.

## Common Pitfalls

- `joblib.load()` executes pickle-based deserialization. Do not load files from users, downloads, or any other untrusted source.
- `Memory` is designed for pure functions. Caching instance methods is fragile because `self` becomes part of the cache key unless you deliberately ignore it.
- Cache keys can invalidate after upgrading `joblib` or some third-party libraries because object hashing can change across versions.
- `mmap_mode="r"` returns read-only memmaps. `r+` and `w+` can mutate the on-disk cache and corrupt it; use `c` if you need copy-on-write semantics.
- Close or delete memmap-backed results on Windows to avoid file-locking issues.
- Default process workers serialize arguments and results. For tiny tasks, `Parallel` can be slower than a plain loop.
- Nested native thread pools can cause severe oversubscription. Use `inner_max_num_threads` or reduce library thread counts when combining joblib with NumPy or BLAS-heavy code.
- In reusable libraries, do not hard-code `backend="threading"` or similar unless you truly must. Upstream recommends soft hints such as `prefer` and external overrides via `parallel_config()`.
- `parallel_backend()` is in the deprecated section of the official docs; prefer `parallel_config()`.

## Version-Sensitive Notes For 1.5.x

- `joblib 1.5.3` on PyPI was released on 2025-12-15 and supports Python 3.9 through 3.13.
- The 1.5.0 line dropped Python 3.8 support and added Python 3.13 free-threaded support.
- `Memory` now auto-creates a `.gitignore` in its cache directory, and `1.5.3` specifically avoids overwriting an existing `.gitignore`.
- `1.5.2` fixed collisions for temporary files in shared cache directories accessed concurrently on cluster filesystems.
- `1.5.3` hardened evaluation of `pre_dispatch` expressions to avoid excessive memory allocation and crashes. Keep `pre_dispatch` values simple and numeric.
- `1.5.3` also updated vendored `cloudpickle` to `3.1.2`, which matters if you serialize interactively defined abstractions on Python 3.14+.

## Official Sources

- Docs root: `https://joblib.readthedocs.io/en/stable/`
- Install guide: `https://joblib.readthedocs.io/en/stable/installing.html`
- Memory guide: `https://joblib.readthedocs.io/en/stable/memory.html`
- Parallel guide: `https://joblib.readthedocs.io/en/stable/parallel.html`
- Persistence guide: `https://joblib.readthedocs.io/en/stable/persistence.html`
- Changelog / latest changes: `https://joblib.readthedocs.io/en/stable/developing.html#latest-changes`
- PyPI package page: `https://pypi.org/project/joblib/`
