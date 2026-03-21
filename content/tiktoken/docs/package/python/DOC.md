---
name: package
description: "tiktoken package guide for Python with model-aware encodings, token counting, cache setup, special tokens, and custom encoding notes"
metadata:
  languages: "python"
  versions: "0.12.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "openai,tiktoken,python,tokens,tokenizer,bpe,llm"
---

# tiktoken Python Package Guide

## Golden Rule

- Use `tiktoken` when you need exact tokenizer behavior for OpenAI encodings, not rough token estimates.
- Prefer `tiktoken.encoding_for_model(...)` when you know the target model name.
- Prefer `tiktoken.get_encoding(...)` when you need a fixed encoding that should not change with future model-to-encoding remaps.

## Version-Sensitive Notes

- This entry is pinned to the version used here `0.12.0`.
- PyPI currently shows `0.12.0` as the latest published version as of `2026-03-12`, so the version used here matches current upstream.
- `tiktoken` `0.12.0` requires Python `>=3.9`.
- Upstream's `0.12.0` changelog notes add the `o200k_harmony` encoding, improve plugin type annotations, remove the need for `blobfile` when reading local files, and drop Python `3.8`.
- `encoding_for_model(...)` depends on model-name mappings shipped in the package. If you upgrade `tiktoken`, a newer release can change which encoding a model name resolves to. Pin the package version if token accounting must stay stable across deployments.

## Install

Pin the version when token counts need to be reproducible:

```bash
python -m pip install "tiktoken==0.12.0"
```

With `uv`:

```bash
uv add "tiktoken==0.12.0"
```

If a compatible wheel is unavailable for your platform, `pip` may need to build the Rust extension from source.

## Recommended Setup

Start by choosing whether your code should follow a model name or a fixed encoding name.

Use a fixed encoding when you want stable behavior independent of model remaps:

```python
import tiktoken

enc = tiktoken.get_encoding("cl100k_base")
```

Use a model name when your code should follow OpenAI's model mapping in this package version:

```python
import tiktoken

enc = tiktoken.encoding_for_model("gpt-4o-mini")
```

To inspect what encodings are available in the installed package:

```python
import tiktoken

print(tiktoken.list_encoding_names())
```

## Core Usage

### Count Tokens

```python
import tiktoken

enc = tiktoken.encoding_for_model("gpt-4o-mini")
text = "Context matters."

token_ids = enc.encode(text)
print(token_ids)
print(len(token_ids))
```

For plain string token counting, `len(enc.encode(text))` is the common pattern.

### Decode Tokens Back To Text

```python
import tiktoken

enc = tiktoken.get_encoding("cl100k_base")
token_ids = enc.encode("hello world")

decoded = enc.decode(token_ids)
print(decoded)
```

### Work With Single Tokens Safely

Single tokens do not always align to UTF-8 character boundaries. Use `decode_single_token_bytes(...)` for per-token inspection:

```python
import tiktoken

enc = tiktoken.get_encoding("cl100k_base")
token_id = enc.encode("hello")[0]

print(enc.decode_single_token_bytes(token_id))
```

### Encode Multiple Strings

Use batch encoding when you already have many strings to tokenize:

```python
import tiktoken

enc = tiktoken.get_encoding("o200k_base")

rows = ["alpha", "beta", "gamma"]
encoded_rows = enc.encode_batch(rows)
print(encoded_rows)
```

### Ignore Special Tokens In Ordinary Text

`encode_ordinary(...)` is useful when you want normal tokenization without special-token handling:

```python
import tiktoken

enc = tiktoken.get_encoding("cl100k_base")
token_ids = enc.encode_ordinary("plain text only")
print(token_ids)
```

### Allow Specific Special Tokens

By default, special tokens are rejected. Allow them explicitly when you intend to preserve them:

```python
import tiktoken

enc = tiktoken.get_encoding("gpt2")
text = "hello <|endoftext|>"

token_ids = enc.encode(text, allowed_special={"<|endoftext|>"})
print(token_ids)
```

## Choosing The Right Encoding

Reach for these APIs in this order:

- `encoding_for_model(model_name)`: best when you are counting tokens for a known OpenAI model.
- `get_encoding(encoding_name)`: best when you need a stable base encoding such as `cl100k_base` or `o200k_base`.
- `list_encoding_names()`: best when debugging or validating what the installed package exposes.

Handle unknown model names defensively:

```python
import tiktoken

model_name = "my-unknown-model"

try:
    enc = tiktoken.encoding_for_model(model_name)
except KeyError:
    enc = tiktoken.get_encoding("o200k_base")
```

That fallback should be a deliberate choice. Do not assume every unknown model belongs to `o200k_base`.

## Config And Environment

`tiktoken` does not require API keys or service authentication. Most usage is local once the encoding assets are available.

For built-in encodings, the package caches tokenizer data on first use. Relevant environment variables from upstream source:

- `TIKTOKEN_CACHE_DIR`: preferred cache location
- `DATA_GYM_CACHE_DIR`: legacy fallback cache location

If neither variable is set, `tiktoken` falls back to the system temp directory.

```bash
export TIKTOKEN_CACHE_DIR="$HOME/.cache/tiktoken"
```

In CI, containers, or other restricted environments, prewarming this cache can avoid repeated downloads.

If `TIKTOKEN_CACHE_DIR` is set to the empty string, upstream disables caching rather than writing files.

## Offline And Restricted Environments

- The first load of a built-in encoding may need to fetch encoding data and cache it locally.
- If your runtime cannot reach public package assets, prewarm the cache during image build or ship the needed encoding files with your environment.
- If you build custom extension packages that load encodings from non-HTTP blob URLs, upstream may require the optional `blobfile` dependency for those paths.

## Custom Encodings

The maintainers expose `tiktoken.Encoding(...)` plus a plugin mechanism under the `tiktoken_ext` namespace.

Use this only when you need custom merge ranks or custom special tokens. For most agent tasks, built-in encodings are the right choice.

The README's extension guidance matters:

- custom plugin packages live under `tiktoken_ext`
- the namespace package should omit `tiktoken_ext/__init__.py`
- the package exposes `ENCODING_CONSTRUCTORS`

If you only need extra special tokens, it is usually simpler to derive from an existing encoding than to build a new tokenizer from scratch.

## Common Pitfalls

- `encoding_for_model(...)` can raise `KeyError` for new or private model names. Catch it and choose an explicit fallback.
- Token counts from `tiktoken` are for the text you pass in. Full API request overhead for message wrappers, tool schemas, or structured payloads is outside the tokenizer itself.
- `decode()` on individual tokens can be misleading because a token may contain partial UTF-8 bytes. Use `decode_single_token_bytes(...)` when inspecting one token at a time.
- Special tokens are disallowed by default. If your input intentionally contains them, pass `allowed_special=...` instead of stripping them silently.
- Do not hard-code a model-to-encoding assumption copied from an old blog post. Use the installed package's mapping or pin a known encoding name.
- First-use cache misses can look like random latency in ephemeral environments. Prewarm the cache if startup consistency matters.

## Practical Patterns

### Count A Prompt Before Sending It Elsewhere

```python
import tiktoken

def count_tokens(model: str, text: str) -> int:
    enc = tiktoken.encoding_for_model(model)
    return len(enc.encode(text))

print(count_tokens("gpt-4o-mini", "Summarize this file."))
```

### Reuse One Encoding Object

Create the encoding once and reuse it across many strings:

```python
import tiktoken

enc = tiktoken.get_encoding("cl100k_base")

def count_many(texts: list[str]) -> list[int]:
    return [len(tokens) for tokens in enc.encode_batch(texts)]
```

### Inspect Why Two Strings Tokenize Differently

```python
import tiktoken

enc = tiktoken.get_encoding("o200k_base")

for text in ["camelCase", "camel case"]:
    token_ids = enc.encode(text)
    print(text, token_ids, len(token_ids))
```

## Official Source URLs

- Repository: `https://github.com/openai/tiktoken`
- README: `https://github.com/openai/tiktoken/blob/main/README.md`
- Changelog: `https://github.com/openai/tiktoken/blob/main/CHANGELOG.md`
- PyPI release: `https://pypi.org/project/tiktoken/0.12.0/`
