---
name: package
description: "Kedro package guide for Python projects covering project creation, configuration, sessions, and data catalogs"
metadata:
  languages: "python"
  versions: "1.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "kedro,python,data-engineering,data-science,pipelines,mlops"
---

# Kedro Python Package Guide

## Golden Rule

Use `kedro==1.2.0` on Python 3.10+ and treat `kedro-datasets` as a separate dependency for real dataset connectors. As of March 12, 2026, PyPI lists `kedro 1.2.0` with `Requires: Python >=3.10`; some Kedro docs search snippets still mention Python 3.9+, so use the PyPI metadata as the install floor.

## Install

Use a virtual environment and pin Kedro explicitly:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "kedro==1.2.0"
```

`uv` also works:

```bash
uv venv
source .venv/bin/activate
uv pip install "kedro==1.2.0"
```

Useful extras from PyPI:

```bash
python -m pip install "kedro[jupyter]==1.2.0"
python -m pip install "kedro[docs]==1.2.0"
```

Most real projects also need dataset implementations from `kedro-datasets`:

```bash
python -m pip install "kedro-datasets[pandas]"
```

Verify the CLI is available:

```bash
kedro info
```

## Create A Project

The current docs recommend `uvx kedro new` when you want to generate a project without first installing Kedro globally:

```bash
uvx kedro new --name=spaceflights --tools=lint,test,data --example=y
cd spaceflights
uv pip install -r requirements.txt
kedro run
```

If Kedro is already installed in your environment, use the same workflow with `kedro new`:

```bash
kedro new --name=my-project --tools=lint,test,data --example=n
cd my-project
python -m pip install -r requirements.txt
```

Important generated project locations:

- `pyproject.toml`: project metadata and Kedro package settings
- `conf/base/`: shared config such as catalog and parameters
- `conf/local/`: developer-local overrides and credentials
- `src/<package_name>/pipeline_registry.py`: registers named pipelines
- `src/<package_name>/settings.py`: config loader, hooks, and project settings

## Core Workflow

### Add a modular pipeline

Create a new pipeline scaffold:

```bash
kedro pipeline create data_processing
```

Minimal node and pipeline code:

```python
# src/my_project/pipelines/data_processing/nodes.py
import pandas as pd

def add_total_price(orders: pd.DataFrame) -> pd.DataFrame:
    enriched = orders.copy()
    enriched["total_price"] = enriched["quantity"] * enriched["unit_price"]
    return enriched
```

```python
# src/my_project/pipelines/data_processing/pipeline.py
from kedro.pipeline import node, pipeline

from .nodes import add_total_price

def create_pipeline(**kwargs):
    return pipeline(
        [
            node(
                func=add_total_price,
                inputs="orders",
                outputs="orders_enriched",
                name="add_total_price",
            )
        ]
    )
```

Register it so the CLI can run it by name:

```python
# src/my_project/pipeline_registry.py
from kedro.framework.project import find_pipelines

def register_pipelines():
    pipelines = find_pipelines()
    pipelines["__default__"] = sum(pipelines.values())
    return pipelines
```

Run the whole project or a specific pipeline:

```bash
kedro run
kedro run --pipeline=data_processing
kedro run --env=prod
kedro run --params=batch_size=500,model.random_state=7
kedro run --runner=ThreadRunner
```

Runner notes:

- `SequentialRunner` is the default
- `ParallelRunner` uses multiprocessing
- `ThreadRunner` is the safer concurrency option when you need threaded I/O and is the documented workaround when `SparkDataset` does not behave well with `ParallelRunner`

### Use Kedro programmatically

When you are outside the CLI and want to run or inspect a project from Python, bootstrap the project first:

```python
from pathlib import Path

from kedro.framework.session import KedroSession
from kedro.framework.startup import bootstrap_project

project_path = Path("/path/to/your-kedro-project")
bootstrap_project(project_path)

with KedroSession.create(
    project_path=project_path,
    env="local",
    runtime_params={"sample_size": 100},
) as session:
    context = session.load_context()
    catalog = context.catalog
    companies = catalog.load("companies")
    result = session.run(pipeline_name="data_processing")
```

Use `bootstrap_project()` for source-tree projects. The Kedro session docs note that packaged projects can use `configure_project()` instead.

## Configuration And Credentials

Kedro loads configuration through `OmegaConfigLoader` by default.

The normal layout is:

```text
conf/
  base/
    catalog.yml
    parameters.yml
  local/
    credentials.yml
```

Key behavior from the stable configuration docs:

- `conf/base` is the shared default
- `conf/local` overrides `base`
- duplicate top-level keys inside the same environment raise a `ValueError`
- `local` is for user-specific or secret config and should not be committed

Example catalog plus credentials split:

```yaml
# conf/base/catalog.yml
orders:
  type: pandas.CSVDataset
  filepath: data/01_raw/orders.csv

