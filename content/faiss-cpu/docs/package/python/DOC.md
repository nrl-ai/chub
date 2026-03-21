---
name: package
description: "Faiss CPU package guide for Python vector search, index selection, training, persistence, and common pitfalls"
metadata:
  languages: "python"
  versions: "1.13.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "faiss,faiss-cpu,python,vector-search,similarity-search,ann,embeddings"
---

# Faiss CPU Python Package Guide

## What It Is

`faiss-cpu` is the PyPI wheel for Faiss, Meta's library for dense vector similarity search and clustering. In Python you install `faiss-cpu` but import it as `faiss`.

Use it when you need:

- exact nearest-neighbor search on dense vectors
- approximate nearest-neighbor search on larger datasets
- vector indexing with IVF, PQ, HNSW, scalar quantization, or factory strings
- local in-process search without a remote service

## Install

Install the older pinned package version if you need to match an existing environment:

```bash
python -m pip install faiss-cpu==1.13.2
```

Verify the installed package and import path:

```bash
python - <<'PY'
from importlib.metadata import version
import faiss

print("faiss-cpu:", version("faiss-cpu"))
print("faiss module:", faiss.__file__)
PY
```

## Core Data Rules

The most important Faiss Python rules are structural:

- vectors are usually 2D arrays shaped `(n, d)`
- `d` is fixed when you build the index
- use `float32` arrays unless an index explicitly documents another format
- keep arrays C-contiguous before `train`, `add`, or `search`

If your data is not already in the expected format, normalize it first:

```python
import numpy as np

def as_faiss_matrix(x: np.ndarray) -> np.ndarray:
    return np.ascontiguousarray(x, dtype="float32")
```

## Exact Search

For exact search on CPU, start with a flat index.

### L2 distance

```python
import numpy as np
import faiss

d = 384
xb = np.random.random((10_000, d)).astype("float32")
xq = np.random.random((5, d)).astype("float32")

index = faiss.IndexFlatL2(d)
index.add(xb)

distances, ids = index.search(xq, 10)

print(index.ntotal)
print(ids.shape, distances.shape)
```

### Inner product and cosine similarity

Use `IndexFlatIP` for maximum inner product. For cosine similarity, normalize both the database vectors and query vectors to unit length, then search with inner product.

```python
import numpy as np
import faiss

d = 768
xb = np.random.random((50_000, d)).astype("float32")
xq = np.random.random((3, d)).astype("float32")

faiss.normalize_L2(xb)
faiss.normalize_L2(xq)

index = faiss.IndexFlatIP(d)
index.add(xb)

scores, ids = index.search(xq, 5)
```

If you skip normalization, `IndexFlatIP` is inner product search, not cosine similarity.

## Choosing An Index

Use the simplest index that matches the dataset size and latency target:

- `IndexFlatL2` or `IndexFlatIP`: exact search, no training, best recall, memory cost grows with raw vectors
- `HNSW,Flat` via `index_factory`: approximate search, no training, good CPU default when exact search is too slow
- `IVF...,Flat`: train once, then trade recall for speed with `nprobe`
- `IVF...,PQ...` or `PQ...`: lower memory footprint, more approximation, better for large collections

If you do not know what to start with, begin with `IndexFlatL2` or `IndexFlatIP` to validate shapes, metrics, and result quality before switching to an approximate index.

## Approximate Indexes And Training

Many Faiss indexes need a training step before you can add vectors. IVF, PQ, OPQ, and combinations built from these usually require `train(...)`. Flat indexes and HNSW do not.

```python
import numpy as np
import faiss

d = 128
xb = np.random.random((20_000, d)).astype("float32")
xq = np.random.random((5, d)).astype("float32")

index = faiss.index_factory(d, "IVF256,Flat", faiss.METRIC_L2)

assert not index.is_trained
index.train(xb[:10_000])
assert index.is_trained

index.add(xb)
index.nprobe = 16

distances, ids = index.search(xq, 10)
```

Practical guidance:

- train on representative vectors from the same distribution as the vectors you will add
- do not call `add(...)` before `is_trained` is true for trainable indexes
- tune `nprobe` on IVF indexes for the recall/latency tradeoff you want

## Factory Strings

Faiss exposes `index_factory(...)` so you can describe an index with a compact string instead of manually wiring every component.

