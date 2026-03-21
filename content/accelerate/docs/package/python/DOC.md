---
name: package
description: "Hugging Face Accelerate for distributed PyTorch training and inference in Python"
metadata:
  languages: "python"
  versions: "1.13.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "accelerate,huggingface,pytorch,distributed-training,mixed-precision,fsdp,deepspeed"
---

# accelerate Python Package Guide

## What It Is

`accelerate` is Hugging Face's thin orchestration layer for running mostly-standard PyTorch code across CPU, single-GPU, multi-GPU, TPU, mixed-precision, FSDP, and DeepSpeed setups without rewriting the whole training loop.

Use it when you still want to own the training loop. Do not reach for it if you want a high-level trainer abstraction instead.

## Version Covered

- Package: `accelerate`
- Ecosystem: `pypi`
- Version: `1.13.0`
- Docs root: `https://huggingface.co/docs/accelerate/`
- Registry: `https://pypi.org/project/accelerate/`

## Install

Install PyTorch first for your platform, then install `accelerate`:

```bash
pip install torch accelerate
```

Common extras from PyPI metadata:

```bash
pip install "accelerate[deepspeed]"
pip install "accelerate[rich]"
pip install "accelerate[sagemaker]"
```

Other official install paths:

```bash
conda install -c conda-forge accelerate
pip install git+https://github.com/huggingface/accelerate
```

## Python Requirement

For `accelerate==1.13.0`, PyPI metadata requires `Python >=3.10`.

Version-sensitive note:
- The Hugging Face install page still says Accelerate is "tested on Python 3.8+". For a real `1.13.0` environment, treat the PyPI requirement as authoritative and use Python 3.10+.

## Core Pattern

The normal migration path is:

1. Create `Accelerator()` early.
2. Replace manual device selection with `accelerator.device` or let `prepare()` handle placement.
3. Pass your model, optimizer, dataloaders, and schedulers through `accelerator.prepare(...)`.
4. Replace `loss.backward()` with `accelerator.backward(loss)`.
5. Launch the script with `accelerate launch` or `torchrun`.

Minimal training script:

```python
import torch
from torch import nn
from torch.utils.data import DataLoader, TensorDataset
from accelerate import Accelerator

def main() -> None:
    accelerator = Accelerator()

    model = nn.Sequential(nn.Linear(16, 32), nn.ReLU(), nn.Linear(32, 2))
    optimizer = torch.optim.AdamW(model.parameters(), lr=1e-3)

    x = torch.randn(128, 16)
    y = torch.randint(0, 2, (128,))
    dataloader = DataLoader(TensorDataset(x, y), batch_size=8, shuffle=True)

    model, optimizer, dataloader = accelerator.prepare(model, optimizer, dataloader)

    model.train()
    for batch_x, batch_y in dataloader:
        optimizer.zero_grad()
        logits = model(batch_x)
        loss = nn.functional.cross_entropy(logits, batch_y)
        accelerator.backward(loss)
        optimizer.step()

    accelerator.print("done")

if __name__ == "__main__":
    main()
```

## Setup And Launch

Interactive config:

```bash
accelerate config
```

Barebones config:

```bash
python -c "from accelerate.utils import write_basic_config; write_basic_config(mixed_precision='fp16')"
```

Inspect the detected environment and current config:

```bash
accelerate env
```

Launch the script:

```bash
accelerate launch train.py --arg1 foo
```

You can also pass launch flags directly instead of relying on a saved config:

```bash
accelerate launch --multi_gpu --num_processes 2 train.py
accelerate launch --cpu train.py
python -m accelerate.commands.launch --num_processes=2 train.py
```

Config location priority from the official docs:

1. `HF_HOME/accelerate`
2. `XDG_CACHE_HOME/huggingface/accelerate`
3. `~/.cache/huggingface/accelerate`

Use `--config_file` when you need multiple launch profiles for different machines or clusters.

## Common `Accelerator` Options

These are the options most likely to matter in generated code:

- `mixed_precision="no" | "fp16" | "bf16"` for autocast and mixed precision behavior
- `gradient_accumulation_steps=N` for accumulation-aware backward/step coordination
- `cpu=True` to force CPU execution
- `log_with=...` to enable experiment trackers
- `project_dir="..."` to control local logs and checkpoint storage

Example:

```python
from accelerate import Accelerator

accelerator = Accelerator(
    mixed_precision="bf16",
    gradient_accumulation_steps=4,
    log_with="tensorboard",
    project_dir="runs/exp1",
)
```

## Gradient Accumulation

Prefer Accelerate's accumulation helpers instead of hand-rolled modulo logic in distributed code:

```python
from accelerate import Accelerator

accelerator = Accelerator(gradient_accumulation_steps=4)
model, optimizer, dataloader, scheduler = accelerator.prepare(
    model, optimizer, dataloader, scheduler
)

for batch in dataloader:
    with accelerator.accumulate(model):
        outputs = model(batch["input_ids"])
        loss = outputs.loss
        accelerator.backward(loss)
        if accelerator.sync_gradients:
            accelerator.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()
        scheduler.step()
        optimizer.zero_grad()
```

