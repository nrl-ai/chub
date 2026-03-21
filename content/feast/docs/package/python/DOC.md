---
name: package
description: "feast package guide for Python feature repositories, feature views, materialization, online retrieval, and remote serving"
metadata:
  languages: "python"
  versions: "0.61.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "feast,python,feature-store,ml,mlops,features,serving"
---

# feast Python Package Guide

## Golden Rule

- Treat Feast as a feature repository plus registry and serving workflow, not as a standalone helper library.
- Start from `feast init` and keep the real source of truth in `feature_store.yaml` and your Python feature definitions.
- Use the local provider only for development and experiments. For shared or production use, move to the documented remote registry and online store backends.

## Install

Base install:

```bash
python -m pip install "feast==0.61.0"
```

Feast publishes provider and integration extras on PyPI. Install the extra that matches your actual backend stack instead of pulling everything into the environment.

Common examples:

```bash
python -m pip install "feast[aws]==0.61.0"
python -m pip install "feast[gcp]==0.61.0"
python -m pip install "feast[azure]==0.61.0"
python -m pip install "feast[snowflake,redis]==0.61.0"
```

If your repo uses a specific offline store, online store, or infra integration, keep the matching extra pinned with the base package version.

## Initialize A Feature Repository

Create a repo with the built-in scaffold:

```bash
feast init my_feature_repo
cd my_feature_repo/feature_repo
```

The generated repo centers on `feature_store.yaml`. A local development config looks like this:

```yaml
project: my_project
provider: local
registry: data/registry.db
online_store:
  type: sqlite
  path: data/online_store.db
offline_store:
  type: file
entity_key_serialization_version: 3
```

The local registry documentation explicitly says this mode is intended for experimentation only. Do not treat `registry.db` on a local filesystem as a shared production registry.

## Minimal Definitions

Feast repositories are ordinary Python files that declare entities, data sources, feature views, and optional feature services:

```python
from datetime import timedelta

from feast import Entity, FeatureService, FeatureView, Field, FileSource
from feast.types import Float32, Int64

driver = Entity(name="driver", join_keys=["driver_id"])

driver_stats_source = FileSource(
    name="driver_hourly_stats_source",
    path="data/driver_stats.parquet",
    timestamp_field="event_timestamp",
    created_timestamp_column="created",
)

driver_stats_fv = FeatureView(
    name="driver_hourly_stats",
    entities=[driver],
    ttl=timedelta(days=1),
    schema=[
        Field(name="conv_rate", dtype=Float32),
        Field(name="acc_rate", dtype=Float32),
        Field(name="avg_daily_trips", dtype=Int64),
    ],
    online=True,
    batch_source=driver_stats_source,
)

driver_activity_v1 = FeatureService(
    name="driver_activity_v1",
    features=[driver_stats_fv],
)
```

After editing definitions, update the registry:

```bash
feast apply
```

`feast apply` is not optional. Editing Python files without applying them leaves the registry and serving state stale.

## Core Usage

### Open The Store

```python
from feast import FeatureStore

store = FeatureStore(repo_path=".")
```

### Materialize Batch Data Into The Online Store

Use CLI materialization for the common case:

```bash
feast materialize-incremental now
```

Or call it from Python when you already control the process:

```python
from datetime import datetime, timezone

from feast import FeatureStore

store = FeatureStore(repo_path=".")
store.materialize_incremental(end_date=datetime.now(timezone.utc))
```

### Retrieve Historical Features

Historical retrieval uses an entity dataframe and returns a retrieval job:

```python
import pandas as pd
from feast import FeatureStore

store = FeatureStore(repo_path=".")

entity_df = pd.DataFrame.from_dict(
    {
        "driver_id": [1001, 1002],
        "event_timestamp": [
            "2024-01-01T10:00:00Z",
            "2024-01-01T11:00:00Z",
        ],
    }
)

training_df = store.get_historical_features(
    entity_df=entity_df,
    features=[
        "driver_hourly_stats:conv_rate",
        "driver_hourly_stats:acc_rate",
        "driver_hourly_stats:avg_daily_trips",
    ],
).to_df()
```

### Retrieve Online Features

Online retrieval expects entity rows and feature references or a feature service:

```python
from feast import FeatureStore

store = FeatureStore(repo_path=".")

online_response = store.get_online_features(
    features=[
        "driver_hourly_stats:conv_rate",
        "driver_hourly_stats:acc_rate",
    ],
    entity_rows=[{"driver_id": 1001}, {"driver_id": 1002}],
).to_dict()
```