```python
import faiss

d = 384
index = faiss.index_factory(d, "HNSW32,Flat", faiss.METRIC_INNER_PRODUCT)
```

Factory strings are useful when you want to:

- switch between exact and approximate layouts quickly
- keep index configuration in a config file or experiment script
- mirror common combinations from the Faiss wiki without manually composing transforms and sub-indexes

## Custom IDs

Plain flat indexes use implicit sequential ids (`0..ntotal-1`). If your application needs stable external ids, wrap the base index in `IndexIDMap`.

```python
import numpy as np
import faiss

d = 256
xb = np.random.random((1_000, d)).astype("float32")
ids = np.arange(10_000, 11_000, dtype=np.int64)

index = faiss.IndexIDMap(faiss.IndexFlatL2(d))
index.add_with_ids(xb, ids)

distances, found_ids = index.search(xb[:3], 5)
print(found_ids)
```

## Save And Load Indexes

Use `write_index(...)` and `read_index(...)` to persist CPU indexes.

```python
import faiss

faiss.write_index(index, "vectors.faiss")
loaded = faiss.read_index("vectors.faiss")
```

Treat serialized indexes as trusted artifacts only. Upstream explicitly warns that loading a crafted file can trigger out-of-memory conditions or even code execution.

## Configuration And Environment Notes

Faiss has no API authentication or remote service configuration. The important configuration is local:

- index dimension `d`
- metric choice: L2 vs inner product
- index family: flat, HNSW, IVF, PQ, scalar quantization
- training parameters and search-time knobs such as `nprobe`

For Python projects:

- install into a virtual environment instead of the system interpreter
- keep `numpy` and `faiss-cpu` in the same environment
- pin the package version if your search quality benchmarks depend on exact index behavior

## Common Pitfalls

### Package name and import name differ

Install `faiss-cpu`, but write `import faiss`.

### Wrong dtype or memory layout

Most Python examples assume `float32` row-major arrays. Convert explicitly:

```python
xb = np.ascontiguousarray(xb, dtype="float32")
xq = np.ascontiguousarray(xq, dtype="float32")
```

### Dimension mismatch

Every vector passed to a given index must have the same `d` that the index was built with.

### Metric mismatch for cosine search

Cosine similarity requires normalized vectors plus an inner-product index. `IndexFlatL2` is not a drop-in cosine index.

### Forgetting to train

If an IVF or PQ-style index is not trained, `add(...)` will fail or behave incorrectly for the intended workflow. Check `index.is_trained`.

### Expecting custom ids on a base flat index

If you need your own ids, use `IndexIDMap` or another id-aware wrapper instead of assuming the base index stores your application ids.

### Loading untrusted index files

Do not call `read_index(...)` on files from untrusted users or arbitrary storage.

### Expecting GPU features from `faiss-cpu`

This package is the CPU wheel. If a guide depends on Faiss GPU resources or CUDA-only features, verify that it applies to your environment before copying it.

## Version-Sensitive Notes For 1.13.2

- The version covered here is the PyPI package version `1.13.2`.
- PyPI metadata for `1.13.2` lists Python support as `>=3.10, <3.15`.
- The upstream wiki is a rolling documentation source, not a version-pinned `1.13.2` snapshot. The examples here stay on stable APIs that are documented across the current wiki and repository docs.
- Upstream install documentation covers broader Faiss build paths, including conda and source builds. For this package entry, prefer the official `faiss-cpu` wheel unless you specifically need a non-PyPI build variant.

## Official Sources Used For This Guide

- PyPI project page: https://pypi.org/project/faiss-cpu/
- Faiss wiki home: https://github.com/facebookresearch/faiss/wiki
- Getting started: https://github.com/facebookresearch/faiss/wiki/Getting-started
- Guidelines to choose an index: https://github.com/facebookresearch/faiss/wiki/Guidelines-to-choose-an-index
- MetricType and distances: https://github.com/facebookresearch/faiss/wiki/MetricType-and-distances
- The index factory: https://github.com/facebookresearch/faiss/wiki/The-index-factory
- Index IO, cloning and hyper parameter tuning: https://github.com/facebookresearch/faiss/wiki/Index-IO%2C-cloning-and-hyper-parameter-tuning
- Repository README: https://github.com/facebookresearch/faiss
- Install notes: https://github.com/facebookresearch/faiss/blob/main/INSTALL.md
