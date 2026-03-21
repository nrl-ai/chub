---
name: package
description: "JAX package guide for Python projects using transform-based array computing, autodiff, and accelerator backends"
metadata:
  languages: "python"
  versions: "0.9.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "jax,python,autodiff,numpy,xla,gpu,tpu"
---

# JAX Python Package Guide

## Golden Rule

Use `jax` together with the `jaxlib` build selected by the official installer path for your hardware. Write transformed code in terms of `jax.numpy`, `jax.lax`, and other JAX-aware APIs, not raw NumPy inside `jit`/`grad`/`vmap`.

## Install

JAX installs as two pieces:

- `jax`: pure Python package
- `jaxlib`: compiled backend/runtime package selected for your OS and accelerator

For most projects, let the official extras resolve the matching backend package.

### CPU

```bash
python -m pip install --upgrade pip
python -m pip install "jax==0.9.1"
```

### NVIDIA GPU

CUDA 13 wheels:

```bash
python -m pip install --upgrade pip
python -m pip install "jax[cuda13]==0.9.1"
```

CUDA 12 wheels:

```bash
python -m pip install "jax[cuda12]==0.9.1"
```

Notes:

- JAX supports NVIDIA GPUs with SM `5.2+` on CUDA 12 and `7.5+` on CUDA 13.
- For CUDA 13 on Linux, the NVIDIA driver must be `>= 580`.
- Native Windows GPU is not supported; WSL2 GPU support is experimental.

### Google Cloud TPU VM

```bash
python -m pip install "jax[tpu]==0.9.1"
```

### Other extras published on PyPI

PyPI currently lists extras including `minimum-jaxlib`, `cpu`, `tpu`, `cuda`, `cuda12`, `cuda13`, `cuda12-local`, `cuda13-local`, `rocm`, `k8s`, and `xprof`.

## Initialize And Verify The Backend

JAX chooses the best available backend by default. Check what it actually initialized before assuming GPU or TPU execution:

```python
import jax
import jax.numpy as jnp

print(jax.__version__)
print(jax.default_backend())
print(jax.devices())

x = jnp.arange(5)
print(x)
```

If you need to force CPU for reproducibility or debugging:

```bash
export JAX_PLATFORMS=cpu
```

JAX has no package-level authentication. Any auth or IAM setup is handled by the surrounding platform, such as access to a Cloud TPU VM or cluster runtime, not by `jax` itself.

## Core Usage

### Array programming with `jax.numpy`

Use `jax.numpy` as the default array API:

```python
import jax.numpy as jnp

x = jnp.linspace(0.0, 1.0, 5)
y = jnp.sin(x) + 2 * x
print(y)
```

### Automatic differentiation with `jax.grad`

```python
import jax
import jax.numpy as jnp

def loss(x):
    return jnp.sum(jnp.log1p(jnp.exp(x)))

grad_loss = jax.grad(loss)
print(grad_loss(jnp.array([0.0, 1.0, 2.0])))
```

### JIT compilation with `jax.jit`

```python
import jax
import jax.numpy as jnp

def normalize_rows(x):
    x = x - x.mean(axis=0)
    return x / x.std(axis=0)

normalize_rows_jit = jax.jit(normalize_rows)

x = jnp.arange(12.0).reshape(4, 3)
result = normalize_rows_jit(x)
print(result.block_until_ready())
```

Call `block_until_ready()` when benchmarking or timing JAX code, because execution is asynchronous.

### Vectorization with `jax.vmap`

```python
import jax
import jax.numpy as jnp

def squared_norm(x):
    return jnp.sum(x * x)

batched_squared_norm = jax.vmap(squared_norm)

x = jnp.arange(12.0).reshape(4, 3)
print(batched_squared_norm(x))
```

### Random numbers use explicit keys

JAX random state is explicit and stateless. Split keys instead of reusing them:

```python
from jax import random

key = random.key(0)
key1, key2 = random.split(key)

sample_a = random.normal(key1, (2, 3))
sample_b = random.normal(key2, (2, 3))
```

