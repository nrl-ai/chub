---
name: package
description: "OmegaConf Python package guide for hierarchical configuration, YAML loading, merges, interpolation, and structured configs"
metadata:
  languages: "python"
  versions: "2.3.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "omegaconf,config,yaml,dataclass,interpolation,python"
---

# OmegaConf Python Package Guide

## Golden Rule

Use `omegaconf` when you need hierarchical configuration that can be loaded from YAML, Python containers, dataclasses, and CLI-style overrides through one API. For package version `2.3.0`, prefer the version-pinned `2.3_branch` docs instead of the previous `latest` URL, because `latest` currently tracks the `2.4.0.dev` branch.

## Install

Pin the package version your project expects:

```bash
python -m pip install "omegaconf==2.3.0"
```

Common alternatives:

```bash
uv add "omegaconf==2.3.0"
poetry add "omegaconf==2.3.0"
```

## Initialize And Load Config

OmegaConf can create configs from Python containers, YAML strings, YAML files, and dotlist or CLI overrides.

### Create from a Python dict

```python
from omegaconf import OmegaConf

cfg = OmegaConf.create(
    {
        "app": {"name": "demo", "debug": False},
        "db": {"host": "localhost", "port": 5432},
    }
)

print(cfg.app.name)      # demo
print(cfg["db"]["port"]) # 5432
```

### Load from YAML

```python
from omegaconf import OmegaConf

cfg = OmegaConf.load("config.yaml")
print(cfg.db.host)
```

You can also parse a YAML string:

```python
from omegaconf import OmegaConf

cfg = OmegaConf.create(
    """
    service:
      host: 127.0.0.1
      port: 8080
    """
)
```

### Create overrides from CLI-style arguments

```python
from omegaconf import OmegaConf

base = OmegaConf.load("config.yaml")
overrides = OmegaConf.from_dotlist(
    [
        "service.host=0.0.0.0",
        "service.port=9000",
        "features.search=true",
    ]
)

cfg = OmegaConf.merge(base, overrides)
```

Use `OmegaConf.from_cli()` when you want to parse the current process arguments directly.

## Core Usage

### Read and update values

```python
from omegaconf import OmegaConf

cfg = OmegaConf.create({"server": {"host": "localhost", "port": 8080}})

cfg.server.port = 8081
cfg.server.host = "127.0.0.1"

print(cfg.server.port)
```

For path-based access:

```python
from omegaconf import OmegaConf

cfg = OmegaConf.create({"server": {"host": "localhost"}})

print(OmegaConf.select(cfg, "server.host"))
OmegaConf.update(cfg, "server.host", "api.internal")
```

`OmegaConf.select()` is safer than direct attribute access when a path may be absent.

### Merge multiple config sources

This is the main OmegaConf workflow: start with defaults, layer file config, then layer environment or CLI overrides.

```python
from omegaconf import OmegaConf

defaults = OmegaConf.create(
    {
        "db": {"host": "localhost", "port": 5432},
        "logging": {"level": "INFO"},
    }
)
file_cfg = OmegaConf.load("config.yaml")
cli_cfg = OmegaConf.from_dotlist(["logging.level=DEBUG"])

cfg = OmegaConf.merge(defaults, file_cfg, cli_cfg)
```

Fast merge is available:

```python
from omegaconf import OmegaConf

cfg = OmegaConf.unsafe_merge(defaults, file_cfg, cli_cfg)
```

Use `unsafe_merge()` only when you do not need the input configs afterward. It is faster, but it destroys the merged inputs.

### Structured configs with dataclasses

Structured configs are the safest way to keep config typed and validated.

```python
from dataclasses import dataclass
from omegaconf import MISSING, OmegaConf

@dataclass
class MySQLConfig:
    host: str = "localhost"
    port: int = 3306

@dataclass
class AppConfig:
    debug: bool = False
    db: MySQLConfig = MISSING

cfg = OmegaConf.structured(AppConfig)
print(cfg.debug)      # False
print(cfg.db.host)    # raises until db is provided
```

Populate it by merging in real values:

```python
runtime_cfg = OmegaConf.merge(
    OmegaConf.structured(AppConfig),
    {"db": {"host": "db.internal", "port": 3307}},
)
```

Typed structured configs matter because OmegaConf validates assignments:

```python
runtime_cfg.db.port = 5432   # ok
# runtime_cfg.db.port = "5432"  # implicit conversion may work for primitives
# runtime_cfg.db.port = "bad"   # raises ValidationError
```

### Interpolation and environment-driven config

Use interpolations to derive values from other nodes or environment variables.

```python
from omegaconf import OmegaConf

cfg = OmegaConf.create(
    {
        "db": {
            "host": "localhost",
            "port": 5432,
            "url": "postgresql://${db.host}:${db.port}/app",
        },
        "workers": "${oc.env:APP_WORKERS,4}",
    }
)

print(cfg.db.url)
print(cfg.workers)
```

