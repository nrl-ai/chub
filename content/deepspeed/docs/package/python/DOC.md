---
name: package
description: "DeepSpeed package guide for distributed PyTorch training, ZeRO, and inference in Python"
metadata:
  languages: "python"
  versions: "0.18.7"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "deepspeed,pytorch,distributed-training,zero,inference"
---

# DeepSpeed Python Package Guide

## Golden Rule

Install a PyTorch build that matches your CUDA runtime first, then install `deepspeed`, keep your DeepSpeed config explicit, and use the `DeepSpeedEngine` methods for training (`backward` and `step`) unless you have a specific reason to drop down to raw PyTorch behavior.

## Installation

PyPI `0.18.7` declares support for Python `>=3.8,<3.13`.

Install PyTorch first, then install DeepSpeed:

```bash
pip install torch
pip install deepspeed==0.18.7
```

Useful install variants:

```bash
pip install deepspeed
DS_BUILD_OPS=1 pip install deepspeed
```

- `pip install deepspeed` is the normal path.
- `DS_BUILD_OPS=1` forces DeepSpeed CUDA/C++ ops to build during install instead of at first use.
- DeepSpeed recommends `ninja` for faster op builds.

Verify what ops are available in the current environment:

```bash
ds_report
```

If you hit slow or repeated JIT builds, set a stable extension cache directory:

```bash
export TORCH_EXTENSIONS_DIR=/path/to/torch-extensions
```

## Minimal Training Setup

DeepSpeed adds CLI flags with `deepspeed.add_config_arguments(parser)` and converts your model into a `DeepSpeedEngine` with `deepspeed.initialize(...)`.

```python
import argparse
import deepspeed
import torch

def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("--local_rank", type=int, default=-1)
    return deepspeed.add_config_arguments(parser)

def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    model = MyModel()
    optimizer = torch.optim.AdamW(model.parameters(), lr=1e-4)

    ds_config = {
        "train_batch_size": 32,
        "train_micro_batch_size_per_gpu": 4,
        "gradient_accumulation_steps": 2,
        "bf16": {"enabled": True},
        "zero_optimization": {
            "stage": 2
        },
        "optimizer": {
            "type": "AdamW",
            "params": {
                "lr": 1e-4,
                "betas": [0.9, 0.999],
                "eps": 1e-8,
                "weight_decay": 0.01
            }
        }
    }

    model_engine, optimizer, _, _ = deepspeed.initialize(
        args=args,
        model=model,
        optimizer=optimizer,
        config=ds_config,
    )

    for batch in dataloader:
        loss = model_engine(**batch)
        model_engine.backward(loss)
        model_engine.step()
```

Notes:

- `config=` can be a Python `dict` or a path to a DeepSpeed JSON config file.
- `train_batch_size` is the global batch size. Keep it consistent with `train_micro_batch_size_per_gpu * gradient_accumulation_steps * data_parallel_world_size`.
- `model_engine.step()` handles the optimizer step and zeroing grads at the right accumulation boundary.

Typical launcher usage:

```bash
deepspeed train.py --deepspeed --deepspeed_config ds_config.json
```

For a single node without the launcher, you can still initialize DeepSpeed inside a process that already set up distributed state, but the launcher is the default operational path.

## Core Config Pattern

Most operational behavior lives in the DeepSpeed config. Start with these keys:

```json
{
  "train_batch_size": 32,
  "train_micro_batch_size_per_gpu": 4,
  "gradient_accumulation_steps": 2,
  "bf16": { "enabled": true },
  "gradient_clipping": 1.0,
  "zero_optimization": {
    "stage": 2
  },
  "optimizer": {
    "type": "AdamW",
    "params": {
      "lr": 0.0001,
      "betas": [0.9, 0.999],
      "eps": 1e-8,
      "weight_decay": 0.01
    }
  }
}
```

Practical defaults:

- Prefer `bf16` on modern NVIDIA GPUs that support it.
- Start with `zero_optimization.stage: 2` for standard distributed training.
- Move to stage 3 only when model states still do not fit in memory or when you need parameter partitioning.
- Add `gradient_clipping` explicitly instead of assuming your training loop or optimizer wrapper will handle it.

## ZeRO and Memory Scaling

DeepSpeed’s ZeRO system reduces memory pressure by partitioning optimizer states, gradients, and eventually parameters:

- Stage 1 partitions optimizer states.
- Stage 2 partitions optimizer states and gradients.
- Stage 3 also partitions model parameters.

Example stage-3 config with CPU offload:

```json
{
  "bf16": { "enabled": true },
  "zero_optimization": {
    "stage": 3,
    "offload_optimizer": {
      "device": "cpu",
      "pin_memory": true
    },
    "offload_param": {
      "device": "cpu",
      "pin_memory": true
    }
  }
}
```

