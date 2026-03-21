---
name: package
description: "charset-normalizer for Python: detect and normalize text encodings from bytes or files"
metadata:
  languages: "python"
  versions: "3.4.5"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "python,encoding,unicode,text,charset,normalization,chardet"
---

# charset-normalizer Python Package Guide

## What It Is

`charset-normalizer` is a Python library for detecting the most likely text encoding for raw bytes or files and converting the result to usable Unicode text. It is commonly used directly, and it is also the maintained alternative to `chardet` in parts of the Python ecosystem.

Use it when you have bytes with no trustworthy declared charset. If the source already declares an encoding you trust, prefer that declared encoding first.

## Install

```bash
pip install charset-normalizer
```

Specific version:

```bash
pip install charset-normalizer==3.4.5
```

Optional Unicode data backport:

```bash
pip install "charset-normalizer[unicode-backport]"
```

## Core Usage

### Detect from bytes

```python
from charset_normalizer import from_bytes

payload = b"\x48\x65\x6c\x6c\x6f"
matches = from_bytes(payload)
best = matches.best()

if best is None:
    raise ValueError("Input does not look like text")

print(best.encoding)
print(best.language)
print(str(best))
```

### Detect from a file path

```python
from charset_normalizer import from_path

best = from_path("data.txt").best()

if best is not None:
    text = str(best)
    utf8_bytes = best.output()
```

### Detect from an open binary file handle

```python
from charset_normalizer import from_fp

with open("data.txt", "rb") as fh:
    best = from_fp(fh).best()
```

## Result Model

The primary APIs return a `CharsetMatches` collection. Call `.best()` to get the preferred `CharsetMatch` or `None`.

Useful `CharsetMatch` properties and methods:

- `encoding`: detected encoding name
- `language`: best-effort detected language, or `"Unknown"`
- `could_be_from_charset`: alternative encodings that decode to the same text
- `encoding_aliases`: normalized alias names
- `raw`: original bytes
- `output(encoding="utf_8")`: re-encode the decoded text, UTF-8 by default
- `str(match)`: decoded Unicode text

Example:

```python
from charset_normalizer import from_bytes

best = from_bytes(blob).best()
if best:
    print(best.encoding)
    print(best.could_be_from_charset)
    normalized = best.output("utf_8")
```

## Choosing the Right API

### Preferred modern APIs

- `from_bytes(...)`
- `from_path(...)`
- `from_fp(...)`
- `is_binary(...)`

These are the stable public interfaces documented with backward-compatibility guarantees.

### Legacy compatibility API

```python
from charset_normalizer import detect

result = detect(blob)
print(result["encoding"])
```

`detect()` exists mainly for `chardet`-style compatibility and migration. Upstream marks it as deprecated, but not planned for removal. Prefer `from_bytes(...).best()` for new code.

## Binary Detection

Use `is_binary(...)` before treating unknown input as text if you need a hard text-vs-binary decision.

```python
from charset_normalizer import is_binary

if is_binary("archive.bin"):
    print("Skip decoding")
```

`is_binary()` accepts a path, bytes payload, or binary file object.

## Configuration And Debugging

There is no service configuration or authentication. Control behavior through function arguments.

Common knobs on `from_bytes`, `from_path`, and `from_fp`:

- `threshold`: maximum chaos tolerated before rejecting a candidate
- `cp_isolation`: only test specific code pages
- `cp_exclusion`: skip specific code pages
- `preemptive_behaviour`: prioritize likely encodings hinted by the content
- `language_threshold`: minimum coherence for language inference
- `enable_fallback`: allow fallback matches when strict detection fails
- `explain=True`: emit debug-oriented detection logs

Example:

```python
from charset_normalizer import from_bytes

best = from_bytes(
    blob,
    threshold=0.15,
    cp_isolation=["utf_8", "cp1252", "iso8859_1"],
    explain=True,
).best()
```

If you want explicit logger wiring instead of the built-in debug helper, use `charset_normalizer.utils.set_logging_handler(...)`.

## CLI Usage

The package installs a `normalizer` CLI.

Inspect a file:

```bash
normalizer sample.txt
```

Minimal output:

```bash
normalizer -m sample.txt
```

Normalize to Unicode output files:

```bash
normalizer -n sample.txt
```

Module form:

```bash
python -m charset_normalizer sample.txt
```

The CLI emits JSON by default. Use it for quick inspection, conversion, or debugging when you do not need Python code.

## Common Pitfalls

- Package name and import name differ: install `charset-normalizer`, import `charset_normalizer`.
- `best()` can return `None`. Treat that as "not confidently text" instead of assuming UTF-8.
- Detection is a fallback tool, not a substitute for trustworthy declared encodings from HTTP headers, file metadata, or protocol specs.
- `detect()` is for compatibility. New code should use `from_bytes`, `from_path`, or `from_fp`.
- Optimized wheels may include a compiled `md__mypyc` module. Some standalone bundlers like PyInstaller can miss that hidden import.
- If you need a pure-Python install without speedups, use `pip install charset-normalizer --no-binary :all:`.

## Version-Sensitive Notes

- PyPI lists `3.4.5` as the latest release, published on `2026-03-06`.
- The Read the Docs pages under `/en/latest/` still identify themselves as `3.4.4` on `2026-03-11`. Treat the API docs as current enough for usage, but note the version banner lag.
- The upstream changelog entry for `3.4.5` only notes a build-related fix for the optimized build (`libm` linkage). There are no documented Python API changes in that release.
- `3.4.2` included a CLI fix for Python deprecation warnings around `argparse.FileType`.
- `3.4.0` added the CLI `--no-preemptive` option and Python 3.13 support.

## Bundling And Deployment Notes

If a frozen executable fails with `ModuleNotFoundError: No module named 'charset_normalizer.md__mypyc'`, upstream recommends either:

- adding the hidden import in the bundler configuration, or
- reinstalling without binary speedups

```bash
pip install charset-normalizer --no-binary :all:
```

## Official Sources

- Docs: https://charset-normalizer.readthedocs.io/en/latest/
- Installation and basic usage: https://charset-normalizer.readthedocs.io/en/latest/user/getstarted.html
- Result handling: https://charset-normalizer.readthedocs.io/en/latest/user/handling_result.html
- CLI: https://charset-normalizer.readthedocs.io/en/latest/user/cli.html
- Miscellaneous and `is_binary()`: https://charset-normalizer.readthedocs.io/en/latest/user/miscellaneous.html
- FAQ: https://charset-normalizer.readthedocs.io/en/latest/community/faq.html
- API reference: https://charset-normalizer.readthedocs.io/en/latest/api.html
- Optional speedup extension: https://charset-normalizer.readthedocs.io/en/latest/community/speedup.html
- PyPI package metadata: https://pypi.org/project/charset-normalizer/
- Upstream changelog: https://github.com/Ousret/charset_normalizer/blob/master/CHANGELOG.md
