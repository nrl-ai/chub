---
name: package
description: "NumPy package guide for Python array computing, broadcasting, typing, random generation, and NumPy 2.x migration"
metadata:
  languages: "python"
  versions: "2.4.3"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "numpy,python,array,ndarray,numerical,scientific-computing,linear-algebra,random"
---

# NumPy Python Package Guide

## Golden Rule

**Use `import numpy as np`, operate on whole arrays instead of Python loops when possible, and check shapes, dtypes, and copy semantics before assuming behavior.**

NumPy is the core Python package for n-dimensional arrays, vectorized math, broadcasting, linear algebra helpers, and random number generation. For most projects, it is the foundation that other scientific and data libraries build on top of.

## Installation

Install with `pip`:

```bash
python -m pip install numpy==2.4.3
```

If you use Conda or Miniconda, install from the `conda-forge` channel:

```bash
conda install -c conda-forge numpy
```

Verify the environment and version:

```bash
python - <<'PY'
import numpy as np
print(np.__version__)
PY
```

## Initialize And Create Arrays

Most code starts with one of:

```python
import numpy as np

a = np.array([1, 2, 3], dtype=np.int64)
b = np.asarray([1.0, 2.0, 3.0], dtype=np.float64)
c = np.zeros((2, 3))
d = np.ones((2, 3), dtype=np.float32)
e = np.arange(0, 10, 2)
f = np.linspace(0.0, 1.0, num=5)
```

Check the properties that control behavior:

```python
arr = np.array([[1, 2, 3], [4, 5, 6]], dtype=np.int32)

print(arr.shape)   # (2, 3)
print(arr.ndim)    # 2
print(arr.dtype)   # int32
print(arr.size)    # 6
```

Use `np.asarray(...)` when you want an array view of array-like input without forcing an unnecessary copy. In NumPy 2.x, `copy=False` is strict and raises if a copy would be required; use `copy=None` to allow a copy when needed, or `copy=True` to force one.

```python
data = [[1, 2], [3, 4]]
arr = np.asarray(data, dtype=np.float64, copy=None)
```

## Core Usage Patterns

### Elementwise math and reductions

NumPy operations usually apply elementwise:

```python
import numpy as np

x = np.array([1.0, 2.0, 3.0])
y = np.array([10.0, 20.0, 30.0])

print(x + y)        # [11. 22. 33.]
print(x * y)        # [10. 40. 90.]
print(np.sqrt(y))   # elementwise ufunc
print(x.sum())      # scalar reduction
print(y.mean())     # scalar reduction
```

Use `axis=` for row/column reductions:

```python
matrix = np.array([[1, 2, 3], [4, 5, 6]])

print(matrix.sum(axis=0))   # column sums
print(matrix.sum(axis=1))   # row sums
```

### Matrix multiplication

`*` is elementwise multiplication, not matrix multiplication. Use `@` or `np.matmul(...)`:

```python
a = np.array([[1, 2], [3, 4]])
b = np.array([[5, 6], [7, 8]])

print(a @ b)
```

### Reshaping and stacking

```python
arr = np.arange(12)
grid = arr.reshape(3, 4)
flat = grid.ravel()
stacked = np.stack([grid, grid])
```

Use `reshape(...)` only when the element count matches the target shape.

## Indexing, Slicing, And Broadcasting

### Basic indexing

```python
arr = np.arange(10)

print(arr[0])      # first element
print(arr[-1])     # last element
print(arr[2:7:2])  # slice
```

For multidimensional arrays:

```python
grid = np.arange(12).reshape(3, 4)

print(grid[1, 2])   # row 1, col 2
print(grid[:, 0])   # first column
print(grid[1:3, :2])
```

### Broadcasting

Broadcasting lets NumPy apply operations to arrays with compatible shapes without materializing repeated copies:

```python
data = np.array([[1.0, 2.0, 3.0],
                 [4.0, 5.0, 6.0]])
offset = np.array([10.0, 20.0, 30.0])

print(data + offset)
```

Broadcasting works when dimensions are equal or one of them is `1`, compared from the trailing dimensions backward. If shapes are incompatible, NumPy raises `ValueError: operands could not be broadcast together`.

## Random Number Generation

Prefer `np.random.default_rng(...)` and a `Generator` instance for new code:

```python
import numpy as np

rng = np.random.default_rng(seed=42)

samples = rng.normal(loc=0.0, scale=1.0, size=(2, 3))
ints = rng.integers(low=0, high=10, size=5)
```

This is better than using legacy global functions such as `np.random.seed(...)` and `np.random.rand(...)` in new code. If you need reproducibility, keep the seed explicit. NumPy documents no compatibility guarantee for the `Generator` bit stream across versions.

## Typing