## Configuration

JAX configuration can be set with environment variables before process start or with `jax.config.update(...)` early in Python startup.

### Enable 64-bit types

JAX defaults `jax_enable_x64` to `False`.

Environment variable:

```bash
export JAX_ENABLE_X64=True
```

Runtime config:

```python
import jax

jax.config.update("jax_enable_x64", True)
```

### Debug transformed code

Disable JIT to debug Python behavior more directly:

```bash
export JAX_DISABLE_JIT=True
```

Raise on NaNs:

```bash
export JAX_DEBUG_NANS=True
```

Useful config options for practical debugging and ops work:

- `JAX_PLATFORMS`: restrict backend initialization order, for example `cpu` or `cpu,tpu`
- `JAX_COMPILATION_CACHE_DIR`: persistent compilation cache directory
- `JAX_EXPLAIN_CACHE_MISSES`: log why JAX missed tracing or compilation caches

## Common Pitfalls

### JAX arrays are immutable

This fails:

```python
x = jnp.arange(5)
x[0] = 10
```

Use indexed updates that return a new array:

```python
x = x.at[0].set(10)
```

### Do not call raw NumPy inside transformed JAX code

This is a common source of `TracerArrayConversionError`:

```python
import numpy as np
from jax import jit

@jit
def f(x):
    return np.sin(x)
```

Use `jax.numpy` inside transformed functions instead.

### `jit` requires static output shapes

Boolean masking that changes output size is incompatible with `jit`:

```python
@jax.jit
def positives(x):
    return x[x > 0]
```

Rewrite logic with shape-preserving operations such as `jnp.where(...)` when possible.

### Python control flow on traced values fails

This often surfaces as `TracerBoolConversionError`:

- `if traced_value: ...`
- `min(x, 0)` or other non-JAX Python helpers inside `jit`

Use `jnp.where`, `jax.lax.cond`, `jax.lax.scan`, or mark true configuration arguments as static with `static_argnums` or `static_argnames`.

### Do not leak traced values through side effects

Appending traced values to outer lists, mutating globals, or keeping references from inside `jit` can lead to `UnexpectedTracerError`. Return values explicitly from the transformed function instead.

### Random keys must be split, not reused

Reusing a key is a logic bug. JAX exposes `KeyReuseError` and related docs because PRNG state is manual.

### Backend assumptions are often wrong

JAX will try to initialize all available platforms and defaults to GPU or TPU when possible, otherwise CPU. If your environment is fragile, set `JAX_PLATFORMS` explicitly and print `jax.default_backend()` / `jax.devices()` at process start.

## Version-Sensitive Notes For `0.9.1`

- As of March 12, 2026, PyPI lists `jax 0.9.1` as the latest release, published on March 2, 2026.
- The docs URL `https://jax.readthedocs.io/en/latest/` redirects to the canonical docs root `https://docs.jax.dev/en/latest/`.
- JAX `0.9.1` adds the debug config `jax_compilation_cache_check_contents`, which helps verify persistent compilation cache content consistency.
- JAX `0.9.0` added `jax.thread_guard()` for detecting multi-thread device misuse in multi-controller JAX.
- JAX `0.9.0` removed `jax_collectives_common_channel_id` and `jax_pmap_no_rank_reduction`.
- JAX `0.9.0` deprecated `jax.numpy.fix()` in favor of `jax.numpy.trunc()`.

## Official Sources

- Docs root: https://docs.jax.dev/en/latest/
- Installation: https://docs.jax.dev/en/latest/installation.html
- Thinking in JAX: https://docs.jax.dev/en/latest/notebooks/thinking_in_jax.html
- Configuration options: https://docs.jax.dev/en/latest/config_options.html
- Errors: https://docs.jax.dev/en/latest/errors.html
- Changelog: https://docs.jax.dev/en/latest/changelog.html
- PyPI: https://pypi.org/project/jax/
