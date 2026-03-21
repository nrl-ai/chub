---
name: package
description: "cffi for Python: call C libraries, build bindings, and package out-of-line modules"
metadata:
  languages: "python"
  versions: "2.0.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "cffi,ffi,c,bindings,python,native"
---

# cffi Python Package Guide

Use `cffi` when Python code needs to call C functions, work with C structs and pointers, or ship Python bindings around an existing C library.

## Golden Rule

- Use `from cffi import FFI` and decide up front whether you need ABI mode or API mode.
- Prefer out-of-line modules with `ffibuilder.set_source(...)` for packaged bindings.
- Treat `ffi.verify()` as legacy compatibility only; upstream marks it deprecated.
- Keep ownership and lifetime explicit. Most CFFI bugs are pointer lifetime, string encoding, or build configuration mistakes.

## Install

For CPython:

```bash
pip install cffi==2.0.0
```

For a project dependency:

```toml
[project]
dependencies = ["cffi==2.0.0"]
```

Upstream notes that `cffi` is distributed with PyPy, but projects should still follow their own dependency pinning policy.

## Runtime and Build Requirements

- Supported upstream on `2.0.0`: CPython `3.9` through `3.14`, plus PyPy.
- PyPI declares `Requires: Python >=3.9`.
- On non-Windows CPython, source builds may need a C compiler and `libffi` development headers.
- On Linux, the usual missing-system-package failure is `libffi-dev`.
- On macOS, upstream documents `pkg-config` plus `libffi` when a wheel is not usable.
- `pycparser` is an automatic dependency of `cffi`.

If `pip install cffi` falls back to building from source on Linux or macOS, fix the system prerequisites first instead of debugging Python import errors.

## Choose a Mode First

### ABI mode

Use ABI mode when the shared library already exists and you only need to describe its interface from Python.

Pros:

- Fastest path for local scripting and internal tooling
- No extension module to compile when using plain in-line `ffi.dlopen(...)`

Tradeoffs:

- Less compile-time checking
- Library loading and symbol lookup issues show up at runtime

### API mode

Use API mode when you are shipping bindings, need compiler-checked declarations, or want a generated module that imports cleanly.

Pros:

- Better packaging story
- Compiler fills in details and validates more of the declarations
- Supports `extern "Python"` patterns for callbacks from C into Python

Tradeoffs:

- Requires a build step
- You must provide correct compiler and linker settings

## Quick Start: ABI Mode

This is the shortest path when a native library already exists on the machine.

```python
from cffi import FFI

ffi = FFI()
ffi.cdef("""
    int puts(const char *s);
""")

lib = ffi.dlopen(None)  # standard C library on Unix
lib.puts(b"hello from cffi")
```

Typical pattern for a real shared library:

```python
from cffi import FFI
from ctypes.util import find_library

ffi = FFI()
ffi.cdef("""
    int sqlite3_libversion_number(void);
""")

lib = ffi.dlopen(find_library("sqlite3"))
print(lib.sqlite3_libversion_number())
```

Use ABI mode when you already know the library path or can resolve it reliably on the target system.

## Quick Start: Packaged API Mode

This is the safer default for reusable bindings.

Build script:

```python
from cffi import FFI

ffibuilder = FFI()
ffibuilder.cdef("""
    int add(int a, int b);
""")

ffibuilder.set_source(
    "_example",
    """
    int add(int a, int b) {
        return a + b;
    }
    """,
)

if __name__ == "__main__":
    ffibuilder.compile(verbose=True)
```

Run the build once:

```bash
python build_example.py
```

Use the generated module:

```python
from _example import lib

result = lib.add(2, 3)
print(result)
```

If you are binding an existing library instead of compiling inline C, pass headers and linker settings to `set_source(...)`:

```python
ffibuilder.set_source(
    "mypkg._native",
    '#include "mylib.h"',
    libraries=["mylib"],
    include_dirs=["/path/to/include"],
    library_dirs=["/path/to/lib"],
)
```

## Core Usage

### Declare C signatures with `cdef()`

`ffi.cdef(...)` takes C-like declarations. Keep them minimal and copy the parts you actually call.

```python
ffi.cdef("""
    typedef struct {
        int x;
        int y;
    } point_t;

    void translate(point_t *p, int dx, int dy);
""")
```

Do not put `#include` directives into `cdef()`. Put declarations there, and use `set_source(...)` for C headers and compilation details.

### Allocate memory with `ffi.new()`

```python
point = ffi.new("point_t *", {"x": 10, "y": 20})
lib.translate(point, 3, -5)
print(point.x, point.y)
```

Important string rule:

