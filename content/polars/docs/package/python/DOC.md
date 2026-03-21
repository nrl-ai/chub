---
name: package
description: "polars package guide for Python DataFrame, LazyFrame, expressions, IO, and cloud-backed analytics workflows"
metadata:
  languages: "python"
  versions: "1.38.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "polars,dataframe,lazyframe,columnar,parquet,sql"
---

# polars Python Package Guide

## What It Is

`polars` is a columnar DataFrame library for Python built around expressions, lazy query planning, and fast parquet/csv/ndjson/database-style data workflows.

Use it when you need:

- DataFrame operations that stay vectorized instead of row-by-row Python loops
- lazy query planning with predicate and projection pushdown
- efficient parquet and IPC/Arrow interop
- SQL-style analytics on local files, in-memory frames, or cloud-backed datasets

For new code, prefer the expression API and the lazy API unless you specifically need eager materialization.

## Install

Base install:

```bash
python -m pip install polars==1.38.1
```

Common alternatives:

```bash
uv add polars==1.38.1
poetry add polars==1.38.1
conda install -c conda-forge polars=1.38.1
```

Useful optional extras from the official package metadata:

```bash
python -m pip install "polars[numpy,fsspec]==1.38.1"
python -m pip install "polars[pyarrow]==1.38.1"
python -m pip install "polars[all]==1.38.1"
```

Notes:

- `fsspec` is commonly needed for filesystem integrations and some remote-storage workflows.
- `pyarrow` is optional for many `polars` tasks, but it is useful when a codebase already depends on Arrow or pandas/Arrow interchange.
- The PyPI page also documents runtime compatibility wheels such as `rtcompat` and `rt64` for specific environments; use them only when the upstream install guidance calls for them.

## Import And First Frame

```python
import polars as pl

df = pl.DataFrame(
    {
        "city": ["sf", "nyc", "la"],
        "count": [10, 7, 5],
        "active": [True, True, False],
    }
)

print(df)
print(df.schema)
```

`polars` uses a few core types repeatedly:

- `pl.DataFrame` for eager tabular data
- `pl.LazyFrame` for deferred query execution
- `pl.Series` for a single column
- `pl.Expr` for reusable column expressions inside `select`, `with_columns`, `filter`, `group_by`, and joins

## Core Usage

### Expressions First

The official user guide treats expressions as the main building block. Write transformations with `pl.col(...)`, literals, and expression methods instead of Python loops.

```python
import polars as pl

result = (
    df
    .filter(pl.col("active"))
    .with_columns(
        count_doubled=pl.col("count") * 2,
        city_upper=pl.col("city").str.to_uppercase(),
    )
    .select("city_upper", "count_doubled")
)
```

`DataFrame.with_columns(...)` returns a new frame but does not copy existing data buffers just to add or replace derived columns.

### Prefer Lazy Execution For Pipelines

Use `scan_*` for file-backed work and keep the query lazy until the end.

```python
import polars as pl

q = (
    pl.scan_parquet("data/events/*.parquet")
    .filter(pl.col("event_date") >= pl.date(2026, 1, 1))
    .group_by("user_id")
    .agg(
        events=pl.len(),
        total_amount=pl.col("amount").sum(),
    )
    .sort("total_amount", descending=True)
)

result = q.collect()
```

Why this matters:

- `scan_parquet`, `scan_csv`, and similar lazy readers allow predicate/projection pushdown
- the optimizer can remove unused columns and push filters closer to the source
- `read_csv(...).lazy()` is usually the wrong starting point because the file was already read eagerly

For large pipelines, the upstream docs also expose streaming execution on supported plans via `collect(engine="streaming")` and sink APIs.

### Read And Write Files

Eager read when the data is small and immediate materialization is fine:

```python
import polars as pl

df = pl.read_csv(
    "input.csv",
    try_parse_dates=True,
    infer_schema_length=10_000,
)
```

Lazy read when the dataset is large or you will filter/select before collecting:

```python
import polars as pl

q = pl.scan_csv(
    "input.csv",
    try_parse_dates=True,
)

result = q.select("id", "ts", "value").filter(pl.col("value") > 0).collect()
```

Write parquet for downstream analytics:

```python
import polars as pl

df.write_parquet(
    "output.parquet",
    compression="zstd",
    statistics=True,
)
```

### Grouping, Joins, And Windowed Logic

```python
import polars as pl

orders = pl.DataFrame(
    {
        "user_id": [1, 1, 2, 3],
        "amount": [40, 60, 10, 25],
    }
)

users = pl.DataFrame(
    {
        "user_id": [1, 2, 3],
        "plan": ["pro", "free", "pro"],
    }
)

summary = (
    orders
    .group_by("user_id")
    .agg(total_amount=pl.col("amount").sum())
    .join(users, on="user_id", how="left")
    .with_columns(
        amount_rank=pl.col("total_amount").rank(descending=True)
    )
)
```

`polars` also supports SQL via `SQLContext`, but the expression API is the primary interface and is usually the best default for generated code.

### Pandas And Arrow Interop