Important for `2.3.0`: use `${oc.env:NAME,default}` for environment lookups. Older `${env:NAME}` examples are obsolete.

### Custom resolvers

Register resolvers once during process startup, then reference them in config.

```python
from omegaconf import OmegaConf

if not OmegaConf.has_resolver("as_int"):
    OmegaConf.register_new_resolver("as_int", lambda value: int(value))

cfg = OmegaConf.create(
    {
        "web": {
            "workers": "${as_int:${oc.env:APP_WORKERS,4}}",
        }
    }
)

print(cfg.web.workers)
```

If tests need a clean resolver registry, remove custom resolvers explicitly:

```python
OmegaConf.clear_resolver("as_int")
```

### Convert back to plain Python or dataclass objects

```python
from dataclasses import dataclass
from omegaconf import OmegaConf

@dataclass
class Settings:
    debug: bool = False

cfg = OmegaConf.structured(Settings)

plain = OmegaConf.to_container(cfg, resolve=True)
obj = OmegaConf.to_object(cfg)
```

Use `to_container(resolve=True)` when you need a plain `dict` or `list` for serialization or for passing to libraries that do not understand `DictConfig` and `ListConfig`.

### Save resolved config

```python
from omegaconf import OmegaConf

cfg = OmegaConf.create({"host": "localhost", "url": "http://${host}:8080"})
OmegaConf.save(cfg=cfg, f="generated.yaml", resolve=True)
```

## Configuration Inputs And Runtime Setup

OmegaConf has no auth model. The practical setup concerns are where your config values come from and how strictly they are validated.

Common input sources:

- checked-in YAML defaults
- environment-specific YAML files
- environment variables via `${oc.env:...}`
- CLI or dotlist overrides from deployment tooling
- dataclass-backed structured configs for validation

A common application pattern is:

```python
from omegaconf import OmegaConf

cfg = OmegaConf.merge(
    OmegaConf.structured(AppConfig),
    OmegaConf.load("defaults.yaml"),
    OmegaConf.load("local.yaml"),
    OmegaConf.from_dotlist(["db.host=db.prod.internal"]),
)
```

Resolve interpolations before handing the config to other libraries if they expect plain Python values:

```python
settings = OmegaConf.to_container(cfg, resolve=True, throw_on_missing=True)
```

## Common Pitfalls

- `OmegaConf.create()` returns `DictConfig` or `ListConfig`, not plain `dict` or `list`. Convert with `OmegaConf.to_container()` when another API expects native containers.
- `OmegaConf.unsafe_merge()` is destructive. Do not reuse the inputs after calling it.
- Missing values use `???` or `MISSING` and raise when accessed. Use `throw_on_missing=True` during export to fail fast before runtime.
- Structured config fields enforce declared types. This is useful, but it can surface `ValidationError` earlier than plain-dict code expects.
- If `struct` mode is enabled, adding unknown keys fails unless you temporarily open the config for mutation.

```python
from omegaconf import OmegaConf, open_dict

cfg = OmegaConf.create({"db": {"host": "localhost"}})
OmegaConf.set_struct(cfg, True)

with open_dict(cfg):
    cfg.db.port = 5432
```

- Environment interpolation examples on blogs often still use `${env:VAR}`. On `2.3.0`, use `${oc.env:VAR}`.
- Dataclass fields without a default are treated as missing. Plan for that when creating structured configs from schemas.

## Version-Sensitive Notes For 2.3.0

- PyPI still lists `2.3.0` as the stable release on March 12, 2026. The `latest` Read the Docs site is for the `2.4.0.dev` branch, so use `https://omegaconf.readthedocs.io/en/2.3_branch/` for stable `2.3.0` behavior.
- The `2.3.0` release adds Python 3.11 support.
- The `2.3.0` release adds support for ignoring structured-config fields whose metadata sets `omegaconf_ignore=True`.
- `2.3.0` fixes a nested structured-config merge bug, so older workarounds around merge exceptions may be stale.
- If you are upgrading from older 2.0 or early 2.1 examples, note that `${env}` is no longer the supported environment resolver and several resolver APIs changed in `2.2`.

## Official Sources

- Stable docs for `2.3.0`: https://omegaconf.readthedocs.io/en/2.3_branch/
- Usage guide: https://omegaconf.readthedocs.io/en/2.3_branch/usage.html
- Structured config guide: https://omegaconf.readthedocs.io/en/2.3_branch/structured_config.html
- Custom resolvers guide: https://omegaconf.readthedocs.io/en/2.3_branch/custom_resolvers.html
- Docs `latest` landing page: https://omegaconf.readthedocs.io/en/latest/
- PyPI package page: https://pypi.org/project/omegaconf/
- Official GitHub releases: https://github.com/omry/omegaconf/releases
