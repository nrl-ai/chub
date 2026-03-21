---
name: package
description: "tokenizers for Python - fast Hugging Face tokenization, tokenizer.json pipelines, training, padding, truncation, and alignment"
metadata:
  languages: "python"
  versions: "0.22.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "tokenizers,huggingface,nlp,bpe,wordpiece,tokenization"
---

# tokenizers Python Package Guide

## What It Is

`tokenizers` is Hugging Face's low-level fast tokenizer engine for Python, backed by the Rust implementation. Use it when you need to:

- load or save a full `tokenizer.json`
- train a tokenizer from files or iterators
- customize normalization, pre-tokenization, model, decoding, and post-processing
- inspect token alignment data such as `offsets` and `word_ids`
- run high-throughput batch tokenization outside the higher-level `transformers` API

If you need model-aware defaults for a pretrained model family, `transformers.AutoTokenizer` is often the higher-level entry point. Use `tokenizers` when you need direct control over the tokenizer pipeline itself.

## Installation

```bash
pip install tokenizers==0.22.2
```

```bash
uv add tokenizers==0.22.2
```

Python requirement for `tokenizers` 0.22.2: Python 3.9+.

The Hugging Face installation page currently says Tokenizers is tested on Python 3.5+, but the current PyPI release metadata for `0.22.2` requires Python `>=3.9`. Treat PyPI as authoritative for install constraints.

## Imports At A Glance

```python
from tokenizers import Tokenizer, models, normalizers, pre_tokenizers, decoders, trainers, processors
from tokenizers.pre_tokenizers import Whitespace
from tokenizers.processors import TemplateProcessing
from tokenizers.trainers import BpeTrainer
```

## Setup Notes

- No auth or remote service setup is required for local files or in-memory training.
- A `Tokenizer` runs a pipeline of normalization, pre-tokenization, model, and post-processing.
- Use `Tokenizer.save(...)` to persist the full tokenizer configuration, not just the vocabulary files.
- `Tokenizer.from_pretrained(identifier, revision="...", token="...")` loads from a Hugging Face Hub repo that contains `tokenizer.json`.
- For private Hub repos, pass the access token with the `token=` argument.
- If you pass pre-tokenized input like `["already", "split"]`, set `is_pretokenized=True`.

## Core Usage

### Load A Saved Or Hub-Hosted Tokenizer

```python
from tokenizers import Tokenizer

local_tokenizer = Tokenizer.from_file("tokenizer.json")

hub_tokenizer = Tokenizer.from_pretrained(
    "bert-base-cased",
    revision="main",
)
```

`from_pretrained(...)` only works when the Hub repo includes a `tokenizer.json` file.

### Encode, Pad, Truncate, And Inspect Alignments

```python
from tokenizers import Tokenizer

tokenizer = Tokenizer.from_pretrained("bert-base-cased")

tokenizer.enable_truncation(max_length=16)
tokenizer.enable_padding(
    direction="right",
    pad_token="[PAD]",
    pad_id=0,
)

encoding = tokenizer.encode("Hello, y'all! How are you?")

print(encoding.tokens)
print(encoding.ids)
print(encoding.attention_mask)
print(encoding.offsets)
print(encoding.word_ids)

decoded = tokenizer.decode(encoding.ids)
print(decoded)
```

Use alignment fields when you need to map model output back to source text spans:

- `offsets` maps tokens back to character spans in the original text
- `word_ids` maps tokens back to word positions, especially useful for pre-tokenized inputs and token classification tasks

### Encode Batches

```python
batch = tokenizer.encode_batch(
    [
        "Hello, y'all!",
        "How are you?",
        ("Question?", "Answer."),
    ]
)

for item in batch:
    print(item.tokens)
```

`encode_batch(...)` is the main high-throughput path. Padding applies across the batch once `enable_padding(...)` is active.

### Train A BPE Tokenizer From Files

```python
from tokenizers import Tokenizer
from tokenizers.models import BPE
from tokenizers.pre_tokenizers import Whitespace
from tokenizers.trainers import BpeTrainer

tokenizer = Tokenizer(BPE(unk_token="[UNK]"))
tokenizer.pre_tokenizer = Whitespace()

trainer = BpeTrainer(
    vocab_size=30_000,
    min_frequency=2,
    special_tokens=["[UNK]", "[CLS]", "[SEP]", "[PAD]", "[MASK]"],
)

tokenizer.train(
    ["data/train.txt", "data/valid.txt"],
    trainer=trainer,
)

tokenizer.save("tokenizer.json")
```

The special token order matters during training. If you list `"[UNK]"` first, it gets the first special-token ID.

### Train From An Iterator Instead Of Files

