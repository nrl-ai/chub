---
name: package
description: "Hamilton PyPI package guide for the legacy 0.1.0 mechanics GUI package"
metadata:
  languages: "python"
  versions: "0.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "hamilton,python,mechanics,gui,legacy,pydy"
---

# Hamilton Python Package Guide

## Golden Rule

Do not assume `pip install hamilton` gives you the modern Apache Hamilton DAG framework.

The PyPI package named `Hamilton` is a separate legacy project published at version `0.1.0` on August 9, 2013. Its maintainer-provided description presents it as a GUI application for determining and solving Hamilton equations, with a workflow centered on launching `GUI-Hamilton.py` from Spyder. If you actually need the modern dataflow framework, use the official `sf-hamilton` package instead.

## What This Package Actually Is

Upstream describes `Hamilton` as a Python application for determining and solving Hamilton equations. The published instructions are oriented around:

- installing scientific Python dependencies manually
- unpacking the source archive
- running `python setup.py install`
- opening `GUI-Hamilton.py` in Spyder and pressing `F5`

This is not documented as a conventional library-first package with a stable import API.

## Install

Prefer a disposable virtual environment or container. The published package is old enough that modern Python compatibility is uncertain.

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip setuptools wheel
python -m pip install "Hamilton==0.1.0"
```

If installation from PyPI fails because the package expects an older scientific stack, inspect the source archive instead of forcing it into a modern production environment.

```bash
python -m pip download "Hamilton==0.1.0"
tar -xf Hamilton-0.1.0.tar.gz
cd Hamilton-0.1.0
python setup.py install
```

## Documented Dependencies

The maintainer-provided PyPI description names these prerequisites:

- `Qt`
- `SIP`
- `PyQt`
- `Sympy`
- `Numpy`
- `Scipy`
- `Matplotlib`
- `Control`
- `Slycot`

Treat that dependency list as historical guidance, not a modern lockfile. Some package names, binary wheels, and Python-version support have changed since 2013.

## Initialize And Run

The only documented startup flow is GUI-based:

```bash
python GUI-Hamilton.py
```

If the installed package does not place `GUI-Hamilton.py` on your path, inspect the unpacked source tree or installed files under `site-packages` to find the entry script.

Because the upstream instructions explicitly reference Spyder, expect an interactive desktop workflow rather than a headless server or importable SDK workflow.

## Core Usage Expectations

There is no maintainer-published API reference for functions, classes, or a supported `import hamilton` surface.

For coding-agent work, the practical approach is:

1. Confirm that `Hamilton` is really the intended dependency and not `sf-hamilton`.
2. Inspect the installed files or unpacked source archive before writing code against it.
3. Treat it as a legacy GUI/scientific application unless your project already has working examples.
4. Avoid inventing module names, CLI flags, or object APIs that are not visible in the source tree you installed.

## Configuration

The official package page does not document environment variables, auth, network configuration, or a structured config file format.

Assume:

- no authentication model
- local desktop execution
- configuration, if any, is embedded in the GUI flow or source files

If you need reproducible automation, inspect the source directly and wrap only the parts you can verify.

## Common Pitfalls

- `pip install hamilton` is easy to confuse with the modern Apache Hamilton project. They are different packages.
- The package is old and may not install cleanly on current Python versions or current Qt bindings.
- Upstream docs are minimal and do not define a stable Python import API.
- The published instructions use `setup.py` and Spyder-era tooling, which is a strong sign that the package predates modern packaging norms.
- Scientific dependencies such as `slycot` can be hard to build on clean machines without platform-specific system libraries.
- The package name is capitalized on PyPI (`Hamilton`), but import paths and script names should be verified from the installed files instead of guessed.

## Version-Sensitive Notes

- The only release shown on PyPI is `0.1.0`.
- The previous docs URL (`http://pypi.python.org/pypi/Hamilton/`) is a legacy PyPI page, not a maintained documentation site.
- As of March 12, 2026, the current PyPI project page still points to the same legacy `0.1.0` release with a 2013 description.
- If your real goal is the actively maintained Hamilton framework, the official package is `sf-hamilton`, and the official install docs say to install that package rather than `hamilton`.

## If You Actually Need Apache Hamilton

Use the official modern package instead:

```bash
python -m pip install "sf-hamilton[visualization]"
```

Official docs for that separate project:

- `https://pypi.org/project/sf-hamilton/`
- `https://hamilton.apache.org/get-started/install/`

Do not mix examples or imports between `Hamilton==0.1.0` and `sf-hamilton`.
