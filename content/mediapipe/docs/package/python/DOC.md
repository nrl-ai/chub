---
name: package
description: "MediaPipe Python package guide for on-device vision, text, and audio tasks"
metadata:
  languages: "python"
  versions: "0.10.32"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "mediapipe,google-ai-edge,vision,text,audio,ml,computer-vision"
---

# MediaPipe Python Package Guide

## What This Package Is For

`mediapipe` is the Python package for MediaPipe Tasks: prebuilt on-device ML APIs for vision, text, and audio workloads. In current upstream docs, the practical Python entry point is the Tasks API under `mediapipe.tasks.python`, not the older `mp.solutions` examples.

Use this package when you need local inference in Python for tasks such as object detection, image classification, hand or pose landmarks, text classification, text embedding, language detection, or audio classification.

## Package Snapshot

- Ecosystem: `pypi`
- Package: `mediapipe`
- Import root: `import mediapipe as mp`
- Version covered: `0.10.32`
- Upstream docs root: `https://ai.google.dev/edge/mediapipe/`
- Registry page: `https://pypi.org/project/mediapipe/`
- Auth: none; inference runs locally

## Installation

Pin the package when you want reproducible agent output:

```bash
python -m pip install "mediapipe==0.10.32"
```

Quick import smoke test:

```bash
python - <<'PY'
import mediapipe as mp
print(mp.__version__)
PY
```

### Python and platform constraints

The official Python setup guide documents support for Python `3.9` through `3.12` on Windows, Mac, Linux, and Raspberry Pi OS 64-bit.

For `0.10.32`, the published PyPI artifacts are more specific:

- `win_amd64`
- `manylinux_2_28_x86_64`
- `macosx_11_0_arm64`
- no source distribution is published on PyPI

If `pip install mediapipe` fails on another architecture or on an older Linux baseline, use the official build-from-source wheel instructions instead of assuming the package should install everywhere.

## Import Layout

The setup guide and task pages use this import structure:

```python
import mediapipe as mp
from mediapipe.tasks.python import vision, text, audio
```

Common aliases from the official examples:

```python
BaseOptions = mp.tasks.BaseOptions
VisionRunningMode = mp.tasks.vision.RunningMode
```

## Core Workflow

Most Python task integrations follow the same shape:

1. Install `mediapipe`.
2. Download a task-compatible model file (`.task` or `.tflite`) from the task overview/model page.
3. Build `BaseOptions` with `model_asset_path` or `model_asset_buffer`.
4. Choose the domain module (`vision`, `text`, or `audio`) and a task-specific `...Options` class.
5. Pick the correct running mode: `IMAGE`, `VIDEO`, or `LIVE_STREAM`.
6. Create the task with `create_from_options(...)`.
7. Convert input data into MediaPipe objects and call the task method for that running mode.

### Minimal image task example

This example uses the vision Task API pattern shown in the official object detection guide:

```python
import mediapipe as mp
from mediapipe.tasks.python import vision

BaseOptions = mp.tasks.BaseOptions
ObjectDetector = vision.ObjectDetector
ObjectDetectorOptions = vision.ObjectDetectorOptions
RunningMode = vision.RunningMode

options = ObjectDetectorOptions(
    base_options=BaseOptions(
        model_asset_path="/absolute/path/to/lite-model_efficientdet_lite0_detection_metadata_1.tflite"
    ),
    running_mode=RunningMode.IMAGE,
    max_results=5,
    score_threshold=0.3,
)

image = mp.Image.create_from_file("/absolute/path/to/image.jpg")

with ObjectDetector.create_from_options(options) as detector:
    result = detector.detect(image)
    print(result)
```

### Live stream pattern

Use `LIVE_STREAM` only when you can provide timestamps and handle async callbacks:

```python
import mediapipe as mp
from mediapipe.tasks.python import vision

BaseOptions = mp.tasks.BaseOptions
ObjectDetector = vision.ObjectDetector
ObjectDetectorOptions = vision.ObjectDetectorOptions
RunningMode = vision.RunningMode

def on_result(result, output_image, timestamp_ms):
    print(timestamp_ms, result)

options = ObjectDetectorOptions(
    base_options=BaseOptions(model_asset_path="/absolute/path/to/model.tflite"),
    running_mode=RunningMode.LIVE_STREAM,
    result_callback=on_result,
)

with ObjectDetector.create_from_options(options) as detector:
    # detector.detect_async(mp_image, frame_timestamp_ms)
    pass
```

