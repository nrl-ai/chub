---
name: package
description: "opencv-python package guide for cv2 installs, image and video workflows, headless variants, and common runtime pitfalls"
metadata:
  languages: "python"
  versions: "4.13.0.92"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "opencv,opencv-python,cv2,computer-vision,image-processing,video"
---

# opencv-python Python Package Guide

## What It Is

`opencv-python` is the PyPI wheel distribution for OpenCV's Python bindings. You install the package as `opencv-python`, but you import it as `cv2`.

Use it when you need:

- image loading, writing, resizing, drawing, and color conversion
- camera and video-file capture
- classic computer vision features from core OpenCV modules
- prebuilt CPU-only wheels instead of compiling OpenCV from source

The official wheel project is for CPU-only builds. If you need CUDA or other custom build options, you need a source build instead of the stock PyPI wheels.

## Version Covered

- Package: `opencv-python`
- Ecosystem: `pypi`
- Version: `4.13.0.92`
- Release date on PyPI: `2026-02-05`
- Import name: `cv2`
- Registry: https://pypi.org/project/opencv-python/
- Docs root used for this guide: https://docs.opencv.org/4.x/
- Packaging repository: https://github.com/opencv/opencv-python

## Install

Prefer an isolated virtual environment.

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip setuptools wheel
python -m pip install opencv-python==4.13.0.92
```

Windows PowerShell:

```powershell
py -m venv .venv
.venv\Scripts\Activate.ps1
py -m pip install --upgrade pip setuptools wheel
py -m pip install opencv-python==4.13.0.92
```

### Choose Exactly One OpenCV Wheel Variant

OpenCV publishes four related PyPI packages that all provide the same `cv2` import namespace. Install exactly one of them in a given environment:

- `opencv-python`: core modules with GUI/backends
- `opencv-contrib-python`: core + contrib modules
- `opencv-python-headless`: no GUI/backends, better for servers and CI
- `opencv-contrib-python-headless`: contrib + headless

If you install multiple variants into the same environment, uninstall them all and reinstall only the one you actually want.

### When To Use Headless

Use `opencv-python-headless` instead of `opencv-python` when:

- you are running in Docker, CI, or a server environment
- you do not need `cv.imshow`, window creation, or other HighGUI features
- you want to avoid desktop GUI dependencies

If your code needs `cv.imshow(...)`, start from `opencv-python`, not a headless package.

## Initialize And Verify

```python
import cv2 as cv

print(cv.__version__)
print(cv.getBuildInformation().splitlines()[0])
```

`cv.getBuildInformation()` is the fastest way to inspect what codecs, modules, and third-party libraries are actually compiled into your installed wheel.

## Core Usage

### Load, Transform, And Save An Image

`cv.imread()` returns a NumPy array, or `None` if the file cannot be read. Check that before continuing.

```python
from pathlib import Path

import cv2 as cv

image_path = Path("input.jpg")
img = cv.imread(str(image_path))
if img is None:
    raise FileNotFoundError(f"Could not read {image_path}")

gray = cv.cvtColor(img, cv.COLOR_BGR2GRAY)
small = cv.resize(gray, (640, 480))

ok = cv.imwrite("output-gray.png", small)
if not ok:
    raise RuntimeError("cv.imwrite failed")
```

Important details:

- OpenCV images use BGR channel order by default, not RGB.
- `cv.imread()` returning `None` usually means the path is wrong or the codec/backend cannot decode that file.
- `cv.imwrite()` returns a boolean; treat `False` as a real failure.

### Display A Window In Desktop Environments

Only do this with a non-headless wheel:

```python
import cv2 as cv

img = cv.imread("input.jpg")
if img is None:
    raise FileNotFoundError("input.jpg")

cv.imshow("preview", img)
cv.waitKey(0)
cv.destroyAllWindows()
```

For headless-safe code, save the output with `cv.imwrite(...)` instead of calling `cv.imshow(...)`.

### Capture Frames From A Camera

Use `cap.isOpened()` and check the boolean from `cap.read()` on every iteration.

```python
import cv2 as cv

cap = cv.VideoCapture(0)
if not cap.isOpened():
    raise RuntimeError("Cannot open camera")

try:
    while True:
        ret, frame = cap.read()
        if not ret:
            raise RuntimeError("Failed to read frame from camera")

        gray = cv.cvtColor(frame, cv.COLOR_BGR2GRAY)
        cv.imshow("camera", gray)

        if cv.waitKey(1) == ord("q"):
            break
finally:
    cap.release()
    cv.destroyAllWindows()
```

If camera or video-file capture behaves differently across machines, inspect `cv.getBuildInformation()` and verify the local multimedia stack. The upstream video tutorial explicitly notes that video capture problems are often caused by FFmpeg or GStreamer installation issues.

### Use Built-In Haar Cascade Data

The wheel packages include Haar cascade files. The package exposes their location through `cv2.data.haarcascades`.

```python
import cv2 as cv

