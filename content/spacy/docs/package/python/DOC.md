---
name: package
description: "spacy package guide for Python with installation, model setup, pipeline usage, serialization, config, and version-sensitive spaCy v3.8 notes"
metadata:
  languages: "python"
  versions: "3.8.11"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "spacy,nlp,python,tokenization,ner,lemmatization,transformers"
---

# spacy Python Package Guide

## Golden Rule

- Install `spacy` and a compatible trained pipeline separately.
- Use `spacy.load(...)` when you want a maintained pipeline package, and `spacy.blank(...)` when you are assembling a custom pipeline from scratch.
- Batch work with `nlp.pipe(...)` instead of calling `nlp(text)` in a tight loop.
- Keep model and pipeline compatibility explicit after upgrades by running `python -m spacy validate`.

## Version-Sensitive Notes

- This entry is pinned to the version used here `3.8.11`.
- PyPI currently lists `spacy 3.8.11` as the latest release, published on `2025-11-17`, so the version used here matches current upstream.
- Current PyPI package metadata requires Python `>=3.9,<3.15`.
- The install docs and long PyPI description still include older compatibility text such as Python `3.7+`; for environment decisions, trust the current package metadata and available wheels for `3.8.11`.
- spaCy v3 uses registered component factories. `nlp.add_pipe(...)` takes a string factory name such as `"sentencizer"` or `"entity_ruler"`, not a bare callable.
- Pipeline packages have their own versions and must stay compatible with your spaCy minor version. After upgrading spaCy, re-run `python -m spacy download ...` or `python -m spacy validate`.
- In spaCy `v3.7+`, transformer pipelines use `spacy-curated-transformers` and `doc._.trf_data` is no longer the older `TransformerData` object used by older `spacy-transformers` examples.
- spaCy `v3.8` added `Language.memory_zone()` for long-running services. Any `Doc`, `Token`, `Span`, or lexeme-backed data created inside the block must not be used after the block exits.

## Install

Use a virtual environment and pin the version when you need reproducible behavior:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install -U pip setuptools wheel
python -m pip install "spacy==3.8.11"
python -m spacy download en_core_web_sm
```

Useful extras from the official install docs and current PyPI metadata:

- `spacy[lookups]`: install lookup tables for lemmatization and lexeme normalization when you are building blank pipelines or training your own models.
- `spacy[transformers]`: install transformer integration for transformer-based workflows.
- `spacy[cuda12x]` or another CUDA extra: install GPU support when the runtime has a compatible CUDA stack.
- `spacy[apple]`: install Apple-optimized ops for Apple Silicon performance improvements.

If you prefer conda:

```bash
conda install -c conda-forge spacy
```

## Recommended Setup

Start with a trained pipeline unless you have a reason to build a blank one:

```python
import spacy

nlp = spacy.load("en_core_web_sm")
doc = nlp("Apple is looking at buying a U.K. startup for $1 billion.")

print([(ent.text, ent.label_) for ent in doc.ents])
print([sent.text for sent in doc.sents])
```

Use a blank language object when you only need tokenization or you are building a custom pipeline:

```python
import spacy

nlp = spacy.blank("en")
nlp.add_pipe("sentencizer")

doc = nlp("First sentence. Second sentence.")
print([sent.text for sent in doc.sents])
```

## Core Usage

### Add Built-In Components

In spaCy v3, add built-ins by registered name:

```python
import spacy

nlp = spacy.blank("en")
nlp.add_pipe("sentencizer")
ruler = nlp.add_pipe("entity_ruler")
ruler.add_patterns(
    [
        {"label": "PRODUCT", "pattern": "spaCy"},
        {"label": "ORG", "pattern": "Explosion"},
    ]
)

doc = nlp("spaCy is built by Explosion.")
print([(ent.text, ent.label_) for ent in doc.ents])
```

If you are writing a custom component, register it with `@Language.component` or `@Language.factory` before adding it to the pipeline.

### Process Text In Batches

For large workloads, stream texts with `nlp.pipe(...)`:

```python
import spacy

nlp = spacy.load("en_core_web_sm")
texts = [
    "spaCy is fast.",
    "Batch processing is usually better than one text at a time.",
]

for doc in nlp.pipe(texts, batch_size=64):
    print(doc.text, [ent.text for ent in doc.ents])
```

If you need to preserve request metadata alongside each document, use `as_tuples=True`:

```python
import spacy

nlp = spacy.load("en_core_web_sm")
items = [
    ("The team uses package docs.", {"id": 1}),
    ("spaCy can stream documents.", {"id": 2}),
]

