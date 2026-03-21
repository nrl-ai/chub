---
name: package
description: "scikit-learn package guide for Python - official machine learning toolkit usage and setup"
metadata:
  languages: "python"
  versions: "1.8.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "scikit-learn,sklearn,machine-learning,data-science,python"
---

# scikit-learn Python Package Guide

## What It Is

`scikit-learn` is the standard Python package for classical machine learning: preprocessing, model selection, pipelines, metrics, and estimators for classification, regression, clustering, and dimensionality reduction.

Agent reminder:

- PyPI package name: `scikit-learn`
- Import namespace: `sklearn`
- This package does not do authentication or remote API calls. Most setup work is dependency management, data preparation, and estimator configuration.

## Install

Use the package version your project expects. For the version covered here:

```bash
python -m pip install "scikit-learn==1.8.0"
```

Upgrade in place:

```bash
python -m pip install -U scikit-learn
```

Create a fresh conda environment:

```bash
conda create -n sklearn-env -c conda-forge scikit-learn
conda activate sklearn-env
```

`scikit-learn` `1.8.0` requires Python `>=3.11`. The install docs also list `numpy`, `scipy`, `joblib`, and `threadpoolctl` as core dependencies.

## Initialize And Verify

```python
import sklearn
from sklearn import show_versions

print(sklearn.__version__)
show_versions()
```

`show_versions()` is useful when builds fail because of binary or BLAS/OpenMP issues.

## Core Workflow

### 1. Split data before training

```python
from sklearn.model_selection import train_test_split

X_train, X_test, y_train, y_test = train_test_split(
    X,
    y,
    test_size=0.2,
    random_state=42,
    stratify=y,
)
```

Set `random_state` when you need reproducible results across runs.

### 2. Build preprocessing and model steps in one pipeline

Use a `Pipeline` so transforms learned on training data are applied consistently at predict time.

```python
from sklearn.compose import ColumnTransformer
from sklearn.impute import SimpleImputer
from sklearn.linear_model import LogisticRegression
from sklearn.pipeline import Pipeline
from sklearn.preprocessing import OneHotEncoder, StandardScaler

numeric_features = ["age", "income"]
categorical_features = ["state", "segment"]

preprocess = ColumnTransformer(
    transformers=[
        (
            "num",
            Pipeline(
                steps=[
                    ("imputer", SimpleImputer(strategy="median")),
                    ("scaler", StandardScaler()),
                ]
            ),
            numeric_features,
        ),
        (
            "cat",
            Pipeline(
                steps=[
                    ("imputer", SimpleImputer(strategy="most_frequent")),
                    ("encoder", OneHotEncoder(handle_unknown="ignore")),
                ]
            ),
            categorical_features,
        ),
    ]
)

model = Pipeline(
    steps=[
        ("preprocess", preprocess),
        ("classifier", LogisticRegression(max_iter=1000)),
    ]
)

model.fit(X_train, y_train)
predictions = model.predict(X_test)
probabilities = model.predict_proba(X_test)
```

### 3. Evaluate with metrics that match the task

```python
from sklearn.metrics import accuracy_score, classification_report, roc_auc_score

print("accuracy:", accuracy_score(y_test, predictions))
print("roc_auc:", roc_auc_score(y_test, probabilities[:, 1]))
print(classification_report(y_test, predictions))
```

For regression, switch to regression metrics such as `mean_squared_error`, `mean_absolute_error`, or `r2_score`.

### 4. Tune hyperparameters with cross-validation

```python
from sklearn.model_selection import GridSearchCV

search = GridSearchCV(
    estimator=model,
    param_grid={
        "classifier__C": [0.1, 1.0, 10.0],
        "classifier__solver": ["lbfgs"],
    },
    cv=5,
    n_jobs=-1,
    scoring="roc_auc",
    refit=True,
)

search.fit(X_train, y_train)

best_model = search.best_estimator_
best_predictions = best_model.predict(X_test)
```

Use step-qualified parameter names like `classifier__C` when tuning inside a `Pipeline`.

## Common Patterns

### Tabular classification or regression

- Start with `Pipeline` plus `ColumnTransformer`.
- For linear models, scale numeric columns.
- For tree-based models, scaling is usually unnecessary but train/test splitting and consistent preprocessing still matter.

