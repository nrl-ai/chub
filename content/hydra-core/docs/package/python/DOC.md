---
name: package
description: "Hydra configuration framework for Python apps with composable configs, structured configs, overrides, and multirun"
metadata:
  languages: "python"
  versions: "1.3.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "hydra,configuration,cli,omegaconf,experiments,python"
---

# Hydra Python Package Guide

## Golden Rule

Use `hydra-core` `1.3.2` with an explicit `version_base` and a deliberate config layout. In Hydra 1.3, the most common breakages come from implicit defaults: omitted `version_base`, unexpected working-directory changes, and confusion about where Hydra looks for configs. Prefer `@hydra.main(version_base=None, config_path="conf", config_name="config")` for normal app entrypoints, and use the Compose API only for notebooks, tests, or embedded flows.

## Install

Pin the package version your project expects:

```bash
python -m pip install "hydra-core==1.3.2"
```

Common alternatives:

```bash
uv add "hydra-core==1.3.2"
poetry add "hydra-core==1.3.2"
```

Version-sensitive note:

- The Hydra `1.3` docs page lists Python `3.6-3.11` for the series, but the `1.3.2` release notes explicitly dropped Python `3.6`. For `1.3.2`, use Python `3.7-3.11`.

## Initialize With YAML Config Files

Hydra works best when your application has a primary config plus optional config groups.

Example layout:

```text
my_app/
  app.py
  conf/
    config.yaml
    db/
      sqlite.yaml
      postgres.yaml
```

Primary config:

```yaml
# conf/config.yaml
defaults:
  - db: sqlite
  - _self_

app_name: demo
debug: false
```

Config group options:

```yaml
# conf/db/sqlite.yaml
driver: sqlite
path: /tmp/demo.db
```

```yaml
# conf/db/postgres.yaml
driver: postgresql
host: localhost
port: 5432
user: app
database: demo
```

Application entrypoint:

```python
import hydra
from omegaconf import DictConfig, OmegaConf

@hydra.main(version_base=None, config_path="conf", config_name="config")
def main(cfg: DictConfig) -> None:
    print(OmegaConf.to_yaml(cfg))

if __name__ == "__main__":
    main()
```

Run with the default config:

```bash
python app.py
```

Override from the command line:

```bash
python app.py db=postgres debug=true
```

Run multiple jobs:

```bash
python app.py --multirun db=sqlite,postgres
```

## Structured Configs With `ConfigStore`

Use structured configs when you want dataclass-backed config validation and editor-friendly types.

```python
from dataclasses import dataclass, field

import hydra
from hydra.core.config_store import ConfigStore

@dataclass
class DBConfig:
    driver: str = "sqlite"
    path: str = "/tmp/demo.db"

@dataclass
class AppConfig:
    app_name: str = "demo"
    debug: bool = False
    db: DBConfig = field(default_factory=DBConfig)

cs = ConfigStore.instance()
cs.store(name="config", node=AppConfig)

@hydra.main(version_base=None, config_name="config")
def main(cfg: AppConfig) -> None:
    print(cfg.db.driver)

if __name__ == "__main__":
    main()
```

Use structured configs when:

- you want type-checked defaults
- you need a Python-native config schema
- you want to register multiple named configs or config groups in code

Use YAML config files when:

- non-Python users need to edit config
- you want config files to live next to deployment assets
- the project already relies on Hydra defaults lists and file-based overrides

## Compose API For Tests, Notebooks, And Embedded Use

The Compose API is useful when `@hydra.main()` is not practical, such as unit tests, Jupyter notebooks, or libraries embedding Hydra config composition.

```python
from hydra import compose, initialize
from omegaconf import OmegaConf

with initialize(version_base=None, config_path="conf"):
    cfg = compose(config_name="config", overrides=["db=postgres", "debug=true"])

print(OmegaConf.to_yaml(cfg))
```

Prefer `@hydra.main()` for normal command-line apps. The Hydra docs warn that the Compose API does not provide the full `@hydra.main()` feature set such as shell tab completion, multirun, working-directory management, and some logging integration.

If your configs live in an importable Python module instead of a path relative to the caller, use `initialize_config_module()` or a `pkg://...` config path. Hydra `1.3.2` specifically added support for non-relative module paths in `config_path`, for example:

```python
import hydra

@hydra.main(version_base=None, config_path="pkg://my_app.conf", config_name="config")
def main(cfg):
    ...
```

## Instantiate Python Objects From Config

Hydra can construct objects directly from config using `_target_`.

Config:

