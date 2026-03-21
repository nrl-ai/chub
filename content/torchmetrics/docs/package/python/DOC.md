---
name: package
description: "TorchMetrics package guide for Python - metric modules and functional metrics for PyTorch and Lightning"
metadata:
  languages: "python"
  versions: "1.9.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "torchmetrics,pytorch,lightning,metrics,evaluation,ml"
---

# torchmetrics Python Package Guide

## Golden Rule

Use `torchmetrics` when you need metrics that work with PyTorch tensors, streaming updates, distributed reduction, and PyTorch Lightning logging.

- Use **module metrics** such as `MulticlassAccuracy` when you need stateful accumulation across batches or epochs.
- Use **functional metrics** for one-off batch calculations where you do not need internal state.

## Install

`torchmetrics` is an add-on to PyTorch. Install the PyTorch build your environment needs first, then install `torchmetrics`.

```bash
pip install torchmetrics==1.9.0
```

`uv` and Poetry equivalents:

```bash
uv add torchmetrics==1.9.0
poetry add torchmetrics==1.9.0
```

Optional extras from the upstream package metadata:

```bash
pip install "torchmetrics[audio]==1.9.0"
pip install "torchmetrics[image]==1.9.0"
pip install "torchmetrics[text]==1.9.0"
pip install "torchmetrics[visual]==1.9.0"
pip install "torchmetrics[all]==1.9.0"
```

## Core Usage

### Functional metrics

Use the functional API when you already have a batch of predictions and targets and only need the result for that batch.

```python
import torch
from torchmetrics.functional.classification import multiclass_accuracy

preds = torch.tensor([[0.05, 0.95, 0.0], [0.1, 0.8, 0.1], [0.7, 0.2, 0.1]])
target = torch.tensor([1, 0, 0])

acc = multiclass_accuracy(preds, target, num_classes=3)
print(acc)  # tensor(0.6667)
```

### Module metrics

Use metric classes when you need streaming updates across many batches.

```python
import torch
from torchmetrics.classification import MulticlassAccuracy

metric = MulticlassAccuracy(num_classes=3)

for preds, target in [
    (torch.tensor([[0.05, 0.95, 0.0], [0.1, 0.8, 0.1]]), torch.tensor([1, 0])),
    (torch.tensor([[0.7, 0.2, 0.1]]), torch.tensor([0])),
]:
    metric.update(preds, target)

epoch_acc = metric.compute()
print(epoch_acc)

metric.reset()
```

### Metric collections

Use `MetricCollection` when you want to share updates across several metrics and compute them together.

```python
import torch
from torchmetrics import MetricCollection
from torchmetrics.classification import MulticlassAccuracy, MulticlassF1Score

metrics = MetricCollection({
    "acc": MulticlassAccuracy(num_classes=3),
    "f1": MulticlassF1Score(num_classes=3),
})

preds = torch.tensor([[0.1, 0.8, 0.1], [0.7, 0.2, 0.1]])
target = torch.tensor([1, 0])

metrics.update(preds, target)
print(metrics.compute())
metrics.reset()
```

## PyTorch Lightning Setup

Keep metric objects as module attributes so Lightning moves them with the model and resets them at the right epoch boundaries when logged correctly.

```python
import torch
from torch import nn
import lightning as L
from torchmetrics.classification import MulticlassAccuracy

class Classifier(L.LightningModule):
    def __init__(self, in_features: int, num_classes: int) -> None:
        super().__init__()
        self.net = nn.Linear(in_features, num_classes)
        self.train_acc = MulticlassAccuracy(num_classes=num_classes)
        self.val_acc = MulticlassAccuracy(num_classes=num_classes)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.net(x)

    def training_step(self, batch, _batch_idx: int) -> torch.Tensor:
        x, y = batch
        logits = self(x)
        loss = nn.functional.cross_entropy(logits, y)
        self.train_acc.update(logits, y)
        self.log("train_acc", self.train_acc, on_step=False, on_epoch=True)
        self.log("train_loss", loss, on_step=True, on_epoch=True)
        return loss

    def validation_step(self, batch, _batch_idx: int) -> None:
        x, y = batch
        logits = self(x)
        self.val_acc.update(logits, y)
        self.log("val_acc", self.val_acc, on_step=False, on_epoch=True)
```

