---
name: package
description: "seaborn package guide for Python statistical visualization with matplotlib and pandas"
metadata:
  languages: "python"
  versions: "0.13.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "seaborn,python,visualization,plotting,matplotlib,pandas,statistics"
---

# seaborn Python Package Guide

## What It Is

`seaborn` is a high-level statistical visualization library for Python. It builds on top of `matplotlib`, works closely with `pandas`-style tabular data, and gives you two main ways to work:

- function-based plotting such as `scatterplot`, `relplot`, `displot`, `catplot`, and `heatmap`
- the newer declarative `seaborn.objects` interface via `seaborn.objects.Plot`

For coding agents, the safe default is:

1. Install `seaborn` into the same environment as `matplotlib`, `pandas`, and `numpy`.
2. Import it as `import seaborn as sns`.
3. Use a tidy `DataFrame` with named columns.
4. Call `sns.set_theme()` once near startup if you want seaborn defaults.
5. In scripts or terminals, call `matplotlib.pyplot.show()` or save the figure explicitly.

## Installation

Use the interpreter-targeted install form when possible:

```bash
python -m pip install "seaborn==0.13.2"
```

Include optional statistical dependencies when needed:

```bash
python -m pip install "seaborn[stats]==0.13.2"
```

Conda options from the official install page:

```bash
conda install seaborn
conda install seaborn -c conda-forge
```

Verify the installed version:

```bash
python -c "import seaborn; print(seaborn.__version__)"
```

If import succeeds but plotting fails, the problem is usually in a compiled dependency such as `matplotlib`, `pandas`, or `scipy`, not in seaborn itself.

## Initialize And First Plot

The docs generally assume these imports:

```python
import numpy as np
import pandas as pd

import matplotlib as mpl
import matplotlib.pyplot as plt

import seaborn as sns
import seaborn.objects as so
```

Minimal example:

```python
import matplotlib.pyplot as plt
import seaborn as sns

sns.set_theme()

penguins = sns.load_dataset("penguins")

g = sns.relplot(
    data=penguins,
    x="bill_length_mm",
    y="bill_depth_mm",
    hue="species",
    col="sex",
)

g.figure.tight_layout()
plt.show()
```

Notes:

- `sns.set_theme()` changes global `matplotlib` rcParams, including non-seaborn plots.
- `sns.load_dataset()` pulls example data from seaborn's online repository, so it requires internet access.
- In notebooks, plots often display automatically. In scripts and terminals, call `plt.show()` yourself.

## Core Usage Patterns

### Axes-level vs figure-level functions

Seaborn has both axes-level and figure-level APIs:

- axes-level functions draw on one matplotlib axes and usually return an `Axes`
- figure-level functions create a larger layout and usually return a seaborn grid object such as `FacetGrid`

Use axes-level functions when you already have `fig, ax = plt.subplots()`:

```python
import matplotlib.pyplot as plt
import seaborn as sns

penguins = sns.load_dataset("penguins")

fig, ax = plt.subplots(figsize=(6, 4))

sns.scatterplot(
    data=penguins,
    x="bill_length_mm",
    y="bill_depth_mm",
    hue="species",
    style="species",
    ax=ax,
)

ax.set_title("Penguin bills")
fig.tight_layout()
```

Use figure-level functions when you want faceting or a higher-level layout:

```python
import seaborn as sns

penguins = sns.load_dataset("penguins")

g = sns.relplot(
    data=penguins,
    x="bill_length_mm",
    y="bill_depth_mm",
    hue="species",
    col="island",
    kind="scatter",
)

g.set_axis_labels("Bill length (mm)", "Bill depth (mm)")
g.set_titles("{col_name}")
g.despine(trim=True)
```

### Distribution plots

Use `displot()` for distribution-focused figures and `histplot()`/`kdeplot()` when you want one axes.

```python
import seaborn as sns

tips = sns.load_dataset("tips")

g = sns.displot(
    data=tips,
    x="total_bill",
    col="time",
    hue="smoker",
    kind="hist",
    kde=True,
)
```

### Categorical plots

`catplot()` is the figure-level entry point for strip, swarm, box, violin, boxen, point, bar, and count plots:

```python
import seaborn as sns

tips = sns.load_dataset("tips")

g = sns.catplot(
    data=tips,
    kind="violin",
    x="day",
    y="total_bill",
    hue="smoker",
    split=True,
)
```

In `0.13.x`, categorical plots can keep numeric or datetime spacing with `native_scale=True`:

```python
import seaborn as sns

tips = sns.load_dataset("tips")

sns.stripplot(
    data=tips,
    x="size",
    y="tip",
    native_scale=True,
)
```

### Matrix plots

Use `heatmap()` for already-aggregated matrix-like data:

```python
import seaborn as sns

penguins = sns.load_dataset("penguins")

table = penguins.pivot_table(
    index="species",
    columns="sex",
    values="body_mass_g",
    aggfunc="mean",
)

ax = sns.heatmap(table, annot=True, fmt=".0f", cmap="crest")
ax.set_title("Mean body mass")
```

Use `clustermap()` only when you have the optional clustering dependencies and actually want hierarchical clustering.

### Pairwise summaries

`pairplot()` is a fast high-level way to inspect relationships across several numeric columns:

```python
import seaborn as sns

penguins = sns.load_dataset("penguins")

g = sns.pairplot(
    data=penguins,
    hue="species",
    corner=True,
)
```

When you need more control, build the same kind of figure with `PairGrid`.

### Declarative objects interface

The `objects` interface is useful when you want a composable, chainable API:

```python
import seaborn as sns
import seaborn.objects as so

penguins = sns.load_dataset("penguins")

p = (
    so.Plot(penguins, x="bill_length_mm", y="bill_depth_mm", color="species")
    .add(so.Dots())
    .facet(col="sex")
    .label(
        title="Penguin bills",
        x="Bill length (mm)",
        y="Bill depth (mm)",
    )
)

p.show()
```

`so.Plot` methods return a copy of the plot specification, so build plots through chaining instead of expecting in-place mutation.

## Configuration And Environment

### Themes and rcParams

Use `set_theme()` as the preferred global theme API:

```python
import seaborn as sns

sns.set_theme(
    style="whitegrid",
    context="talk",
    palette="deep",
    rc={"figure.dpi": 110},
)
```

Important related APIs from the reference:

- `sns.set_theme()`
- `sns.set_style()` / `sns.axes_style()`
- `sns.set_context()` / `sns.plotting_context()`
- `sns.set_palette()` / `sns.color_palette()`
- `sns.despine()`

Avoid starting new code with `sns.set(...)`; it is only an alias for `set_theme()` and may be removed in the future.

### Saving figures

Seaborn uses matplotlib for rendering, so save from the returned matplotlib or grid object:

```python
import seaborn as sns

penguins = sns.load_dataset("penguins")

ax = sns.histplot(data=penguins, x="flipper_length_mm")
ax.figure.savefig("histogram.png", dpi=150, bbox_inches="tight")
```

For figure-level objects:

```python
import seaborn as sns

penguins = sns.load_dataset("penguins")

g = sns.relplot(data=penguins, x="bill_length_mm", y="bill_depth_mm")
g.figure.savefig("relplot.png", dpi=150, bbox_inches="tight")
```

For the objects API:

```python
p.save("objects-plot.png")
```

### Data shape expectations

Seaborn works best with long-form or tidy tabular data:

- one row per observation
- one column per variable
- reference columns by name in plotting calls

Some functions also accept wide-form data or raw vectors, but semantics can change. If a plot needs reliable `hue`, `style`, `col`, `row`, or grouping behavior, use a named `DataFrame`.

### Example datasets

`sns.load_dataset()` is for examples, tests, and quick experiments. It is not a general project data loader:

- it requires network access
- dataset availability comes from seaborn's example-data repository
- production code should usually load real data with `pandas.read_csv`, SQL, parquet readers, or project-specific I/O

## Common Pitfalls

### Plot does not appear

In scripts, IDE consoles, and terminals, seaborn follows normal matplotlib behavior. If nothing displays, call:

```python
import matplotlib.pyplot as plt
plt.show()
```

### Treating figure-level return values like `Axes`

`relplot()`, `displot()`, `catplot()`, `jointplot()`, and `pairplot()` do not return a plain `Axes`. They return grid objects. Use methods such as `g.set_axis_labels(...)`, `g.set_titles(...)`, `g.despine(...)`, or access `g.figure`.

### Forgetting that `set_theme()` changes global matplotlib defaults

Theme changes apply to all later matplotlib and seaborn plots in the process. If you only want a local style change, use matplotlib context managers or limit the change to a controlled section of code.

### Optional dependencies missing

Typical failures:

- advanced regression plotting without `statsmodels`
- clustering workflows such as `clustermap()` without `scipy`
- import/runtime issues caused by compiled dependencies rather than seaborn itself

Install the missing dependency instead of trying to work around the seaborn API.

### Example data loader in offline environments

`sns.load_dataset("penguins")` is convenient but not offline-safe. For reproducible automation, ship a local dataset or load from a known project path.

### Using deprecated or changed older examples

- `distplot()` is deprecated
- `sns.set()` is only an alias for `set_theme()`
- many older categorical-plot examples predate the `0.13` rewrite and may not match current defaults

## Version-Sensitive Notes For 0.13.2

- `0.13.2` is the current stable release shown by the official docs and PyPI as of March 12, 2026.
- The `0.13.2` release note says it contains internal changes to adapt to upcoming pandas deprecations, and upstream recommends updating.
- `0.13.0` was a major release for categorical plots. Existing code can change appearance or behavior because:
  - categorical functions now default to a single main color unless `hue` is assigned
  - passing `palette` without `hue` is deprecated
  - `native_scale=True` can preserve numeric or datetime spacing on the categorical axis
  - `log_scale`, `legend`, `formatter`, `fill`, and other parameters were added or broadened
- `0.13.0` also added provisional support for alternate dataframe libraries through the dataframe exchange protocol, though seaborn still converts inputs to pandas internally.
- The `objects.Plot` interface in `0.13.x` has additional configuration controls for default theme and notebook display behavior.

## Official Sources

- Installation and dependencies: https://seaborn.pydata.org/installing
- API reference: https://seaborn.pydata.org/api.html
- Introduction/tutorial: https://seaborn.pydata.org/tutorial/introduction.html
- FAQ: https://seaborn.pydata.org/faq.html
- `set_theme()` reference: https://seaborn.pydata.org/generated/seaborn.set_theme.html
- `objects.Plot` reference: https://seaborn.pydata.org/generated/seaborn.objects.Plot.html
- `scatterplot()` reference: https://seaborn.pydata.org/generated/seaborn.scatterplot.html
- `displot()` reference: https://seaborn.pydata.org/generated/seaborn.displot.html
- `catplot()` reference: https://seaborn.pydata.org/generated/seaborn.catplot.html
- Release notes index: https://seaborn.pydata.org/whatsnew/index.html
- `v0.13.2` release notes: https://seaborn.pydata.org/whatsnew/v0.13.2.html
- `v0.13.0` release notes: https://seaborn.pydata.org/whatsnew/v0.13.0.html
- PyPI project page: https://pypi.org/project/seaborn/
