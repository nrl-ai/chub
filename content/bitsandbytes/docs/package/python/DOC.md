---
name: package
description: "bitsandbytes Python package for k-bit quantization, 8-bit optimizers, and low-memory PyTorch/Transformers workflows"
metadata:
  languages: "python"
  versions: "0.49.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "bitsandbytes,quantization,pytorch,transformers,llm,qlora,optimizers"
---

# bitsandbytes Python Package Guide

## Golden Rule

Use `bitsandbytes` only with a supported PyTorch and hardware stack, and treat backend compatibility as part of setup. For Hugging Face model loading, prefer `transformers.BitsAndBytesConfig`; for custom PyTorch modules and optimizers, use `bitsandbytes.nn` and `bitsandbytes.optim` directly.

## What It Is For

`bitsandbytes` is a PyTorch-focused low-precision toolkit with three main capabilities:

- 8-bit optimizers to reduce optimizer-state memory usage during training
- LLM.int8() layers for 8-bit inference with mixed-precision handling for outliers
- 4-bit quantization primitives used in QLoRA-style finetuning and low-memory model loading

The package is usually used in one of two ways:

1. Via `transformers`, `accelerate`, or `peft` integration for model loading and finetuning
2. Directly from `bitsandbytes.nn` and `bitsandbytes.optim` inside custom PyTorch code

## Install

Pin the package version your project expects:

```bash
python -m pip install "bitsandbytes==0.49.2"
```

Common alternatives:

```bash
uv add "bitsandbytes==0.49.2"
poetry add "bitsandbytes==0.49.2"
```

Basic verification:

```bash
python - <<'PY'
from importlib.metadata import version
import torch
import bitsandbytes as bnb

print("bitsandbytes", version("bitsandbytes"))
print("torch", torch.__version__)
print("cuda_available", torch.cuda.is_available())
print("mps_available", torch.backends.mps.is_available() if hasattr(torch.backends, "mps") else False)
print("import_ok", bnb is not None)
PY
```

## Platform And Runtime Expectations

Official docs for `0.49.2` list these minimums:

- Python `>=3.10`
- PyTorch `>=2.3`

Officially supported compute targets in the install guide:

- NVIDIA CUDA
- CPU
- Intel XPU
- Intel Gaudi

Experimental or preview support:

- AMD ROCm
- Apple Silicon CPU

Important platform notes:

- On NVIDIA, `LLM.int8()` needs compute capability `7.5+`.
- On NVIDIA, 8-bit optimizers and NF4/FP4 quantization need compute capability `6.0+`.
- Linux wheels require `glibc >= 2.24`.
- Apple Silicon CPU is listed, but `mps` is not supported.
- If your CUDA or platform combination is unusual, upstream recommends compiling from source.

## Configuration

There is no service auth layer. Configuration is about runtime, device placement, and dtypes.

Settings that matter most in practice:

- the installed PyTorch build and its CUDA/XPU backend
- whether the host hardware is actually supported by the selected quantization mode
- compute dtype for 4-bit workflows, especially `torch.bfloat16` on supported hardware
- CUDA library paths when native kernels fail to load

For 4-bit Transformers workflows, upstream recommends `bfloat16` compute when the hardware supports it because it is usually a better tradeoff than `float32` or `float16`.

## Core Usage

### Load a Transformers model in 4-bit

This is the most common application-level entry point:

```python
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig

model_id = "bigscience/bloom-1b7"

quantization_config = BitsAndBytesConfig(
    load_in_4bit=True,
    bnb_4bit_compute_dtype=torch.bfloat16,
    bnb_4bit_use_double_quant=True,
    bnb_4bit_quant_type="nf4",
)

tokenizer = AutoTokenizer.from_pretrained(model_id)
model = AutoModelForCausalLM.from_pretrained(
    model_id,
    device_map="auto",
    quantization_config=quantization_config,
)
```

Use this path when your task is "load an LLM with lower memory use" rather than "manually rewrite linear layers".

### Load a Transformers model in 8-bit

```python
from transformers import AutoModelForCausalLM, BitsAndBytesConfig

quantization_config = BitsAndBytesConfig(load_in_8bit=True)

model = AutoModelForCausalLM.from_pretrained(
    "bigscience/bloom-1b7",
    device_map="auto",
    quantization_config=quantization_config,
)
```

### Replace linear layers directly in PyTorch

Use `bitsandbytes.nn` when you control the model definition yourself.

`Linear8bitLt` example:

```python
import torch
import torch.nn as nn
import bitsandbytes as bnb

fp16_model = nn.Sequential(
    nn.Linear(64, 64),
    nn.Linear(64, 64),
).half()

int8_model = nn.Sequential(
    bnb.nn.Linear8bitLt(64, 64, has_fp16_weights=False),
    bnb.nn.Linear8bitLt(64, 64, has_fp16_weights=False),
)

int8_model.load_state_dict(fp16_model.state_dict())
int8_model = int8_model.to("cuda")
```

`Linear4bit` example:

