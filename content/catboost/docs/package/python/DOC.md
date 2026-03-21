---
name: package
description: "CatBoost Python package guide for gradient-boosted classification, regression, and ranking with categorical, text, and embedding features"
metadata:
  languages: "python"
  versions: "1.2.10"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "catboost,machine-learning,gradient-boosting,classification,regression,ranking"
---

# CatBoost Python Package Guide

## Golden Rule

Use the official `catboost` package, pass raw feature types to CatBoost instead of pre-encoding categorical columns yourself, and build train and validation datasets with `Pool` whenever you need stable feature typing, weights, ranking metadata, or early-stopping behavior.

## Install

Pin the package version your project expects:

```bash
python -m pip install "catboost==1.2.10"
```

Common alternatives:

```bash
uv add "catboost==1.2.10"
poetry add "catboost==1.2.10"
```

Optional packages that upstream calls out:

```bash
python -m pip install graphviz matplotlib plotly
```

Use those only if you need plotting or tree visualization. For custom Python metrics or objectives, upstream recommends `numba`; for CUDA-backed custom metrics or objectives, upstream also recommends `numba-cuda`, and CUDA itself must be installed on the machine.

## Initialize Data Correctly

`Pool` is CatBoost's dataset container. Use it when you need explicit feature typing, labels, sample weights, validation sets, ranking groups, or file-based datasets.

```python
import pandas as pd
from catboost import Pool

train_df = pd.DataFrame(
    [
        {"city": "SF", "device": "ios", "age": 29, "events": 14},
        {"city": "NYC", "device": "web", "age": 41, "events": 5},
        {"city": "SF", "device": "android", "age": 35, "events": 9},
    ]
)
train_labels = [1, 0, 1]

train_pool = Pool(
    data=train_df,
    label=train_labels,
    cat_features=["city", "device"],
)
```

Notes:

- `cat_features`, `text_features`, and `embedding_features` can be passed as column indices or names.
- If you use names instead of indices, give CatBoost actual feature names by using a `pandas.DataFrame`, `polars.DataFrame`, or explicit `feature_names`.
- `Pool` also accepts `weight`, `group_id`, `subgroup_id`, `pairs`, `baseline`, and file paths, so use it instead of ad hoc arrays for ranking or production training pipelines.

## Core Usage

### Binary classification

Use `eval_set` when you want best-iteration selection, overfitting detection, or metric tracking.
Assume `train_df` and `train_labels` come from the previous section, and that `valid_df` and `valid_labels` were prepared with the same schema.

```python
from catboost import CatBoostClassifier, Pool

train_pool = Pool(train_df, label=train_labels, cat_features=["city", "device"])
valid_pool = Pool(valid_df, label=valid_labels, cat_features=["city", "device"])

model = CatBoostClassifier(
    iterations=500,
    learning_rate=0.05,
    depth=8,
    loss_function="Logloss",
    eval_metric="AUC",
    random_seed=42,
)

model.fit(
    train_pool,
    eval_set=valid_pool,
    use_best_model=True,
    early_stopping_rounds=50,
    verbose=50,
)

pred_labels = model.predict(valid_pool)
pred_probs = model.predict_proba(valid_pool)[:, 1]
```

`use_best_model=True` only works when a validation dataset is provided.

### Regression

```python
from catboost import CatBoostRegressor

model = CatBoostRegressor(
    iterations=400,
    learning_rate=0.05,
    depth=8,
    loss_function="RMSE",
    random_seed=42,
)

model.fit(train_pool, eval_set=valid_pool, verbose=50)
predictions = model.predict(valid_pool)
```

### Cross-validation

Use the package-level `cv()` helper instead of writing your own fold loop when you want CatBoost-native metrics and early stopping behavior.

```python
from catboost import Pool, cv

dataset = Pool(train_df, label=train_labels, cat_features=["city", "device"])
scores = cv(
    dataset,
    params={
        "iterations": 300,
        "depth": 8,
        "loss_function": "Logloss",
        "verbose": False,
    },
    fold_count=5,
)
```

### GPU training

The released Linux and Windows packages include CUDA-enabled GPU support. A minimal GPU configuration looks like this:

```python
from catboost import CatBoostClassifier

model = CatBoostClassifier(
    iterations=1000,
    task_type="GPU",
    devices="0",
)
model.fit(train_pool, eval_set=valid_pool, verbose=False)
```

As of CatBoost `1.2.10`, upstream documents released-package support for devices with CUDA compute capability `>= 3.5`, and requires NVIDIA driver `450.80.02` or newer for CUDA-enabled training or inference.

