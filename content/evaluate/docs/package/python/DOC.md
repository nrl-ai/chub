---
name: package
description: "Hugging Face Evaluate Python package for loading metrics, comparisons, measurements, and task evaluators"
metadata:
  languages: "python"
  versions: "0.4.6"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "evaluate,hugging-face,python,metrics,evaluation,ml,nlp"
---

# Hugging Face Evaluate Python Package Guide

## What It Is

`evaluate` is Hugging Face's Python library for loading evaluation modules and computing metrics, comparisons, and measurements. Use it when you need reusable metrics such as accuracy or F1, batched accumulation with `add_batch()`, or task-level evaluation over `transformers` pipelines.

For current LLM-evaluation workflows, Hugging Face now points users to LightEval for newer benchmark-style and judge-style evaluation patterns. Use `evaluate` when you specifically need its metric/module API or task evaluators.

## Install

Pin the package version your project expects:

```bash
python -m pip install "evaluate==0.4.6"
```

Common alternatives:

```bash
uv add "evaluate==0.4.6"
poetry add "evaluate==0.4.6"
```

If you want to scaffold and publish your own evaluation module, install the template extra:

```bash
python -m pip install "evaluate[template]==0.4.6"
```

If you plan to use task evaluators such as `evaluator("text-classification")`, also install the runtime packages that provide the model pipeline and dataset:

```bash
python -m pip install "transformers" "datasets"
```

## Quick Sanity Check

```python
import evaluate

metric = evaluate.load("accuracy")
result = metric.compute(predictions=[0, 1, 1], references=[0, 1, 0])

print(result)
```

If this succeeds, the library can download and cache evaluation modules correctly in your environment.

## Core Usage

### Load a metric and compute in one shot

```python
import evaluate

accuracy = evaluate.load("accuracy")

scores = accuracy.compute(
    predictions=[0, 1, 1, 0],
    references=[0, 1, 0, 0],
)

print(scores["accuracy"])
```

Use this pattern when all predictions and references already fit in memory.

### Accumulate batches incrementally

For larger datasets, add batches first and compute at the end:

```python
import evaluate

f1 = evaluate.load("f1")

f1.add_batch(predictions=[0, 1, 1], references=[0, 1, 0])
f1.add_batch(predictions=[1, 0], references=[1, 0])

result = f1.compute(average="binary")
print(result)
```

This is the main pattern to use when model inference and metric aggregation happen in separate steps.

### Combine multiple metrics

`evaluate.combine()` lets you compute several metrics through one object:

```python
import evaluate

metrics = evaluate.combine(["accuracy", "f1"])

result = metrics.compute(
    predictions=[0, 1, 1, 0],
    references=[0, 1, 0, 0],
)

print(result)
```

Use this when you want one consistent compute call for a standard metric bundle.

### Discover available modules

```python
import evaluate

all_metrics = evaluate.list_evaluation_modules(module_type="metric")
print(all_metrics[:10])
```

Useful module types:

- `metric`: standard metrics such as accuracy, F1, BLEU, or ROUGE
- `comparison`: side-by-side comparisons between systems or outputs
- `measurement`: descriptive measurements over data or predictions

### Evaluate a task end-to-end with a pipeline

The evaluator API is higher-level than raw metrics. It runs a `transformers` pipeline over a dataset and computes metrics for the task:

```python
from datasets import load_dataset
from transformers import pipeline
import evaluate

dataset = load_dataset("imdb", split="test[:100]")

classifier = pipeline(
    "text-classification",
    model="distilbert/distilbert-base-uncased-finetuned-sst-2-english",
)

task_evaluator = evaluate.evaluator("text-classification")
metric = evaluate.combine(["accuracy", "f1"])

results = task_evaluator.compute(
    model_or_pipeline=classifier,
    data=dataset,
    metric=metric,
    input_column="text",
    label_column="label",
    label_mapping={"NEGATIVE": 0, "POSITIVE": 1},
)

print(results)
```

`label_mapping` matters when pipeline outputs use string labels but your dataset labels are integers.

## Configuration And Auth

### Loading options that matter

`evaluate.load()` has a few parameters agents should reach for first:

- `module_type=` to force `metric`, `comparison`, or `measurement` when the name is ambiguous
- `config_name=` when a metric exposes multiple configurations
- `cache_dir=` to control where downloaded modules and intermediate data live
- `revision=` when you need a specific Hub revision for reproducibility
- `num_process=`, `process_id=`, and `experiment_id=` for distributed evaluation
- `keep_in_memory=` only for non-distributed runs where local memory use is acceptable

For distributed evaluation, the docs call out that `cache_dir` must be shared across processes, and `keep_in_memory` cannot be used in that setup.

### Auth and sharing custom modules

Built-in metrics do not usually require authentication. Auth matters when you create or share your own evaluation module on the Hugging Face Hub, or when you need access to private Hub resources.

The documented setup for publishing a module is:

```bash
python -m pip install "evaluate[template]==0.4.6"
huggingface-cli login
evaluate-cli create "my_eval_module"
```

If a private or gated Hub resource is involved, make sure the active Hugging Face login context has access before debugging the metric code itself.

## Common Pitfalls

- The docs installation page still says Evaluate is tested on Python 3.7+, but PyPI metadata for `0.4.6` requires Python `>=3.8`. Treat PyPI as authoritative for install constraints.
- `evaluate` computes metrics; it does not provide a model by itself. `evaluator()` still needs a `transformers` pipeline or model callable plus a dataset.
- Metric inputs must match the metric's expected shape and label type. Classification metrics often expect numeric labels, while text-generation metrics expect strings or tokenized text.
- `f1`, `precision`, and `recall` usually need the correct averaging mode such as `average="binary"` or `average="macro"`. Do not assume the default matches your label shape.
- Distributed runs need a shared `cache_dir` and consistent `experiment_id`; otherwise aggregation can fail or hang.
- If you need reproducible behavior for Hub-hosted modules, pin `revision=` instead of relying on whatever the default branch serves later.
- For newer LLM-eval workflows, check whether LightEval is the intended upstream tool before building new judge-style evaluation code around `evaluate`.

## Version-Sensitive Notes For 0.4.6

- PyPI lists `evaluate 0.4.6` as the current package release for this guide.
- The official `v0.4.6` release notes call out support for `huggingface_hub>=1.0` and `datasets 4`.
- The docs root currently resolves to the Evaluate docs for `v0.4.6`, but some wording on individual pages still reflects older support ranges and should not override the package metadata on PyPI.

## Official Source URLs

- Hugging Face Evaluate docs root: https://huggingface.co/docs/evaluate/
- Installation: https://huggingface.co/docs/evaluate/installation
- Quick tour: https://huggingface.co/docs/evaluate/a_quick_tour
- Loading methods reference: https://huggingface.co/docs/evaluate/package_reference/loading_methods
- Main classes reference: https://huggingface.co/docs/evaluate/package_reference/main_classes
- Evaluator guide: https://huggingface.co/docs/evaluate/base_evaluator
- Creating and sharing modules: https://huggingface.co/docs/evaluate/creating_and_sharing
- PyPI package page: https://pypi.org/project/evaluate/
- GitHub releases: https://github.com/huggingface/evaluate/releases/tag/v0.4.6