```python
import torch
import torch.nn as nn
import bitsandbytes as bnb

fp16_model = nn.Sequential(
    nn.Linear(64, 64),
    nn.Linear(64, 64),
).half()

quantized_model = nn.Sequential(
    bnb.nn.Linear4bit(64, 64, quant_type="nf4", compute_dtype=torch.bfloat16),
    bnb.nn.Linear4bit(64, 64, quant_type="nf4", compute_dtype=torch.bfloat16),
)

quantized_model.load_state_dict(fp16_model.state_dict())
quantized_model = quantized_model.to("cuda")
```

Important behavior:

- For both `Linear8bitLt` and `Linear4bit`, quantization happens when the module is moved to the target device after the original fp16 or bf16 weights are loaded.
- `Linear8bitLt(has_fp16_weights=True)` keeps weights in fp16 and quantizes on the fly during forward passes.
- `Linear8bitLt(has_fp16_weights=False)` stores quantized weights after device transfer.

### Use 8-bit optimizers

For training code that is already otherwise conventional PyTorch:

```python
import bitsandbytes as bnb

optimizer = bnb.optim.AdamW8bit(
    model.parameters(),
    lr=2e-4,
    weight_decay=0.01,
)
```

Useful parameters from the optimizer reference:

- `min_8bit_size=4096` by default, so very small parameter tensors stay out of 8-bit optimization
- `percentile_clipping` can improve stability for difficult training runs
- `block_wise=True` is the default and helps reduce outlier effects

If you need mixed optimizer behavior for specific layers, use `GlobalOptimManager`:

```python
import bitsandbytes as bnb

mng = bnb.optim.GlobalOptimManager.get_instance()
mng.register_parameters(model.parameters())
model = model.cuda()

optimizer = bnb.optim.Adam(model.parameters(), lr=1e-3, optim_bits=8)
mng.override_config(model.fc1.weight, "optim_bits", 32)
```

## Integration Notes

`bitsandbytes` is heavily used through other libraries rather than by itself:

- `transformers`: `BitsAndBytesConfig` for 4-bit and 8-bit model loading
- `peft`: `prepare_model_for_kbit_training()` plus LoRA/QLoRA adapters
- `accelerate`: `BnbQuantizationConfig` with `load_and_quantize_model()`

Prefer those integration layers when the task is model loading, trainer setup, or QLoRA finetuning. Drop down to raw `bitsandbytes` APIs when you need custom layers, custom optimizers, or lower-level debugging.

## Common Pitfalls

- Installing the package is not enough; unsupported hardware or a mismatched PyTorch backend will still fail at runtime.
- `LLM.int8()` has stricter NVIDIA requirements than 4-bit quantization and 8-bit optimizers.
- `Linear4bit` and `Linear8bitLt` do not quantize immediately at construction time; the weight conversion happens after loading fp16 or bf16 weights and moving the module to the target device.
- Apple Silicon users should not assume `mps` support just because macOS appears in package metadata. The current docs list Apple CPU support, but not MPS.
- If you see `No kernel image available` or `fatbinwrap`, check `PATH`, `LD_LIBRARY_PATH`, `CUDA_HOME`, and the CUDA runtime that your PyTorch build expects before recompiling.
- The docs quickstart page is still incomplete in `v0.49.2`; rely on the installation, integrations, and API reference pages for concrete examples.

## Troubleshooting Checklist

1. Confirm the installed versions of `bitsandbytes` and `torch`.
2. Confirm the hardware backend you are actually using: CUDA, CPU, XPU, or Gaudi.
3. On NVIDIA, confirm the GPU compute capability is high enough for the feature you want.
4. If CUDA kernels fail to load, inspect `PATH`, `LD_LIBRARY_PATH`, and `CUDA_HOME` for mixed CUDA installs.
5. If your setup is outside the prebuilt wheel matrix, switch to a source build instead of fighting the wrong wheel.

## Version-Sensitive Notes For 0.49.2

- PyPI lists `0.49.2` as the latest release on February 16, 2026.
- The Hugging Face docs version selector includes `v0.49.2`, so the docs root and API reference can be read against the same release line.
- The current docs state official support for NVIDIA GPU, CPU, Intel XPU, and Intel Gaudi, with AMD ROCm and Apple Silicon still not in the same support tier.
- The package metadata classifies the project as beta, so agents should expect hardware support and wheel coverage to evolve more quickly than typical pure-Python packages.

## Official Sources

- Docs root: `https://huggingface.co/docs/bitsandbytes/en/index`
- Installation guide: `https://huggingface.co/docs/bitsandbytes/en/installation`
- Integrations guide: `https://huggingface.co/docs/bitsandbytes/en/integrations`
- LLM.int8() reference: `https://huggingface.co/docs/bitsandbytes/en/reference/nn/linear8bit`
- 4-bit quantization reference: `https://huggingface.co/docs/bitsandbytes/en/reference/nn/linear4bit`
- Optimizer overview: `https://huggingface.co/docs/bitsandbytes/en/reference/optim/optim_overview`
- Troubleshooting: `https://huggingface.co/docs/bitsandbytes/en/errors`
- PyPI: `https://pypi.org/project/bitsandbytes/`
