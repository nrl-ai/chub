---
name: package
description: "Mage OSS Python package guide for building and running data pipelines with the mage CLI and project-based workflows"
metadata:
  languages: "python"
  versions: "0.9.79"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "mage,mage-ai,python,data-pipelines,orchestration,etl,elt"
---

# Mage OSS Python Package Guide

## Golden Rule

Treat `mage-ai` as an app and project framework first, not as a generic Python SDK. The stable workflow is: install the package, create or open a Mage project, run it with the `mage` CLI, and write decorated pipeline blocks inside that project. Avoid building new code against random internal `mage_ai.*` modules unless the official docs show that import path.

## Install

Pin the package version your project expects:

```bash
python -m pip install "mage-ai==0.9.79"
```

Mage supports optional extras for connectors and integrations. Install only what you need:

```bash
python -m pip install "mage-ai[postgres,s3,dbt]==0.9.79"
python -m pip install "mage-ai[bigquery]==0.9.79"
```

There is also an `all` extra, but it pulls in a very large dependency set and is usually the wrong default for lightweight local development:

```bash
python -m pip install "mage-ai[all]==0.9.79"
```

Upstream quickstart still recommends Docker for the easiest first run, especially on machines where native connector dependencies are painful. For Python-native projects, `pip install` is fine if you keep dependencies pinned.

## Initialize And Start A Project

The explicit project-creation flow is the clearest:

```bash
mage init my_project --project-type standalone
mage start my_project --host 0.0.0.0 --port 6789
```

Then open `http://localhost:6789`.

Important CLI behavior:

- `mage start [project_path]` starts the web UI and scheduler for an existing project path.
- `mage run <project_path> <pipeline_uuid>` runs a pipeline or block from the CLI.
- `mage --help` and `mage <command> --help` are worth checking because the CLI surface is larger than the basic quickstart suggests.

Example pipeline execution:

```bash
mage run my_project my_pipeline_uuid --runtime-vars '{"env": "dev"}' --test
```

`--runtime-vars` overwrites pipeline global variables for that run, which is the clean way to inject execution-time values from automation.

## Core Usage

### Write Python blocks with decorators

The common Mage workflow is to create blocks in the UI, then edit the generated Python code inside the project:

```python
from io import StringIO

import pandas as pd
import requests

if 'data_loader' not in globals():
    from mage_ai.data_preparation.decorators import data_loader
if 'transformer' not in globals():
    from mage_ai.data_preparation.decorators import transformer
if 'test' not in globals():
    from mage_ai.data_preparation.decorators import test

@data_loader
def load_data(*args, **kwargs) -> pd.DataFrame:
    response = requests.get("https://raw.githubusercontent.com/mage-ai/datasets/master/medical.csv")
    response.raise_for_status()
    return pd.read_csv(StringIO(response.text))

@transformer
def transform(df: pd.DataFrame, *args, **kwargs) -> pd.DataFrame:
    return df.loc[:, ["patient_id", "health_score"]].head(100)

@test
def test_output(df, *args) -> None:
    assert df is not None
    assert len(df.index) > 0
```

The function decorated as `data_loader`, `transformer`, `data_exporter`, or similar is the executable entrypoint for the block. Tests run against the block output and can fail the block run.

### Use `io_config.yaml` for data-system credentials

Mage expects `io_config.yaml` in the project root. Start with the standard format:

```yaml
version: 0.1.1
default:
  POSTGRES_DBNAME: analytics
  POSTGRES_USER: analytics_user
  POSTGRES_PASSWORD: "{{ env_var('POSTGRES_PASSWORD') }}"
  POSTGRES_HOST: db.internal
  POSTGRES_PORT: 5432
```

Typical Python block pattern:

```python
from os import path

from mage_ai.io.config import ConfigFileLoader
from mage_ai.io.postgres import Postgres
from mage_ai.settings.repo import get_repo_path

if 'data_loader' not in globals():
    from mage_ai.data_preparation.decorators import data_loader

@data_loader
def load_data(**kwargs):
    config_path = path.join(get_repo_path(), "io_config.yaml")
    config_profile = kwargs.get("profile", "default")

    with Postgres.with_config(ConfigFileLoader(config_path, config_profile)) as loader:
        return loader.load("SELECT * FROM events LIMIT 100")
```

