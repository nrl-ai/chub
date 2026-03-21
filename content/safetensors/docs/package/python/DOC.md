---
name: package
description: "safetensors Python package guide for safe, zero-copy tensor serialization"
metadata:
  languages: "python"
  versions: "0.7.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "safetensors,huggingface,tensors,serialization,model-weights,pytorch,numpy"
---

# safetensors Python Package Guide

## Golden Rule

Use `safetensors` for tensor weights and arrays, not for arbitrary Python objects. The main value is avoiding `pickle`-style code execution while keeping fast, lazy, zero-copy friendly reads.

## Install

```bash
pip install safetensors==0.7.0
```

If you do not need to pin exactly:

```bash
pip install safetensors
```

PyPI metadata for `0.7.0` says `Requires: Python >=3.9`.

Build from source only if you need unreleased changes or local development. The upstream repository says source installs require Rust:

```bash
git clone https://github.com/huggingface/safetensors
cd safetensors/bindings/python
pip install setuptools_rust
pip install -e .
```

## What To Reach For First

- `safe_open(...)`: lazy, selective reads from a `.safetensors` file
- `safetensors.torch.save_file` / `load_file`: the common PyTorch file API
- `safetensors.torch.save` / `load`: bytes-in-memory API
- `safetensors.numpy.load_file` / `save_file`: NumPy file API
- `safetensors.torch.save_model` / `load_model`: use these for `torch.nn.Module` objects with shared tensors

The official docs also expose Python modules for `tensorflow`, `flax`, and `paddle` in addition to `torch` and `numpy`.

## Basic PyTorch Workflow

### Save a state dict

```python
import torch
from safetensors.torch import save_file

tensors = {
    "embedding": torch.zeros((2, 2)),
    "attention": torch.zeros((2, 3)),
}

save_file(tensors, "model.safetensors")
```

### Load everything

```python
from safetensors.torch import load_file

weights = load_file("model.safetensors", device="cpu")
print(weights["embedding"].shape)
```

### Lazy-read selected tensors

Use `safe_open` when you do not want to materialize the whole file immediately.

```python
from safetensors import safe_open

with safe_open("model.safetensors", framework="pt", device="cpu") as f:
    print(list(f.keys()))
    embedding = f.get_tensor("embedding")
```

### Slice without loading the full tensor first

```python
from safetensors import safe_open

with safe_open("model.safetensors", framework="pt", device=0) as f:
    tensor_slice = f.get_slice("embedding")
    rows, cols = tensor_slice.get_shape()
    shard = tensor_slice[:, : cols // 2]
```

This pattern is useful for large model weights and sharded or distributed loading paths.

## NumPy Workflow

```python
import numpy as np
from safetensors.numpy import save_file, load_file

arrays = {
    "x": np.zeros((4, 4), dtype=np.float32),
    "y": np.arange(8, dtype=np.int64),
}

save_file(arrays, "arrays.safetensors")
loaded = load_file("arrays.safetensors")
```

If you already have bytes in memory, use `safetensors.numpy.load(data)` or `safetensors.torch.load(data)`.

## Metadata

`save_file(..., metadata=...)` accepts a string-to-string dictionary only.

```python
import torch
from safetensors.torch import save_file

tensors = {"weight": torch.zeros((8, 8))}
metadata = {
    "format": "pt",
    "model_type": "demo",
}

save_file(tensors, "model.safetensors", metadata=metadata)
```

Do not store nested JSON, numbers, or booleans unless you serialize them to strings yourself. The format allows a special `__metadata__` map, but arbitrary JSON values are not allowed.

## Model-Level Helpers For Shared Tensors

PyTorch models sometimes share buffers, for example tied embeddings and LM heads. `safetensors` does not preserve shared tensors in the plain `Dict[str, Tensor]` file format. The official guidance is:

```python
from safetensors.torch import load_model, save_model

save_model(model, "model.safetensors")
missing, unexpected = load_model(model, "model.safetensors")
```

Prefer `save_model` / `load_model` over:

```python
save_file(model.state_dict(), "model.safetensors")
model.load_state_dict(load_file("model.safetensors"))
```

when the model may have shared tensors. The upstream docs call this out explicitly as the supported workaround.

## Config And Auth

There is no package-level authentication or global config file for local `.safetensors` reads and writes.

- Local files: call the Python APIs directly.
- Remote files: fetch bytes yourself with `requests`, `huggingface_hub`, or another client, then pass bytes to `load(...)` or save them locally first.
- Private Hugging Face Hub assets: handle tokens and Hub auth in `huggingface_hub` or your HTTP client; `safetensors` itself is only the serialization layer.

For metadata-only inspection of remote weights, the official docs show using HTTP Range requests so you can parse header metadata without downloading the full file.

## Common Pitfalls

### `save_file` expects dense, contiguous tensors

The official API docs say tensors must be contiguous and dense. If you are working with views or non-contiguous tensors, call `.contiguous()` first in PyTorch before saving.

### `.safetensors` files are weights, not Python objects

Do not expect `safetensors` to serialize tokenizers, configs, callables, or arbitrary classes. Pair it with JSON, YAML, or framework-native config files for non-tensor metadata.

### Metadata values must be strings

This fails conceptually:

```python
metadata = {"epoch": 3, "is_finetuned": True}
```

Do this instead:

```python
metadata = {"epoch": "3", "is_finetuned": "true"}
```

### `safe_open` is for file paths

Use `safe_open` with an actual safetensors file path. If your data is already in memory, use the module-level `load(...)` function for that framework instead of trying to point `safe_open` at a directory or unrelated path.

### Shared tensor keys can appear "missing"

When using `save_model` / `load_model`, some keys may be dropped from the file because shared buffers are represented once. Review the returned `missing` and `unexpected` lists in `load_model(...)` instead of assuming exact `state_dict()` key symmetry.

## Version-Sensitive Notes For 0.7.0

- PyPI lists `0.7.0` as the current release on November 19, 2025.
- The Hugging Face docs site currently exposes `main` plus stable docs labeled `v0.5.0-rc.0`. That docs version label lags the PyPI package version, so use the docs for API behavior but keep the package pin anchored to PyPI.
- The docs banner says `main` requires installation from source. For normal package usage, prefer `pip install safetensors` unless you specifically need unreleased source changes.
- PyPI metadata says `Python >=3.9`, even though some legacy classifiers and wheel tags still mention older interpreter compatibility. Follow the package metadata requirement for new project setup.

## Practical Decision Rule

- Need safe on-disk tensor exchange: use `save_file` / `load_file`.
- Need lazy or partial reads from a large file: use `safe_open`.
- Need in-memory bytes instead of files: use `save` / `load`.
- Need PyTorch model save/load with tied weights: use `save_model` / `load_model`.

## Official Sources

- Docs landing page: `https://huggingface.co/docs/safetensors/en/index`
- Torch API: `https://huggingface.co/docs/safetensors/en/api/torch`
- NumPy API: `https://huggingface.co/docs/safetensors/en/api/numpy`
- Tensor sharing notes: `https://huggingface.co/docs/safetensors/en/torch_shared_tensors`
- Metadata parsing: `https://huggingface.co/docs/safetensors/en/metadata_parsing`
- Package registry: `https://pypi.org/project/safetensors/`
- Repository: `https://github.com/huggingface/safetensors`
