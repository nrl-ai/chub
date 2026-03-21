---
name: package
description: "timm package guide for Python projects using pretrained vision models, feature extraction, fine-tuning, and Hugging Face Hub integration"
metadata:
  languages: "python"
  versions: "1.0.25"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "timm,pytorch,vision,image-classification,feature-extraction,huggingface"
---

# timm Python Package Guide

## Golden Rule

Use `timm.create_model(...)` plus the model-specific preprocessing config from `resolve_data_config()` and `create_transform()`. For pretrained inference, put the model in eval mode before calling it. Do not assume every `timm` checkpoint uses the same input size, crop percent, interpolation, or normalization values.

## Install

Pin the package version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "timm==1.0.25"
```

Common alternatives:

```bash
uv add "timm==1.0.25"
poetry add "timm==1.0.25"
```

Notes:

- `timm` installs against PyTorch and torchvision. If your environment needs a specific CUDA, ROCm, or CPU-only PyTorch build, choose that PyTorch install deliberately before pinning `timm`.
- The packaged release is for library use. The official training and validation scripts are not included in the wheel, so clone the repository and install from source if you need `train.py`, `validate.py`, `inference.py`, or the shell wrappers.

Source install when you need the repo scripts:

```bash
git clone https://github.com/huggingface/pytorch-image-models
cd pytorch-image-models
python -m pip install -e .
```

## Initialize And Inspect Models

List available pretrained model names:

```python
import timm

models = timm.list_models(pretrained=True)
print(models[:20])
```

Create a pretrained classifier:

```python
import timm

model = timm.create_model(
    "vit_base_patch16_224.augreg_in21k_ft_in1k",
    pretrained=True,
)
model.eval()
```

Useful constructor flags from the official reference:

- `pretrained=True`: load pretrained weights
- `num_classes=...`: replace the classifier head when fine-tuning
- `in_chans=...`: change expected input channels
- `features_only=True`: return intermediate feature maps instead of logits
- `cache_dir=...`: override the default model download cache location

## Preprocessing And Basic Inference

Build transforms from the model's own pretrained config instead of hard-coding ImageNet defaults:

```python
from PIL import Image
import torch
import timm
from timm.data import resolve_data_config
from timm.data.transforms_factory import create_transform

model = timm.create_model(
    "vit_base_patch16_224.augreg_in21k_ft_in1k",
    pretrained=True,
)
model.eval()

data_config = resolve_data_config(model.pretrained_cfg, model=model)
transform = create_transform(**data_config, is_training=False)

image = Image.open("example.jpg").convert("RGB")
inputs = transform(image).unsqueeze(0)

with torch.inference_mode():
    logits = model(inputs)
    probs = logits.softmax(dim=-1)
```

Important details:

- `transform(image)` returns `CHW`; add a batch dimension with `unsqueeze(0)`.
- Use `torch.inference_mode()` or `torch.no_grad()` for inference.
- Many checkpoints expose labels through the associated model card or Hub repo rather than directly from the model object.

## Feature Extraction

### Get the backbone output before the classifier

```python
import timm
import torch

model = timm.create_model("resnet50.a1_in1k", pretrained=True)
model.eval()

x = torch.randn(1, 3, 224, 224)
features = model.forward_features(x)
print(features.shape)
```

### Get pooled embeddings instead of logits

```python
import timm

model = timm.create_model(
    "resnet50.a1_in1k",
    pretrained=True,
    num_classes=0,
    global_pool="avg",
)
model.eval()
```

For an already-created model, the reference docs also expose `reset_classifier(num_classes, global_pool)`.

### Get multi-scale feature maps

```python
import timm

backbone = timm.create_model(
    "resnet50.a1_in1k",
    pretrained=True,
    features_only=True,
    out_indices=(1, 2, 3, 4),
)

feature_maps = backbone(torch.randn(1, 3, 224, 224))
for fmap in feature_maps:
    print(fmap.shape)
```

`features_only=True` is the right choice for detection, segmentation, or other tasks that need pyramid-style intermediate activations.

## Fine-Tuning

For standard transfer learning, replace the classifier head at construction time:

```python
import timm

