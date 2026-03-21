---
name: package
description: "packaging for Python - PEP 440 versions, specifiers, markers, wheel tags, metadata, and lock files"
metadata:
  languages: "python"
  versions: "26.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "packaging,pypi,pep440,requirements,wheels,metadata"
---

# packaging Python Package Guide

## What It Is

`packaging` is the PyPA utility library for parsing and validating Python packaging data. Use it when code needs to work with:

- PEP 440 versions
- version specifiers such as `>=1.2,<2`
- PEP 508 requirement strings
- environment markers
- normalized distribution names
- wheel and sdist filenames
- wheel compatibility tags
- core metadata
- Python lock files

It does not install packages, resolve dependencies, or talk to package indexes. Use `pip`, `uv`, Poetry, or another installer for that.

## Installation

```bash
pip install packaging==26.0
```

```bash
uv add packaging==26.0
```

Python requirement for `packaging` 26.0: Python 3.8+.

`packaging` uses calendar-based versioning (`YY.N`).

## Imports At A Glance

```python
from packaging.version import Version, InvalidVersion, parse
from packaging.specifiers import SpecifierSet
from packaging.requirements import Requirement, InvalidRequirement
from packaging.markers import Marker, default_environment
from packaging.utils import canonicalize_name, canonicalize_version
from packaging.tags import sys_tags
from packaging.metadata import Metadata
```

## Setup Notes

- No auth, credentials, or service configuration is required.
- Most APIs are pure parsing and comparison helpers.
- Invalid inputs raise `InvalidVersion`, `InvalidRequirement`, `InvalidSpecifier`, or metadata exceptions. Catch them at input boundaries.
- Prefer `Version(...)` when you need strict PEP 440 validation. Use `parse(...)` only when you want the same validated object with a shorter call site.

## Core Usage

### Parse And Compare Versions

```python
from packaging.version import Version

current = Version("26.0")
minimum = Version("25.0")

if current > minimum:
    print("new enough")

assert Version("1.0a5") < Version("1.0")
assert Version("1.0.post1") > Version("1.0")
```

Use `Version` instead of string comparison. PEP 440 ordering is not the same as lexical ordering.

### Check Whether Versions Match A Specifier

```python
from packaging.specifiers import SpecifierSet
from packaging.version import Version

spec = SpecifierSet(">=25,<27")

assert Version("26.0") in spec
assert Version("27.0") not in spec

candidates = ["24.2", "25.0", "26.0", "26.0rc1"]
allowed = [v for v in candidates if Version(v) in spec]
print(allowed)  # ['25.0', '26.0']
```

For candidate lists, `SpecifierSet.filter(...)` is often cleaner:

```python
from packaging.specifiers import SpecifierSet

spec = SpecifierSet(">=1.0,<2.0")
print(list(spec.filter(["0.9", "1.0", "1.5", "2.0"])))
```

### Parse PEP 508 Requirements

```python
from packaging.requirements import Requirement

req = Requirement('requests[security]>=2.31; python_version >= "3.10"')

print(req.name)        # requests
print(req.extras)      # {'security'}
print(req.specifier)   # >=2.31
print(req.marker)      # python_version >= "3.10"
print(req.url)         # None
```

This is the right entry point for parsing dependency strings from `pyproject.toml`, lock data, or config files.

### Evaluate Environment Markers

```python
from packaging.markers import Marker, default_environment

marker = Marker('python_version >= "3.10" and sys_platform == "linux"')
env = default_environment()

if marker.evaluate(environment=env, context="requirement"):
    print("dependency applies")
```

Use the `context` argument intentionally:

- `"requirement"` for dependency requirements
- `"metadata"` for core metadata checks
- `"lock_file"` for lock-file evaluation

### Normalize Names And Versions

```python
from packaging.utils import canonicalize_name, canonicalize_version

assert canonicalize_name("My_Package") == "my-package"
assert canonicalize_version("1.4.0.0.0") == "1.4"
```

Use canonicalized names for cache keys, lookups, and deduping. Python package names are case-insensitive and normalize `_`, `.`, and `-`.

### Parse Wheel Filenames And Check Compatibility

```python
from packaging.tags import sys_tags
from packaging.utils import parse_wheel_filename

name, version, build, tags = parse_wheel_filename(
    "packaging-26.0-py3-none-any.whl"
)

supported = set(sys_tags())
if tags & supported:
    print(f"{name} {version} is installable on this interpreter")
```

`parse_sdist_filename(...)` and `parse_wheel_filename(...)` are useful when tooling needs to inspect artifacts before installation.

### Read Core Metadata

```python
from packaging.metadata import Metadata, parse_email

raw = """\
Metadata-Version: 2.4
Name: demo-package
Version: 1.2.3
Requires-Python: >=3.10
Requires-Dist: requests>=2.31
"""

parsed, unparsed = parse_email(raw)
metadata = Metadata.from_raw(parsed)

print(metadata.name)
print(metadata.version)
print(metadata.requires_python)
print(metadata.requires_dist)
print(unparsed)  # {}
```

