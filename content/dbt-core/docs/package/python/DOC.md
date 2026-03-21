---
name: package
description: "dbt Core package guide for Python projects using the dbt CLI, project config, profiles, and adapters"
metadata:
  languages: "python"
  versions: "1.11.7"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "dbt,dbt-core,analytics-engineering,sql,etl,cli,warehouse"
---

# dbt Core Python Package Guide

## Golden Rule

Treat `dbt-core` as the local CLI engine for dbt projects, not as a standalone warehouse connector. Install `dbt-core` together with exactly one adapter package for your platform, keep project behavior in `dbt_project.yml`, keep credentials in `profiles.yml`, and run `dbt debug` before assuming your models or tests are broken.

## Install

Use a virtual environment and pin the version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "dbt-core==1.11.7"
```

For real warehouse work, also install a matching adapter package. Replace `dbt-postgres` with the adapter your project actually uses:

```bash
python -m pip install "dbt-core==1.11.7" "dbt-postgres==1.11.7"
```

Useful checks immediately after install:

```bash
dbt --version
python -m pip show dbt-core
```

Notes:

- `dbt-core` alone gives you the CLI and compiler, but you still need an adapter package to connect to a data platform.
- The dbt v1.11 upgrade guide explicitly recommends installing both `dbt-core` and adapter packages together.
- PyPI marks `1.11.0` and `1.11.1` as yanked; pin to `1.11.7` if you are targeting the current 1.11 line.

## Initialize A Project

Create a new project with the CLI:

```bash
dbt init my_project
cd my_project
```

`dbt init` scaffolds the project and prompts for an adapter/profile setup. In a typical repo, the important files and directories are:

- `dbt_project.yml`: project name, profile name, model paths, materialization defaults, vars, and package install path
- `models/`: SQL models and YAML properties
- `seeds/`: CSV seed data
- `macros/`: reusable Jinja macros
- `tests/`: singular SQL tests
- `target/`: generated artifacts such as compiled SQL and `manifest.json`

Minimal `dbt_project.yml`:

```yaml
name: my_project
version: "1.0.0"
config-version: 2

profile: my_project

model-paths: ["models"]
seed-paths: ["seeds"]
macro-paths: ["macros"]
test-paths: ["tests"]

target-path: "target"
clean-targets:
  - "target"
  - "dbt_packages"

models:
  my_project:
    +materialized: view
```

If the project must stay on the 1.11 series, make that explicit:

```yaml
require-dbt-version: [">=1.11.0", "<1.12.0"]
```

## Authentication And Profiles

dbt stores connection settings in `profiles.yml`. dbt searches for profiles in this order:

1. The `--profiles-dir` CLI flag
2. The `DBT_PROFILES_DIR` environment variable
3. The current working directory
4. `~/.dbt/`

Each profile has one or more named outputs and a `target` that selects the active output. Example:

```yaml
my_project:
  target: dev
  outputs:
    dev:
      type: postgres
      host: "{{ env_var('DBT_HOST') }}"
      user: "{{ env_var('DBT_USER') }}"
      password: "{{ env_var('DBT_ENV_SECRET_PG_PASSWORD') }}"
      port: 5432
      dbname: analytics
      schema: dbt_dev
      threads: "{{ env_var('DBT_THREADS', 4) | int }}"
