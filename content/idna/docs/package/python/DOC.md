---
name: package
description: "idna package guide for Python - IDNA 2008 and UTS #46 domain handling"
metadata:
  languages: "python"
  versions: "3.11"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "idna,dns,unicode,domain,punycode,internationalization"
---

# idna Python Package Guide

## Golden Rule

Use the third-party `idna` package when you need IDNA 2008 or UTS #46 behavior. Do not assume Python's built-in `encodings.idna` codec is equivalent; upstream documents that as the older IDNA 2003 implementation.

## Installation

Install the package:

```bash
python -m pip install idna
```

Pin the documented version when behavior needs to match this entry exactly:

```bash
python -m pip install "idna==3.11"
```

Alternative package managers:

```bash
uv add idna==3.11
```

```bash
poetry add idna==3.11
```

PyPI lists `idna` `3.11` as supporting Python `>=3.8`.

## Setup

There is no service initialization, network client, or authentication step. Setup is just importing the module and deciding which conversion mode fits your input:

- `idna.encode()` / `idna.decode()` for full domains
- `idna.alabel()` / `idna.ulabel()` for single labels
- `uts46=True` when you need browser-style compatibility mapping on user input before IDNA conversion

## Core Usage

### Convert a full domain

```python
import idna

ascii_domain = idna.encode("ドメイン.テスト")
print(ascii_domain)  # b'xn--eckwd4c7c.xn--zckzah'

unicode_domain = idna.decode(ascii_domain)
print(unicode_domain)  # ドメイン.テスト
```

`encode()` returns `bytes`. `decode()` returns `str`.

### Convert a single label

Use label helpers when you already split the hostname or only need one component:

```python
import idna

label = idna.alabel("测试")
print(label)  # b'xn--0zwm56d'

unicode_label = idna.ulabel(label)
print(unicode_label)  # 测试
```

### Normalize user input with UTS #46

Pure IDNA 2008 is strict. Real user input often needs compatibility mapping first:

```python
import idna

try:
    idna.encode("Königsgäßchen")
except idna.InvalidCodepoint as exc:
    print(exc)

normalized = idna.encode("Königsgäßchen", uts46=True)
print(normalized)  # b'xn--knigsgchen-b4a3dun'
```

Use `uts46=True` when accepting domains from forms, CLIs, or copied text. Skip it if your input is already normalized and you want strict IDNA 2008 validation.

### Use the codec interface

If you need codec-style integration, upstream exposes `idna.codec` and the `idna2008` codec:

```python
import idna.codec

wire_value = "домен.испытание".encode("idna2008")
print(wire_value)

text_value = wire_value.decode("idna2008")
print(text_value)
```

### Validate and normalize for application code

```python
import idna

def normalize_domain(value: str) -> str:
    try:
        return idna.encode(value, uts46=True).decode("ascii")
    except idna.IDNAError as exc:
        raise ValueError(f"invalid internationalized domain: {value!r}") from exc
```

This is a common boundary pattern when your application stores hostnames as ASCII but accepts Unicode input.

## Config And Auth

There is no API key, credential flow, or external service configuration.

The runtime choices that matter are:

- Whether to use strict IDNA 2008 processing or `uts46=True`
- Whether to process whole domains or individual labels
- Whether downstream code expects `bytes` or ASCII `str`
- Whether legacy migration code still passes `transitional=True`

In `3.11`, upstream notes that Unicode `16.0.0` removed transitional processing from UTS #46. That means `transitional=True` no longer changes results in current behavior. Treat it as legacy compatibility noise, not an active tuning knob.

## Error Handling

Catch `idna.IDNAError` for generic conversion failures. Upstream also documents more specific exceptions:

- `idna.IDNABidiError` for invalid bidirectional labels
- `idna.InvalidCodepoint` for disallowed characters
- `idna.InvalidCodepointContext` for context-sensitive code point failures

Prefer converting these into application-level validation errors instead of letting raw exceptions surface to users.

## Common Pitfalls

- `idna.encode()` returns `bytes`, not text. Call `.decode("ascii")` if the rest of your code expects `str`.
- Python's built-in `encodings.idna` codec is older IDNA 2003 behavior, not a drop-in replacement for this package.
- `uts46=True` is a compatibility-mapping step for user input. It is not the same as strict validation.
- `transitional=True` is effectively obsolete in `3.11` because Unicode `16.0.0` removed transitional processing from UTS #46.
- Emoji domains and other labels outside the IDNA rules are rejected on purpose.
- Process domains as dot-separated labels. If you already split them, use `alabel()` / `ulabel()` instead of repeatedly rejoining.
- Do not rely on very old blog posts that say `idna` supports Python `3.6` or `3.7`; `3.11` requires Python `>=3.8`.
- If you handle untrusted input, stay on modern releases. Upstream's `3.7` release fixed a crafted-input performance issue in `encode()`.

## Version-Sensitive Notes

- This doc covers `idna` `3.11`, released on `2025-10-12`.
- The version used here `3.11` matches the current PyPI package version shown on the official project page as of `2026-03-12`.
- Upstream's `v3.11` release switched the generated tables back to Unicode `16.0.0`, which changes some UTS #46 compatibility behavior compared with `3.10`.
- The `v3.11` release also notes support for Python `3.14`.
- `3.10` had temporarily reverted to Unicode `15.1.0`; if you are diffing behavior between `3.10` and `3.11`, check UTS #46 mapping outcomes rather than assuming they are identical.

## Official Sources

- Repository and README: `https://github.com/kjd/idna`
- PyPI release page for `3.11`: `https://pypi.org/project/idna/3.11/`
- Releases index: `https://github.com/kjd/idna/releases`
- `v3.11` release note: `https://github.com/kjd/idna/releases/tag/v3.11`
