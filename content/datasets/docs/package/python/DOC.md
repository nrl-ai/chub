---
name: package
description: "Hugging Face Datasets package guide for loading, processing, streaming, and caching datasets in Python"
metadata:
  languages: "python"
  versions: "4.7.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "datasets,huggingface,data,ml,arrow,streaming"
---

# Hugging Face Datasets Python Package Guide

## Golden Rule

Use `datasets` for dataset loading and preprocessing, but choose the execution model up front:

- use `Dataset` or `DatasetDict` when you need random access, materialized transforms, `save_to_disk()`, or framework formatting
- use `streaming=True` only when the dataset is too large to download eagerly or you need sequential iteration over remote shards

Agents often fail by writing code for `Dataset` and then silently switching to `IterableDataset` later. Those two paths do not support the same operations.

## Install

Pin the package version your project expects:

```bash
python -m pip install "datasets==4.7.0"
```

Common alternatives:

```bash
uv add "datasets==4.7.0"
poetry add "datasets==4.7.0"
```

Useful extras from the PyPI package metadata:

```bash
python -m pip install "datasets[audio]==4.7.0"
python -m pip install "datasets[vision]==4.7.0"
python -m pip install "datasets[streaming]==4.7.0"
python -m pip install "datasets[torch]==4.7.0"
```

## Authentication And Environment

Public datasets on the Hugging Face Hub can be loaded without credentials. Private or gated datasets require a Hugging Face token.

Preferred setup:

```bash
hf auth login
```

Then load the dataset with the stored token:

```python
from datasets import load_dataset

dataset = load_dataset("org/private-dataset", split="train", token=True)
```

You can also pass a token string directly:

```python
from datasets import load_dataset

dataset = load_dataset("org/private-dataset", split="train", token="hf_...")
```

Cache-related environment variables:

```bash
export HF_HOME="$PWD/.hf"
export HF_DATASETS_CACHE="$PWD/.hf/datasets"
```

Use these when CI, containers, or shared machines need deterministic cache locations.

## Core Usage

### Load a dataset from the Hub

```python
from datasets import load_dataset

dataset = load_dataset("cornell-movie-review-data/rotten_tomatoes")

print(dataset)
print(dataset["train"][0])
```

Load just one split when you do not need the full `DatasetDict`:

```python
from datasets import load_dataset

train = load_dataset(
    "cornell-movie-review-data/rotten_tomatoes",
    split="train",
)
```

### Load local files

Use a dataset builder name such as `csv`, `json`, or `parquet` for local or direct file ingestion:

```python
from datasets import load_dataset

csv_ds = load_dataset("csv", data_files="data/train.csv")
json_ds = load_dataset("json", data_files={"train": "data/train.jsonl"})
parquet_ds = load_dataset("parquet", data_files="data/events.parquet")
```

### Inspect metadata before downloading

`load_dataset_builder()` is the fastest way to inspect features, splits, and dataset info without materializing the data:

```python
from datasets import load_dataset_builder

builder = load_dataset_builder("nyu-mll/glue", "mrpc")

print(builder.info.features)
print(builder.info.splits)
```

### Preprocess with `map()`, `filter()`, and column operations

```python
from datasets import load_dataset

dataset = load_dataset("nyu-mll/glue", "mrpc", split="train")

dataset = dataset.filter(lambda row: row["label"] == 1)
dataset = dataset.rename_column("label", "labels")
dataset = dataset.remove_columns(["idx"])
dataset = dataset.train_test_split(test_size=0.1, seed=42)
```

Use `map()` when you are creating or rewriting columns:

```python
from datasets import load_dataset
from transformers import AutoTokenizer

tokenizer = AutoTokenizer.from_pretrained("bert-base-uncased")
dataset = load_dataset("nyu-mll/glue", "mrpc", split="train")

def tokenize(batch):
    return tokenizer(batch["sentence1"], batch["sentence2"], truncation=True)

dataset = dataset.map(tokenize, batched=True, num_proc=4)
dataset = dataset.with_format(
    "torch",
    columns=["input_ids", "token_type_ids", "attention_mask", "label"],
)
```

Notes:

- `with_format()` returns a new dataset object; `set_format()` mutates in place
- `num_proc` can speed up CPU preprocessing, but the mapped function must be picklable

### Save and reload processed datasets