cascade = cv.CascadeClassifier(
    cv.data.haarcascades + "haarcascade_frontalface_default.xml"
)
if cascade.empty():
    raise RuntimeError("Failed to load Haar cascade")
```

This is the most reliable way to reference the bundled cascade files without hard-coding a site-packages path.

## Configuration And Environment Notes

`opencv-python` does not use API credentials or service authentication. Configuration is mostly about:

- selecting the correct wheel variant
- using the intended Python interpreter and virtual environment
- making sure the input files, camera devices, and codecs you rely on exist at runtime
- understanding what your installed wheel was built with

### IDE Interpreter Mismatch

If `import cv2` works in your terminal but fails in VS Code, PyCharm, or another IDE, the IDE is probably using a different interpreter than the one where you installed the package.

Verify both of these:

```bash
python -c "import sys, cv2; print(sys.executable); print(cv2.__version__)"
python -m pip show opencv-python
```

### Runtime Build Inspection

When behavior depends on compiled backends or bundled modules, inspect the wheel instead of guessing:

```python
import cv2 as cv

info = cv.getBuildInformation()
print(info)
```

This is especially useful for debugging:

- video I/O backend availability
- codec support
- enabled modules
- platform-specific wheel differences

## Common Pitfalls

### Package Name And Import Name Differ

Install:

```bash
python -m pip install opencv-python
```

Import:

```python
import cv2
```

Do not try `import opencv_python`.

### Mixing Multiple OpenCV Wheel Variants

If both `opencv-python` and `opencv-python-headless` are installed, or if you mix base and contrib variants, import behavior becomes unreliable because all of them provide the same `cv2` namespace.

Start over cleanly if needed:

```bash
python -m pip uninstall -y opencv-python opencv-python-headless opencv-contrib-python opencv-contrib-python-headless
python -m pip install opencv-python==4.13.0.92
```

### Expecting CUDA In The PyPI Wheels

The official wheel project provides prebuilt CPU-only packages. If your task requires CUDA-enabled OpenCV, the PyPI wheel is the wrong artifact; build from source with the options you need.

### `cv.imshow()` In Containers Or Servers

If you run on CI, Docker, or a machine without GUI libraries, `cv.imshow()` and other window APIs are the wrong default. Use a headless wheel and write images to disk instead.

### BGR vs RGB Confusion

OpenCV images are BGR by default. If you pass frames directly into libraries that expect RGB, convert them first:

```python
rgb = cv.cvtColor(img, cv.COLOR_BGR2RGB)
```

### Slow Per-Pixel Python Loops

The upstream tutorials explicitly warn that direct per-pixel NumPy loops are slow and that `cv.split()` is relatively costly. Prefer vectorized NumPy indexing or OpenCV operations over Python-level loops.

### Old Packaging Tooling Or Unsupported Wheels

If `pip` starts building from source unexpectedly, or you see "No matching distribution found":

- upgrade `pip`, `setuptools`, and `wheel`
- verify your Python version and platform match a published wheel
- try a mainstream interpreter version in a fresh virtual environment

## Version-Sensitive Notes For `4.13.0.92`

- PyPI lists `4.13.0.92` as the current `opencv-python` release as of `2026-02-05`.
- The OpenCV docs root `https://docs.opencv.org/4.x/` is a rolling `4.x` site and currently renders `OpenCV 4.14.0-pre`, generated on `2026-03-02`.
- That means the official docs site can be slightly ahead of the specific PyPI wheel version pinned in this guide.
- For code that depends on newly added APIs or backend behavior, prefer checking `cv.__version__` and `cv.getBuildInformation()` in the target environment before assuming the rolling `4.x` docs exactly match your installed wheel.
- The current official pip-install tutorial states that the OpenCV team maintains the PyPI packages; Conda and vendor-specific builds may differ from the official wheels.

## Official Sources Used

- PyPI package page: https://pypi.org/project/opencv-python/
- OpenCV pip install guide: https://docs.opencv.org/4.x/db/dd1/tutorial_py_pip_install.html
- OpenCV Python tutorials root: https://docs.opencv.org/4.x/d6/d00/tutorial_py_root.html
- OpenCV basic image operations tutorial: https://docs.opencv.org/4.x/d3/df2/tutorial_py_basic_ops.html
- OpenCV video capture tutorial: https://docs.opencv.org/4.x/dd/d43/tutorial_py_video_display.html
- OpenCV core utility reference (`cv.getBuildInformation`): https://docs.opencv.org/4.x/db/de0/group__core__utils.html
- OpenCV packaging repository: https://github.com/opencv/opencv-python
- OpenCV 4.13.0 release page: https://github.com/opencv/opencv/releases/tag/4.13.0