warehouse_orders:
  type: pandas.CSVDataset
  filepath: s3://my-bucket/orders.csv
  credentials: dev_s3
```

```yaml
# conf/local/credentials.yml
dev_s3:
  client_kwargs:
    aws_access_key_id: ${oc.env:AWS_ACCESS_KEY_ID}
    aws_secret_access_key: ${oc.env:AWS_SECRET_ACCESS_KEY}
```

Important constraints:

- `oc.env` is intended for `credentials.yml`; the advanced config docs say not to use it in catalog or parameter files
- if you set `KEDRO_ENV=prod`, Kedro will load that environment for `kedro run`, `kedro ipython`, and Kedro Jupyter commands
- `--env=<name>` overrides `KEDRO_ENV`
- `--conf-source=<path>` lets you load config from another folder, a `.tar.gz`, a `.zip`, or supported remote storage

If you need direct access in Python, load config first and build a catalog from it:

```python
from pathlib import Path

from kedro.config import OmegaConfigLoader
from kedro.framework.project import settings
from kedro.io import DataCatalog

project_path = Path("/path/to/project")
conf_path = str(project_path / settings.CONF_SOURCE)

conf_loader = OmegaConfigLoader(
    conf_source=conf_path,
    base_env="base",
    default_run_env="local",
)

catalog = DataCatalog.from_config(
    catalog=conf_loader["catalog"],
    credentials=conf_loader["credentials"],
)
```

## Data Catalog Usage

Kedro's `DataCatalog` handles dataset I/O so nodes can stay pure.

Programmatic catalog construction:

```python
from kedro.io import DataCatalog
from kedro_datasets.pandas import CSVDataset

catalog = DataCatalog(
    {
        "orders": CSVDataset(filepath="data/01_raw/orders.csv"),
        "orders_enriched": CSVDataset(filepath="data/04_feature/orders_enriched.csv"),
    }
)

orders = catalog.load("orders")
catalog.save("orders_enriched", orders)
```

Useful catalog inspection commands in a project:

```bash
kedro catalog describe-datasets --pipeline=data_processing
kedro catalog list-patterns
kedro registry list
kedro registry describe data_processing
```

Avoid calling catalog I/O directly inside node functions unless you are deliberately breaking Kedro's pure-function model.

## Common Pitfalls

- Install `kedro-datasets` separately. Since Kedro `0.19.0`, dataset implementations are no longer shipped in core `kedro`.
- Use the new dataset class names. Since `kedro-datasets 2.0.0`, `CSVDataSet`-style names became `CSVDataset`.
- Keep secrets in `conf/local/credentials.yml` or environment variables, not in `conf/base`.
- When multiprocessing re-imports code under `ParallelRunner` on macOS or Windows, project bootstrap/configuration matters. If concurrency is needed and multiprocessing causes trouble, test `ThreadRunner` first.
- Avoid dots in dataset names unless you intentionally want namespace semantics; the pipeline docs warn that dot notation can produce disconnected or confusing pipelines.
- Do not perform `catalog.load()` or `catalog.save()` inside normal nodes. Let Kedro inject datasets as node inputs and outputs.

## Version-Sensitive Notes

- Version used here and current PyPI release agree: `1.2.0` was published on January 29, 2026.
- The stable docs remain the right canonical docs root for review, but some search-indexed Kedro pages still surface older `0.19.x` URLs. Double-check the URL path before copying examples.
- PyPI metadata currently says `Requires: Python >=3.10`; if you see older install snippets mentioning Python 3.9+, treat those as stale for `1.2.0`.
- `kedro-datasets` is now a separate package, and modern examples should import classes from `kedro_datasets.*`, not the old in-core dataset modules.

## Official Sources

- PyPI: https://pypi.org/project/kedro/
- Stable docs root: https://docs.kedro.org/en/stable/
- New project tools: https://docs.kedro.org/en/stable/create/new_project_tools/
- CLI reference: https://docs.kedro.org/en/stable/getting-started/commands_reference/
- Configuration basics: https://docs.kedro.org/en/stable/configure/configuration_basics/
- Advanced configuration: https://docs.kedro.org/en/stable/configure/advanced_configuration/
- Credentials: https://docs.kedro.org/en/stable/configuration/credentials.html
- Data catalog usage: https://docs.kedro.org/en/stable/catalog-data/advanced_data_catalog_usage/
- Data catalog YAML examples: https://docs.kedro.org/en/stable/data/data_catalog_yaml_examples.html
- KedroSession lifecycle: https://docs.kedro.org/en/stable/kedro_project_setup/session.html