```python
from datasets import load_dataset, load_from_disk

dataset = load_dataset("cornell-movie-review-data/rotten_tomatoes", split="train")
dataset.save_to_disk("artifacts/rotten_tomatoes_train")

reloaded = load_from_disk("artifacts/rotten_tomatoes_train")
print(reloaded[0])
```

This is the right persistence path for preprocessed datasets. Do not replace it with ad hoc `pickle` or custom JSON dumps.

## Streaming

`streaming=True` returns an `IterableDataset`, which is designed for sequential access and remote-scale iteration:

```python
from datasets import load_dataset

stream = load_dataset(
    "cornell-movie-review-data/rotten_tomatoes",
    split="train",
    streaming=True,
)

for index, row in enumerate(stream):
    print(row)
    if index == 2:
        break
```

For Parquet-based data sources, the loader also supports column projection and row filtering:

```python
from datasets import load_dataset

stream = load_dataset(
    "parquet",
    data_files="data/events/*.parquet",
    streaming=True,
    columns=["event_id", "created_at"],
    filters=[("event_type", "==", "click")],
)
```

Use streaming when dataset size is the problem. Do not use it by default for small training or evaluation sets, because many convenient `Dataset` operations are unavailable or behave differently.

## Cache And Storage

Important cache facts:

- the datasets cache lives under the Hugging Face cache root and can grow quickly
- `HF_DATASETS_CACHE` controls the Arrow/cache files used by `datasets`
- `download_mode` can force re-download or cache reuse when debugging stale data

Clean up stale cache entries explicitly when needed:

```python
from datasets import load_dataset

dataset = load_dataset("cornell-movie-review-data/rotten_tomatoes", split="train")
removed = dataset.cleanup_cache_files()
print(removed)
```

## Common Pitfalls

- `revision=` on `load_dataset()` selects the dataset repository git ref on the Hub, not the installed `datasets` package version.
- `streaming=True` returns `IterableDataset`, so random indexing, many materialized transforms, and `save_to_disk()`-style workflows are not interchangeable with `Dataset`.
- `remove_columns()` is cheaper and clearer than doing a no-op `map()` just to drop fields.
- `with_transform()` and `set_transform()` apply transforms lazily at read time; they do not eagerly rewrite the stored Arrow table.
- Cache growth is easy to miss in CI and notebooks. Set `HF_HOME` or `HF_DATASETS_CACHE` deliberately for reproducible environments.
- Multiprocessing plus CUDA needs care. The processing guide notes that GPU-dispatched `map()` code should use the `spawn` start method to avoid CUDA re-initialization errors.
- If you only need schema inspection, call `load_dataset_builder()` first. Downloading full data just to inspect columns is wasted time and disk.

## Version-Sensitive Notes

- The `4.0.0` release removed dataset loading scripts and dropped `trust_remote_code` support in `load_dataset()`. If old blog posts tell you to rely on custom loading scripts, those instructions are stale for `4.x`.
- The `4.0.0` release introduced the `Column` object when indexing a single column. Code that assumed a plain Python list from `dataset["column_name"]` can behave differently.
- The `4.0.0` release switched audio and video decoding to `torchcodec`. Media pipelines now have extra runtime requirements beyond a bare `pip install datasets`.
- The `4.0.0` release added `IterableDataset.push_to_hub()`, which matters if older internal notes still claim streaming datasets cannot be pushed.
- The `4.7.0` release added a `Json()` feature type and `on_mixed_types="use_json"` support in `Dataset.from_dict()`, `Dataset.from_list()`, and `map()`. Revisit older mixed-JSON normalization workarounds before copying them forward.

## Official Links

- Docs root: `https://huggingface.co/docs/datasets/en/index`
- Installation: `https://huggingface.co/docs/datasets/en/installation`
- Quickstart: `https://huggingface.co/docs/datasets/en/quickstart`
- Processing guide: `https://huggingface.co/docs/datasets/en/process`
- Streaming guide: `https://huggingface.co/docs/datasets/en/stream`
- Cache guide: `https://huggingface.co/docs/datasets/en/cache`
- Loading methods reference: `https://huggingface.co/docs/datasets/en/package_reference/loading_methods`
- Main classes reference: `https://huggingface.co/docs/datasets/en/package_reference/main_classes`
- PyPI package: `https://pypi.org/project/datasets/`
