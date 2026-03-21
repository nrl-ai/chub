---
name: package
description: "Kornia Python package guide for differentiable computer vision and image augmentation with PyTorch tensors"
metadata:
  languages: "python"
  versions: "0.8.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "kornia,pytorch,computer-vision,image-processing,augmentation,deep-learning"
---

# Kornia Python Package Guide

## Golden Rule

Use `kornia` as a PyTorch-first computer vision library. Install a compatible `torch` build first, keep image tensors in channels-first layout, and check dtype and value range before composing color, geometry, or augmentation operators.

## Install

Install a matching PyTorch build first, then pin Kornia:

```bash
python -m pip install "kornia==0.8.2"
```

Common alternatives:

```bash
uv add "kornia==0.8.2"
poetry add "kornia==0.8.2"
```

Optional Rust-backed image I/O:

```bash
python -m pip install "kornia_rs"
```

The official `kornia.io` docs note that `kornia_rs` is a separate install and currently Linux-only.

Verify the installed version:

```python
import kornia

print(kornia.__version__)
```

## Setup

Kornia does not use API keys or service configuration. The important runtime setup is:

- install the correct `torch` wheel for your CPU or CUDA environment before adding Kornia
- keep tensors and models on the same device
- use channels-first image layout: `(C, H, W)` or `(B, C, H, W)`
- check dtype and value range before applying operators

Typical starter setup:

```python
import torch
import kornia

device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
images = torch.rand(4, 3, 256, 256, device=device)  # BCHW, float32, [0, 1]
```

## Core Usage

### Geometry and color transforms

Most Kornia operators work directly on PyTorch tensors and stay differentiable.

```python
import torch
from kornia.color import rgb_to_grayscale
from kornia.geometry.transform import resize

images = torch.rand(2, 3, 256, 256)

smaller = resize(images, (128, 128), antialias=True)
gray = rgb_to_grayscale(smaller)

print(smaller.shape)  # (2, 3, 128, 128)
print(gray.shape)     # (2, 1, 128, 128)
```

Use Kornia operators inside normal PyTorch modules when you want gradients to flow through preprocessing or geometric transformations.

### Build augmentation pipelines

`kornia.augmentation` provides module-style augmentations that fit naturally into training code.

```python
import torch
import kornia.augmentation as K

aug = K.AugmentationSequential(
    K.RandomHorizontalFlip(p=0.5),
    K.RandomAffine(degrees=15.0, p=0.5),
    data_keys=["input"],
    same_on_batch=False,
)

images = torch.rand(8, 3, 224, 224)
augmented = aug(images)
```

Use `same_on_batch=True` only when every sample in the batch should receive the same random transform.

### Load images with `kornia_rs`

If you install `kornia_rs`, the `kornia.io` helpers can decode directly to tensors:

```python
import kornia as K
from kornia.io import ImageLoadType

image = K.io.load_image("cat.png", ImageLoadType.RGB32, device="cpu")
batch = image.unsqueeze(0)

print(image.shape)  # (3, H, W)
print(image.dtype)  # torch.float32 in [0, 1]
```

Use `ImageLoadType.RGB8` or `GRAY8` when you explicitly want `uint8` tensors in `[0, 255]`.

### Convert between NumPy-style images and tensors

Kornia also exposes helpers for moving between HWC NumPy arrays and channels-first tensors:

```python
import numpy as np
from kornia.image import image_to_tensor, tensor_to_image

np_image = np.zeros((64, 64, 3), dtype=np.uint8)
tensor = image_to_tensor(np_image, keepdim=False)  # (1, 3, 64, 64)
round_trip = tensor_to_image(tensor)

print(tensor.shape)
print(round_trip.shape)
```

This is useful at dataset boundaries. Stay in tensor form after conversion if you want Kornia ops to remain differentiable.

## Configuration Notes

- No auth or credentials are required.
- The biggest configuration choice is your PyTorch runtime: CPU-only vs CUDA/MPS build, plus matching device placement in code.
- Kornia examples and most operator docs assume tensor inputs, not PIL images.
- Prefer explicit module imports such as `kornia.augmentation`, `kornia.color`, `kornia.geometry.transform`, `kornia.io`, and `kornia.image` instead of guessing top-level aliases.

## Common Pitfalls

- Install `torch` first. Kornia depends on PyTorch and follows its device and dtype rules.
- Do not pass HWC arrays directly into tensor operators. Convert to `(C, H, W)` or `(B, C, H, W)` first.
- Watch value ranges. Many image functions expect float tensors in `[0, 1]`, while some I/O helpers can return `uint8` in `[0, 255]`.
- `kornia.io` is optional. If `kornia_rs` is not installed, do not assume `K.io.load_image(...)` is available in every environment.
- Mixing CPU tensors, CUDA tensors, or mismatched dtypes inside one pipeline will fail just like ordinary PyTorch code.
- Converting tensors back to NumPy for intermediate processing breaks gradient flow and often creates unnecessary host-device copies.

## Version-Sensitive Notes For 0.8.2

- As of March 12, 2026, PyPI lists `kornia 0.8.2`, so the version used here matches the current package release.
- The docs URL `https://kornia.readthedocs.io/en/latest/` currently resolves to the same 0.8.2 documentation surfaced under the stable docs tree, so `https://kornia.readthedocs.io/en/stable/` is the cleaner canonical root for this doc.
- The latest landing pages advertise newer multi-framework helpers such as `to_numpy()`, `to_tensorflow()`, and `to_jax()`, but the core installation and operator docs for `0.8.2` still center on PyTorch tensors. For coding agents, PyTorch-first usage is still the safest default.
- The official `kornia.io` page still documents `kornia_rs` as a separate package with Linux-only support, so treat Rust-backed file I/O as optional rather than a baseline assumption.

## Official Sources

- PyPI package: `https://pypi.org/project/kornia/`
- PyPI JSON metadata: `https://pypi.org/pypi/kornia/json`
- Docs root: `https://kornia.readthedocs.io/en/latest/`
- Canonical docs root used here: `https://kornia.readthedocs.io/en/stable/`
- Introduction: `https://kornia.readthedocs.io/en/stable/get-started/introduction.html`
- Installation: `https://kornia.readthedocs.io/en/stable/get-started/installation.html`
- Augmentations: `https://kornia.readthedocs.io/en/stable/applications/image_augmentations.html`
- Image I/O: `https://kornia.readthedocs.io/en/stable/io.html`
- Image conversion helpers: `https://kornia.readthedocs.io/en/stable/image.html`
- GitHub releases: `https://github.com/kornia/kornia/releases`
