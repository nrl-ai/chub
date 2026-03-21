---
name: package
description: "Optuna package guide for Python hyperparameter optimization, study storage, and parallel workflows"
metadata:
  languages: "python"
  versions: "4.7.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "optuna,python,hyperparameter-optimization,ml,study,pruning,samplers"
---

# Optuna Python Package Guide

## Golden Rule

Use `optuna` when you need programmatic hyperparameter optimization in Python, and keep the objective function focused on three things:

1. Sample parameters from `trial`.
2. Train or evaluate the model.
3. Return one numeric objective value.

Set study direction and storage explicitly instead of relying on defaults from old examples.

```bash
pip install optuna==4.7.0
```

## What Optuna Is Good At

Optuna is a define-by-run hyperparameter optimization framework. The core concepts are:

- `Study`: the optimization run and its storage-backed history
- `Trial`: one evaluation of your objective function
- Sampler: how Optuna chooses the next parameters
- Pruner: how Optuna stops weak trials early

The default sampler is `TPESampler`.

## Installation

### pip

```bash
pip install optuna
```

### uv

```bash
uv add optuna
```

### Poetry

```bash
poetry add optuna
```

### Conda

```bash
conda install -c conda-forge optuna
```

## Quick Start

This is the basic shape to copy into application code.

```python
import optuna

def objective(trial: optuna.Trial) -> float:
    learning_rate = trial.suggest_float("learning_rate", 1e-5, 1e-1, log=True)
    max_depth = trial.suggest_int("max_depth", 3, 12)
    subsample = trial.suggest_float("subsample", 0.5, 1.0)

    # Replace this block with real training and validation.
    score = (max_depth - 7) ** 2 + abs(subsample - 0.8) + learning_rate
    return score

sampler = optuna.samplers.TPESampler(seed=42)
study = optuna.create_study(
    study_name="example-study",
    direction="minimize",
    sampler=sampler,
)
study.optimize(objective, n_trials=50)

print(study.best_value)
print(study.best_params)
```

Notes:

- `create_study()` minimizes by default. Set `direction="maximize"` for metrics such as accuracy or AUC.
- Use `directions=[...]` for multi-objective studies.
- Read results from `study.best_trial`, `study.best_value`, `study.best_params`, or `study.trials_dataframe()`.

## Pruning And Intermediate Metrics

Use pruning when your training loop can report intermediate scores by epoch, boosting round, or step.

```python
import optuna

def objective(trial: optuna.Trial) -> float:
    alpha = trial.suggest_float("alpha", 1e-5, 1e-1, log=True)

    for step in range(20):
        validation_loss = train_one_step(alpha=alpha, step=step)
        trial.report(validation_loss, step=step)

        if trial.should_prune():
            raise optuna.TrialPruned()

    return validation_loss

study = optuna.create_study(
    direction="minimize",
    pruner=optuna.pruners.SuccessiveHalvingPruner(),
)
study.optimize(objective, n_trials=100)
```

Practical guidance:

- `MedianPruner` appears in many examples, but the Optuna tutorial notes that `SuccessiveHalvingPruner` and `HyperbandPruner` generally outperform it.
- Raise `optuna.TrialPruned()` when `trial.should_prune()` returns `True`.
- Framework-specific pruning callbacks may live in `optuna-integration`, not only in `optuna` itself.

## Storage, Persistence, And Resume

Choose storage based on how many workers you need and whether results must survive process restarts.

### Single process or throwaway runs

The default in-memory storage is enough:

```python
study = optuna.create_study(direction="minimize")
study.optimize(objective, n_trials=20)
```

### Local persistence with SQLite

Use SQLite when you want a simple local file you can resume later.

```python
import optuna

study = optuna.create_study(
    study_name="sqlite-study",
    storage="sqlite:///example.db",
    load_if_exists=True,
    direction="minimize",
)
study.optimize(objective, n_trials=50)
```

### File-backed shared storage with `JournalStorage`

Use journal storage for shared file-backed workflows and multi-process runs on one machine.

```python
import optuna
from optuna.storages import JournalStorage
from optuna.storages.journal import JournalFileBackend

storage = JournalStorage(JournalFileBackend("./optuna_journal_storage.log"))
study = optuna.create_study(
    study_name="journal-study",
    storage=storage,
    load_if_exists=True,
    direction="minimize",
)
study.optimize(objective, n_trials=50)
```