Use separate metric instances for train, validation, and test. Do not reuse one metric object across phases unless you manually control resets.

## Configuration

`torchmetrics` has no auth or service-level configuration. Most setup is metric-specific constructor arguments.

Common knobs to set explicitly:

- `task` for generic classification APIs such as `Accuracy`
- `num_classes` or `num_labels`
- `average`, `top_k`, `threshold`, `ignore_index`
- `sync_on_compute` and distributed settings when you are running DDP

If you need device placement outside Lightning, move the metric to the same device as the model:

```python
metric = metric.to(device)
```

## Common Pitfalls

### Prefer task-specific metric classes

In current TorchMetrics releases, classification APIs are task-oriented. `Accuracy(task="binary" | "multiclass" | "multilabel", ...)` and task-specific classes such as `MulticlassAccuracy` are the safe defaults. If you omit required task-specific arguments like `num_classes`, your code will fail or behave differently than older examples.

### Understand how floating predictions are interpreted

For classification metrics, TorchMetrics will often convert floating predictions before scoring:

- binary and multilabel metrics treat floats outside `[0, 1]` as logits and apply sigmoid before thresholding
- multiclass metrics apply `argmax` across the class dimension when you pass floating tensors

This is convenient, but it can hide bugs if your tensor shape, threshold, or `top_k` setting is wrong.

### Reset stateful metrics

Metric modules accumulate internal state. Call `reset()` after `compute()` when you are managing the lifecycle yourself. Reusing a metric instance across epochs or dataset splits without a reset will leak state.

### Keep metrics inside registered modules

Plain Python lists and dicts do not move contained metrics to the correct device. Use `ModuleList`, `ModuleDict`, or `MetricCollection` if you need multiple metrics attached to a model.

### Non-scalar metrics are not direct `self.log` drop-ins

PyTorch Lightning logging expects scalar outputs for direct metric logging. Metrics such as confusion matrices, ROC curves, mean average precision, or ROUGE variants often return tensors or dictionaries and usually need manual `compute()` handling.

### Distributed test evaluation can be slightly biased

The upstream docs call out a DDP edge case: if the dataset size is not divisible by the number of ranks, PyTorch may replicate some samples so each rank has equal work. Metrics computed over those replicated samples can be slightly biased. For exact benchmarking, prefer a single device or use PyTorch's join-context approach for uneven inputs.

### Large list states can grow memory

Some metrics keep list-based states and can grow with every update. If memory usage climbs over a long epoch, inspect `metric.metric_state` and prefer metrics or settings that reduce retained history.

### State is not saved unless you opt in

Metric states are not persisted in `state_dict` by default. If you need checkpoint persistence for metric state, call `metric.persistent(True)`.

## Version-Sensitive Notes For 1.9.0

- The upstream stable docs and PyPI package currently both point at `1.9.0`, so the version used here matches the live package version as of 2026-03-12.
- The official 1.9.0 changelog notes that support for Python 3.9 was removed. Verify your runtime before pinning this version into older projects.
- The 1.9.0 changelog also notes a change to `DiceScore`: the default `average` changed from `"micro"` to `"macro"`. Set `average=` explicitly if you need stable behavior across upgrades.
- A 1.9.0 fix addressed a device mismatch issue in the base `Metric` class when states were updated across devices. If you were working around device placement bugs in older releases, re-test those workarounds before keeping them.

## Minimal Custom Metric Pattern

Subclass `Metric`, declare state with `add_state`, mutate it in `update`, and return the final tensor in `compute`.

```python
import torch
from torchmetrics import Metric

class RunningMean(Metric):
    def __init__(self) -> None:
        super().__init__()
        self.add_state("total", default=torch.tensor(0.0), dist_reduce_fx="sum")
        self.add_state("count", default=torch.tensor(0), dist_reduce_fx="sum")

    def update(self, values: torch.Tensor) -> None:
        self.total += values.sum()
        self.count += values.numel()

    def compute(self) -> torch.Tensor:
        return self.total / self.count
```

Prefer `add_state` over manual attributes so distributed reduction, reset behavior, and device moves keep working.
