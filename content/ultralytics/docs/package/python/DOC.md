---
name: package
description: "Ultralytics package guide for Python: install, predict, train, export, and configure YOLO workflows"
metadata:
  languages: "python"
  versions: "8.4.21"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "ultralytics,yolo,computer-vision,object-detection,training,inference,export"
---

# ultralytics Python Package Guide

## Golden Rule

Use the official `ultralytics` package for Python YOLO workflows, and prefer the current Ultralytics docs over older YOLOv8 blog posts or examples. In the current upstream docs for `8.4.21`, the standard Python entry point is `from ultralytics import YOLO`, and current examples center on the YOLO26 model family.

## Installation

### Recommended install

```bash
pip install ultralytics==8.4.21
```

### Upgrade to the latest stable release

```bash
pip install -U ultralytics
```

### Headless servers and containers

If you are running on a VM, CI runner, Docker container, or other environment without display libraries, use the headless package variant:

```bash
pip install ultralytics-opencv-headless
```

This keeps the same API but avoids OpenCV GUI dependencies that can cause `libGL` and display-related failures.

### CUDA and PyTorch note

Ultralytics documents `Python>=3.8` and `PyTorch>=1.8`. If you need a CUDA-specific setup, install the PyTorch build for your OS and CUDA version first, then install `ultralytics`.

## Initialize and run your first prediction

```python
from ultralytics import YOLO

# Load a pretrained model
model = YOLO("yolo26n.pt")

# Run inference on an image URL
results = model("https://ultralytics.com/images/bus.jpg")

for result in results:
    print(result.boxes.xyxy)
    print(result.boxes.conf)
    print(result.boxes.cls)
```

You can also call `predict()` explicitly when you want to pass more arguments:

```python
from ultralytics import YOLO

model = YOLO("yolo26n.pt")
model.predict(
    source="https://ultralytics.com/images/bus.jpg",
    save=True,
    imgsz=320,
    conf=0.25,
)
```

## Core Python workflows

### Build from YAML or load pretrained weights

```python
from ultralytics import YOLO

# New model from YAML
model = YOLO("yolo26n.yaml")

# Recommended starting point for most training workflows
model = YOLO("yolo26n.pt")
```

### Train on a dataset

```python
from ultralytics import YOLO

model = YOLO("yolo26n.pt")
results = model.train(
    data="path/to/dataset.yaml",
    epochs=100,
    imgsz=640,
    device="cpu",
)
```

The docs also support:

- `device=[0, 1]` for explicit multi-GPU training
- `device=-1` or `device=[-1, -1]` for auto-selecting the most idle GPU(s)
- `device="mps"` for Apple silicon

### Resume an interrupted run

```python
from ultralytics import YOLO

model = YOLO("path/to/last.pt")
results = model.train(resume=True)
```

Resume requires a real saved checkpoint such as `last.pt`; upstream docs note that you need at least one completed epoch for resume to work.

### Validate

```python
from ultralytics import YOLO

model = YOLO("yolo26n.yaml")
model.train(data="path/to/dataset.yaml", epochs=5)
metrics = model.val()
```

### Predict on larger sources without loading everything into memory

For directories, long videos, streams, webcams, or segmentation-heavy runs, use `stream=True` so prediction returns a generator instead of a full in-memory list:

```python
from ultralytics import YOLO

model = YOLO("yolo26n.pt")
results = model.predict(source="path/to/video.mp4", stream=True)

for result in results:
    print(result.boxes.xyxy)
```

### Export for deployment

```python
from ultralytics import YOLO

model = YOLO("path/to/best.pt")
onnx_path = model.export(format="onnx", dynamic=True)
```

Ultralytics supports many export targets. For coding-agent work, ONNX is usually the safest first deployment format because it is well documented and the upstream export examples are straightforward.

## CLI parity

The documented command format is:

```bash
yolo TASK MODE ARGS
```

Examples:

```bash
yolo detect train data=path/to/dataset.yaml model=yolo26n.pt epochs=100 imgsz=640
yolo predict model=yolo26n.pt source='https://ultralytics.com/images/bus.jpg'
yolo export model=path/to/best.pt format=onnx dynamic=True
yolo settings
```

The Python API and CLI use the same core modes and largely the same argument names.

## Configuration and auth

Ultralytics exposes a settings manager for local experiment paths and optional platform integrations.

### Inspect settings

```python
from ultralytics import settings

print(settings)
print(settings["runs_dir"])
```

### Update settings

```python
from ultralytics import settings

settings.update({
    "datasets_dir": "/absolute/path/to/datasets",
    "runs_dir": "/absolute/path/to/runs",
})
```

### CLI settings management

```bash
yolo settings
yolo settings runs_dir='/absolute/path/to/runs'
yolo settings reset
```

### What matters in practice

- `datasets_dir` controls where datasets are stored
- `weights_dir` controls where model weights are stored
- `runs_dir` controls where experiment outputs are written
- `sync` controls analytics/crash syncing to Ultralytics Platform
- `api_key` is the Ultralytics Platform API key field

Local prediction, validation, training, and export do not require an API key. Auth only becomes relevant if you are using Ultralytics Platform features or other enabled integrations.

## Common pitfalls

- Current upstream docs and examples use YOLO26 model names like `yolo26n.pt` and `yolo26n.yaml`. Do not blindly mix those with older YOLOv8-era snippets unless your project already depends on older checkpoints.
- `predict()` returns a list by default. On large folders, streams, or long videos, use `stream=True` to avoid unnecessary memory growth.
- On headless Linux servers, prefer `ultralytics-opencv-headless` instead of debugging GUI-linked OpenCV errors after the fact.
- If outputs or dataset lookups are landing in unexpected locations, inspect `settings["datasets_dir"]`, `settings["weights_dir"]`, and `settings["runs_dir"]` first.
- Resume training only from a real checkpoint such as `last.pt`, not from an arbitrary exported model.
- Export behavior is format-specific. If the deployment target needs variable input sizes, prefer `dynamic=True` for ONNX export.

## Version-sensitive notes for 8.4.21

- `ultralytics 8.4.21` was released on `2026-03-05`.
- The official `v8.4.21` release notes call out a Rockchip RKNN export path fix as the main package change in this release.
- The same release also mentions improved Ray Tune trial isolation and clearer YOLO26 optimizer guidance.
- The current official docs emphasize YOLO26 for new examples, while some release notes and older examples still reference YOLOv8-family compatibility. For new code, follow the current docs; for existing projects, match the checkpoint family already in use.

## Official Sources

- Quickstart: `https://docs.ultralytics.com/quickstart/`
- Python usage: `https://docs.ultralytics.com/usage/python/`
- Configuration: `https://docs.ultralytics.com/usage/cfg/`
- Training mode: `https://docs.ultralytics.com/modes/train/`
- Prediction mode: `https://docs.ultralytics.com/modes/predict/`
- Export mode: `https://docs.ultralytics.com/modes/export/`
- API reference root: `https://docs.ultralytics.com/reference/`
- PyPI version page: `https://pypi.org/project/ultralytics/8.4.21/`
- Release notes: `https://github.com/ultralytics/ultralytics/releases/tag/v8.4.21`