### Server-backed storage for multi-node runs

Use an RDB backend when multiple machines must share the same study.

```python
import os
import optuna

study = optuna.create_study(
    study_name="distributed-study",
    storage=os.environ["OPTUNA_STORAGE_URL"],
    load_if_exists=True,
    direction="minimize",
)
study.optimize(objective, n_trials=100)
```

Example environment variable:

```bash
export OPTUNA_STORAGE_URL='mysql://username:password@127.0.0.1:3306/example'
```

If you need very large multi-node throughput, Optuna also documents `GrpcStorageProxy` on top of `RDBStorage`.

## Config And Auth

Optuna does not have package-level auth settings. Configuration is mostly:

- study name
- objective direction or directions
- sampler and pruner choices
- storage backend
- storage connection string or storage object

Authentication only applies to the backing storage you choose:

- SQLite: no auth, local file path
- Journal storage: no auth, local file path
- MySQL/PostgreSQL or another RDB: credentials are embedded in the storage URL or handled by the SQLAlchemy backend you configure

For production code:

- keep storage URLs in environment variables or secrets management
- use `load_if_exists=True` when multiple runs may attach to the same study
- set a stable `study_name` for resumable or reproducible workflows

## Parallelization

Optuna supports three common parallel patterns.

### Threads in one process

```python
study.optimize(objective, n_trials=100, n_jobs=4)
```

Use this only when threading is actually useful for your workload. The Optuna FAQ notes that `n_jobs != 1` uses multi-threading and Python's GIL means CPU-bound Python code usually will not speed up much.

### Multiple processes on one machine

Use shared storage such as `JournalStorage` or `RDBStorage`, then let each process attach to the same study.

### Multiple machines

Use `RDBStorage`, and consider `GrpcStorageProxy` if you need very high concurrency.

## Reproducibility

For reproducible runs:

```python
import optuna

sampler = optuna.samplers.TPESampler(seed=10)
study = optuna.create_study(
    study_name="reproducible-study",
    direction="minimize",
    sampler=sampler,
)
study.optimize(objective, n_trials=50)
```

Important caveats from the official FAQ:

- reproducibility is much harder in parallel or distributed optimization
- `HyperbandPruner` reproducibility also depends on a fixed `study_name`
- storage does not persist the in-memory state of sampler or pruner instances

If you resume a study and need sampler continuity, persist and restore the sampler object yourself, for example with `pickle`.

## Common Pitfalls

- Forgetting `direction="maximize"` for accuracy-like metrics. The default is minimize.
- Treating `n_jobs` as multiprocessing. It is threading.
- Using `InMemoryStorage` across processes. It is not shareable across processes.
- Assuming storage resumes sampler state automatically. It does not.
- Returning `float("nan")` from the objective. Optuna marks that trial as failed.
- Letting exceptions escape the objective when you intended the study to continue. Use `catch=(YourRecoverableError,)` in `study.optimize(...)` when appropriate.
- Running large numbers of trials without memory cleanup. If memory grows, use `gc_after_trial=True` or call `gc.collect()` from a callback.
- Copying old tutorials that import removed multi-objective APIs from `optuna.multi_objective`.
- Expecting every integration to stay in core `optuna`. Many third-party integrations are migrating to `optuna-integration`.

## Version-Sensitive Notes For 4.7.0

- This guide targets Optuna `4.7.0`, the current PyPI release published on `2026-01-19`.
- Optuna `4.7.0` requires Python `3.9` or newer.
- Since Optuna `4.0`, old `optuna.multi_objective.*` APIs are removed. Use the normal `Study` and `Trial` APIs with `directions=[...]`.
- Since Optuna `4.0`, `JournalStorage` is officially supported and the reorganized `optuna.storages.journal.JournalFileBackend` path is the stable form to use.
- The `optuna.integration` docs explicitly note that third-party integrations are migrating to the separate `optuna-integration` package.

## Official Context

- Stable docs: https://optuna.readthedocs.io/en/stable/
- Installation: https://optuna.readthedocs.io/en/stable/installation.html
- Tutorial: https://optuna.readthedocs.io/en/stable/tutorial/
- API reference: https://optuna.readthedocs.io/en/stable/reference/
- FAQ: https://optuna.readthedocs.io/en/stable/faq.html
- PyPI: https://pypi.org/project/optuna/
- Repository: https://github.com/optuna/optuna
