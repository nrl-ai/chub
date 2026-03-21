---
name: package
description: "Cython package guide for Python: building compiled extension modules from .pyx or annotated .py code"
metadata:
  languages: "python"
  versions: "3.2.4"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "cython,python,c-extension,build,compiler"
---

# Cython Python Package Guide

## What Cython Is

Cython compiles Python-like source into C or C++ extension modules for CPython. Use it when you need one of these:

- faster CPU-bound loops or array-heavy code
- direct interop with C or C++
- Python-callable wrappers around C-level code

For normal application code, prefer plain Python first. Reach for Cython when profiling shows a hot path or when you need native-library bindings.

## Install

```bash
python -m pip install Cython==3.2.4
```

Common alternatives:

```bash
uv add Cython==3.2.4
poetry add Cython==3.2.4
```

You also need a working native build toolchain for your platform because Cython does not remove the need for a C/C++ compiler.

- macOS: Xcode Command Line Tools
- Linux: `gcc` or `clang`, Python headers, and the usual build essentials
- Windows: MSVC Build Tools matching your Python build

## Minimal Build Setup

For modern projects, define build requirements in `pyproject.toml` and use `setuptools` with `cythonize()`.

`pyproject.toml`

```toml
[build-system]
requires = ["setuptools>=68", "Cython==3.2.4"]
build-backend = "setuptools.build_meta"
```

`setup.py`

```python
from setuptools import Extension, setup
from Cython.Build import cythonize

extensions = [
    Extension("mypkg.primes", ["src/mypkg/primes.pyx"]),
]

setup(
    ext_modules=cythonize(
        extensions,
        compiler_directives={"language_level": 3},
        annotate=True,
    ),
)
```

Project layout:

```text
src/
  mypkg/
    __init__.py
    primes.pyx
pyproject.toml
setup.py
```

Build it:

```bash
python -m pip install -e .
```

For quick local iteration on a single module, Cython also documents:

```bash
cythonize -a -i src/mypkg/primes.pyx
```

- `-i` builds an importable extension in place
- `-a` generates the HTML annotation report so you can see where Python interaction remains

## Core Usage

### `.pyx` module example

`src/mypkg/primes.pyx`

```cython
def primes(int nb_primes):
    cdef int n, i, length
    cdef int[1000] values

    length = 0
    n = 2
    while length < nb_primes:
        for i in range(length):
            if n % values[i] == 0:
                break
        else:
            values[length] = n
            length += 1
        n += 1

    return [values[i] for i in range(length)]
```

Use it from Python after the build:

```python
from mypkg.primes import primes

print(primes(10))
```

### Pure Python mode

Cython 3.x also supports compiling regular `.py` files that use `cython` annotations and decorators.

`src/mypkg/fastsum.py`

```python
import cython

@cython.ccall
def sum_to_n(n: cython.int) -> cython.longlong:
    i: cython.int
    total: cython.longlong = 0
    for i in range(n):
        total += i
    return total
```

Build the `.py` file with `cythonize()` the same way you would build a `.pyx` file.

Use pure Python mode when you want one source file that still runs under CPython without compilation. Use `.pyx` when you want the clearest separation between normal Python modules and compiled modules.

### Choosing `def`, `cdef`, and `cpdef`

- `def`: normal Python-callable function; can still use typed local variables
- `cdef`: C-only function or variable; not directly callable from Python
- `cpdef`: generates both a fast C entry point and a Python-callable wrapper

Default to `def` unless you need C-level calling or C-only data structures. Use `cpdef` only when you genuinely need both call paths.

### `.pxd` files and `cimport`

Use `.pxd` files for C-level declarations you want to share across Cython modules. Import those declarations with `cimport`, not normal Python `import`.

Typical split:

- `.pyx`: implementation
- `.pxd`: Cython declarations for functions, extension types, constants, and C APIs

## Build Configuration

### Important compiler directives

Set important directives explicitly instead of depending on defaults:

```python
cythonize(
    extensions,
    compiler_directives={
        "language_level": 3,
        "embedsignature": True,
    },
    annotate=True,
)
```

Practical defaults:

- `language_level=3`: keep Python 3 semantics explicit
- `embedsignature=True`: helps Python introspection and tooling
- `annotate=True`: useful while optimizing

Only disable safety checks such as `boundscheck` or `wraparound` after correctness is established. Those directives can turn mistakes into memory corruption or crashes.

### NumPy and other native dependencies

If you `cimport numpy` or depend on native headers, add those build requirements explicitly.

```python
from setuptools import Extension, setup
from Cython.Build import cythonize
import numpy

extensions = [
    Extension(
        "mypkg.arrays",
        ["src/mypkg/arrays.pyx"],
        include_dirs=[numpy.get_include()],
    )
]

setup(ext_modules=cythonize(extensions, compiler_directives={"language_level": 3}))
```

In that case, include `numpy` in `build-system.requires` as well so isolated builds succeed.

### No auth or runtime credentials

Cython does not need API keys, tokens, or service credentials. The configuration surface is your build backend, compiler toolchain, and compiler directives.

## Common Pitfalls

### `Cython` vs `cython`

These are related but not interchangeable:

- install from PyPI as `Cython`
- import build helpers from `Cython.Build`
- use `import cython` inside pure-Python-mode modules for decorators and type markers

### Translating to C is not the same as building an extension

The `cython` command can generate C or C++ source, but you still need a native build step to produce an importable extension module. For most projects, `cythonize()` plus `setuptools` is the shortest path.

### Pure Python annotations are not always C types

In pure Python mode, a normal annotation like `x: int` remains a Python-object annotation. Use Cython type markers such as `cython.int`, `cython.double`, typed memoryviews, or decorators like `@cython.locals` when you need C-level typing.

### `cython.cimports` only works in compiled code

Imports like `from cython.cimports.libc.math import sin` are for compiled Cython modules. Plain CPython execution will fail on them.

### `pyximport` is convenient but limited

`pyximport.install()` is useful for quick experiments, but it is not the best default for production packaging. It gives you less control over build options, dependencies, and reproducible builds than a normal package build configuration.

### Cython speedups are not automatic

Compilation alone does not guarantee major speed gains. The biggest wins usually come from:

- moving hot loops into typed code
- reducing Python object boxing and unboxing
- using typed memoryviews or C arrays
- avoiding repeated Python callback boundaries

## Version-Sensitive Notes For 3.2.4

- The version used here `3.2.4` matches the current PyPI release page used for this entry.
- For Cython 3.x, Python 3 semantics are the default language level. Keep `language_level=3` explicit in your build config anyway so mixed environments and old examples do not surprise you.
- If you are upgrading old 0.29-era code, review the 3.x migration guide before assuming the same behavior. Upstream calls out changed defaults such as Python-language semantics and `binding=True`.
- Avoid introducing new uses of deprecated compile-time `DEF` and `IF`.
- The `3.2.4` change log is incremental rather than a packaging reset; the bigger compatibility concerns are still the broader 3.x migration changes.

## Official Sources

- Stable docs root: `https://cython.readthedocs.io/en/stable/`
- Install guide: `https://cython.readthedocs.io/en/stable/src/quickstart/install.html`
- Build quickstart: `https://cython.readthedocs.io/en/stable/src/quickstart/build.html`
- Pure Python mode: `https://cython.readthedocs.io/en/stable/src/tutorial/pure.html`
- Source files and compilation guide: `https://cython.readthedocs.io/en/stable/src/userguide/source_files_and_compilation.html`
- Migration guide: `https://cython.readthedocs.io/en/stable/src/userguide/migrating_to_cy30.html`
- Change log: `https://cython.readthedocs.io/en/stable/src/changes.html`
- PyPI release page: `https://pypi.org/project/Cython/3.2.4/`