for doc, meta in nlp.pipe(items, as_tuples=True):
    print(meta["id"], doc.text)
```

If you only need part of a trained pipeline for a block of work, disable components temporarily:

```python
import spacy

nlp = spacy.load("en_core_web_sm")

with nlp.select_pipes(disable=["parser", "tagger", "attribute_ruler", "lemmatizer"]):
    for doc in nlp.pipe(["Apple released a new product."]):
        print([(ent.text, ent.label_) for ent in doc.ents])
```

For non-transformer pipelines, this is a good pattern when you only need NER. For transformer pipelines, keep the `transformer` component enabled because downstream components depend on it.

### Rule-Based Matching

The `Matcher` must share the same vocab as the `Doc` objects it will process:

```python
import spacy
from spacy.matcher import Matcher

nlp = spacy.blank("en")
matcher = Matcher(nlp.vocab)
matcher.add("HELLO_WORLD", [[{"LOWER": "hello"}, {"LOWER": "world"}]])

doc = nlp("hello world from spaCy")
matches = matcher(doc)
print(matches)
```

### Save Pipelines And Serialized Docs

Use `to_disk` and `from_disk` for pipeline state:

```python
import spacy

nlp = spacy.load("en_core_web_sm")
nlp.to_disk("./artifact/spacy-pipeline")

loaded = spacy.load("./artifact/spacy-pipeline")
doc = loaded("Reloaded pipeline.")
print(doc.ents)
```

Use `DocBin` when you need to serialize many docs efficiently:

```python
import spacy
from spacy.tokens import DocBin

nlp = spacy.load("en_core_web_sm")
docbin = DocBin(store_user_data=True)

for doc in nlp.pipe(["one document", "another document"]):
    docbin.add(doc)

docbin.to_disk("docs.spacy")
```

## Config And Runtime Setup

spaCy itself does not require API keys or remote authentication. Configuration is local:

- runtime configuration lives in the pipeline package and its `config.cfg`
- custom training and custom components should keep `config.cfg` in version control
- model selection is a local package choice such as `en_core_web_sm` vs `en_core_web_trf`

For training or custom packaged pipelines, use the spaCy CLI instead of ad hoc scripts:

```bash
python -m spacy init fill-config base.cfg config.cfg
python -m spacy debug config config.cfg
```

If you are starting from a partial config or a quickstart-generated config, fill it before training so the effective settings are explicit and reproducible.

## Long-Running Services

For persistent services that process unbounded traffic, spaCy `v3.8` adds `memory_zone()`:

```python
import spacy

nlp = spacy.load("en_core_web_sm")

with nlp.memory_zone():
    doc = nlp("Process this request inside a bounded memory scope.")
    print(doc.ents)
```

This reduces cache growth in `Vocab` and `StringStore`, but anything created inside the block must be treated as invalid once the block exits.

## Common Pitfalls

- Installing `spacy` does not install English or other pretrained pipelines. You still need `python -m spacy download en_core_web_sm` or another explicit model install.
- `nlp.pipe(...)` returns a generator, not a list. Wrap it with `list(...)` only if you actually need all docs in memory.
- Do not copy spaCy v2 examples that pass component callables directly to `nlp.add_pipe(...)`; spaCy v3 expects registered factory names.
- Disabling or reordering components can break dependent annotations. In many languages, the lemmatizer depends on POS information from `tagger` plus `attribute_ruler` or from `morphologizer`.
- When you only need sentence segmentation, prefer `senter` or `sentencizer` over running the full dependency parser.
- Multiprocessing with `n_process` can add noticeable startup and copy overhead on macOS and Windows because Python uses `spawn`.
- If you install or upgrade spaCy from source on a platform without suitable wheels, build constraints may be required to avoid `numpy` ABI mismatches.
- `Matcher` and other rule-based objects should be created with `nlp.vocab`, not a separate vocab.
- Docs created inside `memory_zone()` must not escape that context manager.

## Official Links

- API reference: `https://spacy.io/api`
- Usage docs: `https://spacy.io/usage`
- Models directory: `https://spacy.io/models`
- Install docs: `https://spacy.io/usage`
- Processing pipelines guide: `https://spacy.io/usage/processing-pipelines`
- Saving and loading guide: `https://spacy.io/usage/saving-loading`
- Memory management guide: `https://spacy.io/usage/memory-management`
- Registry: `https://pypi.org/project/spacy/`
- Releases: `https://github.com/explosion/spaCy/releases`
