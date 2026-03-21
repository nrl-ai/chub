---
name: package
description: "Pillow package guide for Python image processing, formats, and installation-dependent features"
metadata:
  languages: "python"
  versions: "12.1.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "pillow,pil,images,imaging,python"
---

# Pillow Python Package Guide

## Golden Rule

Install `Pillow`, but import from `PIL`:

```bash
python3 -m pip install --upgrade Pillow
```

```python
from PIL import Image
```

Do not use `import pillow`. Pillow is the maintained fork of PIL, but the runtime import namespace is still `PIL`.

If an old `PIL` package is installed in the same environment, remove it before installing Pillow. The upstream docs say Pillow and PIL cannot co-exist.

## Installation

### pip

```bash
python3 -m pip install --upgrade pip
python3 -m pip install --upgrade Pillow
```

### uv

```bash
uv add Pillow
```

### Poetry

```bash
poetry add Pillow
```

### Optional companion packages

The official install docs call out these optional packages:

```bash
python3 -m pip install --upgrade defusedxml olefile
```

- `defusedxml`: lets Pillow read XMP data
- `olefile`: lets Pillow read FPX and MIC images

### Source builds and native libraries

Official wheels usually include support for common formats. If you compile Pillow from source, format support depends on system libraries:

- Required by default: `zlib` and `libjpeg`
- Common optional libraries: `libtiff`, `libfreetype`, `littlecms2`, `libwebp`, `openjpeg`, `libavif`
- Text layout features need `libraqm` plus its dependencies
- Official binaries do not ship with `libimagequant`

If a project depends on a format such as WebP, AVIF, JPEG 2000, or advanced font layout, verify the compiled feature set after installation.

## Initialize And Verify Features

Use `PIL.features` to inspect what your current build can actually do:

```python
from PIL import features

print("webp:", features.check_module("webp"))
print("avif:", features.check_module("avif"))
print("jpg_2000:", features.check_codec("jpg_2000"))
print("raqm:", features.check_feature("raqm"))
```

For a full report:

```bash
python3 -m PIL
```

This matters because two machines can have the same `Pillow==12.1.1` version but different codec or text-layout support.

## Core Usage

### Open, inspect, and save images

```python
from pathlib import Path
from PIL import Image

source = Path("input.png")
target = Path("output.jpg")

with Image.open(source) as img:
    print(img.format, img.size, img.mode)

    rgb = img.convert("RGB")
    rgb.save(target, format="JPEG", quality=90, optimize=True)
```

Notes:

- `Image.open()` raises `OSError` if the file cannot be opened
- `open()` identifies an image from file contents, not the filename
- `save()` uses the destination filename extension unless you pass `format=...`

### Resize safely for thumbnails and web output

```python
from PIL import Image

with Image.open("photo.webp") as img:
    preview = img.copy()
    preview.thumbnail((512, 512))
    preview.save("photo-preview.webp")

with Image.open("photo.webp") as img:
    resized = img.resize((1200, 800), Image.Resampling.LANCZOS)
    resized.save("photo-1200.webp")
```

Use `thumbnail()` when you want to preserve aspect ratio and cap the maximum size. It modifies the image in place, so copy first if you still need the original object.

### Crop, paste, and compose

```python
from PIL import Image

with Image.open("base.png") as base, Image.open("overlay.png") as overlay:
    card = base.convert("RGBA")
    badge = overlay.convert("RGBA")

    region = card.crop((40, 40, 360, 240))
    region.save("region.png")

    card.alpha_composite(badge, dest=(24, 24))
    card.save("composited.png")
```

For masked paste workflows, convert to `RGBA` first so transparency behaves predictably.

### Create images from scratch

```python
from PIL import Image, ImageDraw

img = Image.new("RGB", (400, 200), "white")
draw = ImageDraw.Draw(img)
draw.rectangle((20, 20, 380, 180), outline="black", width=3)
draw.text((40, 80), "Generated with Pillow", fill="black")
img.save("generated.png")
```

### Read from bytes or file-like objects

```python
import io
from pathlib import Path
from PIL import Image

raw_bytes = Path("input.png").read_bytes()

with Image.open(io.BytesIO(raw_bytes)) as img:
    print(img.size)
```

When using a file-like object, keep it open until Pillow has loaded the data. Pillow may call `seek()` while reading headers or pixel data.

### Convert NumPy arrays to images

```python
import numpy as np
from PIL import Image

arr = np.zeros((128, 128, 3), dtype=np.uint8)
arr[:, :, 0] = 255

img = Image.fromarray(arr)
img.save("red.png")
```

`Image.fromarray()` is the standard bridge from NumPy-style image buffers into Pillow objects.

### Work with multiframe images

```python
from PIL import Image, ImageSequence

with Image.open("animation.gif") as img:
    for i, frame in enumerate(ImageSequence.Iterator(img)):
        frame.convert("RGBA").save(f"frame-{i}.png")
```

If you need to keep additional frames available, do not assume the underlying file can be closed after the first `load()`.

## Config, Environment, And Safety Controls

Pillow has no credential setup. The practical runtime controls are feature availability and resource limits.

### Decompression bomb protection

Pillow warns when an image exceeds `Image.MAX_IMAGE_PIXELS` and raises `DecompressionBombError` above twice that threshold.

```python
import warnings
from PIL import Image

warnings.simplefilter("error", Image.DecompressionBombWarning)
```

Only disable the pixel limit if the input is trusted and you understand the memory cost:

```python
from PIL import Image

Image.MAX_IMAGE_PIXELS = None
```

### Deterministic file handling

Use a context manager or call `close()` yourself. For single-frame images, Pillow can close its underlying file after `load()`, but multi-frame images may need the file to stay open for `seek()`.

If you need the image after the context manager exits, load or copy it inside the `with` block.

## Common Pitfalls

- Install name and import name differ: `pip install Pillow`, `from PIL import Image`
- Legacy PIL and Pillow cannot co-exist in one environment
- `thumbnail()` mutates in place; use `copy()` if you still need the original size
- JPEG does not support alpha; convert `RGBA` or `P` images to `RGB` before saving as JPEG
- `save()` infers format from the output extension unless `format=` is provided explicitly
- Copies and derived images lose file-format-specific state like `format` and `fp`
- A closed file-like object can break delayed image loading; keep binary streams open until data is loaded
- Feature support is build-dependent; check WebP, AVIF, JPEG 2000, and Raqm explicitly on production hosts

## Version-Sensitive Notes For 12.1.x

- `12.1.1` is a patch release from `2026-02-11`
- The `12.1.1` release fixes an out-of-bounds write issue triggered by crafted PSD files; the upstream note says this affects Pillow `>=10.3.0`
- `12.1.0` deprecated `Image.getdata()`; prefer `Image.get_flattened_data()`
- Pillow uses semantic versioning and quarterly main releases; patch releases are reserved for security, installation, or critical bug fixes
- The current PyPI metadata for `12.1.1` requires Python `>=3.10`

## Official Sources

- Docs root: `https://pillow.readthedocs.io/en/stable/`
- Package index: `https://pypi.org/project/pillow/`
- Reference index: `https://pillow.readthedocs.io/en/stable/reference/`
- Release notes: `https://pillow.readthedocs.io/en/stable/releasenotes/`