```

Credential guidance:

- Keep `profiles.yml` out of version control unless it only contains placeholders.
- Use environment variables for secrets and environment-specific values.
- `DBT_ENV_SECRET_*` variables are intended for secrets. They are allowed in `profiles.yml` and `packages.yml`, scrubbed from logs, and blocked elsewhere.
- Quote the full Jinja expression in YAML, especially when casting values such as `| int`.

Quick connectivity check:

```bash
dbt debug
```

`dbt debug` validates the project file, profile file, adapter loading, and database connection.

## Core Workflow

Install packages before parsing or building:

```bash
dbt deps
```

`dbt deps` resolves dependencies from `packages.yml` or `dependencies.yml`, installs them, and writes `package-lock.yml`.

Parse and validate project structure without connecting to the warehouse:

```bash
dbt parse
```

This is the fastest way to catch many syntax, config, and graph issues early.

Build selected models only:

```bash
dbt run --select staging.customers+
```

Run models plus associated tests, snapshots, seeds, and other buildable resources:

```bash
dbt build --select marts.finance
```

Run tests only:

```bash
dbt test --select marts.finance
```

Generate documentation artifacts from the compiled manifest:

```bash
dbt docs generate
```

Useful local development loop:

```bash
dbt debug
dbt deps
dbt parse
dbt build --select my_model+
```

## Package And Project Configuration

### Dependency files

dbt supports two dependency files:

- `dependencies.yml`: preferred when you only need package declarations
- `packages.yml`: still required if you need Jinja rendering or private-package features

After `dbt deps`, dbt writes `package-lock.yml`. Commit the lock file if you want reproducible installs in CI and across developer machines.

Installed packages go to `dbt_packages/` by default. You can override that with `packages-install-path` in `dbt_project.yml`.

### Target selection and directories

These project settings are commonly relevant to automation:

- `profile`: which profile name dbt should load from `profiles.yml`
- `target-path`: where compiled SQL and artifacts are written
- `clean-targets`: paths removed by `dbt clean`
- `packages-install-path`: where `dbt deps` installs packages

If an automation step runs dbt from outside the project root, pass the project directory explicitly:

```bash
dbt build --project-dir /path/to/project --profiles-dir /path/to/profiles
```

## Practical Example

Example layout:

```text
my_project/
  dbt_project.yml
  models/
    staging/
      stg_orders.sql
      stg_orders.yml
  macros/
  packages.yml
```

Example model:

```sql
select
  id as order_id,
  customer_id,
  created_at
from {{ source('raw', 'orders') }}
where created_at >= current_date - interval '30 day'
```

Useful commands for that project:

```bash
dbt build --select stg_orders
dbt test --select stg_orders
dbt run --select path:models/staging
```

## Common Pitfalls

- Installing only `dbt-core` and forgetting the adapter package. `dbt debug` will fail before any real warehouse work happens.
- Treating `dbt run` as equivalent to `dbt build`. `run` executes models; `build` also handles tests and other buildable resources.
- Skipping `dbt deps` after editing `packages.yml` or `dependencies.yml`.
- Committing secrets in `profiles.yml` instead of using environment variables.
- Forgetting that `dbt parse` does not connect to the warehouse. A successful parse does not prove credentials or warehouse objects are valid.
- Running dbt from the wrong directory without `--project-dir` or `--profiles-dir`, then debugging the wrong project or profile.
- Repeatedly invoking dbt Core from the same long-lived Python process. The official docs call this unsafe; prefer separate CLI invocations per run.
- Assuming all adapters share the same credential fields. The profile structure is stable, but the adapter-specific keys are not.

## Version-Sensitive Notes For 1.11

- PyPI currently lists `dbt-core 1.11.7` as the latest release, uploaded on March 4, 2026.
- `dbt-core 1.11.7` requires Python `>=3.10`.
- The dbt support matrix lists v1.11 as the latest track and shows end of active support on December 18, 2026.
- The v1.11 upgrade guide recommends explicitly installing both `dbt-core` and adapter packages in the same environment.
- dbt 1.11 expands build support for UDF resources, so older 1.10-era guidance around `dbt build` coverage can be incomplete.
- If you see examples pinned to 1.11.0 or 1.11.1, treat them cautiously because those releases were yanked on PyPI.

## Official Sources

- PyPI package page: `https://pypi.org/project/dbt-core/`
- Installation overview: `https://docs.getdbt.com/docs/core/installation-overview`
- Supported versions: `https://docs.getdbt.com/docs/dbt-versions/core`
- Upgrade to v1.11: `https://docs.getdbt.com/docs/dbt-versions/core-upgrade/upgrading-to-v1.11`
- Profiles: `https://docs.getdbt.com/docs/core/connect-data-platform/profiles.yml`
- Supported data platforms: `https://docs.getdbt.com/docs/core/connect-data-platform/about-core-connections`
- `dbt_project.yml` reference: `https://docs.getdbt.com/reference/dbt_project.yml`
- `dbt debug`: `https://docs.getdbt.com/reference/commands/debug`
- `dbt deps`: `https://docs.getdbt.com/reference/commands/deps`
- `dbt parse`: `https://docs.getdbt.com/reference/commands/parse`
- `dbt run`: `https://docs.getdbt.com/reference/commands/run`
- `dbt build`: `https://docs.getdbt.com/reference/commands/build`
- `dbt test`: `https://docs.getdbt.com/reference/commands/test`
- Packages: `https://docs.getdbt.com/docs/build/packages`
- `env_var`: `https://docs.getdbt.com/reference/dbt-jinja-functions/env_var`
