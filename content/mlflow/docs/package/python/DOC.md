---
name: package
description: "mlflow package guide for Python with experiment tracking, model logging, tracking servers, auth, and serving"
metadata:
  languages: "python"
  versions: "3.10.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "mlflow,python,ml,tracking,experiments,models,serving"
---

# mlflow Python Package Guide

## What This Package Is For

`mlflow` is the main Python package for MLflow. Use it when you need to:

- track experiment runs with parameters, metrics, tags, and artifacts
- log models from supported ML libraries and reload or serve them later
- point training code at a local or remote tracking server
- compare runs, register models, and move model metadata between training and deployment workflows

If a project only needs to serialize a single model locally, `joblib` or framework-native save/load may be enough. Reach for `mlflow` when run tracking, reproducibility, model packaging, or a shared tracking backend matters.

## Version-Sensitive Notes

- This entry is pinned to the version used here `3.10.1`.
- PyPI currently publishes `3.10.1` as the latest package version, so the version used here matches current upstream.
- `mlflow` `3.10.1` requires Python `>=3.10`.
- The docs URL `https://mlflow.org/docs/latest/python_api/` resolves to the current API reference path `https://mlflow.org/docs/latest/api_reference/python_api/`.
- MLflow 3 introduces first-class Logged Models and model IDs. Newer docs may use `model_id`, `models:/<model_id>`, and `search_logged_models()`. Older examples from blogs or codebases may still assume only run-artifact URIs such as `runs:/<run_id>/model`.
- `mlflow.autolog()` is useful, but support depends on the integration and the underlying ML library version. Do not assume identical behavior across frameworks.

## Install

Pin the package when you want reproducible examples and client behavior:

```bash
python -m pip install "mlflow==3.10.1"
```

If you are using `uv` or Poetry:

```bash
uv add mlflow==3.10.1
poetry add mlflow==3.10.1
```

## Recommended Setup

For local-only work, MLflow uses a local file backend by default and writes runs under `./mlruns`.

For a remote or explicit tracking backend, set the tracking URI before you start runs:

```bash
export MLFLOW_TRACKING_URI="http://127.0.0.1:5000"
```

```python
import mlflow

mlflow.set_tracking_uri("http://127.0.0.1:5000")
mlflow.set_experiment("fraud-model-dev")
```

`set_tracking_uri()` accepts a local path, `file:` URI, or `http` / `https` tracking server URI. Set it once near process startup so every run in the process goes to the intended backend.

## Core Tracking Workflow

This is the basic shape to copy when you need params, metrics, tags, and artifacts recorded for one run:

```python
from pathlib import Path
import json

import mlflow

mlflow.set_experiment("fraud-model-dev")

report_path = Path("artifacts/summary.json")
report_path.parent.mkdir(parents=True, exist_ok=True)
report_path.write_text(json.dumps({"rows": 1250, "source": "train.csv"}))

with mlflow.start_run(run_name="baseline") as run:
    mlflow.log_param("model_type", "xgboost")
    mlflow.log_param("max_depth", 6)
    mlflow.log_metric("roc_auc", 0.9134)
    mlflow.set_tag("stage", "dev")
    mlflow.log_artifact(str(report_path), artifact_path="reports")

print(run.info.run_id)
```

Important defaults:

- If you do not call `mlflow.set_experiment(...)`, runs usually land in the `Default` experiment.
- If you do not set a tracking URI, runs go to the local backend under `./mlruns`.

## Model Logging And Loading

### Logging a scikit-learn model

The quickstart pattern is to train inside a run, infer a signature, then log the model with `mlflow.sklearn.log_model(...)`.