```python
import pandas as pd
import polars as pl

pandas_df = pd.DataFrame({"x": [1, 2, 3]})
pl_df = pl.from_pandas(pandas_df)

round_trip = pl_df.to_pandas()
```

Use Arrow/pandas interop when a surrounding library requires it, but keep your main transformation pipeline in native `polars` when possible.

## Config And Auth

`polars` has no service-level authentication model for ordinary local DataFrame work. The main configuration surfaces are display/runtime settings and storage credentials for remote IO.

### Display And Session Config

Use `pl.Config` to adjust table formatting or set temporary display behavior:

```python
import polars as pl

with pl.Config(tbl_rows=20, fmt_str_lengths=80):
    print(df)
```

For debugging environment issues, the API reference also exposes `pl.show_versions()`.

### Cloud And Remote Storage Credentials

Remote readers/writers such as `scan_parquet`, `read_parquet`, and related APIs support `storage_options=` and, for credentialed workflows, `credential_provider=`.

Typical shape:

```python
import polars as pl

q = pl.scan_parquet(
    "s3://analytics-bucket/events/date=2026-03-12/*.parquet",
    storage_options={
        "aws_region": "us-west-2",
    },
)
```

If the codebase needs a shared default credential source, `pl.Config.set_default_credential_provider(...)` is the official API surface to look at. In practice, many projects rely on provider-native environment variables and let the underlying cloud auth chain resolve credentials.

## Common Pitfalls

- Eager-vs-lazy confusion: do not start with `read_csv(...).lazy()` when `scan_csv(...)` can avoid unnecessary upfront IO.
- Python loops: avoid `for`-loop row processing and `DataFrame.apply`-style patterns when an expression exists. Generated code should stay inside the expression engine whenever possible.
- Schema inference surprises: for CSV and loosely typed input, set `schema=`, `schema_overrides=`, or a larger `infer_schema_length` when type stability matters.
- Row orientation: when constructing a frame from row-like records in ambiguous cases, pass `orient="row"` explicitly.
- Strict casting and construction: version 1 tightened several behaviors. Expect invalid values to error unless you intentionally opt into non-strict behavior.
- Lazy schema access: on modern Polars, `LazyFrame.schema`, `dtypes`, `columns`, and `width` can trigger work and generate performance warnings. Prefer `collect_schema()` when you need schema information for a lazy plan.
- String/object assumptions from pandas: Polars is stricter about dtypes and null handling than pandas. Cast deliberately instead of assuming implicit coercion.
- SQL is optional: SQL support exists, but most docs, optimizations, and community examples are built around expressions. If SQL and expression examples disagree, prefer the expression-native API.

## Version-Sensitive Notes

- The version used here `1.38.1` matches the current PyPI release as of `2026-03-12`.
- The Python API reference URL in this package brief is the stable docs root, not a version-pinned `1.38.1` snapshot. Re-check the changelog or release notes before copying behavior into code that must exactly match older releases.
- The official Version 1 upgrade guide is still relevant for current code because it changed strictness defaults and several APIs that older blog posts still use.
- From the Version 1 guide, the constructor is stricter, ambiguous row-oriented construction should use `orient="row"`, many `replace` patterns moved to `replace_strict`, and lazy schema inspection should use `collect_schema()`.
- Streaming execution is opt-in and plan-dependent. Do not assume every lazy query automatically runs in streaming mode.

## Practical Guidance For Agents

1. Start with `import polars as pl` and keep transformations in `select`, `with_columns`, `filter`, `group_by`, and joins.
2. Use `scan_parquet` / `scan_csv` for file pipelines, then `.collect()` only at the boundary where materialized results are actually needed.
3. Prefer explicit casts and schema overrides when reading CSV, JSON, or heterogeneous Python objects.
4. Keep cloud IO auth simple: start with provider-native environment credentials, then add `storage_options` or `credential_provider` only when the upstream docs require it.
5. If a project mixes pandas and Polars, convert at the edges rather than bouncing repeatedly between the two in the middle of a pipeline.

## Official Sources

- Docs root: `https://docs.pola.rs/`
- Installation: `https://docs.pola.rs/user-guide/installation/`
- User guide: `https://docs.pola.rs/user-guide/`
- Expressions and contexts: `https://docs.pola.rs/user-guide/concepts/expressions-and-contexts/`
- SQL interface: `https://docs.pola.rs/user-guide/sql/intro/`
- Python API reference: `https://docs.pola.rs/api/python/stable/reference/`
- `LazyFrame` reference: `https://docs.pola.rs/api/python/stable/reference/lazyframe/`
- `scan_parquet`: `https://docs.pola.rs/api/python/stable/reference/api/polars.scan_parquet.html`
- `Config.set_default_credential_provider`: `https://docs.pola.rs/api/python/stable/reference/api/polars.Config.set_default_credential_provider.html`
- `show_versions`: `https://docs.pola.rs/api/python/stable/reference/api/polars.show_versions.html`
- Version 1 upgrade guide: `https://docs.pola.rs/releases/upgrade/1/`
- PyPI package page: `https://pypi.org/project/polars/`