model = timm.create_model(
    "resnet50.a1_in1k",
    pretrained=True,
    num_classes=10,
)
```

If you already built the model, update the head in place:

```python
model.reset_classifier(num_classes=10)
```

Keep the pretrained transform pipeline unless you intentionally change image size or augmentation behavior for training:

```python
from timm.data import resolve_data_config
from timm.data.transforms_factory import create_transform

data_config = resolve_data_config(model.pretrained_cfg, model=model)
train_transform = create_transform(**data_config, is_training=True)
val_transform = create_transform(**data_config, is_training=False)
```

## Hugging Face Hub Integration

`timm` integrates with Hugging Face Hub for loading and sharing model weights. For a public Hub model, load directly by prefixing the repo id:

```python
import timm

model = timm.create_model("hf-hub:timm/eca_nfnet_l0", pretrained=True)
model.eval()
```

Use `cache_dir` when you need a deterministic or project-local cache path:

```python
model = timm.create_model(
    "hf-hub:timm/eca_nfnet_l0",
    pretrained=True,
    cache_dir=".cache/huggingface",
)
```

Authentication notes:

- Public pretrained weights do not require auth.
- Private or gated Hub repos do require a token.
- The current Hugging Face auth flow is:

```bash
hf auth login
```

After authentication, the same `hf-hub:...` loading pattern works for repos your token can access.

## Official Training Scripts

The repo ships ready-made training, validation, and inference scripts, but the maintainer docs explicitly note these are not included in the pip release package. If your task requires:

- distributed training
- EMA, AMP, advanced augmentation, mixup, or RandAugment defaults
- reproducing official benchmark commands
- bulk validation across checkpoints

use the GitHub repository checkout instead of expecting those scripts to be available after `pip install timm`.

## Common Pitfalls

- Do not reuse a generic torchvision normalization pipeline unless you have checked the model's `pretrained_cfg`. `timm` models vary by image size, crop percentage, interpolation, mean, and std.
- Do not skip `model.eval()` for pretrained inference; dropout and batchnorm behavior will be wrong.
- `forward_features()` does not always return the same shape across architectures. CNNs commonly return feature maps; transformers may return token or spatial representations depending on the model.
- `features_only=True` changes the model output to a list of intermediate tensors, not classifier logits.
- `num_classes=0` removes the classifier head for embedding extraction. If you still need logits, keep a positive `num_classes`.
- The wheel does not include the repo scripts. If an example references `train.py` or shell launchers, you need a source checkout.
- Current Hugging Face Hub docs use the `hf-hub:` prefix. Older `timm` docs pages and snippets may show `hf_hub:` instead; prefer the current `hf-hub:` form when writing new code.

## Version-Sensitive Notes For `1.0.25`

- The version used here `1.0.25` matches the current PyPI release history and the Hugging Face docs navigation, which marks `v1.0.25` as the stable branch.
- PyPI's `1.0.25` release notes call out two recent loading-related changes that matter when copying older examples: `create_model` accepts `cache_dir`, and the Hugging Face Hub loading path passes through `trust_remote_code`.
- Some docs pages default to `main`, which can describe behavior newer than the wheel you have pinned. For a project locked to `1.0.25`, prefer the stable docs branch over `main` when examples conflict.
- The project lives under the Hugging Face-maintained `huggingface/pytorch-image-models` repository. Older blog posts may still refer to the previous `rwightman/pytorch-image-models` location.

## Official Sources Used For This Entry

- `https://huggingface.co/docs/timm/`
- `https://huggingface.co/docs/timm/quickstart`
- `https://huggingface.co/docs/timm/installation`
- `https://huggingface.co/docs/timm/feature_extraction`
- `https://huggingface.co/docs/timm/reference/models`
- `https://huggingface.co/docs/hub/en/timm`
- `https://pypi.org/project/timm/`
- `https://github.com/huggingface/pytorch-image-models`
- `https://raw.githubusercontent.com/huggingface/pytorch-image-models/main/pyproject.toml`