Use `numpy.typing` for type hints:

```python
from typing import Any
import numpy as np
import numpy.typing as npt

def normalize(x: npt.ArrayLike) -> npt.NDArray[np.float64]:
    arr = np.asarray(x, dtype=np.float64)
    denom = np.linalg.norm(arr)
    return arr if denom == 0.0 else arr / denom
```

Useful typing names:

- `npt.ArrayLike` for function inputs that can become arrays
- `npt.NDArray[dtype]` for ndarray return types

If you run `mypy`, NumPy provides a plugin (`numpy.typing.mypy_plugin`) to resolve platform-specific scalar precisions more accurately.

## Configuration And Environment Notes

NumPy itself does not use API authentication.

The main things that behave like "configuration" for coding tasks are:

- `dtype=`: controls numeric precision and casting behavior
- `copy=`: controls whether data may be shared or must be copied
- `order=`: memory layout (`"C"` or `"F"`) for some constructors and conversions
- RNG seed: controls reproducibility in random workflows
- `np.set_printoptions(...)`: useful when tests or logs need stable array rendering

For isolated projects, install NumPy into a virtual environment or Conda environment instead of the system interpreter.

## Common Pitfalls

### Views vs copies

Basic slicing usually creates a **view**, so writing through the slice can mutate the original array:

```python
arr = np.array([1, 2, 3, 4])
view = arr[1:3]
view[0] = 99

print(arr)   # [ 1 99  3  4]
```

Advanced indexing creates a **copy**, not a view:

```python
arr = np.array([1, 2, 3, 4])
copy = arr[[1, 3]]
copy[0] = 99

print(arr)   # unchanged
```

### `np.arange(...)` with float steps

`np.arange(...)` is fine for integer ranges. For evenly spaced floats, prefer `np.linspace(...)` to avoid surprising endpoint and precision behavior.

### Dtype promotion and in-place operations

Mixed dtypes can change result dtypes, and in-place writes can fail if NumPy would need an unsafe cast:

```python
arr = np.array([1, 2, 3], dtype=np.int64)

# This can raise because the float result cannot be cast back safely.
arr += 0.5
```

Be explicit about the dtype you want before combining arrays or scalars with different numeric types.

### Shape mismatches

Most runtime NumPy bugs are shape bugs. Inspect `.shape` before assuming broadcasting or matrix multiplication will work:

```python
assert features.ndim == 2
assert weights.shape == (features.shape[1],)
```

## Version-Sensitive Notes For NumPy 2.x

- The version used here for this package session is `2.4.3`, and PyPI lists `2.4.3` as the current release.
- NumPy 2.0 changed type-promotion behavior in some mixed-dtype operations. If a project was written against 1.x behavior, check the NumPy 2.0 migration guide before "fixing" results by guesswork.
- NumPy 2.0 changed `np.asarray(...)` semantics by adding `copy=` and `device=` keyword arguments. `copy=False` now means "never copy" and can raise if the conversion cannot honor that requirement.
- Several legacy aliases and compatibility shims were removed in 2.x. Old blog posts or snippets may fail even if the overall idea is still correct.
- For extension modules that compile against the NumPy C API, NumPy 2.0 is a binary-compatibility boundary. Pure Python code using public NumPy APIs is usually much less affected.

## Practical Workflow For Agents

1. Install `numpy` into the active project environment and confirm `np.__version__`.
2. Convert external inputs with `np.asarray(...)` only once near the boundary.
3. Check `.shape` and `.dtype` before writing vectorized operations.
4. Prefer array expressions, ufuncs, and reductions over Python loops.
5. Use `default_rng(...)` for new random code.
6. When behavior looks surprising, inspect broadcasting rules and whether you are working with a view or a copy.
7. If the project recently moved from NumPy 1.x to 2.x, check the migration guide before assuming a bug is in your own code.

## Official Sources Used

- NumPy install guide: https://numpy.org/install
- NumPy absolute beginners guide: https://numpy.org/doc/stable/user/absolute_beginners.html
- NumPy quickstart: https://numpy.org/doc/stable/user/quickstart.html
- NumPy broadcasting guide: https://numpy.org/doc/stable/user/basics.broadcasting.html
- NumPy copies and views guide: https://numpy.org/doc/stable/user/basics.copies.html
- `numpy.asarray` reference: https://numpy.org/doc/stable/reference/generated/numpy.asarray.html
- Random `Generator` reference: https://numpy.org/doc/stable/reference/random/generator.html
- NumPy typing reference: https://numpy.org/doc/stable/reference/typing.html
- NumPy 2.0 migration guide: https://numpy.org/doc/stable/numpy_2_0_migration_guide.html
- PyPI package page: https://pypi.org/project/numpy/
