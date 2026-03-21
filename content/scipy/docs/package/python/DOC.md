---
name: package
description: "SciPy package guide for Python numerical computing, optimization, sparse data, and scientific algorithms"
metadata:
  languages: "python"
  versions: "1.17.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "scipy,python,numerical-computing,optimization,linear-algebra,sparse"
---

# SciPy Python Package Guide

## Golden Rule

Use **NumPy for arrays** and **SciPy for algorithms** built on top of those arrays.

Prefer explicit public imports:

```python
import numpy as np
from scipy import integrate, linalg, optimize, sparse, stats
```

Do not rely on private modules such as `scipy._lib...`, and do not assume third-party blog examples match current SciPy behavior.

## Installation

Install into a virtual environment and keep NumPy/SciPy aligned:

```bash
python -m pip install "numpy>=1.25.2" "scipy==1.17.1"
```

If NumPy is already managed by the project, installing SciPy alone is usually enough:

```bash
python -m pip install "scipy==1.17.1"
```

Verify the runtime versions before writing code that depends on new APIs:

```bash
python - <<'PY'
import numpy as np
import scipy

print("numpy", np.__version__)
print("scipy", scipy.__version__)
PY
```

## Initialization And Setup

SciPy is a local numerical library. There is:

- no API key
- no service authentication
- no global client object to initialize

Most projects should start with:

```python
import numpy as np
from scipy import integrate, linalg, optimize, sparse, stats
```

Use NumPy arrays as the default input type:

```python
x = np.asarray([1.0, 2.0, 3.0], dtype=float)
```

If your code receives Python lists, pandas objects, or mixed dtypes, normalize them before passing them into SciPy routines.

## Choose The Right Module

- `scipy.linalg`: dense linear algebra
- `scipy.optimize`: minimization, root finding, curve fitting
- `scipy.integrate`: quadrature and ODE solving
- `scipy.sparse`: sparse matrices and sparse linear algebra helpers
- `scipy.stats`: probability distributions and statistical tests
- `scipy.signal`: filtering, transforms, spectral tools
- `scipy.spatial`: distances, KD-trees, geometry helpers

## Core Usage

### Solve A Dense Linear System

```python
import numpy as np
from scipy import linalg

A = np.array([[3.0, 2.0], [1.0, 2.0]])
b = np.array([5.0, 5.0])

x = linalg.solve(A, b)
print(x)  # [0.  2.5]
```

Use `scipy.linalg` when you need decompositions, matrix functions, or linear algebra routines beyond what plain `numpy.linalg` gives you.

### Minimize An Objective Function

```python
import numpy as np
from scipy import optimize

def objective(x: np.ndarray) -> float:
    return (x[0] - 1.5) ** 2 + (x[1] + 2.0) ** 2

result = optimize.minimize(objective, x0=np.array([0.0, 0.0]), method="BFGS")

print(result.x)        # near [1.5, -2.0]
print(result.success)  # True on a successful solve
```

Check `result.success`, `result.message`, and algorithm-specific fields instead of assuming convergence.

### Solve An ODE

```python
import numpy as np
from scipy import integrate

def decay(t: float, y: np.ndarray) -> np.ndarray:
    return -0.5 * y

solution = integrate.solve_ivp(
    decay,
    t_span=(0.0, 5.0),
    y0=[1.0],
    t_eval=np.linspace(0.0, 5.0, 6),
)

print(solution.t)
print(solution.y[0])
```

Use `solve_ivp` for new ODE code; keep `t_span`, `y0`, tolerances, and event functions explicit.

### Work With Sparse Data

```python
import numpy as np
from scipy import sparse

A = sparse.csr_matrix([
    [0.0, 1.0, 0.0],
    [2.0, 0.0, 3.0],
    [0.0, 0.0, 4.0],
])

x = np.array([1.0, 2.0, 3.0])
y = A @ x

print(y)
print(A.nnz)  # number of stored nonzero entries
```

Use sparse structures when the matrix is mostly zeros. Dense conversions can erase the performance and memory benefit.

### Use Statistical Distributions

```python
from scipy import stats

dist = stats.norm(loc=0.0, scale=1.0)

print(dist.cdf(1.96))
print(dist.ppf(0.975))
```

For statistical tests and distributions, check the docstring for parameterization details instead of guessing from distribution names.

## Configuration And Environment

SciPy has no auth configuration. The main setup concerns are runtime compatibility and native dependencies:

- keep Python and NumPy inside the supported version range for the installed SciPy release
- prefer standard wheels unless you intentionally need a source build
- use a clean virtual environment when debugging binary import or ABI issues

If import errors mention compiled extensions, version skew between Python, NumPy, and SciPy is a more likely cause than application logic.

## Common Pitfalls

- **Passing loosely typed inputs:** many functions work best with explicit `ndarray` inputs. Normalize with `np.asarray(..., dtype=float)` when numeric precision matters.
- **Importing private or undocumented symbols:** public submodules are stable entry points; private internals are not.
- **Guessing algorithm defaults:** optimization, integration, interpolation, and statistics APIs often have important defaults for tolerances, bounds, axis handling, or NaN behavior.
- **Shape mistakes:** SciPy routines often distinguish between `(n,)`, `(n, 1)`, and `(m, n)` inputs. Wrong shapes can change broadcasting or solver behavior.
- **Accidental densification:** mixing sparse and dense operations can materialize large dense arrays unexpectedly.
- **Overusing top-level `scipy` access:** explicit submodule imports are clearer and safer for agent-generated code.

## Version-Sensitive Notes

- PyPI currently lists `1.17.1` as the release covered here, published on **2026-02-23**.
- The reference manual is still labeled `v1.17.0` on **2026-03-11**. That looks like an official docs lag, not a different major/minor API line.
- PyPI metadata for `1.17.1` requires Python `>=3.12` and NumPy `>=1.25.2`. Do not assume older interpreter support.
- SciPy `1.15.0` release notes call out lazy loading for submodules. In practice, write explicit imports such as `from scipy import optimize` or `from scipy.optimize import minimize` instead of relying on fragile access patterns.

## Official Sources

- SciPy package on PyPI: https://pypi.org/project/scipy/
- SciPy reference manual: https://docs.scipy.org/doc/scipy/reference/
- SciPy API overview: https://docs.scipy.org/doc/scipy/reference/api.html
- SciPy 1.15.0 release notes: https://docs.scipy.org/doc/scipy/reference/release.1.15.0.html