```python
name = ffi.new("char[]", b"hello")  # C string with trailing NUL
```

`ffi.new("char *")` allocates only one character, not a C string buffer.

### Convert C strings and buffers back to Python

```python
message = ffi.new("char[]", b"hello")
print(ffi.string(message).decode("utf-8"))
```

For byte-oriented reads and writes:

```python
image = ffi.new("unsigned char[]", 1024)
view = ffi.buffer(image)
```

### Keep owners alive

Only the original owning cdata returned by `ffi.new()` keeps the memory alive.

Correct:

```python
buf = ffi.new("char[]", b"hello")
ptr = ffi.new("char **", buf)
lib.consume(ptr)
```

Wrong:

```python
ptr = ffi.new("char **", ffi.new("char[]", b"hello"))
```

The inner allocation can be freed immediately, leaving `ptr` with a dangling pointer.

## Build and Packaging Configuration

For out-of-line modules, `set_source(...)` is the main configuration point.

Common arguments:

- `libraries=[...]`: linker library names
- `include_dirs=[...]`: C header search paths
- `library_dirs=[...]`: linker search paths
- `define_macros=[(...)]`: compile-time macros
- `extra_compile_args=[...]`: compiler flags
- `extra_link_args=[...]`: linker flags

If the dependency exposes `pkg-config` metadata, prefer `set_source_pkgconfig(...)` instead of hard-coding search paths.

For setuptools-based builds, upstream documents:

```python
setup_requires=["cffi>=1.0.0"]
cffi_modules=["package/foo_build.py:ffibuilder"]
install_requires=["cffi>=1.0.0"]
```

Do not start new packaging work on deprecated `distutils` flows.

## Callbacks and Passing Python Context

For callbacks from C into Python, prefer API mode with `extern "Python"` or use a module-level `@ffi.callback(...)`.

Typical context-passing pattern:

```python
handle = ffi.new_handle(py_object)
```

Later, inside the callback:

```python
obj = ffi.from_handle(handle)
```

Keep the handle alive on the Python side for as long as C may call back with it.

## Threading and Concurrency

CFFI does not make an unsafe C library thread-safe. If you publish bindings around a native library, document the thread-safety contract of the wrapped library and add locking in Python when necessary.

`2.0.0` adds support for free-threaded CPython, but only for `3.14t+`. Upstream explicitly says `3.13t` is not supported.

## Common Pitfalls

- `char *` maps to Python `bytes`, not `str`. Encode before passing into C, decode after reading from C.
- `ffi.new("char *")` is not a string allocation. Use `ffi.new("char[]", data)` for C strings.
- In out-of-line ABI mode, `ffi.dlopen("foo")` does not do extra name resolution. Pass a real path or use `ctypes.util.find_library("foo")`.
- `ffi.dlopen(None)` for the standard C library is a Unix pattern; do not assume it is portable to Windows.
- `ffi.verify()` is still supported for compatibility, but upstream marks it deprecated.
- If you store a callback function pointer in C, keep the Python callback object alive for the entire time C may call it.
- If you use `ffi.gc()` for resource cleanup, pair it with explicit release logic when lifetime matters.
- On PyPy, large native allocations may need explicit memory-pressure hints if cleanup depends on GC timing.

## Version-Sensitive Notes for `2.0.0`

- `2.0.0` supports CPython `3.14`, including free-threaded `3.14t+`.
- For free-threaded builds, upstream says extension builds must set `py_limited_api=False`.
- `3.13t` is not supported because upstream reports segfaults from synchronization differences.
- `2.0.0` drops Python `3.8`.
- Since `1.16.0`, projects using CFFI features that depend on `distutils` at runtime on Python `3.12+` need an explicit `setuptools` dependency.

## Practical Agent Guidance

- Reach for ABI mode when the job is "call a known shared library from one Python process".
- Reach for out-of-line API mode when the job is "ship bindings in a package" or "support callbacks cleanly".
- Debug install failures as native build problems first: compiler, `libffi`, headers, linker paths, or wheel availability.
- Debug runtime crashes as ownership, callback lifetime, wrong declarations, or thread-safety problems first.

## Official Sources

- Documentation: https://cffi.readthedocs.io/en/stable/
- Installation: https://cffi.readthedocs.io/en/stable/installation.html
- Overview: https://cffi.readthedocs.io/en/stable/overview.html
- Preparing and distributing modules: https://cffi.readthedocs.io/en/stable/cdef.html
- Using `ffi`/`lib`: https://cffi.readthedocs.io/en/stable/using.html
- What's new: https://cffi.readthedocs.io/en/stable/whatsnew.html
- PyPI registry: https://pypi.org/project/cffi/
