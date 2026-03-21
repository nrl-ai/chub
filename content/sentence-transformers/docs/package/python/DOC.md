---
name: package
description: "sentence-transformers package guide for Python embeddings, retrieval, reranking, and Hugging Face model loading"
metadata:
  languages: "python"
  versions: "5.2.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "sentence-transformers,embeddings,retrieval,reranking,semantic-search,hugging-face,transformers"
---

# sentence-transformers Python Package Guide

## What It Is

`sentence-transformers` is the maintainer-supported Python library for:

- dense embeddings with `SentenceTransformer`
- reranking and pair scoring with `CrossEncoder`
- sparse retrieval models with `SparseEncoder`

It is a local Python package, not a hosted API. In normal use it downloads models from the Hugging Face Hub or loads them from a local path.

## Golden Rules

- Install `sentence-transformers` and load an explicit model ID such as `all-MiniLM-L6-v2` or `cross-encoder/ms-marco-MiniLM-L-6-v2`.
- Use `encode_query()` and `encode_document()` for retrieval tasks in v5 when you have a real query/document split.
- Use `CrossEncoder` for reranking or pair classification, not for precomputing reusable embeddings.
- Treat `trust_remote_code=True` as code execution. Only enable it for repositories you trust.

## Install

Pin the package version your project expects:

```bash
python -m pip install "sentence-transformers==5.2.3"
```

Common alternatives:

```bash
uv add "sentence-transformers==5.2.3"
poetry add "sentence-transformers==5.2.3"
```

Useful extras from the official installation docs and package metadata:

```bash
python -m pip install "sentence-transformers[train]==5.2.3"
python -m pip install "sentence-transformers[onnx]==5.2.3"
python -m pip install "sentence-transformers[onnx-gpu]==5.2.3"
python -m pip install "sentence-transformers[openvino]==5.2.3"
```

Install notes:

- The project recommends Python `3.10+` and PyTorch `1.11.0+`.
- PyPI metadata for `5.2.3` requires `transformers>=4.41.0,<6.0.0`.
- If you need CUDA, install the matching PyTorch build first from the official PyTorch instructions, then install `sentence-transformers`.

## Model Loading And Setup

Basic embedding model:

```python
from sentence_transformers import SentenceTransformer

model = SentenceTransformer("all-MiniLM-L6-v2")
```

Useful constructor options for real projects:

```python
import os
from sentence_transformers import SentenceTransformer

model = SentenceTransformer(
    "BAAI/bge-base-en-v1.5",
    device="cuda",
    token=os.getenv("HF_TOKEN"),
    revision="main",
    cache_folder="/tmp/sentence-transformers-cache",
)
```

Configuration points that matter:

- `token=` is the supported auth argument for private or gated Hugging Face models.
- `use_auth_token=` is deprecated; use `token=` instead.
- `revision=` can pin a branch, tag, or commit on the Hugging Face Hub.
- `cache_folder=` overrides the model cache path. You can also set `SENTENCE_TRANSFORMERS_HOME`.
- `local_files_only=True` avoids network calls and only loads already-downloaded local artifacts.
- `backend=` can be `torch`, `onnx`, or `openvino`; the non-`torch` backends require the matching extra.

## Core Usage

### Create embeddings

Use `SentenceTransformer` for semantic similarity, clustering, search indexing, and retrieval.

```python
from sentence_transformers import SentenceTransformer

model = SentenceTransformer("all-MiniLM-L6-v2")

sentences = [
    "The weather is lovely today.",
    "It is sunny outside.",
    "He drove to the stadium.",
]

embeddings = model.encode(
    sentences,
    normalize_embeddings=True,
    convert_to_numpy=True,
)

print(embeddings.shape)
```

If you need pairwise similarity after encoding:

```python
from sentence_transformers import SentenceTransformer

model = SentenceTransformer("all-MiniLM-L6-v2")
embeddings = model.encode(["A cat sits outside.", "A dog plays in the yard."])
similarities = model.similarity(embeddings, embeddings)
print(similarities)
```

### Retrieval: use `encode_query()` and `encode_document()`

In v5, these methods are the preferred path for information retrieval because they can apply saved prompts and Router task routing when the model supports them.

