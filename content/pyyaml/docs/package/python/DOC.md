---
name: package
description: "PyYAML package guide for Python YAML parsing and emitting"
metadata:
  languages: "python"
  versions: "6.0.3"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "pyyaml,yaml,serialization,configuration,python"
---

# PyYAML Python Package Guide

## What It Is

`PyYAML` is the standard Python package for parsing YAML into Python objects and emitting Python objects as YAML.

- Package name: `PyYAML`
- Import name: `yaml`
- Version covered: `6.0.3`
- Python requirement on PyPI: `>=3.8`

## Installation

Install the package version your project expects:

```bash
pip install PyYAML==6.0.3
```

Basic import:

```python
import yaml
```

## Initialization And Setup

PyYAML does not require network setup, authentication, or service credentials.

The main setup decision is which loader and dumper you use:

- `safe_load` / `safe_dump`: default choice for untrusted or ordinary YAML
- `load(..., Loader=...)`: only when you intentionally need a specific loader
- `CLoader` / `CDumper` or `CSafeLoader` / `CSafeDumper`: faster LibYAML-backed implementations when available

Official docs recommend this fallback pattern when you want C bindings if available:

```python
import yaml

try:
    from yaml import CSafeLoader as SafeLoader, CSafeDumper as SafeDumper
except ImportError:
    from yaml import SafeLoader, SafeDumper
```

## Core Usage

### Parse YAML Safely

Use `safe_load()` for normal config files and request payloads:

```python
import yaml

data = yaml.safe_load("""
debug: true
port: 8080
hosts:
  - api.internal
  - worker.internal
""")

assert data == {
    "debug": True,
    "port": 8080,
    "hosts": ["api.internal", "worker.internal"],
}
```

### Parse Multiple YAML Documents

Use `safe_load_all()` when the input contains `---` document separators:

```python
import yaml

documents = list(yaml.safe_load_all("""
---
name: app
---
name: worker
"""))
```

### Dump Python Objects As YAML

```python
import yaml

payload = {
    "service": "billing",
    "replicas": 3,
    "features": ["invoices", "refunds"],
}

text = yaml.safe_dump(payload, sort_keys=False)
print(text)
```

Use `safe_dump_all()` for multi-document output:

```python
import yaml

documents = [
    {"kind": "ConfigMap", "name": "app"},
    {"kind": "Secret", "name": "app-secret"},
]

yaml_text = yaml.safe_dump_all(documents, sort_keys=False)
```

### Read And Write Files

```python
from pathlib import Path
import yaml

config_path = Path("config.yaml")
config = yaml.safe_load(config_path.read_text())

config["debug"] = False
config_path.write_text(yaml.safe_dump(config, sort_keys=False))
```

## Custom Types And Tags

If you need custom YAML tags, register constructors and representers explicitly instead of switching everything to unsafe loading.

Example with a custom `!env` tag:

```python
import os
import yaml

def env_constructor(loader, node):
    key = loader.construct_scalar(node)
    return os.environ[key]

class EnvLoader(yaml.SafeLoader):
    pass

EnvLoader.add_constructor("!env", env_constructor)

config = yaml.load("token: !env API_TOKEN\n", Loader=EnvLoader)
```

For custom Python classes, upstream docs also support using `yaml.YAMLObject` plus explicit safe loader configuration when you want that class to work with safe loading.

## Configuration Notes

### Safe Defaults

For most agent-written code:

- parse with `yaml.safe_load()`
- emit with `yaml.safe_dump(..., sort_keys=False)`
- use a custom `SafeLoader` subclass only when you need a controlled extension such as a custom tag

### Flow Style vs Block Style

PyYAML chooses block style for collections with nested collections and flow style for flat collections unless you override `default_flow_style`.

If you need stable block-style output:

```python
yaml.safe_dump(data, sort_keys=False, default_flow_style=False)
```

### C Bindings

LibYAML-backed loaders and dumpers are usually much faster. Upstream docs note there are some subtle behavioral differences between the pure Python and LibYAML implementations, so keep tests around formatting-sensitive output.

## Common Pitfalls

### Do Not Use `yaml.load()` On Untrusted Input

The upstream docs are explicit: `yaml.load` is as powerful as `pickle.load`. Treat it as unsafe for untrusted YAML.

Prefer:

```python
data = yaml.safe_load(text)
```

Only use `yaml.load(text, Loader=...)` when you intentionally need non-safe tags or a custom loader.

### Do Not Copy Legacy One-Argument `yaml.load(...)` Examples

PyYAML's long-lived documentation page still contains older examples and is not tightly version-pinned. For `6.0.3`, use `safe_load()` or pass an explicit `Loader=...`.

### Remember The Import Name

Install with `pip install PyYAML`, but import with:

```python
import yaml
```

### `safe_load()` Will Not Construct Arbitrary Python Objects

That limitation is intentional. If you need custom tags, register them on a `SafeLoader` subclass or use an explicit loader only for trusted input.

### C Loader Availability Depends On The Environment

Do not assume `CSafeLoader` or `CDumper` exists everywhere. Keep the import fallback shown above if your code needs to run across multiple environments.

## Version-Sensitive Notes For 6.0.3

- PyPI currently lists `Requires: Python >=3.8` for `6.0.3`.
- The official wiki documentation page is the main upstream landing page, but it is broad rather than version-scoped and still includes some legacy examples. This is an inference from the current docs content, not a separate upstream compatibility statement.
- The upstream changelog for `6.0.3` is minimal and package-focused; treat loader safety guidance and current PyPI metadata as the main practical constraints for agent-written code.

## Official Sources

- Docs: https://pyyaml.org/wiki/PyYAMLDocumentation
- PyPI: https://pypi.org/project/PyYAML/
- Source repository: https://github.com/yaml/pyyaml
- Changelog / releases: https://github.com/yaml/pyyaml/releases
