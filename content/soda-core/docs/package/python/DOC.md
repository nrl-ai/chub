---
name: package
description: "Soda Core Python package and CLI guide for local data contract verification and data quality checks"
metadata:
  languages: "python"
  versions: "4.0.7"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "soda,soda-core,data-quality,data-contracts,cli,validation"
---

# Soda Core Python Package Guide

## Golden Rule

Use the current Soda v4 docs for workflow and syntax, but treat package naming carefully:

- the Python import namespace is still `soda_core`
- the CLI command is `soda`
- current Soda v4 docs tell you to install `soda` or `soda-<data_source>` from Soda's public package index
- the public PyPI page for `soda-core` is sparse and, as of March 12, 2026, exposes `4.0.7`, not the previous `4.1.1`

If a project is already pinned to `soda-core`, keep that pin consistent. For new v4 setups, prefer the official `soda` or `soda-<data_source>` install flow from the Soda docs.

## Install

Create and activate a virtual environment first:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
```

### Recommended v4 install flow

The current Soda docs recommend installing from Soda's public package index and picking the package that matches your data source:

```bash
python -m pip install --index-url https://pypi.cloud.soda.io/simple "soda-postgres>=4,<5"
```

If you want the general umbrella package described in the v4 docs:

```bash
python -m pip install --index-url https://pypi.cloud.soda.io/simple "soda>=4,<5"
```

Notes:

- `soda` is the current v4 "umbrella" package in Soda's public index.
- `soda-<data_source>` packages such as `soda-postgres`, `soda-bigquery`, and `soda-duckdb` are the practical installs for real work.
- The docs say the public package index hosts the open-source Soda Core packages.

### If the project explicitly depends on `soda-core`

Use the public PyPI artifact that is actually available:

```bash
python -m pip install "soda-core==4.0.7"
```

Use this only when the environment is already pinned to `soda-core` or you intentionally want the package published on `pypi.org`. For new v4 automation, the official docs are written around the `soda` and `soda-<data_source>` names.

## Initialize And Configure

Generate a starter data-source config:

```bash
soda data-source create -f ds_config.yml
```

By default, the generated template is for PostgreSQL. A minimal PostgreSQL config looks like:

```yaml
type: postgres
name: my_postgres
connection:
  user: ${env.POSTGRES_USER}
  host: ${env.POSTGRES_HOST}
  port: 5432
  password: ${env.POSTGRES_PW}
  database: ${env.POSTGRES_DB}
```

Test the connection before writing or running contracts:

```bash
soda data-source test --data-source ds_config.yml
```

Soda's data-source reference expects each config file to include:

- `type`
- `name`
- `connection`

Use environment variables for credentials rather than hardcoding secrets in YAML.

## Connect To Soda Cloud

Soda Cloud is optional for local verification, but required if you want to publish contracts or push verification results.

Create the config file:

```bash
soda cloud create -f sc_config.yml
```

Then fill in the generated file with the API key material from Soda Cloud. The official docs say to create these keys from your avatar menu under `Profile > API Keys`.

A minimal config shape looks like this:

```yaml
soda_cloud:
  host: cloud.us.soda.io
  api_key_id: ${env.SODA_CLOUD_API_KEY_ID}
  api_key_secret: ${env.SODA_CLOUD_API_KEY_SECRET}
```

Use `cloud.us.soda.io` for US-region organizations and the region-specific host your Soda tenant uses.

## Core Usage

### 1. Write a contract

Soda v4 centers the CLI and Python API around data contracts. A minimal contract needs:

- a top-level `dataset:` key
- a `checks:` block, a `columns:` list, or both

Example:

```yaml
dataset: my_postgres/analytics/orders

checks:
  - schema:
  - row_count:

columns:
  - name: id
    checks:
      - missing:
  - name: status
    checks:
      - invalid:
          valid_values: ["pending", "paid", "failed"]
```

### 2. Verify a contract with the CLI

Run the contract locally with Soda Core:

```bash
soda contract verify --data-source ds_config.yml --contract contract.yaml
```

You can override variables defined in the contract at runtime:

```bash
soda contract verify \
  --data-source ds_config.yml \
  --contract contract.yaml \
  --set START_DATE=2026-03-01 \
  --set COUNTRY=US
```

If you only want a subset of checks, the CLI also supports `--check-paths` for local execution.

### 3. Verify a contract with the Python API

The current v4 Python API is contract-oriented:

```python
from soda_core import configure_logging
from soda_core.contracts import verify_contract_locally

