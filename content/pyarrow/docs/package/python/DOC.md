---
name: package
description: "pyarrow package guide for Python: Arrow arrays and tables, Parquet, datasets, filesystems, and pandas interop"
metadata:
  languages: "python"
  versions: "23.0.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "pyarrow,apache-arrow,columnar,parquet,dataset,pandas"
---

# pyarrow Python Package Guide

## What This Package Is For

`pyarrow` is the Python binding for Apache Arrow. Use it when you need:

- in-memory columnar data structures such as `Array`, `Table`, `ChunkedArray`, and `RecordBatch`
- Parquet, IPC/Feather, CSV, JSON, ORC, and dataset IO
- fast conversion between Arrow and pandas
- filesystem access for local files and object stores such as S3
- vectorized compute kernels through `pyarrow.compute`

If a project only needs pandas CSV or JSON loading, `pyarrow` may be unnecessary. Reach for it when Arrow-native types, Parquet, datasets, or cross-language columnar data matter.

## Install

`pyarrow` 23.0.1 supports Python 3.10 and newer.

```bash
pip install pyarrow==23.0.1
```

Common alternatives:

```bash
uv add pyarrow==23.0.1
poetry add pyarrow==23.0.1
conda install -c conda-forge pyarrow=23.0.1
```

For timezone-aware timestamp work on Windows or minimal Linux images, also install timezone data if your environment does not already provide it:

```bash
pip install tzdata
```

## Import Pattern

```python
import pyarrow as pa
import pyarrow.compute as pc
import pyarrow.parquet as pq
import pyarrow.dataset as ds
```

Add other modules only when needed:

```python
from pyarrow import csv
from pyarrow import fs
```

## Core Data Structures

### Arrays and Schemas

Use Arrow types explicitly when nullability, timestamp precision, decimals, nested types, or cross-language compatibility matter.

```python
import pyarrow as pa

schema = pa.schema(
    [
        ("id", pa.int64()),
        ("name", pa.string()),
        ("score", pa.float64()),
        ("active", pa.bool_()),
    ]
)

ids = pa.array([1, 2, 3], type=pa.int64())
names = pa.array(["a", "b", None], type=pa.string())
```

### Tables

`pa.table(...)` is the quickest way to build a table from Python objects. Use an explicit schema when you need stable downstream typing.

```python
import pyarrow as pa

table = pa.table(
    {
        "id": [1, 2, 3],
        "name": ["alice", "bob", "carol"],
        "score": [9.2, 8.7, 9.8],
    }
)

print(table.schema)
print(table.num_rows, table.num_columns)
```

For batch-oriented pipelines:

```python
batch = pa.record_batch(
    [
        pa.array([1, 2, 3]),
        pa.array(["x", "y", "z"]),
    ],
    names=["id", "label"],
)
```

## Compute Kernels

Use `pyarrow.compute` for vectorized operations over Arrow arrays and table columns.

```python
import pyarrow as pa
import pyarrow.compute as pc

values = pa.array([1, 2, None, 4, 5])

clean = pc.fill_null(values, 0)
filtered = pc.filter(clean, pc.greater(clean, 2))
total = pc.sum(clean).as_py()

print(filtered)
print(total)
```

For dataset scanning, build expressions with `pc.field(...)`:

```python
predicate = (pc.field("year") == 2026) & (pc.field("amount") > 0)
```

## Pandas Interop

### pandas to Arrow

Use `Table.from_pandas` when Arrow is an IO or compute layer under a pandas workflow.

```python
import pandas as pd
import pyarrow as pa

df = pd.DataFrame(
    {
        "id": [1, 2, 3],
        "name": ["alice", "bob", None],
        "created_at": pd.to_datetime(
            ["2026-03-01T10:00:00Z", "2026-03-02T10:00:00Z", "2026-03-03T10:00:00Z"],
            utc=True,
        ),
    }
)

table = pa.Table.from_pandas(df, preserve_index=False)
```

### Arrow to pandas

```python
round_tripped = table.to_pandas()
```

Important behavior:

- `preserve_index=False` avoids writing pandas index columns unless you need them.
- pandas `object` columns infer poorly when values are mixed or mostly null; pass an explicit schema if types matter.
- Arrow preserves more exact typing than pandas. Converting back to pandas can widen integer columns with nulls unless you opt into pandas nullable dtypes in your own pipeline.

## Parquet

`pyarrow.parquet` is the direct Parquet API. Use it for single-file reads and writes.

### Write a Parquet file

```python
import pyarrow as pa
import pyarrow.parquet as pq

table = pa.table(
    {
        "id": [1, 2, 3],
        "country": ["us", "ca", "us"],
        "amount": [125.5, 89.0, 42.25],
    }
)

pq.write_table(
    table,
    "data/example.parquet",
    compression="zstd",
    row_group_size=100_000,
)
```

### Read a Parquet file

```python
import pyarrow.parquet as pq

table = pq.read_table("data/example.parquet")
```

### Read selected columns or filtered rows

```python
import pyarrow.parquet as pq

table = pq.read_table(
    "data/example.parquet",
    columns=["id", "amount"],
    filters=[("country", "=", "us")],
)
```

Use `filters` only when the underlying file or dataset layout can benefit from predicate pushdown. For partitioned directory trees, `pyarrow.dataset` is usually a better fit than direct `pq.read_table`.

