---
name: package
description: "Great Expectations Python package guide for GX Core data validation workflows with Data Contexts, Expectations, Validation Definitions, and Checkpoints"
metadata:
  languages: "python"
  versions: "1.14.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "great-expectations,gx,data-quality,validation,expectations,checkpoints,pandas,spark"
---

# Great Expectations Python Package Guide

## Golden Rule

Use `great_expectations` as `import great_expectations as gx`, and be explicit about the Data Context mode you want. A large share of GX confusion comes from `gx.get_context()` returning a different context than you expected because it found an existing File Data Context in the current directory or under `GX_HOME`.

## Install

Pin the package version your project expects:

```bash
python -m pip install "great_expectations==1.14.0"
```

If you use pandas examples, install pandas in the same environment:

```bash
python -m pip install "pandas>=2"
```

Representative optional extras from the official docs and PyPI metadata:

```bash
python -m pip install "great_expectations[gcp]==1.14.0"
python -m pip install "great_expectations[aws_secrets]==1.14.0"
python -m pip install "great_expectations[spark]==1.14.0"
```

Notes:

- GX supports Python `3.10` through `3.13`.
- PyPI metadata normalizes some extras with hyphens while docs examples sometimes use underscores. If pip rejects one spelling, use the PyPI-normalized spelling shown on the package page.

## Choose A Data Context

GX has three important context modes:

- `file`: persistent config and metadata on disk; use this for reusable suites, checkpoints, and Data Docs
- `ephemeral`: in-memory only; use this for notebooks, experiments, and one-off validations
- `cloud`: GX Cloud-backed context using cloud credentials

The official `get_context()` behavior matters:

- no arguments: returns an existing `FileDataContext` if GX finds one in the current directory tree, otherwise returns an `EphemeralDataContext`
- `mode="file"`: requires or scaffolds a File Data Context
- `mode="ephemeral"`: always returns a fresh in-memory context
- `mode="cloud"`: returns a `CloudDataContext`

Typical setup:

```python
import great_expectations as gx

# Persistent local project config
file_context = gx.get_context(mode="file", project_root_dir=".")

# One-off exploration
ephemeral_context = gx.get_context(mode="ephemeral")

# GX Cloud
cloud_context = gx.get_context(mode="cloud")
```

## Core Usage

### Fast in-memory validation with a DataFrame

Use this for notebooks, ETL debugging, and test-time validation where you do not need persisted metadata:

```python
import great_expectations as gx
import pandas as pd

df = pd.read_csv("orders.csv")

context = gx.get_context(mode="ephemeral")
data_source = context.data_sources.add_pandas("pandas")
data_asset = data_source.add_dataframe_asset(name="orders_df")
batch_definition = data_asset.add_batch_definition_whole_dataframe("default")
batch = batch_definition.get_batch(batch_parameters={"dataframe": df})

expectation = gx.expectations.ExpectColumnValuesToBeBetween(
    column="total_amount",
    min_value=0,
    severity="warning",
)

validation_result = batch.validate(expectation)
print(validation_result)
```

Key detail: dataframe-backed assets do not persist the dataframe itself. You must pass the actual dataframe at runtime with `batch_parameters={"dataframe": df}`.

### Persist a reusable suite, validation definition, and checkpoint

Use a File Data Context when you want rerunnable validation artifacts:

```python
import great_expectations as gx

context = gx.get_context(mode="file", project_root_dir=".")

connection_string = "postgresql+psycopg2://user:password@host/dbname"
data_source = context.data_sources.add_postgres(
    "warehouse",
    connection_string=connection_string,
)
data_asset = data_source.add_table_asset(name="orders", table_name="orders")
batch_definition = data_asset.add_batch_definition_whole_table("all_rows")

suite = gx.ExpectationSuite(name="orders_suite")
suite.add_expectation(
    gx.expectations.ExpectColumnValuesToNotBeNull(
        column="order_id",
        severity="critical",
    )
)
suite.add_expectation(
    gx.expectations.ExpectColumnValuesToBeBetween(
        column="total_amount",
        min_value=0,
        severity="warning",
    )
)
suite = context.suites.add_or_update(suite)

validation_definition = gx.ValidationDefinition(
    name="orders_validation",
    data=batch_definition,
    suite=suite,
)
validation_definition = context.validation_definitions.add_or_update(
    validation_definition
)

checkpoint = gx.Checkpoint(
    name="orders_checkpoint",
    validation_definitions=[validation_definition],
)
checkpoint = context.checkpoints.add_or_update(checkpoint)

checkpoint_result = checkpoint.run()
print(checkpoint_result.describe())
```

Why this matters:

- `ExpectationSuite` groups expectations
- `ValidationDefinition` binds a suite to a specific batch definition
- `Checkpoint` is the production-oriented entry point for running one or more validation definitions

If you only need ad hoc validation, `batch.validate(expectation)` or `batch.validate(suite)` is simpler. Use checkpoints when you need repeatability and post-validation actions.