configure_logging(verbose=True)

result = verify_contract_locally(
    data_source_file_path="ds_config.yml",
    contract_file_path="contract.yaml",
    publish=False,
)

if result.has_errors:
    raise RuntimeError(result.get_errors_str())

print(result.is_ok)
print(result.get_logs_str())
```

Important result properties from the official Python API:

- `is_ok`: true when verification has no failed checks and no execution errors
- `is_failed`: true when one or more checks failed
- `has_errors`: true when execution itself failed
- `get_logs_str()` and `get_errors_str()` for readable diagnostics

### 4. Publish or verify against Soda Cloud

Publishing requires a Soda Cloud config file and the matching permissions:

```python
from soda_core.contracts import publish_contract

result = publish_contract(
    contract_file_path="contract.yaml",
    soda_cloud_file_path="sc_config.yml",
)
```

Or verify locally and publish the results:

```python
from soda_core.contracts import verify_contract_locally

result = verify_contract_locally(
    data_source_file_path="ds_config.yml",
    contract_file_path="contract.yaml",
    soda_cloud_file_path="sc_config.yml",
    publish=True,
)
```

## In-Memory And DataFrame Workflows

One of Soda Core's practical strengths is local execution for in-memory data. The v4 docs show DuckDB-based verification for Pandas and Polars data frames.

Pandas example:

```python
import duckdb
import pandas as pd
from soda_core.contracts import verify_contract_locally
from soda_duckdb import DuckDBDataSource

df = pd.read_parquet("orders.parquet")
conn = duckdb.connect(database=":memory:")
cursor = conn.cursor()
cursor.register(view_name="orders", python_object=df)

result = verify_contract_locally(
    data_sources=[DuckDBDataSource.from_existing_cursor(cursor, name="duckdb")],
    contract_file_path="orders.contract.yaml",
)
```

Use this pattern when the dataset is already materialized in Python and you want contract verification without creating a durable warehouse table first.

## Common Pitfalls

- Do not assume `pip install soda-core` is the preferred v4 install path. Current official docs are written around `soda` and `soda-<data_source>` from `https://pypi.cloud.soda.io/simple`.
- Do not confuse the package name with the import namespace. The package name may be `soda-core` or `soda-postgres`, but the documented contract API imports come from `soda_core`.
- Run `soda data-source test --data-source ds_config.yml` before debugging contract syntax. Connection issues are a separate class of failure.
- Keep the `name` in `ds_config.yml` aligned with the Soda Cloud data source name if you want local Soda Core verification and Soda Agent / Soda Cloud results to map to the same source.
- Avoid hardcoding passwords or API keys in YAML. The official data-source docs explicitly support `${env.VAR_NAME}` interpolation.
- Contract verification is not the same as observability. The v4 docs describe Soda Core as local contract execution without observability features.
- GitHub release tags are not a reliable source for current v4 package selection. The public repo's release page still exposes `v3.5.6` as latest even though current v4 docs and PyPI artifacts exist.

## Version-Sensitive Notes

- On March 12, 2026, the official public PyPI page for `soda-core` shows `4.0.7` as the latest visible release.
- Current Soda v4 docs have shifted package naming from old v3-style `soda-core-<datasource>` guidance toward `soda` and `soda-<data_source>`.
- Current official docs state support for Python `3.9` through `3.12`, and note that Python `3.13+` has no known constraints even though `3.12` is the highest officially supported version.
- If you are maintaining an older v3-era codebase, expect older docs and examples to use `configuration.yml`, `checks.yml`, and older package naming. Re-check every command against the v4 CLI reference before copying it into production code.

## Official Sources

- Docs root: https://docs.soda.io/
- Soda Python Libraries: https://docs.soda.io/deployment-options/soda-python-libraries
- CLI reference: https://docs.soda.io/reference/cli-reference
- Python API: https://docs.soda.io/reference/python-api
- Contract language reference: https://docs.soda.io/reference/contract-language-reference
- Data source reference: https://docs.soda.io/reference/data-source-reference-for-soda-core
- PostgreSQL data source example: https://docs.soda.io/soda-v4/reference/data-source-reference-for-soda-core/postgresql
- DuckDB advanced usage: https://docs.soda.io/reference/data-source-reference-for-soda-core/duckdb/duckdb-advanced-usage
- Generate API keys: https://docs.soda.io/reference/generate-api-keys
- PyPI package page: https://pypi.org/project/soda-core/
- Source repository: https://github.com/sodadata/soda-core
