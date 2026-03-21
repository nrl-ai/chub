---
name: package
description: "xgboost package guide for python - gradient boosting with native and scikit-learn interfaces"
metadata:
  languages: "python"
  versions: "3.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "xgboost,machine-learning,gradient-boosting,scikit-learn,gpu"
---

# xgboost Python Package Guide

## Golden Rule

Use the official `xgboost` Python package and match examples to the installed package version. For this entry, that version is `3.2.0`.

## What XGBoost Exposes In Python

The Python package has three main interfaces:

- Native interface: `xgb.DMatrix`, `xgb.train`, `xgb.Booster`
- Scikit-learn interface: `XGBClassifier`, `XGBRegressor`, `XGBRanker`
- Dask interface: distributed training when you already have a Dask cluster

For most application code, start with the scikit-learn estimators. Use the native interface when you need lower-level control, cached prediction behavior, or direct `Booster` APIs.

## Installation

```bash
pip install xgboost==3.2.0
```

If you just need the CPU-only build on supported `x86_64` Linux or Windows systems:

```bash
pip install xgboost-cpu==3.2.0
```

Additional optional extras published on PyPI include `scikit-learn`, `pandas`, `plotting`, `dask`, and `pyspark`.

## Platform And Environment Notes

- Python requirement: `>=3.10`
- Linux wheels now target `manylinux_2_28`; older distros need a newer base image or a source build
- Windows requires the Visual C++ Redistributable unless the needed runtime is already present via Visual Studio
- `pip install xgboost` ships GPU support on supported Linux and Windows wheels
- macOS wheels are CPU-only

## Verify The Install

```python
import xgboost as xgb

print(xgb.__version__)
```

## Choosing An Interface

### Use The scikit-learn API By Default

Prefer `XGBClassifier` or `XGBRegressor` when:

- you already use sklearn pipelines, metrics, or model selection
- you want `fit`, `predict`, `predict_proba`, `score`
- you want early stopping with `eval_set`

### Use The Native API When You Need Booster-Level Control

Prefer `xgb.train` with `DMatrix` when:

- you want direct control over watchlists and boosting rounds
- you need `Booster` methods like `predict`, `save_model`, `dump_model`
- you want cached prediction or lower-level integration behavior

## Quick Start: scikit-learn Interface

```python
from sklearn.datasets import load_breast_cancer
from sklearn.model_selection import train_test_split
from sklearn.metrics import roc_auc_score
import xgboost as xgb

X, y = load_breast_cancer(return_X_y=True)
X_train, X_valid, y_train, y_valid = train_test_split(
    X, y, test_size=0.2, random_state=42, stratify=y
)

model = xgb.XGBClassifier(
    n_estimators=500,
    max_depth=6,
    learning_rate=0.05,
    subsample=0.8,
    colsample_bytree=0.8,
    objective="binary:logistic",
    eval_metric="auc",
    tree_method="hist",
    device="cpu",
    random_state=42,
)

model.fit(
    X_train,
    y_train,
    eval_set=[(X_valid, y_valid)],
    verbose=False,
)

proba = model.predict_proba(X_valid)[:, 1]
print("AUC:", roc_auc_score(y_valid, proba))

model.save_model("model.json")
```

Notes:

- `tree_method="hist"` is the usual default choice for new code
- set `device="cuda"` to use a supported NVIDIA GPU
- with sklearn estimators, use `n_estimators`; with the native API, use `num_boost_round`

## Quick Start: Native Interface

```python
import numpy as np
import xgboost as xgb

rng = np.random.default_rng(42)
X_train = rng.normal(size=(1000, 20))
y_train = (X_train[:, 0] + X_train[:, 1] > 0).astype(int)
X_valid = rng.normal(size=(200, 20))
y_valid = (X_valid[:, 0] + X_valid[:, 1] > 0).astype(int)

dtrain = xgb.DMatrix(X_train, label=y_train)
dvalid = xgb.DMatrix(X_valid, label=y_valid)

params = {
    "objective": "binary:logistic",
    "eval_metric": "auc",
    "max_depth": 6,
    "eta": 0.05,
    "subsample": 0.8,
    "colsample_bytree": 0.8,
    "tree_method": "hist",
    "device": "cpu",
}

bst = xgb.train(
    params=params,
    dtrain=dtrain,
    num_boost_round=500,
    evals=[(dtrain, "train"), (dvalid, "valid")],
    early_stopping_rounds=20,
    verbose_eval=False,
)

pred = bst.predict(dvalid, iteration_range=(0, bst.best_iteration + 1))
bst.save_model("model.ubj")
```

## Data Input

`xgboost` accepts a wide range of inputs in Python. Common ones:

- `numpy.ndarray`
- `scipy.sparse` matrices
- `pandas.DataFrame`
- `cudf.DataFrame` and other GPU-backed data structures where supported

Use `DMatrix` directly for native training:

```python
dtrain = xgb.DMatrix(X, label=y, missing=float("nan"))
```

If you pass pandas data to the sklearn API, XGBoost will internally build `DMatrix` or `QuantileDMatrix` depending on the algorithm and input.

## Core Parameters To Reach For First

### Shared Across Most Tree Models

- `objective`: learning task, for example `binary:logistic`, `reg:squarederror`, `multi:softprob`
- `eval_metric`: validation metric, for example `auc`, `logloss`, `rmse`
- `max_depth`: tree depth
- `learning_rate` or `eta`: shrinkage
- `subsample`: row subsampling
- `colsample_bytree`: feature subsampling per tree
- `n_jobs` or `nthread`: CPU parallelism

