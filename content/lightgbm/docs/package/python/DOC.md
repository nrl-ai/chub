---
name: package
description: "LightGBM Python package guide for gradient boosting models using the official LightGBM 4.6.0 docs"
metadata:
  languages: "python"
  versions: "4.6.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "lightgbm,gradient-boosting,machine-learning,scikit-learn,python"
---

# LightGBM Python Package Guide

## Golden Rule

Use the official `lightgbm` package, import it as `import lightgbm as lgb`, and prefer the scikit-learn estimators (`LGBMClassifier`, `LGBMRegressor`, `LGBMRanker`) when you need pipeline or cross-validation compatibility. Use the native `lgb.train()` API when you need tighter control over datasets, callbacks, ranking groups, or model persistence.

## Install

Pin the package version your project expects:

```bash
python -m pip install "lightgbm==4.6.0"
```

Common alternatives:

```bash
uv add "lightgbm==4.6.0"
poetry add "lightgbm==4.6.0"
```

Optional extras from PyPI:

```bash
python -m pip install "lightgbm[pandas,scikit-learn]==4.6.0"
python -m pip install "lightgbm[dask]==4.6.0"
python -m pip install "lightgbm[pyarrow]==4.6.0"
```

Installation notes:

- `numpy` and `scipy` are required core dependencies.
- The `pandas`, `scikit-learn`, `dask`, and `pyarrow` extras are only needed if your code uses those integrations.
- The PyPI page notes that the Dask integration is only tested on Linux.
- On macOS, LightGBM commonly needs OpenMP installed separately, for example `brew install libomp`.
- On Linux, an import error mentioning `libgomp.so.1` usually means the OpenMP runtime is missing from the system image.
- The install guide documents experimental GPU-enabled wheels for Windows and Linux. For CUDA-specific builds or custom build flags, follow the upstream installation guide instead of assuming the default wheel will fit the environment.

## Initialize And Prepare Data

LightGBM can train from NumPy arrays, SciPy sparse matrices, pandas DataFrames, H2O DataTables, and PyArrow Tables. For the native API, convert training data into `lgb.Dataset` objects.

```python
import lightgbm as lgb

train_data = lgb.Dataset(X_train, label=y_train)
valid_data = lgb.Dataset(X_valid, label=y_valid, reference=train_data)
```

Start with a small, explicit parameter set:

```python
params = {
    "objective": "binary",
    "metric": "binary_logloss",
    "learning_rate": 0.05,
    "num_leaves": 31,
    "feature_fraction": 0.9,
    "bagging_fraction": 0.8,
    "bagging_freq": 1,
    "verbosity": -1,
}
```

## Core Usage

### Native training API

Use `lgb.train()` when you want direct access to `Dataset`, callbacks, ranking groups, or low-level training control.

```python
import lightgbm as lgb

train_data = lgb.Dataset(X_train, label=y_train)
valid_data = lgb.Dataset(X_valid, label=y_valid, reference=train_data)

params = {
    "objective": "binary",
    "metric": "binary_logloss",
    "learning_rate": 0.05,
    "num_leaves": 31,
    "verbosity": -1,
}

booster = lgb.train(
    params,
    train_data,
    num_boost_round=300,
    valid_sets=[valid_data],
    callbacks=[
        lgb.early_stopping(stopping_rounds=20),
        lgb.log_evaluation(period=20),
    ],
)

predictions = booster.predict(X_valid, num_iteration=booster.best_iteration)
```

### Scikit-learn estimators

Use the sklearn wrappers for `Pipeline`, `GridSearchCV`, and the usual estimator API.

```python
import lightgbm as lgb

model = lgb.LGBMClassifier(
    objective="binary",
    n_estimators=300,
    learning_rate=0.05,
    num_leaves=31,
    random_state=42,
)

model.fit(
    X_train,
    y_train,
    eval_set=[(X_valid, y_valid)],
    eval_metric="binary_logloss",
    callbacks=[lgb.early_stopping(stopping_rounds=20)],
)

probabilities = model.predict_proba(X_valid)[:, 1]
labels = model.predict(X_valid)
```