```python
import mlflow
import mlflow.sklearn
from mlflow.models import infer_signature
from sklearn.ensemble import RandomForestRegressor
from sklearn.model_selection import train_test_split

X = training_df.drop(columns=["target"])
y = training_df["target"]

X_train, X_test, y_train, y_test = train_test_split(X, y, random_state=42)

with mlflow.start_run():
    model = RandomForestRegressor(n_estimators=200, random_state=42)
    model.fit(X_train, y_train)

    predictions = model.predict(X_test)
    signature = infer_signature(X_test, predictions)

    mlflow.log_metric("rmse", rmse(predictions, y_test))
    model_info = mlflow.sklearn.log_model(
        sk_model=model,
        name="fraud-model",
        signature=signature,
        input_example=X_test.iloc[:5],
    )

print(model_info.model_uri)
```

### Loading a logged model

Use the returned `model_uri` with the generic `pyfunc` loader when you want a prediction interface that is independent of the original training library:

```python
import mlflow.pyfunc

loaded = mlflow.pyfunc.load_model(model_info.model_uri)
predictions = loaded.predict(X_test.iloc[:5])
print(predictions)
```

For MLflow 3 style model references, you may also see URIs like `models:/<model_id>`.

## Autologging

If you want MLflow to automatically capture parameters, metrics, model artifacts, and framework-specific details for a supported library, enable autologging before training:

```python
import mlflow

mlflow.autolog()
```

Use this when you want fast instrumentation, but verify the exact integration behavior for your framework. The official docs note that compatibility varies by integration and library version.

## Tracking Server And Auth Configuration

For remote tracking, the environment-variable path is usually the least invasive:

```bash
export MLFLOW_TRACKING_URI="https://mlflow.example.com"
```

If your tracking server has authentication enabled, MLflow documents these common options:

```bash
export MLFLOW_TRACKING_USERNAME="alice"
export MLFLOW_TRACKING_PASSWORD="secret"
```

Or token-based auth:

```bash
export MLFLOW_TRACKING_TOKEN="token-value"
```

For custom TLS handling, MLflow also documents:

- `MLFLOW_TRACKING_SERVER_CERT_PATH`
- `MLFLOW_TRACKING_CLIENT_CERT_PATH`
- `MLFLOW_TRACKING_INSECURE_TLS=true`

Only set the auth and TLS variables you actually need for the target tracking server.

## Serving A Logged Model

Once you have a model URI, you can serve it locally from the CLI:

```bash
mlflow models serve -m "$MODEL_URI" -p 5000 --env-manager local
```

Then send requests to the local scoring endpoint:

```bash
curl http://127.0.0.1:5000/invocations \
  -H 'Content-Type: application/json' \
  -d '{"inputs": [{"feature_a": 1.2, "feature_b": 3.4}]}'
```

For quick local testing, this is often enough. For production deployment, treat the served model environment and dependency set as explicit deployment concerns instead of assuming the training environment automatically matches.

## Common Pitfalls

- Older MLflow examples often use run-artifact paths and older registry flows. For MLflow 3 code, expect more `model_id`-centric examples.
- `mlflow.autolog()` must be enabled before model training to capture the full run automatically.
- Logging a model without a signature or input example may be fine for simple internal use, but it makes downstream serving and validation less predictable.
- A wrong or missing `MLFLOW_TRACKING_URI` silently sends runs to the wrong backend, often the local `./mlruns` store.
- Tracking server auth variables are only relevant when the target server is configured for auth. Do not add them blindly to local setups.

## Official Source URLs

- https://mlflow.org/docs/latest/
- https://mlflow.org/docs/latest/api_reference/python_api/
- https://mlflow.org/docs/latest/ml/tracking/quickstart/
- https://mlflow.org/docs/latest/self-hosting/architecture/tracking-server/
- https://mlflow.org/docs/latest/ml/tracking/autolog/
- https://mlflow.org/docs/latest/ml/tracking/tracking-api/
- https://mlflow.org/docs/latest/ml/traditional-ml/tutorials/creating-custom-pyfunc/notebooks/basic-pyfunc/
- https://mlflow.org/docs/latest/ml/migrate/
- https://pypi.org/project/mlflow/