Use multiple profiles such as `development`, `staging`, and `production` rather than rewriting one shared `default` profile for every environment.

### Trigger downstream pipelines from code

Mage includes orchestration helpers for pipeline chaining:

```python
from mage_ai.orchestration.triggers.api import trigger_pipeline

if 'data_loader' not in globals():
    from mage_ai.data_preparation.decorators import data_loader

@data_loader
def trigger_downstream(*args, **kwargs):
    trigger_pipeline(
        "downstream_pipeline",
        variables={"env": "dev"},
        check_status=True,
        error_on_failure=True,
        poll_interval=30,
    )
```

This is useful when an agent needs to stitch together batch workflows without reimplementing scheduling logic outside Mage.

## Configuration And Authentication

Mage has four main variable types:

- Environment variables, referenced as `{{ env_var() }}`
- Runtime variables, referenced through `kwargs` or `{{ variables() }}`
- Block variables, accessed through `kwargs["configuration"][key]`
- Secrets, referenced as `{{ mage_secret_var() }}`

For `0.9.79`, user authentication is enabled by default.

Default owner behavior:

- If no owner exists, Mage creates one automatically.
- Default credentials are `admin@admin.com` / `admin`.
- Supported customization env vars are `DEFAULT_OWNER_EMAIL`, `DEFAULT_OWNER_PASSWORD`, and `DEFAULT_OWNER_USERNAME`.

Change those defaults immediately on any non-throwaway instance.

Useful environment variables for real deployments:

- `USER_CODE_PATH`: in Docker, point this to `/home/src/<project_name>` so Mage starts the intended project and installs its `requirements.txt`
- `MAGE_DATABASE_CONNECTION_URL`: moves orchestration state off the default local SQLite database
- `MAGE_PUBLIC_HOST`: sets the public URL used in notifications and external links
- `MAGE_ACCESS_TOKEN_EXPIRY_TIME`: customizes access-token lifetime

If you automate against Mage APIs, assume authenticated endpoints need both the Mage API key and, when auth is enabled, a valid session token created through `/api/sessions`.

## Common Pitfalls

- Do not treat `mage-ai` as a stable general-purpose SDK. The supported surface is the CLI, project layout, documented decorators, and documented integration helpers.
- `mage start my_project` and `mage run my_project ...` both depend on the correct project path. Agents often run commands from the wrong directory and then debug the wrong failure.
- Keep `io_config.yaml` in the project root. Many connector examples assume `get_repo_path()` plus that filename.
- Do not hard-code passwords in `io_config.yaml`. Use `env_var`, `mage_secret_var`, or an external secrets backend instead.
- In Docker, missing `USER_CODE_PATH` causes confusing behavior: Mage starts, but the wrong project or missing `requirements.txt` install path breaks execution later.
- The `all` extra is convenient but heavy. Prefer targeted extras to reduce dependency conflicts and startup time.
- API auth changed in newer `0.9.x`; older blog posts that assume unauthenticated local access are often wrong for `0.9.79`.

## Version-Sensitive Notes

- PyPI lists `0.9.79` published on January 21, 2026.
- `mage-ai 0.9.79` still requires Python `>=3.9`.
- Mage OSS versions `0.9.78` and above enable user authentication by default; versions `0.8.4` through `0.9.77` treated it as optional.
- PyPI release history shows several older `0.9.x` yanked releases, including `0.9.55` and `0.9.56` for scheduler/trigger bugs. Avoid broad unreviewed `0.9.*` assumptions in automation.

## Official Sources

- Docs: https://docs.mage.ai/getting-started/setup
- CLI reference: https://docs.mage.ai/development/cli-commands
- Variables: https://docs.mage.ai/development/variables/overview
- Environment variables: https://docs.mage.ai/development/variables/environment-variables
- IO config: https://docs.mage.ai/development/io_config
- IO config setup: https://docs.mage.ai/development/io_config_setup
- Authentication: https://docs.mage.ai/production/authentication/overview
- Repository: https://github.com/mage-ai/mage-ai
- PyPI: https://pypi.org/project/mage-ai/