### Text features

```python
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.pipeline import Pipeline
from sklearn.svm import LinearSVC

text_model = Pipeline(
    steps=[
        ("tfidf", TfidfVectorizer()),
        ("classifier", LinearSVC()),
    ]
)

text_model.fit(train_texts, train_labels)
predicted_labels = text_model.predict(test_texts)
```

### Unsupervised learning

```python
from sklearn.cluster import KMeans

kmeans = KMeans(n_clusters=8, random_state=42, n_init="auto")
labels = kmeans.fit_predict(X)
```

For clustering and dimensionality reduction, check whether scaling should happen before fitting.

## Configuration And Performance

## Authentication

None. `scikit-learn` is a local Python library.

## Global configuration

Use `sklearn.set_config()` for process-wide defaults and `sklearn.config_context()` for temporary overrides.

```python
from sklearn import config_context, set_config

set_config(
    transform_output="pandas",
    enable_metadata_routing=False,
)

with config_context(assume_finite=True):
    model.fit(X_train, y_train)
```

Useful options from the official API docs:

- `transform_output`: return transformer output in `"default"`, `"pandas"`, or `"polars"` form when supported.
- `working_memory`: cap temporary memory usage for some algorithms.
- `enable_metadata_routing`: controls whether metadata-routing behavior is enabled for estimators that support it.

## Parallelism

Many estimators and tools expose `n_jobs`. Use:

- `n_jobs=1` for predictable debugging
- `n_jobs=-1` to use all CPUs when the estimator supports it

`n_jobs` controls joblib-level parallelism. Native libraries used under NumPy, SciPy, or OpenMP may still use their own threads. If you see CPU oversubscription, explicitly limit threads with environment variables before Python starts:

```bash
OMP_NUM_THREADS=1
OPENBLAS_NUM_THREADS=1
MKL_NUM_THREADS=1
python train.py
```

## Model Persistence

For trusted local artifacts, `joblib.dump()` and `joblib.load()` are common:

```python
import joblib

joblib.dump(best_model, "model.joblib")
loaded_model = joblib.load("model.joblib")
```

Use `skops.io` when you need a more inspectable and safer format than raw pickle-based persistence. Do not assume pickle or joblib artifacts are safe to load from untrusted sources.

## Common Pitfalls

## Inconsistent preprocessing

Do not fit transforms on training data and then forget to apply the same transforms to test or production data. Put preprocessing and the estimator in one `Pipeline`.

## Data leakage

Do not fit scalers, imputers, feature selectors, or target-aware preprocessing on the full dataset before splitting. Split first, then fit the pipeline on training data only.

## Wrong metric or split strategy

- Classification with class imbalance often needs stratified splitting and metrics beyond raw accuracy.
- Time series data should not use random shuffles unless the task genuinely allows it.

## Sparse and dense mismatches

Some preprocessing steps return sparse matrices while some estimators expect dense inputs. Check estimator requirements before forcing `.toarray()`, which can explode memory usage.

## Version-Sensitive Notes For 1.8.0

- `1.8.0` is the version currently shown on PyPI and on the main stable scikit-learn docs used for this guide.
- `scikit-learn` `1.8.0` requires Python `3.11` or newer.
- Serialized models are not a stable interchange format across scikit-learn versions. Rebuild or re-export artifacts when upgrading environments.
- The package import remains `sklearn`, not `scikit_learn`.
- Some config behavior, metadata routing support, and estimator defaults can change across releases. Recheck the official API page for the exact estimator you are using before copying older snippets.

## Official Sources

- Main docs: https://scikit-learn.org/stable/
- Install guide: https://scikit-learn.org/stable/install.html
- Getting started: https://scikit-learn.org/stable/getting_started.html
- Common pitfalls: https://scikit-learn.org/stable/common_pitfalls.html
- API reference: https://scikit-learn.org/stable/api/
- `set_config` API: https://scikit-learn.org/stable/modules/generated/sklearn.set_config.html
- Parallelism guide: https://scikit-learn.org/stable/computing/parallelism.html
- Model persistence: https://scikit-learn.org/stable/model_persistence.html
- PyPI package page: https://pypi.org/project/scikit-learn/
