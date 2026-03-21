---
name: package
description: "Matplotlib package guide for Python plotting, figures, styles, and backends"
metadata:
  languages: "python"
  versions: "3.10.8"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "matplotlib,python,plotting,visualization,figures"
---

# matplotlib Python Package Guide

## What It Is

`matplotlib` is the standard plotting library for Python. Use it to build static figures, basic interactive plots, and files for reports, notebooks, dashboards, and test artifacts.

For coding agents, the safest default is:

1. Install `matplotlib` into the project environment.
2. Use the object-oriented `Figure` and `Axes` API via `plt.subplots()`.
3. Choose a backend explicitly in headless or notebook environments when display behavior matters.
4. Save files with `fig.savefig(...)` for deterministic output.

## Installation

```bash
pip install matplotlib==3.10.8
```

With `uv`:

```bash
uv add matplotlib==3.10.8
```

Verify the installed version:

```bash
python -c "import matplotlib; print(matplotlib.__version__)"
```

PyPI for `3.10.8` requires Python `>=3.10`. If interactive windows fail to open, the usual problem is not `matplotlib` itself but a missing GUI toolkit or an unsuitable backend for the current environment.

## First Working Plot

Use `plt.subplots()` and operate on the returned `Axes` object instead of building larger scripts around the implicit pyplot state.

```python
import numpy as np
import matplotlib.pyplot as plt

x = np.linspace(0, 2 * np.pi, 200)
y = np.sin(x)

fig, ax = plt.subplots(figsize=(6, 4))
ax.plot(x, y, label="sin(x)")
ax.set_title("Sine wave")
ax.set_xlabel("x")
ax.set_ylabel("y")
ax.legend()
fig.tight_layout()

fig.savefig("sine.png", dpi=150)
plt.show()
```

## Core Usage Patterns

### Preferred interface

Matplotlib documents two major interfaces:

- The explicit object-oriented `Axes` interface
- The implicit `pyplot` interface

Prefer the explicit `Axes` interface for functions, libraries, tests, and any code that needs predictable behavior.

```python
import matplotlib.pyplot as plt

fig, ax = plt.subplots()
ax.plot([1, 2, 3], [2, 4, 3], marker="o")
ax.set(xlabel="step", ylabel="value", title="Run metrics")
fig.tight_layout()
```

### Multiple plots

```python
import matplotlib.pyplot as plt

fig, axes = plt.subplots(nrows=1, ncols=2, figsize=(10, 4))

axes[0].bar(["A", "B", "C"], [3, 5, 2])
axes[0].set_title("Bars")

axes[1].scatter([1, 2, 3, 4], [4, 1, 3, 2])
axes[1].set_title("Scatter")

fig.tight_layout()
fig.savefig("summary.png", dpi=150)
```

### Use NumPy arrays directly

Most examples in the official docs pass lists or NumPy arrays directly into plotting calls. For numeric work, keep the data as arrays and let `Axes.plot`, `Axes.scatter`, `Axes.imshow`, and similar methods consume them directly.

### Save files for agents and CI

For scripts, tests, or non-interactive jobs, prefer explicit file output:

```python
fig.savefig("plot.svg")
fig.savefig("plot.png", dpi=200, bbox_inches="tight")
```

`PNG`, `PDF`, and `SVG` are the usual safe defaults.

## Configuration And Environment

### Backends

The backend controls where figures render: GUI windows, notebooks, or image files. Matplotlib documents three main ways to configure it:

1. `rcParams["backend"]` in configuration
2. The `MPLBACKEND` environment variable
3. `matplotlib.use()`

For headless jobs, set `Agg` before importing `pyplot`:

```python
import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt
```

If you need interactive windows locally, use an interactive backend supported by your environment instead of forcing `Agg`.

### Styles and rcParams

Use styles for broad appearance changes and `rcParams` for targeted defaults.

```python
import matplotlib as mpl
import matplotlib.pyplot as plt

plt.style.use("ggplot")
mpl.rcParams["figure.dpi"] = 150
mpl.rcParams["axes.grid"] = True
```

Project-level defaults can live in a `matplotlibrc` file. Runtime settings override configuration-file defaults.

### Config directory

`MPLCONFIGDIR` points Matplotlib at the directory used for configuration and cache files. Set it in locked-down CI or container environments if the default user cache/config location is not writable.

```bash
export MPLCONFIGDIR="$PWD/.matplotlib"
```

## Notebooks And Interactive Workflows

In notebooks, inline rendering is usually enough:

```python
import matplotlib.pyplot as plt

plt.plot([1, 2, 3], [1, 4, 9])
plt.show()
```

If a project expects richer widget-based interactivity, confirm the notebook backend and any extra dependency such as `ipympl` before writing code around pan/zoom callbacks or live updates.

## Common Pitfalls

### No display in CI, containers, or servers

Symptom: `plt.show()` does nothing useful or backend import errors appear.

Use a non-interactive backend and save files instead:

```python
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
```

### Switching backends too late

`matplotlib.use()` must happen before figures are created. In practice, do it before importing `matplotlib.pyplot`.

### Mixing pyplot state with reusable library code

Stateful pyplot calls are fine for short scripts, but library code is easier to test and compose if it accepts an `Axes` and returns a `Figure` or `Axes`.

```python
def render_series(ax, xs, ys):
    ax.plot(xs, ys)
    ax.set_title("Series")
    return ax
```

### Interactive backend missing dependencies

If the default backend fails on a local desktop, install or enable a supported GUI toolkit for that environment, or fall back to file output.

### Blank or incomplete saved images

Apply layout before saving and save from the `Figure` object you actually modified:

```python
fig.tight_layout()
fig.savefig("figure.png", dpi=150)
```

## Version-Sensitive Notes For 3.10.8

- This guide targets `matplotlib` `3.10.8`, which is also the current stable docs version linked above.
- PyPI for `3.10.8` requires Python `>=3.10`.
- Use the stable documentation for API details that vary by minor release, especially backend behavior, configuration defaults, and notebook integration.
- If you use `uv` and want the `tkagg` backend with Python builds managed by `uv`, Matplotlib's install docs call out `uv` `0.8.7` or later for bundled `tkinter` support.

## Official Sources

- Matplotlib stable API index: https://matplotlib.org/stable/api/index.html
- Getting started: https://matplotlib.org/stable/users/getting_started/
- Installation: https://matplotlib.org/stable/users/installing/index.html
- Application interfaces: https://matplotlib.org/stable/users/explain/figure/api_interfaces.html
- Backends: https://matplotlib.org/stable/users/explain/figure/backends.html
- Customizing with styles and rcParams: https://matplotlib.org/stable/users/explain/customizing.html
- Configuration API: https://matplotlib.org/stable/api/matplotlib_configuration_api.html
- PyPI package page: https://pypi.org/project/matplotlib/