### Re-running persisted objects

When a script may be run more than once, prefer `add_or_update(...)` instead of `add(...)` for suites, validation definitions, and checkpoints. The official factories support both, but `add(...)` will fail if the named object already exists.

## Configuration And Credentials

### File Data Context configuration

A File Data Context stores GX configuration on disk in a project config rooted around `great_expectations.yml`. GX looks for it in:

1. the path in `GX_HOME`
2. the current directory
3. parent directories of the current directory

That lookup order is why implicit `gx.get_context()` calls can attach to the wrong project.

Useful patterns:

```bash
export GX_HOME="/absolute/path/to/gx-project"
```

```python
context = gx.get_context(mode="file", project_root_dir="/absolute/path/to/project")
```

GX also supports `runtime_environment` overrides when you need to inject config values at runtime instead of hard-coding them into project files.

### Secret management

GX Core supports:

- environment variables
- `config_variables.yml`
- AWS Secrets Manager
- Google Cloud Secret Manager
- Azure Key Vault

Official docs examples for secret-manager support use extras such as:

```bash
python -m pip install "great_expectations[aws_secrets]"
python -m pip install "great_expectations[gcp]"
```

Practical guidance:

- keep raw passwords, tokens, and connection strings out of `great_expectations.yml`
- store stable references in `config_variables.yml` and resolve the sensitive value via environment variables or a supported secret manager
- prefer runtime environment variables in CI/CD and containerized jobs

### GX Cloud authentication

GX Cloud uses these environment variables:

```bash
export GX_CLOUD_ACCESS_TOKEN="<user_access_token>"
export GX_CLOUD_ORGANIZATION_ID="<organization_id>"
export GX_CLOUD_WORKSPACE_ID="<workspace_id>"
```

Then create the context with:

```python
import great_expectations as gx

context = gx.get_context(mode="cloud")
```

GX Cloud also ships with a built-in `pandas_default` datasource, so simple CSV reads can start immediately:

```python
batch = context.data_sources.pandas_default.read_csv("https://example.com/data.csv")
```

## Common Pitfalls

- `gx.get_context()` is environment-sensitive. If a nearby File Data Context exists, GX will use it instead of creating an Ephemeral context.
- `mode="ephemeral"` does not persist suites, checkpoints, or other metadata after the Python process exits.
- DataFrame assets still need runtime `batch_parameters`. Defining the datasource, asset, and batch definition does not save the dataframe itself.
- Checkpoints should be added to the context before running them. The API documents `CheckpointNotAddedError` and `CheckpointNotFreshError` if you modify or run unmanaged checkpoint objects incorrectly.
- Batch parameter keys depend on the batch definition. A whole-dataframe batch definition expects `{"dataframe": df}`; whole-table definitions typically need no batch parameters.
- The package is not officially Windows-supported in the compatibility reference. Mac and Linux are the supported operating systems.
- Many third-party examples still use pre-1.0 or 0.18 APIs. Do not copy those patterns into a 1.x project.

## Version-Sensitive Notes

- This doc is pinned to version used here `1.14.0`, which PyPI lists as released on `2026-03-04`.
- PyPI already lists `1.15.0` as the latest release on `2026-03-11`, so check release drift before copying exact method names into a newly created project.
- The docs URL `https://docs.greatexpectations.io/docs/reference/` is not a stable package guide. During validation on `2026-03-12`, it resolved to a version-drifting API landing page, while guide pages were on `1.15.0` and some API pages still showed `1.14.0` or `1.11.3`.
- When the docs disagree, trust the page's own version banner plus the PyPI release history, and keep 0.18 content out of 1.x work unless you are explicitly maintaining a legacy deployment.
- The compatibility reference says GX supports library versions `>=1.0`, and that `0.18` reached end of life on `2025-10-01`.

## Source URLs

- Docs home: https://docs.greatexpectations.io/docs/
- Try GX Core: https://docs.greatexpectations.io/docs/core/introduction/try_gx/
- Create a Data Context: https://docs.greatexpectations.io/docs/core/set_up_a_gx_environment/create_a_data_context/
- `get_context()` API reference: https://docs.greatexpectations.io/docs/reference/api/data_context/data_context/context_factory/
- Pandas dataframe workflow: https://docs.greatexpectations.io/docs/core/connect_to_data/dataframes/
- ValidationDefinition API reference: https://docs.greatexpectations.io/docs/reference/api/validationdefinition_class/
- Checkpoint API reference: https://docs.greatexpectations.io/docs/reference/api/checkpoint_class/
- Compatibility reference: https://docs.greatexpectations.io/docs/help/compatibility_reference
- GX Cloud Python connection guide: https://docs.greatexpectations.io/docs/cloud/connect/connect_python/
- Secrets managers guide: https://docs.greatexpectations.io/docs/core/configure_project_settings/access_secrets_managers/
- PyPI package page: https://pypi.org/project/great-expectations/
- GitHub repository: https://github.com/great-expectations/great_expectations
