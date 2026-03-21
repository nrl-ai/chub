---
name: package
description: "petl Python package guide for lazy ETL pipelines over CSV, Excel, databases, pandas, and remote files"
metadata:
  languages: "python"
  versions: "1.7.17"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "petl,python,etl,csv,excel,sql,pandas,data"
---

# petl Python Package Guide

## Golden Rule

Use `petl` as a lazy table-transformation toolkit, not as an in-memory dataframe library. Start with `import petl as etl`, read from a concrete source such as CSV, Excel, or a database query, apply explicit transforms, then materialize with `look()`, `tocsv()`, `toxlsx()`, `todb()`, or another sink. Convert types deliberately because text-based sources arrive as strings.

## Install

Pin the package version your project expects:

```bash
python -m pip install "petl==1.7.17"
```

Common alternatives:

```bash
uv add "petl==1.7.17"
poetry add "petl==1.7.17"
```

Optional extras published on PyPI:

```bash
python -m pip install "petl[db,pandas,remote,xlsx]==1.7.17"
```

Available extras on PyPI include `avro`, `db`, `pandas`, `remote`, `xls`, `xlsx`, `xml`, and `all`.

Use extras only for the formats you need:

- `db`: database helpers; you still need the actual DB driver and in some cases SQLAlchemy-compatible tooling
- `pandas`: dataframe bridge helpers
- `remote`: remote URLs through `fsspec`; install provider packages such as `s3fs` when the protocol needs them
- `xlsx`: modern Excel `.xlsx` support via `openpyxl`
- `xls`: legacy Excel `.xls` support

## Initialize And Mental Model

`petl` works with table objects that are usually lazy. A table describes a pipeline; it does not necessarily read all rows immediately.

Typical workflow:

1. Read from a source such as `fromcsv()`, `fromxlsx()`, `fromdb()`, or `fromdataframe()`
2. Apply transforms such as `cut()`, `select()`, `convert()`, `addfield()`, `sort()`, or joins
3. Materialize into a sink such as `tocsv()`, `toxlsx()`, `todb()`, `todataframe()`, or a preview such as `look()`

The root import already exposes the fluent interface, so modern code normally uses `import petl as etl`.

## Core Usage

### Build a CSV pipeline

```python
import petl as etl

table = (
    etl
    .fromcsv("orders.csv")
    .convert("amount", float)
    .addfield("is_large", lambda row: row.amount >= 1000)
    .cut("order_id", "customer_id", "amount", "is_large")
)

table.tocsv("orders-normalized.csv")
```

This is the common pattern for file-to-file ETL:

- read with `fromcsv()`
- normalize types with `convert()`
- derive new columns with `addfield()`
- project only needed fields with `cut()`
- write with `tocsv()`

### Join two datasets

```python
import petl as etl

orders = etl.fromcsv("orders.csv")
customers = etl.fromcsv("customers.csv")

enriched = etl.join(orders, customers, key="customer_id")

print(etl.look(enriched))
```

Use `join()` when both sides can have multiple matching rows. Use `lookupjoin()` only when taking the first matching row from the right-hand table is acceptable.

### Read from a database query

```python
import sqlite3
import petl as etl

conn = sqlite3.connect("app.db")

users = etl.fromdb(conn, "SELECT id, email, created_at FROM users")
recent = users.cut("id", "email")

print(etl.look(recent))
```

For non-SQLite backends, install the appropriate DB driver. If you need SQLAlchemy-backed database helpers, check the version note below about SQLAlchemy 2.x support.

### Bridge to pandas

```python
import pandas as pd
import petl as etl

df = pd.read_csv("orders.csv")
table = etl.fromdataframe(df)

clean = table.convert("amount", float)
result_df = etl.todataframe(clean)
```

Use `petl` when you want deterministic extract/transform/load steps around files and databases, and convert to pandas only when you actually need dataframe operations downstream.

## Configuration And Auth

`petl` has no package-level authentication system and almost no global runtime configuration. Configuration happens at the integration boundary:

- file inputs/outputs: local paths or URLs
- remote filesystems: `fsspec` plus backend-specific packages and credentials
- databases: DB-API connection objects, connection strings, or SQLAlchemy-compatible configuration depending on the helper you use
- Excel/format support: install the optional dependency for the format before calling the corresponding loader or writer

Examples of external auth/config that `petl` relies on rather than manages:

- S3 URLs usually rely on `fsspec` plus `s3fs`, with AWS credentials coming from the normal AWS environment or config chain
- database access relies on the driver or engine configuration you provide
- Excel readers/writers rely on packages such as `openpyxl`, `xlrd`, or `xlwt`

## Common Pitfalls

- CSV and other text sources yield strings by default. Call `convert()` for numeric, date, boolean, or structured fields before doing real logic.
- Pipelines are lazy. Re-iterating a table can re-read the source or recompute the transform chain. Use `cache()` when you need to materialize an intermediate table once and reuse it.
- `look()` is only a preview. It is useful for inspection, not as proof that the full pipeline has already run.
- `lookupjoin()` keeps only the first matching row from the right-hand side. Do not use it when you need full many-to-one or many-to-many semantics.
- Some sources infer headers or schema from the data. When reproducibility matters, define headers and type conversion explicitly instead of relying on inference.
- Remote URLs are not automatic just because the path starts with `s3://` or similar. You need the `remote` extra or equivalent `fsspec` install, plus any provider-specific package.
- Database examples on the web often omit the driver dependency. `petl` does not replace `psycopg`, `pymysql`, `pyodbc`, or other backend drivers.
- For large files, be careful with transforms that force eager materialization or repeated scans. Keep the pipeline streaming where possible and materialize only when necessary.

## Version-Sensitive Notes

- `1.x` uses `import petl as etl` as the normal entry point. Older imports such as `petl.fluent` and `petl.interactive` were retired long ago and should not appear in new code.
- Since the `1.0` line, text I/O was unified around `fromcsv()` and `tocsv()`. Legacy helpers such as `fromucsv()` and `toucsv()` are obsolete.
- Remote filesystem support was added through `fsspec` in the `1.6` line. Prefer the current remote-path approach instead of older ad hoc remote file recipes.
- The stable changelog notes that `1.7.14` does not support SQLAlchemy `2.0`. If your pipeline uses SQLAlchemy-mediated DB helpers, pin SQLAlchemy `<2` unless you have verified newer compatibility in the repository.
- The docs also note that `fromxlsx(read_only=True)` can truncate some LibreOffice-generated `.xlsx` files at `65536` rows. Leave `read_only=False` unless you have a confirmed memory or performance reason to change it.
- The `1.3` line changed `fromxlsx()` offset arguments from `row_offset` and `column_offset` to `min_row` and `min_col`. Old blog posts still use the removed names.

## Official Sources

- PyPI package metadata and release history: `https://pypi.org/project/petl/`
- Maintainer repository: `https://github.com/petl-developers/petl`
- Maintainer docs: `https://petl.readthedocs.io/latest/`
- Stable changelog: `https://petl.readthedocs.io/stable/changes.html`