Why this matters:
- `Accelerator.accumulate(...)` keeps synchronization behavior aligned with distributed execution.
- `accelerator.backward(...)` applies the right scaling behavior for the configured setup.

## Distributed Evaluation And Metrics

Do not compute metrics independently on each worker and then trust rank 0's local batch. Gather first:

```python
preds = accelerator.gather_for_metrics(preds)
labels = accelerator.gather_for_metrics(labels)
```

`gather_for_metrics()` exists specifically for metric computation across processes and handles duplicate-dropping in the last distributed batch.

## Saving And Restoring

For training checkpoints that you intend to resume in the same kind of environment:

```python
accelerator.save_state(output_dir="checkpoints/step_1000")
accelerator.load_state("checkpoints/step_1000")
```

For custom stateful objects:

```python
accelerator.register_for_checkpointing(lr_scheduler)
```

For model export or safe save patterns in distributed jobs:

```python
accelerator.wait_for_everyone()
unwrapped_model = accelerator.unwrap_model(model)
accelerator.save_model(unwrapped_model, "exported-model")
```

Important distinction:
- `save_state()` is for resumable training state in the same environment shape.
- `save_model()` / model-specific `save_pretrained(...)` patterns are for exported model artifacts.

## Experiment Tracking

Accelerate can initialize and manage trackers from the `Accelerator` instance:

```python
accelerator = Accelerator(log_with="tensorboard", project_dir="runs/demo")
accelerator.init_trackers("demo-project", config={"lr": 1e-3, "batch_size": 32})

# training

accelerator.end_training()
```

If you enable trackers, call `accelerator.end_training()` before process exit so tracker shutdown and process-group cleanup happen in the expected place.

## Notebook Usage

For notebooks, use `notebook_launcher(...)` and create the `Accelerator` inside the launched function, not at notebook top level:

```python
from accelerate import Accelerator, notebook_launcher

def training_function():
    accelerator = Accelerator(mixed_precision="bf16")
    ...

notebook_launcher(training_function, num_processes=2)
```

This pattern matters for Colab, Kaggle, TPU, and multi-device notebook execution.

## Config And Environment Notes

`accelerate` itself does not use service authentication. The main configuration surface is:

- the generated Accelerate launch config
- device visibility environment variables such as `CUDA_VISIBLE_DEVICES`
- cache location variables such as `HF_HOME` and `XDG_CACHE_HOME`
- runtime env vars like `ACCELERATE_GRADIENT_ACCUMULATION_STEPS` and `ACCELERATE_LOG_LEVEL`

If code needs model or dataset credentials, those usually belong to other libraries such as `huggingface_hub`, not `accelerate`.

## Common Pitfalls

- Do not keep mixing manual `.to(device)` calls with automatic placement unless you are intentionally controlling placement yourself.
- Do not keep `loss.backward()` after migration. Use `accelerator.backward(loss)`.
- Do not forget the `if __name__ == "__main__":` guard when using `accelerate launch`.
- Do not assume the docs page you found is stable. The docs site exposes both `main` and release versions; `main` may describe unreleased behavior that requires installation from source.
- Do not use local per-rank tensors directly for metrics in distributed evaluation. Gather first with `gather_for_metrics()`.
- Do not treat `save_state()` as a generic model export format.
- If you use experiment tracking, remember `accelerator.end_training()`.
- If you only need plain inference and no mixed precision, you may not need to `prepare()` the model at all.

## Version-Sensitive Notes For `1.13.0`

- `1.13.0` is on PyPI as of March 4, 2026.
- PyPI metadata for `1.13.0` requires Python 3.10+, even though some official docs text still references Python 3.8+.
- The Hugging Face docs UI currently exposes both `main` and `v1.13.0`. Prefer stable `v1.13.0` content for pip-installed `1.13.0`; use `main` only when you intentionally install from source.
- The package exposes extras on PyPI including `deepspeed`, `rich`, and `sagemaker`.

## Official Sources

- Docs root: https://huggingface.co/docs/accelerate/
- Installation: https://huggingface.co/docs/accelerate/basic_tutorials/install
- Launching scripts: https://huggingface.co/docs/accelerate/en/basic_tutorials/launch
- Accelerator reference: https://huggingface.co/docs/accelerate/main/package_reference/accelerator
- CLI reference: https://huggingface.co/docs/accelerate/main/package_reference/cli
- Notebook launchers: https://huggingface.co/docs/accelerate/en/package_reference/launchers
- Troubleshooting: https://huggingface.co/docs/accelerate/en/basic_tutorials/troubleshooting
- Gradient accumulation guide: https://huggingface.co/docs/accelerate/en/usage_guides/gradient_accumulation
- PyPI: https://pypi.org/project/accelerate/
- GitHub repository: https://github.com/huggingface/accelerate