Use stage 3 deliberately:

- It unlocks much larger models, but it changes checkpointing and module execution behavior.
- DeepSpeed documents `DeepSpeedCPUAdam` as the preferred optimizer when using ZeRO-Offload.

If your model has dynamic control flow or mixture-of-experts style routing under ZeRO-3, define the unstable parent as a ZeRO leaf module so all ranks gather the same parameter set during execution.

## Inference

`deepspeed.init_inference(...)` wraps an existing model for optimized inference. You can pass a config dict, keyword arguments, or both.

```python
import deepspeed
import torch

model = MyModel().eval().cuda()

engine = deepspeed.init_inference(
    model,
    config={
        "dtype": torch.float16,
        "replace_with_kernel_inject": True,
        "tensor_parallel": {"tp_size": 1},
    },
)

with torch.inference_mode():
    outputs = engine(**batch)
```

Notes:

- Keyword arguments and config dict values are merged; explicit kwargs win on conflict.
- `replace_with_kernel_inject=True` is the common path for supported transformer-style models.
- For tensor parallel inference, initialize the distributed environment before calling `init_inference`.

## Runtime Environment and Multi-Node Setup

DeepSpeed does not have package-level API auth. The operational setup is environment and cluster configuration instead:

- `CUDA_VISIBLE_DEVICES` can restrict GPU selection on a node.
- Multi-node runs typically use the DeepSpeed launcher plus a hostfile.
- DeepSpeed can propagate selected environment variables across nodes from a `.deepspeed_env` file.
- Set `DS_ENV_FILE=/path/to/file` if you do not want to use the default `.deepspeed_env` lookup.

Example `.deepspeed_env` values:

```bash
NCCL_IB_DISABLE=1
NCCL_SOCKET_IFNAME=eth0
```

If your cluster does not use passwordless SSH, use the documented `--no_ssh` flow and launch one DeepSpeed command per node with the correct node rank.

## Checkpointing

Checkpointing is easy to misuse with ZeRO:

```python
model_engine.save_checkpoint("/tmp/checkpoints", tag="global_step100")
```

Rules that matter:

- All ranks must call `save_checkpoint`. Calling it only on rank 0 can hang because each rank writes its own master weights and optimizer shard.
- Under ZeRO-3, you cannot save a checkpoint and then immediately reload it into the same still-partitioned engine instance. Reinitialize before load.
- If you need a plain fp32 `state_dict`, use DeepSpeed’s ZeRO checkpoint conversion utilities rather than assuming the checkpoint is already a standard PyTorch format.

## Common Pitfalls

- Install order matters. Resolve the correct PyTorch and CUDA combination first, then install DeepSpeed.
- Missing build tools cause avoidable pain. Install `ninja` and keep `TORCH_EXTENSIONS_DIR` stable on shared filesystems.
- CUDA version mismatches fail fast by default. `DS_SKIP_CUDA_CHECK=1` exists, but the docs explicitly warn it can lead to unexpected behavior.
- Older blog posts often assume only `engine.backward(loss)` is valid. DeepSpeed documents direct tensor fragment `backward()` support in versions `>= 0.18.3`, but `engine.backward(...)` is still the safest default across older examples and custom loops.
- ZeRO-3 can deadlock on dynamic submodule activation unless you mark the correct parent as a leaf module.
- DeepSpeed config mistakes are usually batch-size math or stage mismatches, not Python syntax errors. Check global batch, micro batch, accumulation, and ZeRO stage together.

## Version-Sensitive Notes for 0.18.7

- This doc is aligned to PyPI version `0.18.7` and the current `latest` DeepSpeed docs.
- The install docs still describe both JIT op building and pre-building during installation; choose one intentionally for your environment.
- Tensor fragment `backward()` support is documented for versions `>= 0.18.3`, so older pre-0.18.3 guidance may be more restrictive than necessary for `0.18.7`.
- If you are copying a DeepSpeed config from an older repository, check whether it assumes `fp16` only. Current DeepSpeed docs and examples commonly show `bf16` for supported hardware.

## Official Source URLs

- Docs root: https://deepspeed.readthedocs.io/en/latest/
- Getting Started: https://www.deepspeed.ai/getting-started/
- Config JSON: https://www.deepspeed.ai/docs/config-json/
- Inference Setup: https://deepspeed.readthedocs.io/en/latest/inference-init.html
- ZeRO docs: https://deepspeed.readthedocs.io/en/latest/zero3.html
- PyPI: https://pypi.org/project/deepspeed/
- Repository: https://github.com/deepspeedai/DeepSpeed
