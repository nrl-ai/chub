---
name: package
description: "SymPy symbolic mathematics library for Python: expressions, algebra, calculus, solving, matrices, and numeric code generation"
metadata:
  languages: "python"
  versions: "1.14.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "sympy,python,symbolic-math,cas,algebra,calculus,matrices"
---

# SymPy Python Package Guide

## Golden Rule

Use SymPy objects end-to-end for symbolic work. Create symbols explicitly, represent equations with `Eq(...)`, keep values exact with SymPy numbers like `Integer`, `Rational`, and `pi`, and only convert to numeric code at the edge with `evalf()` or `lambdify()`.

## Install

Pin the version your project expects:

```bash
python -m pip install "sympy==1.14.0"
```

Common alternatives:

```bash
uv add "sympy==1.14.0"
poetry add "sympy==1.14.0"
```

Notes:

- `pip install sympy` already installs `mpmath`, which SymPy depends on.
- `gmpy2` is a useful optional speedup for heavy integer and rational arithmetic workloads.
- Install `matplotlib` if you need plotting.
- Use `ipython` or Jupyter for a better interactive workflow.

## Initialize And Set Up

For scripts, prefer a normal module import:

```python
import sympy as sp

x, y = sp.symbols("x y", real=True)
expr = (x + y) ** 3

print(sp.expand(expr))
```

For notebooks or REPL use, enable pretty printing:

```python
import sympy as sp

sp.init_printing()
x = sp.symbols("x")
```

For an interactive shell with common imports preloaded, use:

```bash
isympy
```

## Core Usage

### Define symbols with assumptions

Assumptions materially affect simplification and integration behavior, so add them when they are known.

```python
import sympy as sp

x = sp.symbols("x", positive=True)
n = sp.symbols("n", integer=True)

print(sp.sqrt(x**2))  # x
```

### Build and transform expressions

Prefer targeted transforms like `expand()`, `factor()`, `cancel()`, and `collect()` when behavior needs to be predictable.

```python
import sympy as sp

x = sp.symbols("x")
expr = (x + 1) ** 2

print(sp.expand(expr))   # x**2 + 2*x + 1
print(sp.factor(x**2 + 2*x + 1))  # (x + 1)**2
```

### Solve equations

Use `Eq()` for explicit equations, pass the variable you want solved, and use `dict=True` if you need stable programmatic output.

```python
import sympy as sp

x, y = sp.symbols("x y")

solutions = sp.solve(sp.Eq(x**2, y), x, dict=True)
print(solutions)
# [{x: -sqrt(y)}, {x: sqrt(y)}]
```

Use `solveset()` when you want a mathematical set result with an explicit domain:

```python
import sympy as sp

x = sp.symbols("x")
roots = sp.solveset(sp.sin(x), x, domain=sp.Interval(-sp.pi, sp.pi))
print(roots)  # {0, -pi, pi}
```

Use `nsolve()` when you need a numeric root or when exact solving is not implemented:

```python
import sympy as sp

x = sp.symbols("x")
root = sp.nsolve(sp.cos(x) - x, x, 0.5)
print(root)  # 0.739085...
```

### Calculus

```python
import sympy as sp

x = sp.symbols("x")
f = sp.sin(x) * sp.exp(x)

print(sp.diff(f, x))
print(sp.integrate(f, (x, 0, sp.pi)))
```

### Matrices and linear algebra

```python
import sympy as sp

A = sp.Matrix([[1, 2], [3, 4]])

print(A.det())
print(A.eigenvals())
print(A.inv())
```

### Move from symbolic to numeric code with `lambdify()`

Keep symbolic and numeric code separate. Convert symbolic expressions into numeric callables explicitly.

```python
import numpy as np
import sympy as sp

x = sp.symbols("x")
expr = sp.diff(sp.sin(x) * sp.exp(x**2), x)

f = sp.lambdify(x, expr, modules="numpy")
xs = np.linspace(0, 2, 5)
print(f(xs))
```

Set `modules=` explicitly for reproducible behavior. If you omit it, SymPy chooses a backend based on what is installed in the environment.

## Configuration And Environment Notes

SymPy does not use API keys or service auth. The main setup concerns are environment consistency and optional tooling:

- Use a virtual environment so `sympy`, `numpy`, `scipy`, and notebook tooling stay version-aligned.
- Prefer `import sympy as sp` in real code over `from sympy import *`.
- Use `sp.init_printing()` in notebooks for readable output.
- Use `lambdify(..., modules="numpy")`, `modules="mpmath"`, or another explicit backend instead of relying on backend auto-detection.
- Keep symbolic computation in SymPy and numeric array computation in NumPy/SciPy; bridge between them with `lambdify()`.

## Common Pitfalls

- `=` is Python assignment, not an equation. Use `sp.Eq(x, y)` when you mean mathematical equality.
- `==` checks structural equality, not symbolic equivalence. `(x + 1)**2 == x**2 + 2*x + 1` is `False`.
- Python numbers are not always SymPy numbers. Use `sp.Rational(1, 2)` or `sp.S(1) / 2` when exactness matters.
- Do not mix `math` with symbolic expressions. Use `sp.pi`, `sp.sin`, and `expr.evalf()` instead of `math.pi`, `math.sin`, or implicit float conversions.
- SymPy expressions are immutable. `expr.subs(...)` returns a new expression; it does not mutate the original.
- Always pass the variable to `solve()` and `solveset()` when there is any ambiguity.
- `solve()` returns different shapes depending on the problem. Use `dict=True` for stable programmatic handling.
- `simplify()` is heuristic. In production code, prefer targeted transforms like `expand()`, `factor()`, `cancel()`, `collect()`, or `trigsimp()`.
- Do not pass SymPy expressions back into functions produced by `lambdify()`. Those functions are for numeric inputs.

## Version-Sensitive Notes For 1.14.0

- PyPI lists `sympy 1.14.0` with `Requires: Python >=3.9` and classifiers for Python `3.9` through `3.13`.
- The official docs are already versioned as `SymPy 1.14.0`, so the version used here and current upstream docs match.
- The install docs still call out older code that imports `sympy.mpmath`; in current code, import `mpmath` directly.
- `lambdify()` backend selection depends on installed numeric libraries. In one environment it may prefer SciPy or NumPy, and in another it may fall back to `math`/`cmath`/`mpmath`, so explicit `modules=` is safer for agent-generated code.

## Official Links

- Docs root: `https://docs.sympy.org/latest/`
- Reference root: `https://docs.sympy.org/latest/reference/`
- Install guide: `https://docs.sympy.org/latest/install.html`
- Best practices: `https://docs.sympy.org/latest/explanation/best-practices.html`
- Gotchas: `https://docs.sympy.org/latest/explanation/gotchas.html`
- Solving guidance: `https://docs.sympy.org/latest/guides/solving/solving-guidance.html`
- `lambdify()` reference: `https://docs.sympy.org/latest/modules/utilities/lambdify.html`
- PyPI: `https://pypi.org/project/sympy/`
