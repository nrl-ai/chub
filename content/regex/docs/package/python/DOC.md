---
name: package
description: "Python regex package guide for enhanced regular expressions, fuzzy matching, repeated captures, and Unicode-aware text processing"
metadata:
  languages: "python"
  versions: "2026.2.28"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "regex,regular-expressions,pattern-matching,text-processing,unicode"
---

# regex Python Package Guide

## What This Package Is

`regex` is a third-party regular-expression engine for Python. It is mostly compatible with the standard-library `re` module, but adds features that `re` does not have, including repeated captures, overlapped matches, fuzzy matching, partial matches, nested sets, and richer Unicode handling.

- Ecosystem: `pypi`
- Package: `regex`
- Import: `import regex`
- Covered version: `2026.2.28`
- Upstream repo: `https://github.com/mrabarnett/mrab-regex`
- Registry page: `https://pypi.org/project/regex/`

As of Thursday, March 12, 2026, PyPI shows `2026.2.28` as the current release, which matches the version covered here.

## Installation

```bash
pip install "regex==2026.2.28"
```

```bash
uv add "regex==2026.2.28"
```

```bash
poetry add "regex==2026.2.28"
```

PyPI lists `Requires: Python >=3.9` for `2026.2.28`.

## Initialization And Drop-In Use

For most code, start with the same workflow you would use for `re`:

```python
import regex

pattern = regex.compile(r"\b[a-z]+\b", flags=regex.IGNORECASE)
matches = pattern.findall("One two THREE")
print(matches)  # ['One', 'two', 'THREE']
```

If you are migrating code that already imports `re`, the lowest-friction change is often:

```python
import regex as re

if re.fullmatch(r"[A-Z]{3}\d{2}", "ABC12"):
    print("ok")
```

Core entry points that map closely to `re`:

- `regex.compile`
- `regex.search`, `regex.match`, `regex.fullmatch`
- `regex.findall`, `regex.finditer`
- `regex.split`, `regex.sub`, `regex.subn`
- `regex.escape`

## Core Usage Patterns

### Repeated Captures

Repeated groups keep all captures, not just the last one:

```python
import regex

match = regex.search(r"(\w{2})+", "abcd")
print(match.group(1))      # 'cd'
print(match.captures(1))   # ['ab', 'cd']
print(match.starts(1))     # [0, 2]
print(match.ends(1))       # [2, 4]
```

### Overlapped Matches

Use `overlapped=True` when matches should slide over one another:

```python
import regex

print(regex.findall(r"aba", "ababa", overlapped=True))
# ['aba', 'aba']
```

### Fuzzy Matching

Fuzzy constraints allow insertions, deletions, and substitutions:

```python
import regex

match = regex.search(r"(color){e<=1}", "colour")
print(match.group(0))      # colour
print(match.fuzzy_counts)  # e.g. (0, 1, 0)
```

When there are several possible fuzzy matches, `regex.BESTMATCH` is often the safest starting flag.

### Partial Matches

Use `partial=True` for incremental validation or streaming input:

```python
import regex

match = regex.fullmatch(r"\d{4}", "12", partial=True)
print(match is not None)  # True
print(match.partial)      # True
```

### Version 1 Features

Be explicit when you rely on `regex`-specific behavior such as nested sets or full case-folding:

```python
import regex

pattern = regex.compile(r"[[a-z]--[aeiou]]+", regex.VERSION1)
print(pattern.findall("alphabet soup"))
```

If a codebase mixes plain `re`-style patterns and `regex`-specific patterns, call out which ones require `regex.VERSION1` or inline `(?V1)`.

## Configuration And Safety

`regex` is a local library, so there is no service authentication or API credential setup.

Configuration that matters in practice:

- Choose flags explicitly when behavior matters: `IGNORECASE`, `MULTILINE`, `DOTALL`, `BESTMATCH`, `VERSION1`.
- Use `timeout=` for untrusted patterns or large inputs.
- Use `concurrent=True` only when matching against built-in immutable strings that will not change during the operation.

Timeout example:

```python
import regex

try:
    regex.search(r"(a+)+$", "a" * 50_000, timeout=0.2)
except TimeoutError:
    print("match timed out")
```

Concurrent matching example:

```python
import regex

words = regex.findall(r"\w+", "alpha beta gamma", concurrent=True)
print(words)
```

## Common Pitfalls

- `regex` is not part of the standard library. Only use it when the environment explicitly installs it.
- The module is mostly `re`-compatible, not perfectly identical. Re-test patterns that depend on edge-case character classes, Unicode behavior, or backtracking details.
- If you rely on nested sets, full Unicode case-folding, or other Version 1 behavior, set `regex.VERSION1` or `(?V1)` explicitly instead of assuming a default.
- Fuzzy matching, POSIX matching, and catastrophic-backtracking patterns can be expensive. Add `timeout=` for production code that handles user input.
- Match objects keep a reference to the searched string. If you keep many matches for large inputs, call `match.detach_string()` after extracting what you need.
- The upstream README notes that PyPy stores strings as UTF-8 internally, so behavior outside ASCII is not expected to match CPython.
- The GitHub default-branch README is current project documentation, not a release-pinned manual. For version-sensitive work, verify against the versioned PyPI page for `2026.2.28`.

## Version-Sensitive Notes For `2026.2.28`

- This doc is pinned to `2026.2.28`, the release published on PyPI on February 28, 2026.
- PyPI marks this release as requiring Python 3.9 or newer.
- The upstream project documents Unicode-focused behavior and Version 0 versus Version 1 semantics in the README; older blog posts often omit those distinctions.
- If copied examples behave differently across environments, check whether they assumed the standard-library `re` module, an older `regex` release, or `VERSION1` semantics.

## Practical Workflow For Agents

1. Start with a normal `re`-style implementation using `import regex`.
2. Only introduce `overlapped=True`, fuzzy constraints, `partial=True`, or `VERSION1` when the use case actually needs them.
3. Add `timeout=` before using complex patterns against user-controlled text.
4. If behavior is unclear, compare the versioned PyPI page for `2026.2.28` with the current upstream README instead of relying on third-party tutorials.

## Official Sources

- PyPI project page: `https://pypi.org/project/regex/`
- Version-pinned PyPI page: `https://pypi.org/project/regex/2026.2.28/`
- Upstream repository: `https://github.com/mrabarnett/mrab-regex`
- Upstream README: `https://github.com/mrabarnett/mrab-regex/blob/master/README.rst`
