---
name: package
description: "Apache Airflow workflow orchestration platform for authoring, scheduling, and monitoring Python-defined workflows"
metadata:
  languages: "python"
  versions: "3.1.8"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "airflow,apache-airflow,workflow-orchestration,dag,scheduler,etl,data-pipelines"
---

# Apache Airflow Python Package Guide

## Golden Rule

Install `apache-airflow` with the official constraints file, write DAG code against the Airflow 3 public authoring API in `airflow.sdk`, and treat `airflow standalone` plus the default Simple auth manager as local-development shortcuts rather than production deployment defaults.

## Install

Use a virtual environment and the official constraints file. Upstream explicitly warns that plain `pip install apache-airflow` can produce an unusable installation.

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip

AIRFLOW_VERSION=3.1.8
PYTHON_VERSION="$(python -c 'import sys; print(f\"{sys.version_info.major}.{sys.version_info.minor}\")')"
CONSTRAINT_URL="https://raw.githubusercontent.com/apache/airflow/constraints-${AIRFLOW_VERSION}/constraints-${PYTHON_VERSION}.txt"

python -m pip install "apache-airflow==${AIRFLOW_VERSION}" --constraint "${CONSTRAINT_URL}"
```

If you already know which extras you need, install them in the same constrained command:

```bash
python -m pip install "apache-airflow[postgres,google]==${AIRFLOW_VERSION}" --constraint "${CONSTRAINT_URL}"
```

Important installation notes:

- Officially supported installers are `pip` and `uv`; Poetry and `pip-tools` are not officially supported workflows for Airflow installation.
- Install providers separately from Airflow core after the base install. Constraints apply only to the `pip install` command that uses them.
- When adding providers later, pin `apache-airflow` in the same command so `pip` does not silently upgrade or downgrade core:

```bash
python -m pip install "apache-airflow==3.1.8" apache-airflow-providers-google
```

## Initialize A Local Instance

For local development, the quickest path is `airflow standalone`:

```bash
export AIRFLOW_HOME="$PWD/.airflow"
airflow standalone
```

What this does:

- initializes the metadata database
- creates a local admin account
- starts the API server, scheduler, dag processor, and triggerer
- writes `airflow.cfg` under `$AIRFLOW_HOME`

Then open `http://localhost:8080` and use the admin credentials printed in the terminal.

If you want to run components separately instead of `standalone`:

```bash
airflow db migrate
airflow api-server --port 8080
airflow scheduler
airflow dag-processor
airflow triggerer
```

`airflow users create` is only available when the Flask AppBuilder auth manager is enabled; it is not the default auth flow in Airflow 3.

## Write And Load DAGs

Airflow 3's stable DAG authoring interface lives in `airflow.sdk`. Put DAG files in `$AIRFLOW_HOME/dags` or set `AIRFLOW__CORE__DAGS_FOLDER` to another absolute path.

Minimal TaskFlow DAG:

```python
from __future__ import annotations

import pendulum

from airflow.sdk import dag, task

@dag(
    dag_id="hello_airflow",
    schedule=None,
    start_date=pendulum.datetime(2024, 1, 1, tz="UTC"),
    catchup=False,
    tags=["example"],
)
def hello_airflow():
    @task()
    def extract() -> dict[str, int]:
        return {"orders": 3, "returns": 1}

    @task()
    def summarize(stats: dict[str, int]) -> str:
        return f"net={stats['orders'] - stats['returns']}"

    @task()
    def emit(message: str) -> None:
        print(message)

    emit(summarize(extract()))

hello_airflow()
```

Useful local checks:

```bash
airflow dags list
airflow dags show hello_airflow
airflow tasks test hello_airflow emit 2026-03-12
```

Authoring notes for agents:

- Prefer `@dag` and `@task` for ordinary Python workflows.
- `schedule=None` is the explicit "manual only" setting in Airflow 3.
- Set `catchup` explicitly in code even though the global default in Airflow 3 is now `False`.
- Keep top-level DAG file code lightweight. Airflow repeatedly parses DAG files, so expensive imports, network calls, or database queries at module import time will hurt scheduler performance.

## Common Configuration

`airflow.cfg` is the base config file, but environment variables are the safest override mechanism:

```bash
export AIRFLOW__CORE__DAGS_FOLDER="/abs/path/to/dags"
export AIRFLOW__CORE__LOAD_EXAMPLES="False"
export AIRFLOW__CORE__EXECUTOR="LocalExecutor"
export AIRFLOW__DATABASE__SQL_ALCHEMY_CONN="postgresql+psycopg2://airflow:secret@localhost:5432/airflow"
```

Common settings agents usually need:

- `AIRFLOW__CORE__DAGS_FOLDER`: absolute path where DAG files live
- `AIRFLOW__CORE__LOAD_EXAMPLES`: disable example DAGs in real environments
- `AIRFLOW__CORE__EXECUTOR`: defaults to `LocalExecutor`; production stacks often switch to `CeleryExecutor` or `KubernetesExecutor`
- `AIRFLOW__DATABASE__SQL_ALCHEMY_CONN`: metadata DB connection string
- `AIRFLOW__CORE__FERNET_KEY`: key used to encrypt connection passwords in the metastore
- `AIRFLOW__SECRETS__BACKEND`: secrets backend class, which takes precedence over env vars and metastore lookups for Connections and Variables

