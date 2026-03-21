---
name: package
description: "Numba package guide for Python JIT compilation of NumPy-heavy CPU kernels, parallel loops, and CUDA-aware setup"
metadata:
  languages: "python"
  versions: "0.64.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "numba,jit,numpy,performance,parallel,cuda,llvm"
---

# Numba Python Package Guide

## Golden Rule

Use `@njit` on NumPy-heavy, type-stable functions and loops, not on general Python application code. Warm up and benchmark after compilation, keep inputs as NumPy arrays or other homogeneous types, and treat CUDA as a separate dependency path by installing `numba-cuda` when GPU support matters.

## Install

For the package version used here:

```bash
python -m pip install "numba==0.64.0"
```

Common alternatives:

```bash
uv add "numba==0.64.0"
poetry add "numba==0.64.0"
```

Notes:

- `pip install numba` also installs the required `llvmlite` wheel; you do not need a system LLVM install for standard wheel-based usage.
- The official version-support table for `0.64.0` lists `llvmlite 0.46.x`, LLVM `20.x`, and NumPy `1.22 <= version < 1.27` or `2.0 <= version < 2.5`.
- For CUDA work, install the CUDA toolkit as required by your platform and add `numba-cuda` explicitly:

```bash
python -m pip install "numba==0.64.0" "numba-cuda"
```

## Check The Environment

Confirm the package imports and the version is what you expect:

```bash
python - <<'PY'
import numba
print(numba.__version__)
PY
```

For runtime diagnostics, threading-layer availability, CPU features, and CUDA detection:

```bash
numba -s
```

`numba -s` is the fastest official way to debug "why is this machine behaving differently?" issues before rewriting code.

## Core Usage

### Start with `@njit`

Prefer `@njit` over `@jit` when you want predictable performance. `@njit` enforces nopython compilation and fails loudly instead of silently leaving work in slow Python-object paths.

```python
import numpy as np
from numba import njit

@njit(cache=True)
def sum_of_squares(values):
    total = 0.0
    for value in values:
        total += value * value
    return total

arr = np.arange(1_000_000, dtype=np.float64)
print(sum_of_squares(arr))
```

Important behavior:

- The first call compiles the function for the observed argument types.
- Later calls with the same signature reuse the compiled version.
- `cache=True` persists compiled artifacts on disk so repeated runs start faster.

### Loops Are Fine

Numba is designed for numerical loops. You do not need to contort code into vectorized NumPy form just to make it fast under Numba.

```python
import numpy as np
from numba import njit

@njit(cache=True)
def l2_norms(x, y):
    out = np.empty(x.shape[0], dtype=np.float64)
    for i in range(x.shape[0]):
        out[i] = (x[i] * x[i] + y[i] * y[i]) ** 0.5
    return out
```

Keep array dtypes explicit and stable. Mixed Python objects, changing container element types, and shape-dependent Python branching make compilation less reliable.

### Parallel CPU Loops With `prange`

Use `parallel=True` together with `prange` for embarrassingly parallel loops or supported reductions:

```python
import numpy as np
from numba import njit, prange

@njit(parallel=True, cache=True)
def column_sums(matrix):
    out = np.zeros(matrix.shape[1], dtype=np.float64)
    for j in prange(matrix.shape[1]):
        total = 0.0
        for i in range(matrix.shape[0]):
            total += matrix[i, j]
        out[j] = total
    return out
```

Official parallel docs call out two constraints that matter in practice:

- `prange` only runs in parallel when the loop has no unsafe cross-iteration dependencies.
- Mutating shared containers like Python lists, sets, or dicts inside a `prange` region is not thread-safe.

### Build Ufuncs With `@vectorize`

When the natural API is elementwise array math, `@vectorize` can expose a NumPy-style ufunc:

```python
from numba import vectorize

@vectorize(["float64(float64, float64)"], nopython=True, cache=True)
def clipped_add(x, y):
    value = x + y
    return 1.0 if value > 1.0 else value
```

This is useful when you want a callable that behaves like a NumPy ufunc instead of a plain function compiled by `@njit`.

## Configuration And Runtime Controls

Numba has no auth or remote-service setup. Configuration is local process configuration.

Useful controls from the official environment-variable reference:

- `NUMBA_DISABLE_JIT=1`: disable JIT entirely so decorated functions execute as plain Python, useful for stepping through logic in a debugger.
- `NUMBA_NUM_THREADS=8`: cap the CPU thread pool used by `@njit(parallel=True)` and parallel CPU ufunc targets.
- `NUMBA_THREADING_LAYER=safe`: prefer a thread-safe and fork-safe threading backend when mixing Numba parallelism with multiprocessing.
- `NUMBA_CPU_NAME=generic`: emit more portable cached machine code within the same OS and CPU architecture.

You can persist settings in a `.numba_config.yaml` file in the working directory if `pyyaml` is installed:

```yaml
developer_mode: 1
color_scheme: dark_bg
```

Environment variables override the config file.

## Common Pitfalls

- First-call latency is compilation time, not steady-state runtime. Do not benchmark a jitted function from its very first call.
- Prefer `@njit` to force nopython mode. If compilation fails, fix the unsupported code path instead of dropping back to slow object-heavy code.
- Numba is not a general accelerator for Pandas, arbitrary Python classes, or string-heavy business logic. Focus on numerical kernels.
- Keep types homogeneous. Lists with mixed element types, dicts with unstable key or value types, and implicit dtype changes often cause typing errors.
- `parallel=True` is not a magic speed flag. Some loops will not parallelize, and unsafe writes can introduce wrong answers rather than exceptions.
- `fastmath=True` can change floating-point semantics. Use it only when reassociation and relaxed IEEE behavior are acceptable.
- `numba.typed.List` and `numba.typed.Dict` remain the explicit path when you need compiled containers, but the docs still label them experimental. Prefer NumPy arrays unless you truly need dynamic containers.
- Cached machine code is still specialization-sensitive. A different signature or incompatible CPU target will trigger recompilation.

## Version-Sensitive Notes For 0.64.0

- The official support table for `0.64.0` lists Python `3.10.x <= version < 3.15`.
- `0.64.0` adds support for NumPy `2.4`. If you are modernizing code for NumPy 2.4, replace removed APIs such as `np.trapz` with `np.trapezoid` and `np.in1d` with `np.isin`.
- The built-in CUDA target is deprecated. Official deprecation guidance says CUDA development has moved to the separate `numba-cuda` package, and CUDA users should install `numba-cuda`.
- Intel macOS x86_64 is no longer officially supported starting with `0.63.0+`. The deprecation notice recommends pinning `numba<0.63` if you need the last officially supported Intel Mac release line.
- `0.64.0` includes a cache-related fix for functions that call `parallel=True` functions. If you saw stale or incorrect cache behavior in older releases around nested parallel calls, re-test on `0.64.0`.

## Official Source URLs

- Docs root: `https://numba.readthedocs.io/en/stable/`
- Installation and version support: `https://numba.readthedocs.io/en/stable/user/installing.html`
- Quickstart: `https://numba.readthedocs.io/en/stable/user/5minguide.html`
- Parallel CPU docs: `https://numba.readthedocs.io/en/stable/user/parallel.html`
- Environment variables: `https://numba.readthedocs.io/en/stable/reference/envvars.html`
- Deprecations: `https://numba.readthedocs.io/en/stable/reference/deprecation.html`
- Release notes for `0.64.0`: `https://numba.readthedocs.io/en/stable/release/0.64.0-notes.html`
- PyPI: `https://pypi.org/project/numba/`
