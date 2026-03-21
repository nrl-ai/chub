---
name: package
description: "PyTorch torch package guide for tensors, autograd, modules, device placement, and model save/load workflows"
metadata:
  languages: "python"
  versions: "2.10.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "torch,pytorch,python,deep-learning,tensors,autograd,nn,training"
---

# PyTorch `torch` Python Package Guide

## Golden Rule

Use the official `torch` package, import it as `import torch`, choose the execution device explicitly, and prefer `state_dict` save/load flows over pickled whole-model checkpoints. For accelerator installs, use the official PyTorch install selector instead of guessing wheel indexes from older blog posts.

## Install

Base install with an exact pin:

```bash
python -m pip install "torch==2.10.0"
```

Common alternatives:

```bash
uv add "torch==2.10.0"
poetry add "torch==2.10.0"
```

Important install note:

- CPU-only and many local development flows can start from the pinned PyPI package above.
- CUDA or ROCm installs may require an official PyTorch wheel index and a command generated from `https://pytorch.org/get-started/locally/`.
- On Apple Silicon, check `mps` availability after install instead of assuming GPU support is active.

Verify the runtime you actually got:

```python
import torch

print(torch.__version__)
print("cuda:", torch.cuda.is_available())
print("mps:", torch.backends.mps.is_available())
```

## Initialize And Set Up

Pick a device once and move both models and tensors onto it:

```python
import torch

if torch.cuda.is_available():
    device = torch.device("cuda")
elif torch.backends.mps.is_available():
    device = torch.device("mps")
else:
    device = torch.device("cpu")

torch.manual_seed(0)
```

Create tensors directly on the target device when practical:

```python
x = torch.randn(4, 3, device=device)
y = torch.zeros(4, 3, device=device)
```

If you already have a tensor or module on CPU, move it explicitly:

```python
x = x.to(device)
model = model.to(device)
```

## Core Usage

### Tensors And Basic Ops

```python
import torch

x = torch.tensor([[1.0, 2.0], [3.0, 4.0]])
y = torch.tensor([[5.0, 6.0], [7.0, 8.0]])

print(x + y)
print(x @ y)
print(x.shape, x.dtype)
```

### Autograd

```python
import torch

x = torch.tensor([2.0], requires_grad=True)
y = x**2 + 3 * x
y.backward()

print(x.grad)  # dy/dx = 2x + 3
```

### Define And Train A Small Model

```python
import torch
from torch import nn
from torch.utils.data import DataLoader, TensorDataset

if torch.cuda.is_available():
    device = torch.device("cuda")
elif torch.backends.mps.is_available():
    device = torch.device("mps")
else:
    device = torch.device("cpu")

features = torch.randn(256, 10)
labels = torch.randint(0, 2, (256,))
loader = DataLoader(TensorDataset(features, labels), batch_size=32, shuffle=True)

model = nn.Sequential(
    nn.Linear(10, 32),
    nn.ReLU(),
    nn.Linear(32, 2),
).to(device)

loss_fn = nn.CrossEntropyLoss()
optimizer = torch.optim.Adam(model.parameters(), lr=1e-3)

model.train()
for batch_x, batch_y in loader:
    batch_x = batch_x.to(device)
    batch_y = batch_y.to(device)

    optimizer.zero_grad()
    logits = model(batch_x)
    loss = loss_fn(logits, batch_y)
    loss.backward()
    optimizer.step()
```

### Inference

Use `eval()` plus `torch.inference_mode()` for prediction code:

```python
model.eval()

with torch.inference_mode():
    sample = torch.randn(1, 10, device=device)
    probs = torch.softmax(model(sample), dim=-1)
    predicted_class = probs.argmax(dim=-1).item()
```

### Save And Load Weights

Prefer `state_dict` checkpoints:

```python
import torch
from torch import nn

model = nn.Linear(10, 2)
torch.save(model.state_dict(), "model.pt")

restored = nn.Linear(10, 2)
state_dict = torch.load("model.pt", map_location="cpu", weights_only=True)
restored.load_state_dict(state_dict)
restored.eval()
```

## Configuration Notes

There is no service authentication layer in `torch`. The main configuration decisions are runtime and execution behavior:

- Device selection: `cpu`, `cuda`, or `mps`
- Numeric precision and dtype choices
- Batch size and `DataLoader` worker count
- Checkpoint device mapping during `torch.load(...)`

Practical defaults:

- Keep device selection explicit with `torch.device(...)` and `.to(device)`.
- Use `map_location="cpu"` when loading checkpoints on machines that may not have the original accelerator.
- Start with small `num_workers` values in `DataLoader`; worker behavior differs by platform and environment.
- Only introduce `torch.compile(...)` after the model works correctly in eager mode.

## Common Pitfalls

- Model parameters and input tensors must be on the same device. Mixed-device mistakes are one of the most common `RuntimeError` causes.
- Move the model to GPU before creating the optimizer if the optimizer should track GPU parameters.
- `model.train()` and `model.eval()` are not interchangeable. Dropout and batch normalization behave differently.
- Use `torch.inference_mode()` or `torch.no_grad()` for inference; otherwise PyTorch keeps autograd bookkeeping you do not need.
- Prefer `DistributedDataParallel` over `DataParallel` for multi-GPU training.
- Cross-GPU tensor ops are not generally allowed except for explicit copy-like operations such as `to(...)`, `copy_()`, or `cuda(...)`.
- Save and load `state_dict`s rather than whole module objects when you want safer, more portable checkpoints.

## Version-Sensitive Notes For `2.10.0`

- PyPI lists `torch 2.10.0` and Python `>=3.10` for this package version.
- The official PyTorch install-helper pages were not fully in sync on 2026-03-12: `get-started/locally` still surfaced `Stable (2.7.0)` and `previous-versions` still highlighted `v2.9.1`. Do not treat those labels as the package-version source of truth when pinning `2.10.0`.
- The stable serialization notes say that starting in `2.6`, `torch.load()` uses `weights_only=True` by default when `pickle_module` is not passed. Keeping `weights_only=True` explicit is still the clearest choice in agent-written code.
- The stable CUDA notes deprecate older `torch.cuda.amp.autocast(...)` and `torch.cpu.amp.autocast(...)` entry points in favor of `torch.amp.autocast("cuda", ...)` and `torch.amp.autocast("cpu", ...)`.

## Official Sources

- PyPI package page: `https://pypi.org/project/torch/`
- PyPI JSON API: `https://pypi.org/pypi/torch/json`
- PyTorch stable docs: `https://docs.pytorch.org/docs/stable/`
- PyTorch quickstart tutorial: `https://docs.pytorch.org/tutorials/beginner/basics/quickstart_tutorial.html`
- PyTorch CUDA notes: `https://docs.pytorch.org/docs/stable/notes/cuda.html`
- PyTorch serialization notes: `https://docs.pytorch.org/docs/stable/notes/serialization.html`
- PyTorch install selector: `https://pytorch.org/get-started/locally/`
