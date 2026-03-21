---
name: package
description: "pandas package guide for Python 3.0.1: installation, DataFrame workflows, IO, options, and 3.0 migration notes"
metadata:
  languages: "python"
  versions: "3.0.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "pandas,dataframe,data-analysis,io,timeseries,pyarrow"
---

# pandas Python Package Guide

## Golden Rule

Use `pandas` when you need labeled tabular data in Python: `Series` for 1D data and `DataFrame` for 2D data. Treat the official getting started, user guide, and API reference as the source of truth for method behavior and optional IO dependencies.

## Installation

### pip

```bash
pip install pandas==3.0.1
```

### conda-forge

```bash
conda install -c conda-forge pandas
```

### Common optional extras

```bash
pip install "pandas[excel]"
pip install "pandas[performance]"
pip install "pandas[aws]"
pip install "pandas[parquet]"
pip install "pandas[all]"
```

Use extras only when the workflow needs them. pandas raises `ImportError` when you call a method that depends on an optional package you did not install.

Examples from the official docs:

- Excel IO needs engines such as `openpyxl`, `python-calamine`, or `xlsxwriter`
- SQL workflows commonly use `sqlalchemy`
- Remote filesystem URLs rely on `fsspec`-style backends
- Performance-sensitive work can benefit from `numexpr`, `bottleneck`, and `numba`

## Initialize And Inspect Data

```python
import pandas as pd

df = pd.DataFrame(
    {
        "city": ["SF", "NYC", "SEA"],
        "sales": [10, 15, 7],
        "date": pd.to_datetime(["2026-01-01", "2026-01-02", "2026-01-03"]),
    }
)

print(df.dtypes)
print(df.head())
print(df.describe(include="all"))
```

Use `DataFrame` for table-shaped data and `Series` for a single labeled column. The package overview and API reference organize pandas around `Series`, `DataFrame`, `GroupBy`, `Resampling`, `Window`, IO, and options/settings.

## Core Workflows

### Read and write tabular data

```python
import pandas as pd

df = pd.read_csv("input.csv")
df.to_parquet("output.parquet", index=False)
```

`read_*` and `to_*` cover CSV, Excel, JSON, HTML, XML, Parquet, Feather, SQL, and more.

If you need Excel:

```python
df = pd.read_excel("input.xlsx")
df.to_excel("output.xlsx", index=False)
```

Make sure the matching engine is installed first.

### Select, filter, and assign

```python
active = df.loc[df["sales"] > 8, ["city", "sales"]]
first_two_rows = df.iloc[:2, :]

df.loc[:, "sales_with_tax"] = df["sales"] * 1.1
```

Use:

- `[]` for simple column access
- `loc` for label-based row/column selection
- `iloc` for position-based selection

### Aggregate with groupby

```python
summary = (
    df.groupby("city", dropna=False)["sales"]
    .agg(["count", "sum", "mean"])
    .sort_values("sum", ascending=False)
)
```

`groupby` is the main split-apply-combine tool. Use `value_counts()` for quick category counts.

### Join and reshape

```python
customers = pd.DataFrame({"customer_id": [1, 2], "name": ["A", "B"]})
orders = pd.DataFrame({"customer_id": [1, 1, 2], "total": [25, 30, 20]})

joined = customers.merge(orders, on="customer_id", how="left")
```

Use `merge`, `join`, `concat`, `pivot`, `pivot_table`, `stack`, and `unstack` for relational and reporting-style transforms.

### Time series

```python
ts = (
    df.set_index("date")
    .sort_index()
    .resample("MS")["sales"]
    .sum()
)
```

Use `to_datetime`, `date_range`, timezone-aware dtypes, rolling windows, and `resample()` for date-based pipelines.

### Expression-based column creation

`pandas 3.0` added `pd.col()` for expression-style column references:

```python
df = df.assign(double_sales=pd.col("sales") * 2)
```

This is useful in `assign`, `loc`, and similar places that accept callables returning a `Series`.

## Config And Backend Integration

### Display and runtime options

Use the options API for temporary or global formatting changes:

```python
import pandas as pd

pd.set_option("display.max_rows", 200)

with pd.option_context("display.max_columns", 20, "display.width", 120):
    print(df)
```

