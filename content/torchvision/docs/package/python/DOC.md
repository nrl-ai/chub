---
name: package
description: "torchvision package guide for Python computer vision workflows with models, datasets, transforms, and image I/O"
metadata:
  languages: "python"
  versions: "0.25.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "torchvision,pytorch,computer-vision,images,transforms,datasets,models"
---

# torchvision Python Package Guide

## Golden Rule

Install `torch` and `torchvision` together, use pretrained-model `weights=` enums plus `weights.transforms()` for inference, and prefer `torchvision.transforms.v2` for new preprocessing pipelines. If `torch` and `torchvision` wheels are out of sync, image ops and custom C++ operators such as NMS are the first things that usually break.

## Install

For CPU-only environments and many local macOS setups, the basic install is:

```bash
python -m pip install torch torchvision
```

If your project is already pinned, install both packages together instead of upgrading `torchvision` by itself:

```bash
python -m pip install "torch==<exact-version>" "torchvision==0.25.0"
```

For Linux/Windows GPU or ROCm builds, generate the exact command from the official selector:

- https://pytorch.org/get-started/locally/

Verify the environment immediately after install:

```python
import torch
import torchvision

print(torch.__version__)
print(torchvision.__version__)
```

If the import step fails with missing operators or image extension errors, reinstall a matching `torch` and `torchvision` pair in a clean virtual environment.

## Setup Pattern

Most projects use these pieces:

```python
import torch
from torch.utils.data import DataLoader
from torchvision import datasets
from torchvision.transforms import v2
from torchvision.models import resnet50, ResNet50_Weights
```

Use `torch.inference_mode()` for inference, and put the model and tensors on the same device:

```python
device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
```

## Core Usage

### Run inference with pretrained weights

Use weight enums instead of hard-coded preprocessing constants. The enum carries the canonical normalization, resize, crop, and category metadata for that checkpoint.

```python
from PIL import Image
import torch
from torchvision.models import resnet50, ResNet50_Weights

weights = ResNet50_Weights.DEFAULT
model = resnet50(weights=weights).to("cpu").eval()
preprocess = weights.transforms()

image = Image.open("cat.jpg").convert("RGB")
batch = preprocess(image).unsqueeze(0)

with torch.inference_mode():
    probs = model(batch).softmax(dim=1)

top_idx = int(probs.argmax(dim=1))
label = weights.meta["categories"][top_idx]
score = float(probs[0, top_idx])

print(label, score)
```

Notes:

- Call `.eval()` before inference so dropout and batch norm behave correctly.
- `weights.transforms()` already handles resize/crop/normalization for the selected checkpoint. Do not normalize the same image twice.
- For GPU inference, move both `model` and `batch` to the same CUDA device.

### Build training transforms with `transforms.v2`

The official docs now treat `v2` as the modern transforms API. Use it for new pipelines, especially when you may later handle bounding boxes, masks, or videos.

```python
import torch
from torchvision.transforms import v2

train_transforms = v2.Compose(
    [
        v2.ToImage(),
        v2.RandomResizedCrop((224, 224), antialias=True),
        v2.RandomHorizontalFlip(p=0.5),
        v2.ToDtype(torch.float32, scale=True),
        v2.Normalize(
            mean=(0.485, 0.456, 0.406),
            std=(0.229, 0.224, 0.225),
        ),
    ]
)
```

For classification datasets, pass the transform as usual. For older detection/segmentation datasets, the docs expose `torchvision.datasets.wrap_dataset_for_transforms_v2` to adapt targets for `v2`.

### Load an image dataset

```python
import torch
from torch.utils.data import DataLoader
from torchvision.datasets import CIFAR10

# Reuse the v2 pipeline defined above.
train_dataset = CIFAR10(
    root="data/cifar10",
    train=True,
    download=True,
    transform=train_transforms,
)

train_loader = DataLoader(
    train_dataset,
    batch_size=64,
    shuffle=True,
    num_workers=4,
    pin_memory=torch.cuda.is_available(),
)

images, labels = next(iter(train_loader))
print(images.shape, labels.shape)
```

