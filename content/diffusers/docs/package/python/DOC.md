---
name: package
description: "diffusers package guide for Python - Hugging Face Diffusers 0.37.0"
metadata:
  languages: "python"
  versions: "0.37.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "diffusers,hugging-face,diffusion,generative-ai,images,video,audio"
---

# diffusers Python Package Guide

## Golden Rule

Use `DiffusionPipeline.from_pretrained(...)` for the first working version, authenticate with Hugging Face before using gated models, and pin the package version because model-loading behavior and docs examples move quickly.

## What This Package Is For

`diffusers` is Hugging Face's Python library for running and training diffusion models. The main building blocks you use in application code are:

- `DiffusionPipeline` for end-to-end inference
- model classes and schedulers when you need to customize a pipeline
- official training scripts in `diffusers/examples` when you need fine-tuning or full training

It supports image, video, audio, and other diffusion workloads, but the most common coding-agent task is loading a Hub checkpoint and generating outputs through a pipeline.

## Installation

PyPI currently publishes `diffusers 0.37.0` with `Requires: Python >=3.10`. Start there, even though some installation docs pages still describe older Python support ranges.

### Recommended install for PyTorch workflows

```bash
python -m venv .venv
source .venv/bin/activate
pip install "diffusers[torch]==0.37.0" transformers accelerate safetensors
```

### If your project pins PyTorch separately

```bash
pip install "diffusers==0.37.0" transformers accelerate safetensors
```

### Check the installed version

```bash
python -c "import diffusers; print(diffusers.__version__)"
```

## Authentication and Hub Setup

Many popular checkpoints are public, but gated or private models require Hugging Face authentication.

### CLI login

```bash
hf auth login
hf auth whoami
```

### Programmatic login

```python
from huggingface_hub import login

login()
```

### When explicit tokens are useful

- CI or short-lived jobs
- switching between accounts
- accessing gated models without an interactive prompt

Hugging Face documents both machine login and explicit `token=` usage across loading APIs. If a loading example for your exact pipeline class shows `token` or older `use_auth_token` style arguments, follow the signature for the version you have installed.

## Cache and Offline Configuration

Model files are cached locally. These environment variables are the main ones worth knowing:

```bash
export HF_HOME="/path/to/cache-root"
export HF_HUB_CACHE="/path/to/hub-cache"
export HF_HUB_OFFLINE=1
export HF_HUB_DISABLE_TELEMETRY=1
```

Practical meaning:

- `HF_HOME` and `HF_HUB_CACHE` move the download cache
- `HF_HUB_OFFLINE=1` forces local-cache-only behavior
- `HF_HUB_DISABLE_TELEMETRY=1` disables Hub telemetry during `from_pretrained()` loads

## Fastest Path To A Working Pipeline

Start with the generic pipeline loader unless you already know the exact pipeline class you need.

```python
import torch
from diffusers import DiffusionPipeline

model_id = "runwayml/stable-diffusion-v1-5"

pipe = DiffusionPipeline.from_pretrained(
    model_id,
    torch_dtype=torch.float16,
    use_safetensors=True,
)
pipe = pipe.to("cuda")

image = pipe("a cinematic photo of a lighthouse in winter at sunrise").images[0]
image.save("output.png")
```

Notes:

- `DiffusionPipeline` auto-detects the right pipeline class from the checkpoint.
- `use_safetensors=True` is a good default when the checkpoint provides safetensors weights.
- Most official examples assume GPU inference. On CPU, remove `torch.float16` and expect much slower generation.

## Common Loading Patterns

### Load a specific pipeline class

Use this when you already know the task and want explicit behavior.

```python
import torch
from diffusers import StableDiffusionPipeline

pipe = StableDiffusionPipeline.from_pretrained(
    "runwayml/stable-diffusion-v1-5",
    torch_dtype=torch.float16,
    use_safetensors=True,
)
pipe = pipe.to("cuda")
```

### Load from a local directory

```python
from diffusers import DiffusionPipeline

pipe = DiffusionPipeline.from_pretrained("./my-downloaded-model")
```

This uses the local files as-is. If you expect newer Hub weights, refresh the local snapshot instead of assuming `from_pretrained()` will update a copied directory.

### Save a pipeline locally

```python
pipe.save_pretrained("./stable-diffusion-v1-5")
```

### Use a gated model after login

```python
import torch
from diffusers import DiffusionPipeline

pipe = DiffusionPipeline.from_pretrained(
    "black-forest-labs/FLUX.1-dev",
    torch_dtype=torch.bfloat16,
)
pipe = pipe.to("cuda")
```

If this fails with access or license errors, verify that:

1. your Hugging Face account accepted the model terms
2. `hf auth whoami` succeeds on the machine running the code
3. the token has enough access for the repository you are loading

## Memory-Constrained Inference

For large models, start with offloading before rewriting the pipeline structure.

```python
import torch
from diffusers import DiffusionPipeline

pipe = DiffusionPipeline.from_pretrained(
    "runwayml/stable-diffusion-v1-5",
    torch_dtype=torch.float16,
)
pipe.enable_model_cpu_offload()
```

Practical cautions:

- `enable_model_cpu_offload()` is the simple first option when a model barely fits in VRAM.
- `device_map` is a different placement strategy; Diffusers documents values such as `"cuda"` and `"balanced"` for pipeline placement.
- If you loaded with a device map and later need `.to(...)` or offload helpers, reset the device map first with `pipe.reset_device_map()`.

## Swapping Components

One of the main reasons to use `diffusers` instead of a model-specific wrapper is that schedulers and model components are replaceable.

```python
from diffusers import DiffusionPipeline, DPMSolverMultistepScheduler

pipe = DiffusionPipeline.from_pretrained("runwayml/stable-diffusion-v1-5")
pipe.scheduler = DPMSolverMultistepScheduler.from_config(pipe.scheduler.config)
```

Use this when you want to trade off speed and quality without changing the checkpoint.

## Training and Fine-Tuning

Use the official training scripts for real training work. Do not start from an inference-only snippet and improvise a training loop unless you already know the model internals you need.

The upstream training overview describes the scripts as:

- self-contained
- easy to tweak
- beginner-friendly
- single-purpose

Common maintained examples include:

- text-to-image
- DreamBooth
- LoRA
- textual inversion
- ControlNet

For coding agents, the practical rule is:

1. find the closest official example in `diffusers/examples`
2. match its dependency set and launch arguments first
3. only then customize dataset handling, prompts, or model components

## Common Pitfalls

### Installing `diffusers` without the runtime stack

`diffusers` alone is often not enough for useful PyTorch workflows. In practice you usually also need:

- `torch`
- `transformers`
- `accelerate`
- `safetensors`

### Using `main` docs with a pinned PyPI release

The docs site has a version switcher, and `main` may document features not present in `0.37.0`. If an example looks newer than your installation, compare it against the `0.37.0` package version before copying it.

### Assuming every model can be loaded the same way

The generic pipeline loader is the right default, but model cards still matter. Check the model page for:

- license or gated-access requirements
- recommended dtype
- expected hardware
- task-specific inputs such as masks, control images, or reference audio

### Treating a local directory like a Hub repo

`from_pretrained("./path")` loads whatever files are already on disk. It does not magically track upstream updates for that copied directory.

### Mixing device placement helpers incorrectly

If you use `device_map`, offloading, quantization, and `.to(...)` in the same script, apply them deliberately and verify the pipeline state after each step. The loading docs explicitly call out `reset_device_map()` when changing placement strategy.

## Version-Sensitive Notes For 0.37.0

- PyPI lists `0.37.0` as the package version covered here.
- The official GitHub releases page for `v0.37.0` highlights Modular Diffusers as an experimental feature. Treat that API surface as less stable than the long-standing pipeline APIs.
- Some Hugging Face docs pages and search snippets still reference older "latest stable" versions. Prefer the version switcher on the docs site plus the PyPI release page when you need to reconcile examples.
- Authentication examples across Hugging Face docs may show either machine login or explicit token arguments. Use `hf auth login` as the default setup path, and check the exact method signature if you need non-interactive token passing.

## Minimal Agent Workflow

When asked to write code with `diffusers`, this order is usually correct:

1. Confirm the project is actually using `diffusers==0.37.0` or a close version.
2. Confirm whether inference or training is needed.
3. Identify the exact Hub model ID.
4. Check whether the model is public, private, or gated.
5. Start with `DiffusionPipeline.from_pretrained(...)`.
6. Add dtype, `device_map`, or offloading only after the basic load works.
7. For training, switch to the closest official example script instead of building from scratch.

## Official Source URLs

- Docs root: `https://huggingface.co/docs/diffusers/`
- Installation: `https://huggingface.co/docs/diffusers/en/installation`
- Loading pipelines: `https://huggingface.co/docs/diffusers/using-diffusers/loading`
- Training overview: `https://huggingface.co/docs/diffusers/main/en/training/overview`
- PyPI package: `https://pypi.org/project/diffusers/`
- PyPI release `0.37.0`: `https://pypi.org/project/diffusers/0.37.0/`
- GitHub releases: `https://github.com/huggingface/diffusers/releases`
- GitHub release `v0.37.0`: `https://github.com/huggingface/diffusers/releases/tag/v0.37.0`