```python
from tokenizers import Tokenizer, models, trainers
from tokenizers.pre_tokenizers import ByteLevel

tokenizer = Tokenizer(models.BPE())
tokenizer.pre_tokenizer = ByteLevel()

trainer = trainers.BpeTrainer(
    vocab_size=20_000,
    initial_alphabet=ByteLevel.alphabet(),
    special_tokens=["<PAD>", "<BOS>", "<EOS>"],
)

def batch_iterator():
    for batch in [
        ["first example", "second example"],
        ["third example"],
    ]:
        yield batch

tokenizer.train_from_iterator(batch_iterator(), trainer=trainer, length=3)
tokenizer.save("tokenizer.json")
```

Use `train_from_iterator(...)` when your corpus already lives in memory, comes from a streaming dataset, or is too awkward to materialize as training files.

### Add A Post-Processor For Model-Specific Special Tokens

```python
from tokenizers.processors import TemplateProcessing

cls_id = tokenizer.token_to_id("[CLS]")
sep_id = tokenizer.token_to_id("[SEP]")

tokenizer.post_processor = TemplateProcessing(
    single="[CLS] $A [SEP]",
    pair="[CLS] $A [SEP] $B:1 [SEP]:1",
    special_tokens=[
        ("[CLS]", cls_id),
        ("[SEP]", sep_id),
    ],
)
```

After this, `tokenizer.encode(...)` automatically inserts the configured special tokens.

## Custom Pipeline Components

When you need full control, compose the tokenizer from components directly:

```python
from tokenizers import Tokenizer, decoders, models, normalizers, pre_tokenizers

tokenizer = Tokenizer(models.Unigram())
tokenizer.normalizer = normalizers.NFKC()
tokenizer.pre_tokenizer = pre_tokenizers.ByteLevel()
tokenizer.decoder = decoders.ByteLevel()
```

The main component families are:

- `normalizers` for cleanup such as Unicode normalization or lowercasing
- `pre_tokenizers` for splitting text before model tokenization
- `models` for the actual tokenization algorithm, such as BPE or Unigram
- `processors` for adding model-specific wrapping like `[CLS]` / `[SEP]`
- `decoders` for converting token IDs back to readable text

## Config And Auth

### Local Usage

- No config files are required.
- No auth is needed for `from_file(...)`, `train(...)`, `train_from_iterator(...)`, `encode(...)`, or `decode(...)`.

### Hugging Face Hub Usage

```python
from tokenizers import Tokenizer

tokenizer = Tokenizer.from_pretrained(
    "org/private-tokenizer",
    revision="main",
    token="hf_...",
)
```

- `identifier` is the Hub repo ID.
- `revision` can be a branch or commit.
- `token` is only needed for private repositories.

Pin `revision` when reproducibility matters.

## Common Pitfalls

- The public docs site currently surfaces `main v0.20.3` pages. Some examples and install claims are older than the current PyPI release.
- Do not assume the padding token exists. Add it during training or via special tokens before calling `enable_padding(...)`.
- Do not hardcode special-token IDs unless you control tokenizer creation. Use `token_to_id(...)`.
- `Tokenizer.save(...)` writes the full tokenizer pipeline. If you only keep `vocab.json` and `merges.txt`, you can lose processor or normalization details.
- `from_pretrained(...)` needs a Hub repo that actually contains `tokenizer.json`.
- When feeding pre-tokenized input, remember `is_pretokenized=True`; otherwise the input shape is interpreted differently.
- Batch padding is stateful on the tokenizer instance. Call `no_padding()` or reconfigure padding if later calls should not inherit it.
- Truncation is also stateful. Call `no_truncation()` if a later code path expects full sequences.

## Version-Sensitive Notes For 0.22.x

- Version used here `0.22.2` matches the current PyPI latest release.
- PyPI shows `tokenizers 0.22.2` released on January 5, 2026.
- Hugging Face's docs site still exposes a selector headed by `main v0.20.3`, so the usage docs are helpful but not fully synchronized with the newest package release.
- The `0.22.0` release notes mention new stream work and native async binding work.
- The `0.22.2` release notes focus on added-token deserialization, typing stub updates, and a PyO3 0.26 bump.
- If you rely on behavior added in `0.22.x`, verify against the exact release notes or repository code instead of assuming the docs site reflects the latest package state.

## Official Sources

- Docs root: https://huggingface.co/docs/tokenizers/index
- Quicktour: https://huggingface.co/docs/tokenizers/quicktour
- Installation: https://huggingface.co/docs/tokenizers/installation
- Pipeline: https://huggingface.co/docs/tokenizers/pipeline
- Components: https://huggingface.co/docs/tokenizers/components
- Tokenizer API: https://huggingface.co/docs/tokenizers/api/tokenizer
- Encoding API: https://huggingface.co/docs/tokenizers/api/encoding
- Trainers API: https://huggingface.co/docs/tokenizers/api/trainers
- Training from memory: https://huggingface.co/docs/tokenizers/training_from_memory
- PyPI: https://pypi.org/project/tokenizers/
- GitHub releases: https://github.com/huggingface/tokenizers/releases/tag/v0.22.0 and https://github.com/huggingface/tokenizers/releases/tag/v0.22.2