Important options APIs:

- `pd.get_option()`
- `pd.set_option()`
- `pd.reset_option()`
- `pd.describe_option()`
- `pd.option_context()`

Use full option names in code. Short patterns can become ambiguous across releases.

### Remote files and cloud storage

pandas itself does not have package-level authentication. Credentials come from the underlying storage or database backend.

For remote files, use URL-aware readers and pass backend configuration through `storage_options` when needed:

```python
import pandas as pd

df = pd.read_csv(
    "s3://my-bucket/path/data.csv",
    storage_options={"anon": False},
)
```

The IO guide also shows passing backend-specific client settings via `storage_options`, for example `client_kwargs` for S3-compatible endpoints.

### SQL connections

```python
import pandas as pd
from sqlalchemy import create_engine

engine = create_engine("postgresql+psycopg://user:pass@host/db")
df = pd.read_sql("select * from orders", engine)
```

Put connection credentials in your database URL, driver config, environment, or secret manager. pandas reads through the DBAPI/SQLAlchemy connection you provide.

If you care about preserving database types more closely, the IO guide recommends `dtype_backend="pyarrow"` for `read_sql()` round-trips.

## Common Pitfalls

### Missing optional dependencies

Many pandas methods work only when their backend package is installed. Typical failures:

- `read_excel()` without an Excel engine
- `read_parquet()` without Parquet support
- `read_csv("s3://...")` without the appropriate filesystem backend
- `to_markdown()` without `tabulate`

Install the matching extra or dependency before changing code.

### Chained assignment changed in pandas 3.0

In pandas 3.0, chained assignment no longer works as a mutation pattern. Old code like this is wrong:

```python
df[df["sales"] > 0]["sales"] = 0
```

Write the mutation in one step instead:

```python
df.loc[df["sales"] > 0, "sales"] = 0
```

### String dtype assumptions changed

The pandas 3.0 release notes call out a new default string dtype. Code that assumes string columns stay `object` dtype, or code that depends on a specific missing-value sentinel, can break after upgrading.

### Be explicit with indexes before joins and resampling

`merge`, `groupby`, and `resample` are easier to reason about when you normalize dtypes first and make the relevant key or timestamp column explicit.

```python
df["date"] = pd.to_datetime(df["date"], utc=True)
df["customer_id"] = df["customer_id"].astype("int64")
```

### Large-data performance is opt-in

pandas is flexible, not magically distributed. For large datasets:

- install performance extras
- prefer vectorized operations over Python loops
- choose columnar formats like Parquet when possible
- use `dtype_backend="pyarrow"` selectively when it improves downstream interoperability

## Version-Sensitive Notes For 3.0.1

- `pandas 3.0.0` was released on January 21, 2026 and introduced breaking behavior around Copy-on-Write semantics and chained assignment.
- `pandas 3.0.0` also introduced `pd.col()` expression support.
- `pandas 3.0.1` was released on February 17, 2026 and fixes early 3.0 regressions, including:
  - unary operators on `pd.col()`
  - some pyarrow-backed string operations
  - a `merge()` bug involving `NaN` values in pyarrow-backed string join keys on Windows with pyarrow 21
  - Copy-on-Write handling in some constructor paths

If you are on `3.0.0` and using `pd.col()`, pyarrow-backed strings, or Windows joins, prefer `3.0.1`.

## Official Sources

- Docs root: https://pandas.pydata.org/docs/
- API reference: https://pandas.pydata.org/docs/reference/index.html
- Installation: https://pandas.pydata.org/docs/getting_started/install.html
- Package overview: https://pandas.pydata.org/docs/getting_started/overview.html
- Tutorials index: https://pandas.pydata.org/docs/getting_started/intro_tutorials/index.html
- IO guide: https://pandas.pydata.org/docs/user_guide/io.html
- Options guide: https://pandas.pydata.org/docs/user_guide/options.html
- Release notes 3.0.0: https://pandas.pydata.org/docs/whatsnew/v3.0.0.html
- Release notes 3.0.1: https://pandas.pydata.org/docs/whatsnew/v3.0.1.html
- PyPI package: https://pypi.org/project/pandas/3.0.1/