```yaml
trainer:
  _target_: my_app.training.Trainer
  epochs: 10
  lr: 0.001
```

Python:

```python
import hydra
from hydra.utils import instantiate
from omegaconf import DictConfig

@hydra.main(version_base=None, config_path="conf", config_name="config")
def main(cfg: DictConfig) -> None:
    trainer = instantiate(cfg.trainer)
    trainer.run()
```

This pattern is common in ML and experiment codebases. It keeps object wiring in config instead of hardcoding class selection in Python.

## Config, Secrets, And Runtime Behavior

Hydra does not handle service authentication. It only composes configuration. Treat credentials as application concerns and keep them out of committed config files.

Practical guidance:

- Keep secrets in environment variables, a secret manager, or local untracked overrides.
- Do not commit real credentials into `conf/*.yaml`.
- Pass environment-derived values into your app after Hydra composes the config, or inject them at runtime with command-line overrides.

Hydra-specific runtime behavior to configure explicitly:

- `version_base`: set this on every `@hydra.main()` and `initialize()` call
- `hydra.job.chdir`: decide whether each run should change into Hydra's output directory
- `hydra.run.dir` and `hydra.sweep.dir`: set these if your project needs deterministic output locations
- `hydra.searchpath`: use this only in the primary config when you need Hydra to search additional config sources

Example working-directory configuration:

```yaml
hydra:
  job:
    chdir: false
  run:
    dir: outputs/${now:%Y-%m-%d}/${now:%H-%M-%S}
```

In Hydra 1.2+, `version_base=None` implies the newer defaults, including `hydra.job.chdir=False`. If you depend on the older behavior that changes into the run directory, set `hydra.job.chdir=true` explicitly instead of relying on legacy defaults.

## Debugging And Common CLI Helpers

Useful commands when a config is not composing the way you expect:

```bash
python app.py --cfg job
python app.py --cfg hydra
python app.py --info searchpath
python app.py --resolve --cfg job
```

When you need the full Python traceback instead of Hydra's shorter error formatting:

```bash
HYDRA_FULL_ERROR=1 python app.py
```

## Common Pitfalls

- Omitting `version_base`. Hydra will warn, and default behavior can change across releases.
- Omitting `_self_` in a defaults list when composition order matters. Hydra 1.1+ changed defaults-list composition behavior.
- Using the Compose API in place of `@hydra.main()` for a normal CLI app. You lose important Hydra runtime behavior.
- Forgetting that config search path issues are often the real cause of "config not found" failures. Check `--info searchpath`.
- Setting `hydra.searchpath` outside the primary config. Hydra documents that this must be configured in the primary config.
- Assuming Hydra will manage credentials. It will not; that logic belongs in your application.
- Relying on the process working directory without checking `hydra.job.chdir`.
- Decorating a `@hydra.main()` function without preserving `__wrapped__`. Hydra's docs require `@functools.wraps` on custom decorators so it can locate the config path correctly.

## Version-Sensitive Notes For `1.3.2`

- Pin documentation links to `https://hydra.cc/docs/1.3/...` instead of the unversioned docs root when targeting `hydra-core==1.3.2`.
- Hydra `1.3.2` dropped Python `3.6`.
- Hydra `1.3.2` added support for non-relative module paths in `config_path`, which makes packaged config modules easier to use.
- Hydra `1.3.2` added `hydra.utils.get_object` for resolving a dotted path without instantiating it.
- The `1.3` docs line is stable but not actively maintained; if a project is migrating beyond `1.3.x`, re-check the "Next" or newer versioned docs before copying behavior assumptions.

## Official Sources

- Docs root: `https://hydra.cc/docs/1.3/intro/`
- Compose API: `https://hydra.cc/docs/1.3/advanced/compose_api/`
- Search path: `https://hydra.cc/docs/1.3/advanced/search_path/`
- Instantiate objects: `https://hydra.cc/docs/1.3/advanced/instantiate_objects/overview/`
- Structured configs: `https://hydra.cc/docs/1.3/tutorials/structured_config/intro/`
- `version_base` upgrade note: `https://hydra.cc/docs/1.3/upgrades/version_base/`
- Working-directory behavior: `https://hydra.cc/docs/1.3/upgrades/1.1_to_1.2/changes_to_job_working_dir/`
- Decorator guidance: `https://hydra.cc/docs/1.3/advanced/decorating_main/`
- PyPI registry page: `https://pypi.org/project/hydra-core/`
- `1.3.2` release notes: `https://github.com/facebookresearch/hydra/releases/tag/v1.3.2`