### Compute And Tree Construction

- `tree_method="hist"`: the usual choice for modern CPU or GPU training
- `device="cpu"`: explicit CPU training
- `device="cuda"`: GPU training on supported CUDA environments
- `validate_parameters=True`: useful when debugging misspelled parameters

## Early Stopping

Native interface behavior:

- `early_stopping_rounds` requires at least one evaluation set
- if you pass multiple evaluation sets, early stopping uses the last one
- `xgb.train()` returns the model from the last iteration, not automatically the best checkpoint
- use `bst.best_iteration` when predicting after early stopping

Scikit-learn estimator behavior:

- `predict`, `score`, and `apply` automatically use the best iteration when early stopping is enabled
- if you need direct low-level prediction caching or full `Booster` behavior, call `model.get_booster()`

## Categorical Features

For categorical columns, do not one-hot encode by default just because the library is tree-based. XGBoost supports native categorical handling.

```python
import pandas as pd
import xgboost as xgb

X = pd.DataFrame(
    {
        "city": ["ny", "sf", "ny", "la"],
        "age": [10, 20, 30, 40],
    }
)
X["city"] = X["city"].astype("category")
y = [0, 1, 0, 1]

clf = xgb.XGBClassifier(
    tree_method="hist",
    enable_categorical=True,
    device="cpu",
)
clf.fit(X, y)
clf.save_model("categorical-model.json")
```

Rules that matter:

- convert categorical columns to pandas or cuDF `category` dtype first
- set `enable_categorical=True`
- use `tree_method="hist"` or `tree_method="approx"`
- save categorical models as `.json` or `.ubj`; older legacy formats lose categorical metadata

## Model Persistence

Use model files for long-term storage and portability:

```python
model.save_model("model.json")
loaded = xgb.XGBClassifier()
loaded.load_model("model.json")
```

For native boosters:

```python
bst.save_model("model.ubj")
restored = xgb.Booster()
restored.load_model("model.ubj")
```

Prefer `.json` or `.ubj`:

- model files are the stable, backward-compatible format
- pickled boosters are memory snapshots, not stable interchange artifacts
- do not rely on `pickle` for long-term archival across XGBoost or Python version changes

## Plotting And Inspection

Some inspection helpers need optional dependencies:

- `xgb.plot_importance(...)` requires `matplotlib`
- `xgb.plot_tree(...)` requires `matplotlib` and `graphviz`
- PyPI publishes a `plotting` extra if you want a package-managed install path for plotting helpers

## Common Pitfalls

### Wrong Installation Assumptions

- `pip install xgboost` is not the same as `xgboost-cpu`
- on older Linux systems, wheel install failures often mean the host is below `glibc 2.28`
- on Windows, missing runtime DLLs usually means the Visual C++ Redistributable is absent

### Mixing Native And sklearn Parameters

- native API uses `num_boost_round`
- sklearn estimators use `n_estimators`
- native examples often show `eta`; sklearn code often uses `learning_rate`

### Early Stopping Confusion

- `xgb.train()` keeps training state from the last round; use `best_iteration` explicitly when predicting
- sklearn wrappers already use the best iteration for prediction methods when early stopping is active

### Thread Contention

- XGBoost uses all available threads by default
- if you also run sklearn cross-validation with parallelism, set `n_jobs` deliberately to avoid thread thrashing

### Categorical Serialization Mistakes

- categorical models must be saved as `.json` or `.ubj`
- if you forget `enable_categorical=True`, XGBoost will not treat category-typed columns as native categorical features

### Booster Methods

- `Booster.update()` and `Booster.boost()` are internal-oriented APIs
- for normal training, prefer `xgb.train()` instead of manually driving booster updates

## Version-Sensitive Notes For 3.2.0

- PyPI lists `3.2.0` as the latest release on `2026-02-10`
- PyPI requires Python `>=3.10`
- the docs for `3.2.0` describe `device` as the main CPU/GPU switch; older code may still use older GPU-only parameter conventions
- starting with `2.1.0`, XGBoost uses JSON or UBJSON model IO for stable serialization; do not treat pickle snapshots as durable artifacts
- the categorical data tutorial documents Python auto-recoding support added in `3.1`, so category-handling behavior may differ from older `2.x` examples
- `3.2.0` adds new global configuration like `use_cuda_async_pool`; treat it as experimental and only use it when you know your CUDA environment and allocator behavior

## No Auth / Service Configuration

`xgboost` is a local library, not a hosted API client:

- no API keys
- no service endpoint configuration
- the main environment concerns are Python version, CPU vs GPU availability, system libraries, and optional plotting or dataframe dependencies

## Canonical Upstream Links

- Stable docs: https://xgboost.readthedocs.io/en/stable/
- Installation guide: https://xgboost.readthedocs.io/en/stable/install.html
- Python package intro: https://xgboost.readthedocs.io/en/stable/python/python_intro.html
- sklearn estimator guide: https://xgboost.readthedocs.io/en/stable/python/sklearn_estimator.html
- Parameters: https://xgboost.readthedocs.io/en/stable/parameter.html
- Categorical tutorial: https://xgboost.readthedocs.io/en/stable/tutorials/categorical.html
- Model IO: https://xgboost.readthedocs.io/en/stable/tutorials/saving_model.html
- Release notes: https://xgboost.readthedocs.io/en/stable/changes/
- PyPI: https://pypi.org/project/xgboost/