Use metadata parsing when you are validating generated package metadata, building package inspection tooling, or processing `.dist-info/METADATA`.

### Work With Lock Files

`packaging` 26.0 adds the `packaging.pylock` module for the Python lock file specification.

```python
from packaging.pylock import Package, PackageWheel, Pylock
from packaging.utils import NormalizedName
from packaging.version import Version

lock = Pylock(
    lock_version="1.0",
    created_by="example-tool",
    packages=[
        Package(
            name=NormalizedName("packaging"),
            version=Version("26.0"),
            wheels=[
                PackageWheel(
                    url="https://files.pythonhosted.org/packages/example/packaging-26.0-py3-none-any.whl",
                    hashes={"sha256": "abc123"},
                )
            ],
        )
    ],
)

data = lock.to_dict()
round_tripped = Pylock.from_dict(data)
assert round_tripped.packages[0].name == "packaging"
```

Use this only for lock-file data structures. It is not a resolver.

## Common Patterns

### Validate User-Supplied Version Constraints

```python
from packaging.specifiers import SpecifierSet, InvalidSpecifier

def validate_constraint(raw: str) -> bool:
    try:
        SpecifierSet(raw)
    except InvalidSpecifier:
        return False
    return True
```

### Parse Requirements Safely From Config

```python
from packaging.requirements import Requirement, InvalidRequirement

def parse_requirement(raw: str) -> Requirement | None:
    try:
        return Requirement(raw)
    except InvalidRequirement:
        return None
```

### Select The Best Matching Version

```python
from packaging.specifiers import SpecifierSet
from packaging.version import Version

def best_match(available: list[str], constraint: str) -> str | None:
    spec = SpecifierSet(constraint)
    matches = sorted((Version(v) for v in available if Version(v) in spec))
    return str(matches[-1]) if matches else None
```

## Common Pitfalls

### `packaging` Is Strict

Invalid versions and malformed requirement strings fail fast. That is usually desirable in tooling, but it means copying raw text from user input or old metadata can raise exceptions immediately.

### Pre-Releases Are Easy To Misread

Specifier behavior around pre-releases is subtle. Be explicit when checking release candidates and beta versions, and test the exact constraints you plan to generate.

### Requirement URLs Are Not General URL Validators

Since `packaging` 23.2, `Requirement` no longer rejects every unusual URL shape during parsing. If you care about network safety or allowed hosts, validate the URL separately after parsing.

### Name Normalization Matters

Do not compare package names with raw strings from filenames or user input. Normalize them first with `canonicalize_name(...)`.

### Marker Evaluation Depends On Context

Use the right marker evaluation context for the data you are checking. `"requirement"`, `"metadata"`, and `"lock_file"` are not interchangeable.

### This Library Does Not Resolve Dependencies

`packaging` helps parse, compare, and validate packaging data. It will not tell you which transitive dependency set to install.

## Version-Sensitive Notes

- `26.0` adds `packaging.pylock` for Python lock files, `Metadata.import_names` and `Metadata.import_namespaces`, metadata writing helpers, and `Version.__replace__`. It also changes some `Specifier` and marker edge-case behavior, including returning `False` instead of raising for `.contains(...)` on an invalid version.
- `25.0` added support for PEP 751 extras and dependency groups in markers plus new platform tags including Android support.
- `24.2` changed prerelease handling for `<` and `>` specifiers to align with PEP 440 and added support for PEP 639 license expressions and license files.
- `23.2` stopped over-validating requirement URLs during requirement parsing.
- `22.0` removed legacy version and specifier parsing. If old code still imports `LegacyVersion` or `LegacySpecifier`, it needs to be rewritten.

## When To Reach For Other Tools

- Use `pip` or `uv` to install packages.
- Use `importlib.metadata` when you need metadata for already installed distributions in the current environment.
- Use `build`, `hatchling`, `setuptools`, or Poetry when you are creating distributions.
- Use `resolvelib`, Poetry, pip, or uv when you need dependency resolution.

## Official Sources

- Documentation: https://packaging.pypa.io/en/stable/
- Version handling: https://packaging.pypa.io/en/stable/version.html
- Specifiers: https://packaging.pypa.io/en/stable/specifiers.html
- Requirements: https://packaging.pypa.io/en/stable/requirements.html
- Markers: https://packaging.pypa.io/en/stable/markers.html
- Utils: https://packaging.pypa.io/en/stable/utils.html
- Tags: https://packaging.pypa.io/en/stable/tags.html
- Metadata: https://packaging.pypa.io/en/stable/metadata.html
- Changelog: https://packaging.pypa.io/en/stable/changelog.html
- PyPI: https://pypi.org/project/packaging/