## Database And Deployment Notes

SQLite is for development only:

- Airflow documents SQLite as suitable for development and says it should never be used in production.
- For real deployments, move to PostgreSQL or MySQL.
- MariaDB is explicitly unsupported.

Recommended metadata database URI examples:

```bash
export AIRFLOW__DATABASE__SQL_ALCHEMY_CONN="postgresql+psycopg2://airflow:secret@localhost:5432/airflow"
```

```bash
export AIRFLOW__DATABASE__SQL_ALCHEMY_CONN="mysql+mysqldb://airflow:secret@localhost:3306/airflow"
```

PostgreSQL-specific note:

- Use `postgresql://` or `postgresql+psycopg2://`, not `postgres://`.

## Auth, Users, And API Access

Airflow 3 defaults to `SimpleAuthManager`.

That means:

- it is intended for development and testing
- users are configured through Airflow config, not through the legacy webserver/FAB flow
- passwords are auto-generated and stored in a JSON file unless you set a custom password file path

Simple auth manager example:

```bash
export AIRFLOW__CORE__SIMPLE_AUTH_MANAGER_USERS="alice:admin,bob:viewer"
export AIRFLOW__CORE__SIMPLE_AUTH_MANAGER_PASSWORDS_FILE="$AIRFLOW_HOME/simple-auth-passwords.json"
```

Do not use this in production:

```bash
export AIRFLOW__CORE__SIMPLE_AUTH_MANAGER_ALL_ADMINS="True"
```

That setting disables authentication and treats every user as an admin.

If you need the traditional Airflow user-management flow, install the FAB auth manager provider and switch auth managers:

```bash
python -m pip install "apache-airflow==3.1.8" apache-airflow-providers-fab
export AIRFLOW__CORE__AUTH_MANAGER="airflow.providers.fab.auth_manager.FabAuthManager"
```

Then `airflow users create` becomes available.

Public API auth in Airflow 3 uses JWTs. Typical flow:

```bash
ENDPOINT_URL="http://localhost:8080"

TOKEN="$(
  curl -sS -X POST "${ENDPOINT_URL}/auth/token" \
    -H "Content-Type: application/json" \
    -d '{"username":"alice","password":"your-password"}' |
  python -c 'import json,sys; print(json.load(sys.stdin)["access_token"])'
)"

curl -sS "${ENDPOINT_URL}/api/v2/dags" \
  -H "Authorization: Bearer ${TOKEN}"
```

## Connections, Variables, And Secrets

Environment-based config is often the safest path for agents and CI.

Connection via env var:

```bash
export AIRFLOW_CONN_MY_PROD_DB='postgresql://user:pass@db.example.com:5432/app'
```

Connection via JSON env var:

```bash
export AIRFLOW_CONN_AWS_DEFAULT='{"conn_type":"aws","extra":{"region_name":"us-west-2"}}'
```

Variable via env var:

```bash
export AIRFLOW_VAR_DEPLOY_ENV="staging"
export AIRFLOW_VAR_FEATURE_FLAGS='{"use_new_loader": true}'
```

Use these APIs in DAG code:

```python
from airflow.sdk import Connection, Variable

deploy_env = Variable.get("deploy_env")
feature_flags = Variable.get("feature_flags", deserialize_json=True)
conn = Connection.get("my_prod_db")
```

Production guidance:

- Prefer a secrets backend over UI-entered secrets when possible.
- Keep `hide_sensitive_var_conn_fields=True` unless you have a strong reason to expose more detail in logs or the UI.
- Connection testing is disabled by default in UI, API, and CLI unless `core.test_connection` is enabled.

## Common Pitfalls

- Do not install with plain `pip install apache-airflow` and stop there. Use constraints.
- Do not keep using Airflow 2 import paths like `from airflow.models import DAG` or `from airflow.operators.python import PythonOperator`. In Airflow 3, DAG authoring moved to `airflow.sdk`, and standard operators now live in provider packages such as `airflow.providers.standard`.
- Do not use `airflow webserver`, `airflow db init`, or `airflow db upgrade` in new Airflow 3 automation. Use `airflow api-server` and `airflow db migrate`.
- Do not use SQLite for production metadata storage.
- Do not put slow work at module import time in DAG files.
- Do not assume manually triggered or REST-triggered DAG runs always have a `logical_date`. In Airflow 3, `logical_date` can be `None`.
- Do not access the metadata database directly from task code. Airflow 3 expects stateful interactions to go through the REST API or exposed task context.

## Version-Sensitive Notes For Airflow 3.1

- `apache-airflow 3.1.8` is the current Airflow 3 patch release on the stable docs and PyPI as of March 12, 2026.
- Airflow 3 supports Python `3.10`, `3.11`, `3.12`, and `3.13`.
- The stable DAG authoring interface is `airflow.sdk`; older internal imports are deprecated or removed.
- `SimpleAuthManager` is the default auth manager in Airflow 3.
- `catchup_by_default` is `False` in Airflow 3, and the default DAG `schedule` is `None` instead of `@once`.
- `SequentialExecutor` has been removed; `LocalExecutor` is the default executor.
- Future logical dates are no longer supported when triggering DAG runs.
- The core CLI now covers local functionality, while some service-mode remote actions moved into `airflowctl` from the `apache-airflow-client` package.
