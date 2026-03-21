---
name: package
description: "einops for Python: concise tensor reshaping, reduction, repetition, packing, and einsum patterns"
metadata:
  languages: "python"
  versions: "0.8.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "einops,tensors,numpy,pytorch,jax,tensorflow,array-api"
---

# einops Python Package Guide

## What It Is

`einops` is a small tensor manipulation library for writing reshapes, transposes, reductions, repetitions, packing, and einsum-style operations with readable string patterns. It works with common Python tensor backends including NumPy, PyTorch, TensorFlow, JAX, and CuPy.

Use it when raw `reshape`, `transpose`, `permute`, or backend-specific pooling code is getting hard to read or easy to get wrong.

## Installation

Install `einops` itself plus the tensor backend you actually use:

```bash
pip install einops==0.8.2
```

Common setups:

```bash
pip install einops numpy
pip install einops torch
pip install einops jax jaxlib
uv add einops
poetry add einops
```

`einops` does not require authentication and has no global configuration file.

## Setup

For most projects, import the functional API:

```python
from einops import rearrange, reduce, repeat, pack, unpack, einsum, parse_shape
```

If your project uses the Python Array API standard namespace, use the array API variant:

```python
from einops.array_api import rearrange, reduce, repeat
```

PyTorch-specific layers are available here:

```python
from einops.layers.torch import Rearrange, Reduce
```

## Core Usage

### `rearrange`

Use `rearrange` for reshape, transpose, flatten, split, stack, channel moves, and patch extraction in one expression.

```python
import numpy as np
from einops import rearrange

x = np.random.randn(32, 3, 224, 224)  # BCHW

# BCHW -> BHWC
y = rearrange(x, "b c h w -> b h w c")

# Flatten image patches
patches = rearrange(x, "b c (h ph) (w pw) -> b (h w) (c ph pw)", ph=16, pw=16)

# Split channels into heads
q = np.random.randn(8, 128, 512)
multi_head = rearrange(q, "b n (heads d) -> b heads n d", heads=8)
```

### `reduce`

Use `reduce` when dimensions should be grouped and reduced in one step.

```python
import numpy as np
from einops import reduce

x = np.random.randn(32, 3, 224, 224)

# 2x2 average pooling
pooled = reduce(x, "b c (h h2) (w w2) -> b c h w", "mean", h2=2, w2=2)

# Global max over sequence length
seq = np.random.randn(16, 128, 64)
mx = reduce(seq, "b n d -> b d", "max")
```

### `repeat`

Use `repeat` to tile data or add broadcast-like structure with named axes.

```python
import numpy as np
from einops import repeat

token = np.random.randn(64)

# Create a batch of identical vectors
batch = repeat(token, "d -> b d", b=32)

# Expand grayscale to RGB-style channels
image = np.random.randn(224, 224)
rgbish = repeat(image, "h w -> c h w", c=3)
```

### `pack` and `unpack`

Use `pack` when you need to concatenate tensors that share some axes but differ on others, then reverse the operation later.

```python
import numpy as np
from einops import pack, unpack

cls = np.random.randn(2, 1, 64)
tokens = np.random.randn(2, 128, 64)

packed, ps = pack([cls, tokens], "b * d")
restored_cls, restored_tokens = unpack(packed, ps, "b * d")
```

### `einsum`

Use `einops.einsum` when you want readable named axes instead of single-letter einsum dimensions.

```python
import numpy as np
from einops import einsum

query = np.random.randn(2, 8, 128, 64)
key = np.random.randn(2, 8, 128, 64)

scores = einsum(query, key, "batch head token_q dim, batch head token_k dim -> batch head token_q token_k")
```

### `parse_shape`

Use `parse_shape` when later operations need named sizes pulled from a real tensor.

```python
import numpy as np
from einops import parse_shape, rearrange

x = np.random.randn(4, 128, 16, 16)
shape = parse_shape(x, "b c h w")

y = rearrange(x, "b c h w -> b (h w) c")
assert y.shape == (shape["b"], shape["h"] * shape["w"], shape["c"])
```

## PyTorch Module Usage