```python
from sentence_transformers import SentenceTransformer

model = SentenceTransformer("mixedbread-ai/mxbai-embed-large-v1")

query = "How do I export a model to ONNX?"
documents = [
    "Use the export_dynamic_quantized_onnx_model helper for dynamic quantization.",
    "CrossEncoder models score query-document pairs directly.",
    "OpenVINO can improve CPU inference latency.",
]

query_embedding = model.encode_query(query, normalize_embeddings=True)
document_embeddings = model.encode_document(documents, normalize_embeddings=True)

scores = model.similarity(query_embedding, document_embeddings)
best_index = scores[0].argmax().item()
print(documents[best_index])
```

Use plain `encode()` when you do not have a query/document distinction, or when you are doing clustering, deduplication, or semantic similarity between peers.

### Rerank with `CrossEncoder`

Use a cross-encoder after retrieval when you need higher-quality ranking on a smaller candidate set.

```python
from sentence_transformers.cross_encoder import CrossEncoder

query = "How many people live in Berlin?"
documents = [
    "Berlin had a population of 3,520,031 registered inhabitants in an area of 891.82 square kilometers.",
    "Berlin is known for its museums.",
]

model = CrossEncoder("cross-encoder/ms-marco-MiniLM-L-6-v2")
scores = model.predict([(query, doc) for doc in documents])

ranked = sorted(zip(documents, scores), key=lambda item: item[1], reverse=True)
print(ranked[0])
```

Rule of thumb:

- `SentenceTransformer`: fast embeddings you can precompute and index
- `CrossEncoder`: slower but better scoring for query-document pairs

### Faster inference backends

If you installed the matching extra, you can load the same model with a non-default backend:

```python
from sentence_transformers import SentenceTransformer

model = SentenceTransformer("all-MiniLM-L6-v2", backend="onnx")
embeddings = model.encode(["hello world"])
```

Use this when the deployment target is inference-only and you have already validated that the backend-specific export path works for the chosen model.

## Auth And Configuration

This package does not use API keys of its own. Authentication is only relevant when the underlying model is private or gated on the Hugging Face Hub.

Private model example:

```python
import os
from sentence_transformers import SentenceTransformer

model = SentenceTransformer(
    "my-org/private-embedding-model",
    token=os.environ["HF_TOKEN"],
)
```

Operational settings agents often need:

- `device="cpu"`, `"cuda"`, or `"mps"` for explicit hardware selection
- `similarity_fn_name="cosine"` or `"dot"` when you want an explicit similarity mode
- `batch_size=` for throughput tuning
- `truncate_dim=` when using Matryoshka-style models and you want smaller embeddings
- `prompt_name=` or `prompt=` when the model expects prompt-specific encoding behavior

## Common Pitfalls

- The package name is not a model name. `pip install sentence-transformers` does not give you a default embedding model; you still need to load one.
- `CrossEncoder` and `SentenceTransformer` are not interchangeable. Cross-encoders score pairs directly and do not produce reusable corpus embeddings.
- Retrieval models in the v5 line may behave better with `encode_query()` and `encode_document()` than with plain `encode()`, especially when prompts or Router modules are saved with the model.
- `trust_remote_code=True` runs repository-defined code from the model source. Keep it off unless you have reviewed the model repo.
- Offline or air-gapped runs need either a local model path or a warm cache plus `local_files_only=True`.
- Embedding scores are only comparable when you use the same model family and a consistent normalization/similarity setup.
- `encode_multi_process()` is deprecated in the v5 migration guide. Prefer `encode(..., device=[...], chunk_size=...)` for multi-process encoding.

## Version-Sensitive Notes

- This guide covers the package version used here `5.2.3`.
- PyPI now lists `5.3.0` as the latest release on `2026-03-12`, so there is already upstream version drift relative to the pinned package version.
- The v5 line introduced `SparseEncoder`, `encode_query()`, `encode_document()`, Router-based asymmetric routing, and the deprecation of older `encode_multi_process()`-style flows.
- The official docs homepage still foregrounds the `v5.2` release train, while PyPI has already moved to `5.3.0`. When exact patch behavior matters, check both the current docs and the package release history before copying older snippets.
