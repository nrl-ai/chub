---
name: package
description: "dlt package guide for Python data-loading pipelines with destinations, sources, config injection, and incremental loading"
metadata:
  languages: "python"
  versions: "1.23.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "dlt,etl,elt,data-loading,pipeline,python"
---

# dlt Python Package Guide

## Golden Rule

Use explicit `dlt.pipeline(...)` objects for real projects, install the destination extra you actually load into, and treat `pipeline_name` as persistent state. `dlt` stores schemas, load packages, and incremental state under that pipeline name, so accidental reuse changes behavior across runs.

## What dlt Is For

`dlt` is a Python-first data loading library for extracting data, normalizing nested structures into tables, and loading them into a destination such as DuckDB, Postgres, BigQuery, filesystem-backed data lakes, and other supported targets.

The main building blocks are:

- `dlt.pipeline(...)`: runtime, state, destination, and dataset configuration
- `@dlt.resource`: one stream of records, usually one logical table
- `@dlt.source`: a group of resources that should run together
- `pipeline.run(...)`: extract, normalize, and load

## Install

Pin the package version your project expects:

```bash
python -m pip install "dlt==1.23.0"
```

For actual loading, install the destination extra too. Common examples:

```bash
python -m pip install "dlt[duckdb]==1.23.0"
python -m pip install "dlt[postgres]==1.23.0"
python -m pip install "dlt[bigquery]==1.23.0"
```

If you want the CLI scaffolding and workspace features:

```bash
python -m pip install "dlt[cli]==1.23.0"
```

To start from a generated template instead of hand-writing a pipeline:

```bash
dlt init github_api duckdb
python -m pip install -r requirements.txt
```

## Quickest Working Pipeline

For simple Python data structures, start with an explicit pipeline and load a list of dictionaries:

```python
import dlt

pipeline = dlt.pipeline(
    pipeline_name="users_ingest",
    destination="duckdb",
    dataset_name="raw_data",
)

load_info = pipeline.run(
    [
        {"id": 1, "name": "Ada"},
        {"id": 2, "name": "Grace"},
    ],
    table_name="users",
    write_disposition="append",
)

print(load_info)
```

`pipeline.run(...)` returns a `load_info` object with load package and destination details. Use this pattern when you already have Python data in memory.

## Reusable Source And Resource Pattern

Use `@dlt.resource` for one logical feed and `@dlt.source` to group related feeds. This is the pattern to prefer for API integrations and recurring jobs.

```python
import dlt
from dlt.sources.helpers import requests

@dlt.resource(primary_key="id", write_disposition="merge")
def users(
    base_url=dlt.config.value,
    api_token: str = dlt.secrets.value,
    updated_at=dlt.sources.incremental(
        "updated_at",
        initial_value="1970-01-01T00:00:00Z",
    ),
):
    response = requests.get(
        f"{base_url}/users",
        headers={"Authorization": f"Bearer {api_token}"},
        params={"updated_since": updated_at.start_value},
    )
    response.raise_for_status()
    yield response.json()["users"]

@dlt.source
def users_source():
    return users

pipeline = dlt.pipeline(
    pipeline_name="users_ingest",
    destination="duckdb",
    dataset_name="raw_data",
)

load_info = pipeline.run(users_source())
print(load_info)
```

Why this pattern matters:

- `dlt.config.value` marks non-secret config for injection
- `dlt.secrets.value` marks secrets for injection
- `dlt.sources.incremental(...)` keeps cursor state between runs
- `primary_key="id"` plus `write_disposition="merge"` makes updates/upserts practical for mutable entities

## Configuration And Secrets

`dlt` resolves configuration in this order:

1. Environment variables
2. `.dlt/secrets.toml` and `.dlt/config.toml`
3. Supported vault providers
4. Custom providers
5. Default argument values

Use `.dlt/config.toml` for non-secret settings and `.dlt/secrets.toml` for tokens, passwords, and credentials.

Example for the `users_source()` function above:

```toml
# .dlt/config.toml
[sources.users_source]
base_url = "https://api.example.com"
```

```toml
# .dlt/secrets.toml
[sources.users_source]
api_token = "replace-me"
```

Environment variable equivalents:

```bash
export SOURCES__USERS_SOURCE__BASE_URL="https://api.example.com"
export SOURCES__USERS_SOURCE__API_TOKEN="replace-me"
```

Practical notes:

- Environment variables override TOML values.
- `config.toml` can be committed; `secrets.toml` should not be.
- `dlt` also reads `~/.dlt/config.toml` and `~/.dlt/secrets.toml`, merged behind project-local values.
- If a secret is in the wrong place, `dlt` can raise instead of silently accepting it.