## Feature Types And Data Containers

CatBoost supports numerical, categorical, text, and embedding features. Keep the feature declarations close to the dataset rather than burying them in preprocessing code.

When performance matters and your input pipeline is already validated, `FeaturesData` can build `Pool` objects faster than generic DataFrames or ndarrays, especially for mostly numerical datasets with some categorical columns:

```python
import numpy as np
from catboost import FeaturesData, Pool

features = FeaturesData(
    num_feature_data=np.array([[29.0, 14.0], [41.0, 5.0]], dtype=np.float32),
    cat_feature_data=np.array([["SF", "ios"], ["NYC", "web"]], dtype=object),
)
dataset = Pool(features, label=[1, 0])
```

Use `FeaturesData` only when you already know the input is correct. Upstream explicitly warns that it performs no input validation, and categorical feature values must be strings stored in an `object` array.

## Model Persistence And Export

Use CatBoost's native binary format unless you have a specific interoperability requirement.

```python
from catboost import CatBoostClassifier

model.save_model("model.cbm")

loaded = CatBoostClassifier()
loaded.load_model("model.cbm")
```

Useful export formats from `save_model()`:

- `cbm`: native CatBoost binary format
- `json`: portable JSON representation
- `python`: standalone Python code
- `cpp`: standalone C++ code
- `onnx` and `coreml`: only supported for datasets without categorical features

If you export a model with categorical features to `json`, `python`, or `cpp`, pass the training `pool=` argument to `save_model()` so CatBoost has the categorical metadata it needs.

## Configuration Notes

- Choose the estimator class for the problem shape: `CatBoostClassifier`, `CatBoostRegressor`, or `CatBoostRanker`.
- Keep the training objective explicit with `loss_function` instead of relying on defaults when the task is not obvious.
- Core tuning knobs agents usually need first: `iterations`, `learning_rate`, `depth`, `l2_leaf_reg`, `loss_function`, `eval_metric`, `random_seed`.
- For long runs, `save_snapshot=True` plus `snapshot_file` lets CatBoost resume interrupted training.
- Ranking tasks usually need `Pool(..., group_id=..., pairs=...)` or related ranking metadata, so do not treat them like plain classification arrays.

## Common Pitfalls

- Do not one-hot encode categorical columns during preprocessing. CatBoost's docs explicitly warn that this hurts both training speed and model quality.
- `use_best_model=True` requires `eval_set`; without a validation set, best-iteration truncation cannot work.
- GPU training does not support multiple validation datasets.
- If you specify categorical, text, or embedding features by name, make sure the dataset object actually carries feature names.
- `FeaturesData` is a performance tool, not a safe default. It does no input checks.
- If you train with categorical features and then export to `onnx` or `coreml`, the export will fail because those formats only support datasets without categorical features.
- For `pmml` export, categorical features must effectively be one-hot encoded during training via `one_hot_max_size`, which conflicts with the normal CatBoost guidance for standard training workflows.

## Version-Sensitive Notes For 1.2.10

- The version used here `1.2.10` matches the current PyPI release as of `2026-03-12`.
- PyPI lists `catboost 1.2.10` as released on `2026-02-18`.
- The official GitHub release `v1.2.10` was published on `2026-02-19` and only calls out JVM applier and Spark changes, not Python API changes.
- The official docs are version-light and mostly describe the active `1.2.x` behavior rather than a separate per-version Python reference. When exact runtime support matters, validate against the `1.2.10` PyPI files page and release tag instead of older blog posts.

## Official Links

- Docs root: `https://catboost.ai/en/docs/`
- Canonical docs root used for authoring: `https://catboost.ai/docs/en/`
- Python installation: `https://catboost.ai/docs/en/concepts/python-installation`
- Usage examples: `https://catboost.ai/docs/en/concepts/python-usages-examples`
- Pool reference: `https://catboost.ai/docs/en/concepts/python-reference_pool`
- `fit()` reference: `https://catboost.ai/docs/en/concepts/python-reference_catboostclassifier_fit`
- `save_model()` reference: `https://catboost.ai/docs/en/concepts/python-reference_catboost_save_model`
- Categorical features: `https://catboost.ai/docs/en/features/categorical-features`
- FeaturesData: `https://catboost.ai/docs/en/concepts/python-features-data__desc`
- PyPI release: `https://pypi.org/project/catboost/1.2.10/`
- GitHub release: `https://github.com/catboost/catboost/releases/tag/v1.2.10`
