---
name: package
description: "Dask package guide for Python projects using the official Dask and distributed docs"
metadata:
  languages: "python"
  versions: "2026.1.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "dask,python,parallel,distributed,dataframe,array,scheduling"
---

# Dask Python Package Guide

## Golden Rule

Use Dask for lazy parallel collections and task graphs, and install the extra set that matches the features you actually need. Plain `dask` is enough for some core graph APIs, but `dask.dataframe`, the distributed scheduler, dashboard, and many filesystem integrations require optional dependencies.

## Install

Pin the version your project expects.

For most projects, the safest default is:

```bash
python -m pip install "dask[complete]==2026.1.2"
```

Targeted installs are smaller:

```bash
python -m pip install "dask[array]==2026.1.2"
python -m pip install "dask[dataframe]==2026.1.2"
python -m pip install "dask[distributed]==2026.1.2"
python -m pip install "dask[diagnostics]==2026.1.2"
```

Common alternatives:

```bash
uv add "dask[complete]==2026.1.2"
poetry add "dask[complete]==2026.1.2"
```

Notes:

- Use `dask[complete]` when you are not sure which collections, dataframe engines, or scheduler features the code path will need.
- Remote filesystems usually need backend packages in addition to Dask, such as `s3fs` for `s3://...`, `gcsfs` for `gcs://...`, or `adlfs` for Azure paths.
- If you only install `dask` and then import `dask.dataframe` or `dask.distributed`, import errors are usually missing optional dependencies, not bad Dask syntax.

## Execution Model And Setup

Dask collections are lazy. Build a graph first, then trigger execution with `.compute()`, `.persist()`, or a distributed `Client`.

Local single-process or threaded usage does not require a client:

```python
import dask.array as da

x = da.random.random((50_000, 1_000), chunks=(5_000, 1_000))
mean = x.mean()

print(mean.compute())
```

Start a client when you want the distributed scheduler, dashboard, futures API, or multi-process execution:

```python
from dask.distributed import Client

client = Client()
print(client)
```

For explicit local cluster sizing:

```python
from dask.distributed import Client, LocalCluster

cluster = LocalCluster(
    n_workers=4,
    threads_per_worker=2,
    memory_limit="4GiB",
)
client = Client(cluster)
```

## Core Usage

### DataFrame

Use `dask.dataframe` when pandas would be too large for one process or you want parallel file IO and shuffles.

```python
import dask.dataframe as dd

ddf = dd.read_parquet("data/events/*.parquet")

daily_totals = (
    ddf[["user_id", "amount", "event_date"]]
    .groupby(["event_date", "user_id"])
    .amount.sum()
    .reset_index()
)

result = daily_totals.compute()
print(result.head())
```

If you will reuse an expensive intermediate more than once, persist it once instead of recomputing the whole graph each time:

```python
filtered = ddf[ddf.amount > 0].persist()
top_users = filtered.groupby("user_id").amount.sum()
print(top_users.nlargest(10).compute())
```

### Array

Use `dask.array` for NumPy-like chunked arrays:

```python
import dask.array as da

x = da.from_zarr("data/training-array.zarr")
y = (x - x.mean(axis=0)) / x.std(axis=0)

print(y[:1000].compute())
```

Choose chunk sizes so several chunks fit comfortably in worker memory. Tiny chunks create large graphs and scheduler overhead; giant chunks cause memory pressure.

### Delayed

Use `dask.delayed` for Python functions that are not already expressed as Dask collections:

```python
import dask

@dask.delayed
def load_value(path: str) -> int:
    with open(path) as f:
        return int(f.read().strip())

@dask.delayed
def combine(values: list[int]) -> int:
    return sum(values)

values = [load_value(path) for path in ["a.txt", "b.txt", "c.txt"]]
total = combine(values)

print(total.compute())
```

### Futures And Interactive Work

Use the futures API for request-style workloads, simulations, or task queues where you want to submit work incrementally:

```python
from dask.distributed import Client

client = Client()

futures = client.map(lambda x: x * 2, range(10))
results = client.gather(futures)

print(results)
```

Prefer collections for dataframe and array pipelines. Prefer futures when work arrives dynamically or tasks do not map cleanly onto Dask collections.

## Configuration And Auth

Dask configuration can come from YAML files, environment variables, or `dask.config`.

Typical config locations:

- `~/.config/dask/`
- `/etc/dask/`

Python override:

```python
import dask

with dask.config.set({"temporary-directory": "/mnt/fast-ssd/dask"}):
    ...
```

Example YAML:

```yaml
temporary-directory: /mnt/fast-ssd/dask
distributed:
  worker:
    memory:
      target: 0.75
      spill: 0.85
      pause: 0.95
      terminate: 0.98
```

Example environment variables:

```bash
export DASK_DISTRIBUTED__SCHEDULER__WORK_STEALING=True
export DASK_TEMPORARY_DIRECTORY=/mnt/fast-ssd/dask
```

Authentication notes:

- Core Dask itself does not use API keys.
- Access to `s3://`, `gcs://`, Azure storage, SQL sources, or remote clusters is handled by the corresponding filesystem or cluster backend, not by Dask package config alone.
- Keep cloud credentials in the provider-standard environment or config locations. Do not hard-code them into Dask task functions.

## Common Pitfalls

- Installing only `dask` and then assuming `dask.dataframe` or `dask.distributed` is ready. Use the correct extras or install missing dependencies explicitly.
- Calling `.compute()` inside loops. Build multiple results and execute them together with `dask.compute(...)` when possible.
- Creating extremely small partitions or chunks. This produces huge graphs and poor scheduler efficiency.
- Building extremely large task graphs for work that could be expressed with chunk-aware collection operations like `map_partitions`, `map_blocks`, `read_parquet`, or batched futures.
- Ignoring the dashboard when a job is slow or memory-bound. The official best-practices guidance treats the dashboard as a primary debugging tool.
- Forgetting that remote URLs need filesystem backends and credentials. `dd.read_parquet("s3://...")` will not work with only the base package installed.
- Treating the distributed scheduler as mandatory. Many local workloads are fine with the default scheduler; add `Client()` when you need cluster features, diagnostics, or better control.

## Version-Sensitive Notes For `2026.1.2`

- PyPI lists `dask 2026.1.2` as the current package version, released on January 30, 2026.
- The separately hosted `distributed.dask.org` stable docs may still display `2026.1.1` on some pages as of March 12, 2026. Use PyPI and the Dask changelog as the package-version authority, and use the distributed site for API behavior and scheduler guidance.
- The `2026.1.2` changelog notes that `dask.dataframe` now requires `pyarrow>=16`. If your environment pins an older PyArrow, dataframe imports or parquet-heavy code paths can fail in ways that look unrelated at first glance.
- If you use Zarr writes, `2026.1.2` moved `mode` to an explicit `to_zarr` argument and removed `read_kwargs` and `zarr_array_kwargs`. Older blog examples for those keyword arguments are stale.