### Regression

```python
import lightgbm as lgb

model = lgb.LGBMRegressor(
    objective="regression",
    n_estimators=500,
    learning_rate=0.05,
    num_leaves=63,
    random_state=42,
)

model.fit(X_train, y_train)
predictions = model.predict(X_test)
```

### Save and load a model

```python
import lightgbm as lgb

booster.save_model("model.txt")

loaded = lgb.Booster(model_file="model.txt")
predictions = loaded.predict(X_test)
```

For sklearn estimators, use the wrapped booster when you need LightGBM-native serialization:

```python
model.booster_.save_model("model.txt")
```

## Configuration Notes

- There is no service-side authentication layer. Runtime behavior is controlled by model parameters, callbacks, and local build/runtime dependencies.
- The most important training parameters to set intentionally are usually `objective`, `metric`, `learning_rate`, `num_leaves`, `max_depth`, `min_data_in_leaf`, `feature_fraction`, `bagging_fraction`, and `bagging_freq`.
- Use `random_state` on sklearn estimators or `seed`-style parameters in the native API when you need reproducible runs.
- Use `num_threads` or estimator `n_jobs` carefully in constrained CI or container environments; otherwise LightGBM will use multithreading aggressively.
- Ranking tasks require group information. With the native API, set `group` on the `Dataset`; with sklearn ranking wrappers, pass the group data explicitly to fit and evaluation.
- LightGBM supports categorical features, but the values must be represented as integer-coded categories or pandas categorical columns. Do not pass raw object/string columns and expect native categorical handling to work automatically.

## Common Pitfalls

- Old blog posts often use `early_stopping_rounds=` directly in estimator calls. In LightGBM 4.x, prefer the documented callback path such as `callbacks=[lgb.early_stopping(...)]`.
- The sklearn wrapper docs explicitly warn that `**kwargs` is not supported well in sklearn mode and may cause unexpected behavior. Pass documented estimator arguments directly.
- `num_leaves` is the main complexity control. If you also set a positive `max_depth`, keep `num_leaves <= 2^max_depth` or you can create an inconsistent tree configuration.
- The upstream tuning guide recommends increasing `min_data_in_leaf` to combat overfitting. Very small values often make models look great on the training set and unstable on validation data.
- GPU usage is not just a training parameter. It is also an installation and runtime-dependency issue. Confirm that the environment matches the documented OpenCL or CUDA setup before turning on GPU-related parameters.
- When using early stopping, predict with `best_iteration` or `best_iteration_` instead of blindly using all configured boosting rounds.
- LightGBM accepts missing values natively, but mixed dtypes and accidental object columns from pandas can still break dataset construction or categorical handling.

## Version-Sensitive Notes For 4.6.0

- PyPI and the official docs align on `4.6.0` for the version covered here, but the docs URL used the floating `latest` docs tree. This doc intentionally uses the versioned docs root `https://lightgbm.readthedocs.io/en/v4.6.0/`.
- The `4.6.0` release notes add Python `3.13` support and introduce the `bagging_by_query` parameter, so older tuning examples will not mention that option.
- The `4.5.0` line added the sklearn-compatible `feature_names_in_` attribute. If you are working with older 4.x examples that inspect feature names manually, check whether your installed version already exposes that attribute.

## Official Sources

- LightGBM docs root: `https://lightgbm.readthedocs.io/en/v4.6.0/`
- Installation guide: `https://lightgbm.readthedocs.io/en/v4.6.0/Installation-Guide.html`
- Python package intro: `https://lightgbm.readthedocs.io/en/v4.6.0/Python-Intro.html`
- Python API: `https://lightgbm.readthedocs.io/en/v4.6.0/Python-API.html`
- Parameters tuning: `https://lightgbm.readthedocs.io/en/v4.6.0/Parameters-Tuning.html`
- PyPI package page: `https://pypi.org/project/lightgbm/`
