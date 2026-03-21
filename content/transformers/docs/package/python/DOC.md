---
name: package
description: "Hugging Face Transformers Python package for loading, running, fine-tuning, and publishing pretrained transformer models"
metadata:
  languages: "python"
  versions: "5.3.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "transformers,hugging-face,python,ml,llm,nlp,vision,audio,pytorch"
---

# Hugging Face Transformers Python Package Guide

## Golden Rule

Use `transformers` for model loading, tokenization, pipelines, and training workflows, but match the checkpoint to the correct task-specific class. In `5.3.0`, prefer current Hugging Face Hub auth and cache settings (`hf auth login`, `HF_TOKEN`, `HF_HOME`, `HF_HUB_CACHE`) instead of copying older blog snippets blindly.

## Install

For most Python projects, install the package pinned to the version you expect and include the PyTorch extra:

```bash
python -m pip install "transformers[torch]==5.3.0"
```

Common alternatives:

```bash
uv add "transformers[torch]==5.3.0"
poetry add "transformers[torch]==5.3.0"
```

If you manage your deep-learning stack separately, install your hardware-specific PyTorch build first, then install `transformers`:

```bash
python -m pip install torch
python -m pip install "transformers==5.3.0"
```

Packages commonly needed around `transformers` workflows:

```bash
python -m pip install datasets evaluate accelerate
```

Add `sentencepiece`, `timm`, or other model-specific dependencies only when the checkpoint or processor you picked requires them.

## Authentication And Cache Setup

Public model downloads usually work without login. Use authentication when you need gated or private repos, or when your organization rate-limits anonymous access.

Interactive login:

```bash
hf auth login
```

Environment-based setup:

```bash
export HF_TOKEN="hf_your_token"
export HF_HOME="$HOME/.cache/huggingface"
export HF_HUB_CACHE="$HF_HOME/hub"
```

Notes:

- `HF_TOKEN` overrides the token saved on disk.
- `HF_HOME` changes the Hugging Face local data root.
- `HF_HUB_CACHE` changes where downloaded models, tokenizers, and configs are cached.
- Pass `token=...` to `from_pretrained()` when you need explicit per-call auth.

## Core Usage

### Fastest path: pipeline

`pipeline()` is the shortest path for common inference tasks:

```python
from transformers import pipeline

classifier = pipeline(
    task="sentiment-analysis",
    model="distilbert/distilbert-base-uncased-finetuned-sst-2-english",
)

result = classifier("The new setup makes agent handoff faster.")
print(result)
```

Use this when you want working inference quickly and do not need fine control over tokenization, batching, or generation config.

### Explicit model and tokenizer loading

Use Auto classes when you need predictable control over the model class, device placement, or preprocessing:

```python
import torch
from transformers import AutoModelForSequenceClassification, AutoTokenizer

checkpoint = "distilbert/distilbert-base-uncased-finetuned-sst-2-english"

tokenizer = AutoTokenizer.from_pretrained(checkpoint)
model = AutoModelForSequenceClassification.from_pretrained(
    checkpoint,
    device_map="auto",
    dtype="auto",
)

inputs = tokenizer("Transformers handles tokenization for you.", return_tensors="pt")

with torch.inference_mode():
    outputs = model(**inputs)

predicted_label_id = int(outputs.logits.argmax(dim=-1))
print(model.config.id2label[predicted_label_id])
```

Rules that matter:

- Use the task head that matches the checkpoint and task, such as `AutoModelForCausalLM`, `AutoModelForSeq2SeqLM`, or `AutoModelForSequenceClassification`.
- `device_map="auto"` is the easiest way to place large models on available hardware.
- `dtype="auto"` lets the library choose a sensible model dtype for the checkpoint and backend.

### Loading a gated or private model

```python
import os
from transformers import AutoTokenizer, AutoModelForCausalLM

checkpoint = "meta-llama/Llama-4-Scout-17B-16E-Instruct"
token = os.environ["HF_TOKEN"]

tokenizer = AutoTokenizer.from_pretrained(checkpoint, token=token)
model = AutoModelForCausalLM.from_pretrained(
    checkpoint,
    token=token,
    device_map="auto",
    dtype="auto",
)
```

If the repo is gated, logging in alone is not enough unless the account has been granted access to that checkpoint.

### Fine-tuning with Trainer

The current quicktour uses `Trainer` and `TrainingArguments` for the standard PyTorch fine-tuning path:

```python
from datasets import load_dataset
from transformers import (
    AutoModelForSequenceClassification,
    AutoTokenizer,
    DataCollatorWithPadding,
    Trainer,
    TrainingArguments,
)

checkpoint = "distilbert/distilbert-base-uncased"
tokenizer = AutoTokenizer.from_pretrained(checkpoint)

dataset = load_dataset("yelp_review_full")

def preprocess(batch):
    return tokenizer(batch["text"], truncation=True)

tokenized = dataset.map(preprocess, batched=True)
data_collator = DataCollatorWithPadding(tokenizer=tokenizer)

model = AutoModelForSequenceClassification.from_pretrained(checkpoint, num_labels=5)

training_args = TrainingArguments(
    output_dir="checkpoints",
    eval_strategy="epoch",
    learning_rate=2e-5,
    per_device_train_batch_size=8,
    per_device_eval_batch_size=8,
    num_train_epochs=1,
    weight_decay=0.01,
)

trainer = Trainer(
    model=model,
    args=training_args,
    train_dataset=tokenized["train"].select(range(1000)),
    eval_dataset=tokenized["test"].select(range(500)),
    processing_class=tokenizer,
    data_collator=data_collator,
)

trainer.train()
```

Install `accelerate` before using `Trainer` seriously. For larger jobs, verify mixed precision, checkpointing, and distributed-launch settings explicitly instead of relying on defaults.

### Save locally and reload later

```python
from transformers import AutoModel, AutoTokenizer

checkpoint = "distilbert/distilbert-base-uncased"

tokenizer = AutoTokenizer.from_pretrained(checkpoint)
model = AutoModel.from_pretrained(checkpoint)

tokenizer.save_pretrained("./artifacts/distilbert")
model.save_pretrained("./artifacts/distilbert")

reloaded_tokenizer = AutoTokenizer.from_pretrained("./artifacts/distilbert")
reloaded_model = AutoModel.from_pretrained("./artifacts/distilbert")
```

Use `save_pretrained()` and `from_pretrained()` instead of manually copying config and weight files.

## Configuration Notes

- Prefer environment variables or `hf auth login` for tokens; do not hardcode access tokens in source.
- Keep the cache on local SSD when possible. Network-mounted caches are a common cause of slow or flaky loads.
- For reproducible revisions, pass a model repo commit or tag with `revision=...` to `from_pretrained()`.
- Use `local_files_only=True` when you need offline-only behavior after the model is already cached.

## Common Pitfalls

- Wrong class for the task: base classes like `LlamaModel` or `BertModel` do not include task heads. Use the `AutoModelFor...` variant that matches generation, classification, token classification, seq2seq, or QA.
- Missing model dependencies: some checkpoints need optional packages such as `sentencepiece`, `timm`, or audio/image extras. Read the checkpoint card if `from_pretrained()` fails on an import.
- Assuming login grants access: gated models still require approval on the model page.
- Device mismatch bugs: tokenized tensors and model weights must live on compatible devices. `pipeline()` or `device_map="auto"` reduces this friction.
- Old auth examples: prefer `hf auth login` and the `token=` argument. Many older snippets use deprecated or stale patterns.
- Docs-version drift: the Hugging Face docs site can default to `main` or an older stable selector. Check the version picker when behavior differs from your installed `5.3.0`.

## Version-Sensitive Notes For 5.3.0

- PyPI lists `transformers 5.3.0` with Python `>=3.10`.
- The installation docs still emphasize that `main` requires installation from source and separately call out the latest stable docs series, so do not assume the first docs page you land on matches your installed package exactly.
- For new code, prefer current v5-era APIs and examples from the Hugging Face docs instead of older 4.x-era tutorials or issue comments.

## Official Source URLs

- Hugging Face Transformers docs root: https://huggingface.co/docs/transformers/
- Installation: https://huggingface.co/docs/transformers/main/en/installation
- Quicktour: https://huggingface.co/docs/transformers/main/en/quicktour
- Model loading API: https://huggingface.co/docs/transformers/main/en/main_classes/model
- Hugging Face Hub authentication: https://huggingface.co/docs/huggingface_hub/en/package_reference/authentication
- Hugging Face Hub environment variables: https://huggingface.co/docs/huggingface_hub/en/package_reference/environment_variables
- PyPI package page: https://pypi.org/project/transformers/