## Destination Setup

The destination is selected when you create the pipeline:

```python
pipeline = dlt.pipeline(
    pipeline_name="my_pipeline",
    destination="duckdb",
    dataset_name="analytics",
)
```

For local development, `duckdb` is the lowest-friction default. For warehouse targets like Postgres or BigQuery, install the matching extra and provide destination credentials under `destination.<name>.credentials`.

Typical destination credential sections:

```toml
[destination.postgres.credentials]
database = "analytics"
username = "loader"
password = "replace-me"
host = "localhost"
port = 5432
```

```toml
[destination.bigquery.credentials]
project_id = "my-project"
client_email = "svc@my-project.iam.gserviceaccount.com"
private_key = "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n"
```

## Write Disposition And Incremental Loading

The core choice is how each resource writes data:

- `append`: immutable events or logs
- `merge`: mutable entities you update over time
- `replace`: full snapshot refreshes

Example merge resource:

```python
@dlt.resource(primary_key="id", write_disposition="merge")
def accounts():
    yield [{"id": 1, "email": "a@example.com"}]
```

Example incremental cursor:

```python
@dlt.resource(primary_key="id")
def events(
    updated_at=dlt.sources.incremental(
        "updated_at",
        initial_value="1970-01-01T00:00:00Z",
    )
):
    ...
```

Rules that matter in practice:

- Use `append` for records that never change.
- Use `merge` with `primary_key` or `merge_key` for mutable records.
- `pipeline.run(..., write_disposition="replace")` forces a full refresh for that run and resets incremental state for incremental resources.
- `dlt` can deduplicate incremental results, but a stable `primary_key` makes behavior more predictable and efficient.

## State, Working Directories, And Inspection

Each pipeline keeps local state and artifacts in `~/.dlt/pipelines/<pipeline_name>` by default. Two scripts using the same `pipeline_name` will share that state.

Use these options deliberately:

- `pipeline_name`: stable identifier for stateful production pipelines
- `pipelines_dir`: separate dev, staging, and prod working directories
- `dev_mode=True`: create disposable datasets while iterating on a pipeline

Useful inspection patterns:

```python
with pipeline.sql_client() as client:
    with client.execute_query("SELECT COUNT(*) FROM users") as cursor:
        print(cursor.fetchall())
```

```bash
dlt pipeline users_ingest info
```

## Common Pitfalls

- Installing plain `dlt` without the destination extra and then expecting the destination driver to exist.
- Reusing the same `pipeline_name` across unrelated scripts and unintentionally sharing state, schema, and incremental checkpoints.
- Using `replace` when you meant `merge`, which can wipe and reload tables instead of updating them.
- Defining secrets in `config.toml` instead of `secrets.toml`.
- Assuming `dlt.sources.incremental(...)` filters the upstream API for you. It tracks cursor state, but your resource still needs to pass `start_value` to the API when the API supports server-side filtering.
- Forgetting that `pipeline.run(...)` syncs destination state and processes pending packages before extracting new data.
- Binding config to the wrong source section name. In `1.23.0`, `dlt` added a compact lookup path `sources.<name>.<key>` in addition to the older fully qualified source path. That helps many setups, but it can also unexpectedly resolve values that older versions ignored.

## Version-Sensitive Notes For 1.23.0

- PyPI shows `1.23.0` as the current stable release, published on March 6, 2026.
- PyPI metadata requires Python `>=3.9.2,<3.15`; the project description also says Python 3.14 support is still experimental because some optional extras are not available yet.
- The `1.23.0` release removed the legacy Streamlit dashboard behind `dlt pipeline show`. If older docs or blog posts still mention that command, treat them as pre-`1.23.0`.
- The `1.23.0` release added the compact source config lookup path `sources.<name>.<key>` alongside the older fully qualified source path.

## Official Sources

- Docs root: https://dlthub.com/docs/
- API reference: https://dlthub.com/docs/api_reference/
- Create-a-pipeline walkthrough: https://dlthub.com/docs/walkthroughs/create-a-pipeline
- Configuration and secrets: https://dlthub.com/docs/general-usage/credentials/setup
- Incremental loading: https://dlthub.com/docs/general-usage/incremental-loading
- Cursor-based incremental loading: https://dlthub.com/docs/general-usage/incremental/cursor
- Resource guide: https://dlthub.com/docs/general-usage/resource
- Pipeline guide: https://dlthub.com/docs/general-usage/pipeline
- PyPI package page: https://pypi.org/project/dlt/
- GitHub releases: https://github.com/dlt-hub/dlt/releases
