---
name: package
description: "imageio package guide for Python: read and write images, videos, volumes, and scientific formats with the v3 API and optional plugins"
metadata:
  languages: "python"
  versions: "2.37.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "imageio,images,video,numpy,io,tiff,gif,scientific-imaging"
---

# imageio Python Package Guide

## Golden Rule

Use the v3 API unless you are intentionally maintaining older code:

- Prefer `import imageio.v3 as iio`
- Use plugin extras for the formats you actually need
- Use `imageio.v2` only for legacy code that still relies on the older reader/writer behavior

`imageio` is a NumPy-first IO layer for images, videos, volumetric data, and many scientific file formats. The package is lightweight by default and picks up broader format support through optional plugins.

## Install

Pin the version your project expects:

```bash
python -m pip install "imageio==2.37.3"
```

Common alternatives:

```bash
uv add "imageio==2.37.3"
poetry add "imageio==2.37.3"
```

Optional extras matter in practice because format support is plugin-driven:

```bash
python -m pip install "imageio[pyav]==2.37.3"
python -m pip install "imageio[ffmpeg]==2.37.3"
python -m pip install "imageio[tifffile]==2.37.3"
python -m pip install "imageio[full]==2.37.3"
```

Practical guidance:

- Use `pyav` for modern video decoding/encoding work when possible.
- Use `ffmpeg` when you need the legacy ffmpeg plugin, webcam helpers, or existing ffmpeg-based examples.
- Use `tifffile` for TIFF-heavy scientific workflows.
- Use `full` only when you explicitly want broad optional dependency coverage.

Verify what your environment installed:

```bash
python -c "import imageio; print(imageio.__version__)"
```

## Initialize And Choose An API

Preferred import:

```python
import imageio.v3 as iio
```

Legacy compatibility import:

```python
import imageio.v2 as iio
```

Mental model:

- `imageio.v3` is the current high-level API.
- `imageio.v2` exposes the older API surface and wraps many calls through the v3 backend.
- Most common code paths start with `iio.imread(...)`, `iio.imwrite(...)`, `iio.imiter(...)`, `iio.improps(...)`, or `iio.immeta(...)`.

## Core Usage

### Read a single image into a NumPy array

```python
import imageio.v3 as iio

image = iio.imread("photo.png")
print(image.shape, image.dtype)
```

`imread()` returns an `ndarray`. For a normal RGB image, expect a shape like `(height, width, 3)`.

### Write an array to disk

```python
import imageio.v3 as iio
import numpy as np

image = np.zeros((128, 128, 3), dtype=np.uint8)
image[:, :, 0] = 255

iio.imwrite("red.png", image)
```

When writing to bytes or file-like objects, pass an explicit extension so imageio can pick the correct backend:

```python
import imageio.v3 as iio

payload = iio.imwrite("<bytes>", image, extension=".png")
```

### Iterate video or multi-frame data without loading everything at once

Use `imiter()` for frame-by-frame processing:

```python
import imageio.v3 as iio

for frame in iio.imiter("clip.mp4", plugin="pyav"):
    print(frame.shape)
```

This is usually the safer default for large videos, animated GIFs, and stacks because it avoids loading every frame into memory at once.

### Inspect standardized properties and raw metadata

```python
import imageio.v3 as iio

props = iio.improps("photo.jpg")
meta = iio.immeta("photo.jpg")

print(props.shape)
print(props.dtype)
print(meta)
```

Use `improps()` when you want a stable, plugin-independent summary. Use `immeta()` when you need format-specific metadata from the backend.

### Work with sample assets, URLs, zip paths, and bytes

`imageio` accepts more than ordinary filesystem paths:

```python
import imageio.v3 as iio

sample = iio.imread("imageio:chelsea.png")
remote = iio.imread("https://github.com/imageio/imageio-binaries/raw/master/images/chelsea.png")
```

In-memory streams also work:

```python
from io import BytesIO
import imageio.v3 as iio

with open("photo.jpg", "rb") as f:
    raw = f.read()

image = iio.imread(BytesIO(raw), extension=".jpg")
```

If you are pulling protected URLs or data from custom transports, fetch the bytes yourself with your HTTP client and then pass a file object or `BytesIO` to imageio.

### Use `imopen()` when you need explicit plugin control

`imopen()` is the lower-level entry point when the one-shot helpers are too limiting:

```python
import imageio.v3 as iio

with iio.imopen("clip.mp4", "r", plugin="pyav") as resource:
    frame0 = resource.read(index=0)
    props = resource.properties(index=0)
    print(frame0.shape, props.shape)
```

Reach for `imopen()` when you need:

- an explicitly chosen plugin
- repeated reads or writes through one opened resource
- lower-level access to backend-specific behavior

## Configuration And Environment

There is no auth model built into `imageio`. Configuration is mostly about plugin choice, transport constraints, and environment variables.

Important environment variables from the upstream docs:

- `IMAGEIO_FORMAT_ORDER`: overrides plugin priority for a format, for example `".tiff -TIFF +FEI"`
- `IMAGEIO_NO_INTERNET=1`: disables internet access, including automatic fetching of example images and remote resources
- `IMAGEIO_REQUEST_TIMEOUT=5.0`: changes the timeout for reading remote resources over the network

Practical setup guidance:

- In CI or sandboxed environments, set `IMAGEIO_NO_INTERNET=1` if tests must not reach the network.
- If backend auto-detection picks the wrong plugin, pass `plugin="..."` explicitly instead of relying on global ordering.
- For remote resources behind auth, proxies, signed URLs, or custom headers, do the network request outside imageio and pass the resulting bytes or stream.

## Common Pitfalls

- `imageio.v2` and `imageio.v3` are not interchangeable. New code should not mix both styles casually.
- Optional backends are not installed automatically. A missing plugin dependency often shows up as a runtime read/write failure for only one format family.
- For writes to `<bytes>` or arbitrary file-like objects, pass `extension=".png"` or similar so backend selection is deterministic.
- `imiter()` is usually better than `imread()` for large multi-frame resources. Loading every frame eagerly can consume a lot of memory.
- `immeta()` returns backend-specific metadata and can vary by plugin. Use `improps()` when you need a normalized shape/dtype summary.
- Remote URL support is convenient for public resources, not a full HTTP client abstraction.
- Some example pages still use `imageio:chelsea.png` sample assets or public GitHub URLs. Those patterns are fine for tests and demos, but production code should use your own asset source explicitly.

## Version-Sensitive Notes For 2.37.3

- `2.37.3` is the current version on PyPI as of `2026-03-12`, released on `2026-03-09`.
- The stable docs home and PyPI metadata now target Python `>=3.10`.
- The installation page in the stable docs still says `imageio` supports Python `3.5+`, which is stale relative to the current package metadata. Trust PyPI metadata and the current package configuration when deciding compatibility.
- The v2 API remains available, but the upstream docs treat it as a compatibility layer around the newer v3 machinery. Prefer `imageio.v3` in new code.
- The upstream user API docs explicitly point older `imageio.imread(...)` and similar patterns toward `imageio.v2` for backward-compatible behavior. If you are fixing an old codebase, importing `imageio.v2 as imageio` is often the least disruptive migration step before a fuller v3 rewrite.

## Official Sources

- Docs root: https://imageio.readthedocs.io/en/stable/
- Installation: https://imageio.readthedocs.io/en/stable/getting_started/installation.html
- Core v3 API: https://imageio.readthedocs.io/en/stable/reference/core_v3.html
- User API and v2/v3 migration notes: https://imageio.readthedocs.io/en/stable/reference/userapi.html
- Fancy sources and `BytesIO` examples: https://imageio.readthedocs.io/en/stable/examples.html#read-from-fancy-sources
- Package metadata: https://pypi.org/project/imageio/
- Repository: https://github.com/imageio/imageio