`einops.layers.torch` is useful inside `nn.Sequential` or reusable model blocks:

```python
import torch.nn as nn
from einops.layers.torch import Rearrange, Reduce

model = nn.Sequential(
    Rearrange("b c h w -> b (h w) c"),
    nn.LayerNorm(256),
    Reduce("b n c -> b c", "mean"),
)
```

Use functional `rearrange`/`reduce`/`repeat` in ad hoc tensor code and layer classes when the operation should live inside the model graph as a module.

## Backend Notes

- `einops` operates on tensors from the backend you pass in. It is not a tensor backend by itself.
- For backend-agnostic code using the Python Array API standard, prefer `einops.array_api`.
- Some operations can return a view of the original tensor when the backend supports it. Do not assume the result is always a copy.
- The package also accepts lists of tensors in several APIs, which is useful for stacking or packing operations without manual pre-concatenation.

## Config and Auth

There is no auth layer, credentials flow, or runtime service configuration.

The practical setup choices are:

- install the correct tensor backend separately
- import either the normal API or `einops.array_api`
- for PyTorch, choose between functional calls and `einops.layers.torch`

## Common Pitfalls

- Pattern names are semantic. If you use an axis name on the right-hand side, it must either appear on the left-hand side or be provided in `axes_lengths`.
- Composite axes must divide cleanly. For example, `(h h2)` requires the actual axis length to be divisible by `h2`.
- `rearrange` is not a reduction. If you need pooling or aggregation, use `reduce`.
- `repeat` expands by explicit named dimensions; it is clearer than manual broadcasting, but you still need to specify the added axis sizes.
- `pack`/`unpack` are a pair. Keep the `ps` metadata returned by `pack`, otherwise you cannot reliably reconstruct the original tensors.
- `einops.einsum` takes tensors first and the pattern last, unlike `numpy.einsum` and `torch.einsum`.
- `einops.einsum` does not support singleton axes like `()` or rearrange-style anonymous compositions inside the einsum pattern.
- When performance debugging, check whether the backend already has a fused primitive that is better than composing many small tensor ops, even if the `einops` expression is correct.

## Version-Sensitive Notes For 0.8.2

- `0.8.2` is the current package version on PyPI and the docs site at the time of writing.
- The maintainer docs for `0.8.2` call out MLX backend support.
- `pack` and `unpack` require a newer `einops` line than older blog posts from the `0.4` and `0.5` era; confirm examples before copying legacy snippets.
- `einops.einsum` was added after the earliest `einops` releases, so older examples may still use backend-native einsum calls instead.
- The docs site currently says supported Python versions are `3.10+`, while the PyPI metadata for `0.8.2` lists `Requires: Python >=3.9`. If you target Python 3.9 specifically, verify your environment with a real install and smoke import before depending on it.
- If you use `torch.compile` on the functional operations with PyTorch earlier than `2.4`, the maintainer docs point to `allow_ops_in_compiled_graph`; layer objects do not need that workaround.
- Starting in `0.8.1`, upstream ships packaged tests and documents a `uv`-based test runner. That is useful if you need a quick backend compatibility check in CI or while debugging environment issues.

## Recommended Workflow For Coding Agents

1. Identify the tensor layout before writing the pattern. Write down what each axis means.
2. Prefer `rearrange`/`reduce`/`repeat` over chains of backend-specific reshape and transpose calls when readability matters.
3. Keep axis names meaningful: `batch`, `time`, `head`, `channel`, `patch`, `height`, `width`.
4. For reversible packing logic, store the `ps` object next to the packed tensor.
5. If you are translating from `numpy.einsum` or `torch.einsum`, rewrite the expression carefully because `einops.einsum` uses named axes and a different argument order.

## Official Sources

- Docs: https://einops.rocks/
- Basics tutorial: https://einops.rocks/1-einops-basics/
- API reference: https://einops.rocks/api/rearrange/
- Pack/unpack API: https://einops.rocks/api/pack_unpack/
- Einsum API: https://einops.rocks/api/einsum/
- GitHub repository: https://github.com/arogozhnikov/einops
- PyPI: https://pypi.org/project/einops/