## Datasets

Use `pyarrow.dataset` for partitioned data, directory trees, multi-file scans, and dataset-style writes.

### Open a dataset

```python
import pyarrow.dataset as ds

dataset = ds.dataset("warehouse/events", format="parquet", partitioning="hive")
table = dataset.to_table(columns=["event_id", "year", "amount"])
```

### Scan with a predicate

```python
import pyarrow.compute as pc
import pyarrow.dataset as ds

dataset = ds.dataset("warehouse/events", format="parquet", partitioning="hive")

table = dataset.to_table(
    filter=(pc.field("year") == 2026) & (pc.field("amount") > 0),
    columns=["event_id", "amount", "year"],
)
```

### Write a partitioned dataset

```python
import pyarrow as pa
import pyarrow.dataset as ds

table = pa.table(
    {
        "event_id": [1, 2, 3],
        "year": [2026, 2026, 2025],
        "month": [3, 3, 12],
        "amount": [10, 20, 30],
    }
)

ds.write_dataset(
    table,
    base_dir="warehouse/events",
    format="parquet",
    partitioning=["year", "month"],
    existing_data_behavior="overwrite_or_ignore",
)
```

Common write controls worth remembering:

- `partitioning=...` controls the directory layout.
- `existing_data_behavior=...` determines whether existing files are overwritten, ignored, or cause failure.
- `basename_template=...` is useful when deterministic output filenames matter.

## Filesystems and Cloud/Object Storage

Use `pyarrow.fs` when you want Arrow-native filesystem objects instead of fsspec wrappers.

### Local filesystem

```python
from pyarrow import fs

local = fs.LocalFileSystem()
```

### Build a filesystem from a URI

```python
from pyarrow import fs

filesystem, path = fs.FileSystem.from_uri("s3://my-bucket/datasets/events")
print(type(filesystem), path)
```

### S3

```python
from pyarrow import fs

s3 = fs.S3FileSystem(
    region="us-east-1",
    access_key="AWS_ACCESS_KEY_ID",
    secret_key="AWS_SECRET_ACCESS_KEY",
)
```

`S3FileSystem` can also use ambient AWS credentials, assumed roles (`role_arn=...`), anonymous access (`anonymous=True`), and custom endpoints for S3-compatible stores.

You can combine a filesystem with dataset APIs:

```python
import pyarrow.dataset as ds
from pyarrow import fs

s3 = fs.S3FileSystem(region="us-east-1")
dataset = ds.dataset(
    "my-bucket/warehouse/events",
    filesystem=s3,
    format="parquet",
    partitioning="hive",
)
```

### fsspec interop

If the rest of the project already uses fsspec, wrap it with `PyFileSystem(FSSpecHandler(...))` instead of rewriting the storage layer.

## Initialization Checklist

For most projects, the minimal setup is:

1. Install `pyarrow` at the version your project expects.
2. Decide whether your primary interface is `Table`, `dataset`, or pandas interop.
3. Pin explicit Arrow schemas for columns that must stay stable across files or services.
4. Choose the right IO layer:
   `pyarrow.parquet` for single files, `pyarrow.dataset` for partitioned collections, `pyarrow.fs` for storage access.
5. Verify object-store credentials before debugging dataset reads.

## Common Pitfalls

- Arrow types are stricter than pandas and Python containers. Inference can surprise you when values are mixed, nested, or mostly null.
- `Table.from_pandas` will preserve indexes unless you disable that behavior.
- pandas `object` columns are a common source of unexpected Arrow schemas. Prefer explicit pandas dtypes or an explicit Arrow schema.
- `pq.read_table()` on a directory may work, but `ds.dataset()` is the better abstraction for partitioned datasets and scanner-style filtering.
- Predicate pushdown only helps when the file format and layout support it. Do not assume every filter avoids reading all data.
- S3 issues are often credential or region issues, not Parquet issues. Confirm filesystem setup independently.
- Timezone-aware timestamps may need system tzdata available in the runtime.

## Version-Sensitive Notes For 23.0.1

- The 23.0.1 docs and PyPI metadata both point to Python 3.10+ support. Do not assume older Python versions are still supported.
- Prefer the dataset APIs for modern partitioned-lake patterns rather than older file-by-file loops.
- Use explicit schemas for data interchange boundaries. Arrow and pandas keep evolving their dtype interop, and implicit inference is where regressions usually show up first.
- When copying examples from older blog posts, re-check parameter names against the 23.0.1 API reference, especially dataset and filesystem APIs.

## Official Sources Used

- PyPI package page: https://pypi.org/project/pyarrow/
- Python docs root: https://arrow.apache.org/docs/python/
- API reference: https://arrow.apache.org/docs/python/api.html
- Install guide: https://arrow.apache.org/docs/python/install.html
- Getting started: https://arrow.apache.org/docs/python/getstarted.html
- Compute guide: https://arrow.apache.org/docs/python/compute.html
- Parquet guide: https://arrow.apache.org/docs/python/parquet.html
- Dataset guide: https://arrow.apache.org/docs/python/dataset.html
- Filesystems guide: https://arrow.apache.org/docs/python/filesystems.html
- pandas integration guide: https://arrow.apache.org/docs/python/pandas.html
