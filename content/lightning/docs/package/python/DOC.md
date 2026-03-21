---
name: package
description: "Lightning package guide for PyTorch training loops, Trainer orchestration, callbacks, checkpointing, and multi-device execution"
metadata:
  languages: "python"
  versions: "2.6.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "lightning,pytorch,deep-learning,training,trainer,distributed"
---

# Lightning Python Package Guide

## Golden Rule

Install `lightning`, import the PyTorch API from `lightning.pytorch`, and treat older `pytorch_lightning` examples as legacy unless the project is already pinned to that older package layout.

As of March 12, 2026, both PyPI and the Lightning PyTorch stable docs point to `lightning 2.6.1`.

## Install

If you need a specific CUDA, ROCm, or CPU-only PyTorch build, install `torch` the way PyTorch recommends first, then install Lightning on top.

Pin the package version your project expects:

```bash
python -m pip install "lightning==2.6.1"
```

Common alternatives:

```bash
uv add "lightning==2.6.1"
poetry add "lightning==2.6.1"
```

If you also need PyTorch in the same step and you are using the default wheel channel:

```bash
python -m pip install "torch" "lightning==2.6.1"
```

## Initialize A Minimal Project

Lightning wraps a normal PyTorch model inside a `LightningModule` and delegates orchestration to `Trainer`.

```python
import lightning as L
import torch
from torch import nn
from torch.utils.data import DataLoader, TensorDataset

class Classifier(L.LightningModule):
    def __init__(self, input_dim: int = 32, num_classes: int = 2) -> None:
        super().__init__()
        self.save_hyperparameters()
        self.model = nn.Sequential(
            nn.Linear(input_dim, 64),
            nn.ReLU(),
            nn.Linear(64, num_classes),
        )
        self.loss_fn = nn.CrossEntropyLoss()

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.model(x)

    def training_step(self, batch, batch_idx: int) -> torch.Tensor:
        x, y = batch
        logits = self(x)
        loss = self.loss_fn(logits, y)
        self.log("train_loss", loss, prog_bar=True, on_step=True, on_epoch=True)
        return loss

    def validation_step(self, batch, batch_idx: int) -> None:
        x, y = batch
        logits = self(x)
        loss = self.loss_fn(logits, y)
        preds = logits.argmax(dim=1)
        acc = (preds == y).float().mean()
        self.log("val_loss", loss, prog_bar=True)
        self.log("val_acc", acc, prog_bar=True)

    def configure_optimizers(self):
        return torch.optim.Adam(self.parameters(), lr=1e-3)

def make_loader(num_rows: int) -> DataLoader:
    x = torch.randn(num_rows, 32)
    y = torch.randint(0, 2, (num_rows,))
    return DataLoader(TensorDataset(x, y), batch_size=64, shuffle=True)

model = Classifier()
train_loader = make_loader(2048)
val_loader = make_loader(512)

trainer = L.Trainer(
    max_epochs=5,
    accelerator="auto",
    devices="auto",
    deterministic=True,
)
trainer.fit(model, train_dataloaders=train_loader, val_dataloaders=val_loader)
```

Use `self.log(...)` inside module hooks instead of printing metrics manually. Lightning routes those values to the progress bar, logger, callbacks, and checkpoints.

## Use A LightningDataModule For Reusable Data Setup

For anything beyond a toy script, put data preparation and loader creation in a `LightningDataModule`.

```python
import lightning as L
from torch.utils.data import DataLoader, random_split
from torchvision import transforms
from torchvision.datasets import MNIST

class MNISTDataModule(L.LightningDataModule):
    def __init__(self, data_dir: str = "./data", batch_size: int = 64) -> None:
        super().__init__()
        self.data_dir = data_dir
        self.batch_size = batch_size
        self.transform = transforms.ToTensor()

    def prepare_data(self) -> None:
        MNIST(self.data_dir, train=True, download=True)
        MNIST(self.data_dir, train=False, download=True)

    def setup(self, stage: str | None = None) -> None:
        if stage in ("fit", None):
            full = MNIST(self.data_dir, train=True, transform=self.transform)
            self.mnist_train, self.mnist_val = random_split(full, [55_000, 5_000])
        if stage in ("test", None):
            self.mnist_test = MNIST(self.data_dir, train=False, transform=self.transform)

    def train_dataloader(self) -> DataLoader:
        return DataLoader(self.mnist_train, batch_size=self.batch_size, shuffle=True)

    def val_dataloader(self) -> DataLoader:
        return DataLoader(self.mnist_val, batch_size=self.batch_size)

    def test_dataloader(self) -> DataLoader:
        return DataLoader(self.mnist_test, batch_size=self.batch_size)
```

`prepare_data()` is for download and disk-side work. `setup()` is for assigning state used by each process, such as train/val/test datasets.

## Trainer Configuration That Matters In Real Projects

The main control surface is `Trainer(...)`.

Useful defaults to set explicitly:

```python
from lightning.pytorch import Trainer, seed_everything
from lightning.pytorch.callbacks import ModelCheckpoint

seed_everything(42, workers=True)

trainer = Trainer(
    accelerator="auto",
    devices=1,
    max_epochs=10,
    default_root_dir="artifacts/lightning",
    log_every_n_steps=10,
    deterministic=True,
    callbacks=[
        ModelCheckpoint(
            monitor="val_loss",
            mode="min",
            save_top_k=1,
            filename="epoch{epoch:02d}-val_loss{val_loss:.3f}",
        )
    ],
)
```

Practical notes:

- `accelerator="auto"` and `devices="auto"` let Lightning pick CPU, GPU, MPS, TPU, or other supported hardware automatically.
- If you want exactly one device, set `devices=1` instead of relying on auto-detection.
- `default_root_dir` controls where checkpoints and logs land when you do not pass logger or checkpoint paths explicitly.
- `seed_everything(42, workers=True)` is the standard reproducibility starting point for dataloader workers and training state.
- `fast_dev_run=True` is the quickest smoke test for hook wiring, dataloaders, and shape mismatches.

## Checkpointing, Resume, And Inference

Lightning checkpointing is built around `Trainer.fit(...)` and `LightningModule.load_from_checkpoint(...)`.

```python
from lightning.pytorch import Trainer

trainer = Trainer(max_epochs=10)
trainer.fit(model, datamodule=datamodule, ckpt_path="last")
```

Load a model later:

```python
model = Classifier.load_from_checkpoint("artifacts/lightning/checkpoints/best.ckpt")
```

Run validation, testing, or prediction from a checkpoint:

```python
trainer.validate(model=model, datamodule=datamodule, ckpt_path="best")
trainer.test(model=model, datamodule=datamodule, ckpt_path="best")
predictions = trainer.predict(model=model, datamodule=datamodule, ckpt_path="best")
```

By default, validation, test, and predict run under inference mode. If your evaluation code needs gradients, set `inference_mode=False` on the `Trainer`.

## Config, Secrets, And External Integrations

Lightning itself does not require API keys or service authentication.

Configuration usually comes from:

- your experiment config system or CLI
- environment variables for dataset locations, checkpoint roots, and logger settings
- credentials required by integrations such as Weights & Biases, MLflow, cloud storage, or remote artifact stores

A simple pattern is to keep Lightning config separate from model code:

```python
import os

CHECKPOINT_ROOT = os.getenv("LIGHTNING_ROOT_DIR", "./artifacts/lightning")
MAX_EPOCHS = int(os.getenv("LIGHTNING_MAX_EPOCHS", "10"))
USE_CUDA = os.getenv("LIGHTNING_USE_CUDA", "1") == "1"
```

Then feed those values into `Trainer(...)` rather than hard-coding infrastructure-specific paths.

## Common Pitfalls

- Do not copy old `pytorch_lightning` or `Trainer(gpus=...)` snippets into a `2.6.1` project. Use `lightning` or `lightning.pytorch`, plus `accelerator=` and `devices=`.
- Do not put dataset objects or process-local state in `prepare_data()`. Use `setup()` for that.
- Do not manually add a `DistributedSampler` unless you know why. Lightning injects one automatically for distributed strategies unless you disable `use_distributed_sampler`.
- Do not assume `Trainer()` means single-GPU training. Auto device selection can use all available GPUs on a machine.
- `save_hyperparameters()` is convenient, but exclude non-serializable objects such as dataset handles, file descriptors, or large live clients.
- `fast_dev_run=True` is for debugging; it intentionally changes normal training behavior and suppresses some side effects such as full logging and checkpoint flow.
- Keep `default_root_dir` explicit in scripts and jobs. Otherwise artifacts often end up in an unexpected working directory.
- If your project mixes raw PyTorch and Lightning hooks, keep the ownership boundary clean. Let Lightning drive the loop and let your module define step logic.

## Version-Sensitive Notes For `2.6.1`

- The stable Lightning PyTorch docs and PyPI both resolve to `2.6.1` as of `2026-03-12`.
- PyPI metadata for `lightning 2.6.1` requires Python `>=3.10`.
- Lightning's versioning policy says patch releases are for backward-compatible bug fixes, while minor releases can include backward-incompatible changes with notice. If you need stable training behavior, pin the minor line, for example `lightning~=2.6.1`.
- The current upgrade guide still matters for older codebases: many `1.x` and early `2.0` examples on the web use removed or renamed Trainer arguments and old import paths.

## Official Sources

- Lightning docs root: https://lightning.ai/docs/pytorch/stable/
- Installation guide: https://lightning.ai/docs/pytorch/stable/starter/installation.html
- LightningModule guide: https://lightning.ai/docs/pytorch/stable/common/lightning_module.html
- DataModule guide: https://lightning.ai/docs/pytorch/stable/data/datamodule.html
- Trainer guide: https://lightning.ai/docs/pytorch/stable/common/trainer.html
- GPU basics: https://lightning.ai/docs/pytorch/stable/accelerators/gpu_basic.html
- Upgrade guide: https://lightning.ai/docs/pytorch/stable/upgrade/migration_guide.html
- Versioning policy: https://lightning.ai/docs/pytorch/stable/versioning.html
- PyPI package page: https://pypi.org/project/lightning/