Notes:

- Keep the dataset `root` stable across runs or you will redownload data into multiple cache trees.
- `download=True` is convenient for local development, but production or CI flows often pre-stage datasets instead.
- Some datasets download from third-party mirrors and have their own licenses or manual-download requirements.

### Fine-tune a pretrained classifier

```python
import torch.nn as nn
from torchvision.models import resnet18, ResNet18_Weights

model = resnet18(weights=ResNet18_Weights.DEFAULT)
model.fc = nn.Linear(model.fc.in_features, 10)
```

This pattern is common across many `torchvision.models` architectures: load weights, replace the task-specific head, and train only the layers you need.

### Decode images with `torchvision.io`

If you need a tensor directly from bytes on disk, use the `io` helpers:

```python
from torchvision.io import decode_image, read_file

image = decode_image(read_file("cat.jpg"))
print(image.shape, image.dtype)  # CHW uint8 tensor
```

This is useful when you want tensor-first ingestion without a PIL dependency in the read path.

## Configuration Notes

- Match the wheel family across `torch`, `torchvision`, Python version, OS, and accelerator backend.
- Use `weights.transforms()` for inference and your own explicit `v2.Compose(...)` pipeline for training.
- `pin_memory=True` is mainly useful when you are training on CUDA and moving batches from host RAM to the GPU.
- Many torchvision examples assume channels-first tensors shaped `[C, H, W]` or `[N, C, H, W]`.
- Detection and segmentation models often expect a list of images and per-image target dicts, not a single batched tensor and flat labels.

## Common Pitfalls

- `RuntimeError` around missing ops such as `torchvision::nms` usually means the `torch` and `torchvision` binaries do not match.
- Reusing blog-era `pretrained=True` snippets is brittle. Prefer the current `weights=` enums from the official docs.
- Mixing `weights.transforms()` with your own normalization step often double-normalizes inputs and hurts inference quality.
- Forgetting `.eval()` and `torch.inference_mode()` makes inference slower and can produce unstable results.
- Legacy `torchvision.transforms` examples often assume PIL images only. `transforms.v2` is the safer default for new work.
- The `torchvision.io` docs explicitly deprecate video decoding and encoding in favor of TorchCodec. Do not start new video pipelines on the legacy video APIs.

## Version-Sensitive Notes For `0.25.0`

- As of `2026-03-12`, the official stable torchvision docs root and the PyPI project page both show `0.25.0`.
- Upstream install metadata is not fully synchronized: the PyPI page's compatibility table still stops at `torch 2.9` / `torchvision 0.24`, and the PyTorch "Get Started" selector page still renders a `Stable (2.7.0)` default. Treat both as drift signals rather than authoritative pairing guidance for `0.25.0`.
- Because of that drift, the safest upgrade path is to install `torch` and `torchvision` together in a fresh environment, verify import success, and run a minimal model + image-op smoke test before rolling the upgrade into production.
- The `transforms.v2` docs and the stable models docs are current enough to use as the API baseline for new code.

## Official Source URLs

- Docs root: https://docs.pytorch.org/vision/stable/
- Models: https://docs.pytorch.org/vision/stable/models.html
- Datasets: https://docs.pytorch.org/vision/stable/datasets.html
- Transforms: https://docs.pytorch.org/vision/stable/transforms.html
- V2 start here: https://docs.pytorch.org/vision/stable/auto_examples/transforms/plot_transforms_getting_started.html
- V1 or V2 comparison: https://docs.pytorch.org/vision/stable/auto_examples/transforms/plot_v2_e2e.html
- I/O: https://docs.pytorch.org/vision/stable/io.html
- Install selector: https://pytorch.org/get-started/locally/
- PyPI: https://pypi.org/project/torchvision/
- Repository: https://github.com/pytorch/vision