## Models and Setup

Installing the package is not enough for most tasks. The task guides expect you to download a compatible model and point `BaseOptions` at it:

```python
base_options = mp.tasks.BaseOptions(model_asset_path="/absolute/path/to/model.task")
```

`BaseOptions` also supports `model_asset_buffer` when you already have the model bytes in memory.

Choose the model from the overview page for the exact task you are using. Do not mix a random `.tflite` or `.task` file with a task API that expects packaged metadata from a specific task family.

## Data Preparation

The Python guides use `mediapipe.Image` as the common container for vision inputs.

From file:

```python
mp_image = mp.Image.create_from_file("/absolute/path/to/image.jpg")
```

From a NumPy array:

```python
mp_image = mp.Image(image_format=mp.ImageFormat.SRGB, data=numpy_image)
```

For video or webcam pipelines, the official guides use OpenCV to load frames and then wrap each frame as an `mp.Image`. In practice, make sure the pixel format you pass matches the `image_format` you declare.

## Configuration Notes

There is no remote service configuration or API key flow. Configuration is local and task-specific.

Common settings you will see across task guides:

- `base_options`: model path or bytes
- `running_mode`: `IMAGE`, `VIDEO`, or `LIVE_STREAM`
- `result_callback`: required for async live-stream usage
- result limits or thresholds such as `max_results` and `score_threshold`
- task-specific filters such as category allowlists or denylists

For `VIDEO` and `LIVE_STREAM`, you must pass timestamps alongside frames. The task guides also note an important execution difference:

- image/video methods block until inference completes
- live-stream methods return immediately and deliver results through the callback

## Legacy vs Current API

Prefer the Task APIs under `mediapipe.tasks.python`.

The official MediaPipe Solutions guide says support for the listed legacy solutions ended on **March 1, 2023**, and that the old repository and binaries continue only on an as-is basis. Upgraded replacements include:

- `Hands` -> `HandLandmarker`
- `Pose` -> `PoseLandmarker`
- `Face Detection` -> `FaceDetector`
- `Face Mesh`/`Iris` -> `FaceLandmarker`
- `Selfie Segmentation` -> image segmentation tasks

If you find code using `mp.solutions.hands`, `mp.solutions.pose`, or similar, treat it as legacy unless you have a strong compatibility reason to keep it.

## Common Pitfalls

- Missing model file: many task APIs will import correctly but still fail at runtime until you provide a compatible `.task` or `.tflite` asset.
- Wrong running mode: `detect()`, `detect_for_video()`, and `detect_async()` are mode-specific. Match the method to the mode used when creating the task.
- Missing timestamps: `VIDEO` and `LIVE_STREAM` calls require frame timestamps.
- Dropped live frames: the task guides note that live-stream tasks can ignore new input frames when the task is still busy processing the previous one.
- Platform mismatch: the PyPI package does not publish wheels for every Python/OS/architecture combination. Check the wheel list before assuming install problems are user error.
- Legacy examples: many blog posts still use `mp.solutions.*`; prefer the current task guides and API reference.

## Version-Sensitive Notes for `0.10.32`

- PyPI lists `0.10.32` as the current release, uploaded on `2026-01-22`.
- GitHub releases list `MediaPipe v0.10.32` as the latest release as of `2026-03-12`.
- The Google AI Edge task guides are still labeled "MediaPipe Solutions Preview", and several Python setup/task pages were last updated in 2024 even though the package version is newer.

Treat the docs as the canonical API shape, but verify exact runtime behavior against the package you installed when copying older examples or debugging installation issues.

## Official Links

- Docs root: https://ai.google.dev/edge/mediapipe/
- Tasks overview: https://ai.google.dev/edge/mediapipe/solutions/tasks
- Python setup guide: https://ai.google.dev/edge/mediapipe/solutions/setup_python
- Solutions guide: https://ai.google.dev/edge/mediapipe/solutions/guide
- Python API reference index: https://ai.google.dev/edge/mediapipe/api/solutions
- Build Python wheel: https://ai.google.dev/edge/mediapipe/solutions/build_python
- PyPI: https://pypi.org/project/mediapipe/
- Releases: https://github.com/google-ai-edge/mediapipe/releases