With a feature service:

```python
from feast import FeatureStore

store = FeatureStore(repo_path=".")

online_response = store.get_online_features(
    features=store.get_feature_service("driver_activity_v1"),
    entity_rows=[{"driver_id": 1001}],
).to_dict()
```

If online results are empty, check materialization first. Feast will not infer or backfill the online store automatically just because the offline source has data.

## Config And Auth

### `feature_store.yaml` Is The Primary Config Surface

Project name, provider, registry backend, online store backend, offline store backend, and auth settings belong in `feature_store.yaml`. Avoid scattering the same connection details across helper modules and environment-specific Python code.

### Remote Registry

For a shared registry server, the documented client config is:

```yaml
registry:
  registry_type: remote
  path: localhost:6570
  cache_ttl_seconds: 60
```

Start the registry server with:

```bash
feast serve_registry
```

The registry server exposes:

- REST on `6570`
- gRPC on `6572`

Use the REST port in `feature_store.yaml` for the remote registry `path`. Pointing the remote registry config at `6572` is a common mistake.

### Remote Online Store And Python Feature Server

For online serving through the Python feature server:

```yaml
online_store:
  type: remote
  path: localhost:6566
  secure: false
  auth:
    type: no_auth
```

Start the server with:

```bash
feast serve
```

The default feature server port is `6566`. For TLS, the docs use:

```bash
feast serve --key server.key --cert server.crt
```

The current docs describe `NoAuthManager` as the default auth manager. If you enable OIDC or Kubernetes auth, keep the client-side `auth:` config aligned with the server-side auth manager settings. Feast does not mint credentials for you.

### Cleanup For Remote Clients

The remote online store docs recommend closing the store or using a context manager so pooled connections are cleaned up:

```python
from feast import FeatureStore

with FeatureStore(repo_path=".") as store:
    response = store.get_online_features(
        features=["driver_hourly_stats:conv_rate"],
        entity_rows=[{"driver_id": 1001}],
    ).to_dict()
```

## Common Pitfalls

### Editing Definitions Without Running `feast apply`

Your Python definitions are not live until the registry is updated.

### Missing Backend Extras

Base `feast` does not magically install every provider dependency. Missing extras usually show up as import or connection errors when you switch to cloud or warehouse backends.

### Treating The Local Registry As Production Infrastructure

The local registry docs explicitly position it for experimentation. Shared environments should use the remote registry path instead.

### Querying Online Features Before Materialization

Online retrieval only returns data that has been materialized or pushed into the online store.

### Wrong Port For Remote Services

- Remote registry config uses REST `6570`
- Registry server gRPC is `6572`
- Python feature server defaults to `6566`

### Feature References Must Match Repo Names

Strings like `"driver_hourly_stats:conv_rate"` are name-sensitive. If you rename a feature view or service, update the retrieval code and re-run `feast apply`.

## Version-Sensitive Notes

- The version used here `0.61.0` matches the currently published PyPI version on `2026-03-12`.
- The current Feast docs identify the live content as `v0.61-branch`, which aligns with the version used here for this package session.
- PyPI now requires Python `>=3.10`, so older project environments on Python `3.9` will need either a Feast downgrade or a Python upgrade.
- The current reference docs document remote registry and remote online store configs directly in `feature_store.yaml`; if you are copying older examples that assume only local sqlite-style setups, expect backend and auth config drift.
- The local registry page still warns that the local mode is for experimentation only, so production guidance should prefer remote registry/server-backed setups even though local quickstarts still work.

## Official Sources

- Docs home: `https://docs.feast.dev/`
- Quickstart: `https://docs.feast.dev/getting-started/quickstart`
- Feature views reference: `https://docs.feast.dev/reference/feature-repository/feature-view`
- Auth manager reference: `https://docs.feast.dev/reference/auth-manager`
- Local registry reference: `https://docs.feast.dev/reference/registries/local`
- Remote registry reference: `https://docs.feast.dev/reference/registries/remote`
- Python feature server reference: `https://docs.feast.dev/reference/feature-servers/python-feature-server`
- Remote online store reference: `https://docs.feast.dev/reference/feature-stores/online-stores/remote`
- PyPI package page: `https://pypi.org/project/feast/`
