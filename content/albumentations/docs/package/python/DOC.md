---
name: package
description: "Albumentations image augmentation library for Python computer vision pipelines"
metadata:
  languages: "python"
  versions: "2.0.8"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "albumentations,computer-vision,image-augmentation,opencv,pytorch,augmentation"
---

# Albumentations Python Package Guide

## Golden Rule

Use `albumentations` as `import albumentations as A`, pass NumPy arrays in HWC layout, convert OpenCV images from BGR to RGB before augmentation, and declare every synchronized target up front in one `A.Compose(...)` call. If you augment masks, bounding boxes, keypoints, or multiple images together, configure the corresponding target metadata explicitly instead of assuming transforms will infer it.

## Install

Pin the package version your project expects:

```bash
python -m pip install "albumentations==2.0.8"
```

Common alternatives:

```bash
uv add "albumentations==2.0.8"
poetry add "albumentations==2.0.8"
```

PyPI publishes optional extras including `pytorch`, `text`, and `hub`:

```bash
python -m pip install "albumentations[pytorch]==2.0.8"
```

Notes:

- `albumentations` works on NumPy arrays and is commonly paired with OpenCV, PyTorch, or other training frameworks.
- If you use `albumentations.pytorch`, you still need a compatible `torch` install in your environment.
- There is no API key or account setup. Configuration is entirely local Python code.

## Initialize A Pipeline

The core abstraction is `A.Compose`, which applies the same random decisions to all declared targets in the sample.

```python
import cv2
import albumentations as A

transform = A.Compose(
    [
        A.SmallestMaxSize(max_size=512),
        A.RandomCrop(height=448, width=448),
        A.HorizontalFlip(p=0.5),
        A.RandomBrightnessContrast(p=0.2),
    ],
    strict=True,
)

image = cv2.imread("image.jpg")
image = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)

augmented = transform(image=image)
image_aug = augmented["image"]
```

Why `strict=True` matters:

- it rejects unknown input keys instead of silently ignoring them
- it catches misconfigured pipelines earlier
- it is a good default for agent-written code

## Core Usage

### Image classification or generic image augmentation

```python
import albumentations as A

train_tf = A.Compose(
    [
        A.RandomResizedCrop(size=(224, 224), scale=(0.8, 1.0)),
        A.HorizontalFlip(p=0.5),
        A.Affine(scale=(0.9, 1.1), rotate=(-15, 15), p=0.5),
        A.Normalize(),
    ],
    strict=True,
)
```

Call it with `transform(image=image)` and read the result from `["image"]`.

### Semantic segmentation

Masks stay synchronized automatically when you pass both `image` and `mask`. Geometric transforms affect both; image-only transforms such as `Normalize` affect the image and leave the mask untouched.

```python
import albumentations as A

seg_tf = A.Compose(
    [
        A.RandomCrop(height=512, width=512),
        A.HorizontalFlip(p=0.5),
        A.RandomRotate90(p=0.5),
        A.Normalize(),
    ],
    strict=True,
)

augmented = seg_tf(image=image, mask=mask)
image_aug = augmented["image"]
mask_aug = augmented["mask"]
```

### Object detection with bounding boxes

Declare bounding box metadata with `bbox_params`. Do not pass boxes without it.

```python
import albumentations as A

det_tf = A.Compose(
    [
        A.Resize(height=640, width=640),
        A.HorizontalFlip(p=0.5),
        A.RandomBrightnessContrast(p=0.2),
    ],
    bbox_params=A.BboxParams(
        format="coco",
        label_fields=["class_labels"],
        min_visibility=0.1,
    ),
    strict=True,
)

augmented = det_tf(
    image=image,
    bboxes=bboxes,
    class_labels=class_labels,
)
```

Use the bounding box format your data actually uses: `coco`, `pascal_voc`, `albumentations`, or `yolo`.

### Keypoints

Keypoints require their own metadata, just like bounding boxes:

```python
import albumentations as A

kp_tf = A.Compose(
    [
        A.Resize(height=256, width=256),
        A.HorizontalFlip(p=0.5),
    ],
    keypoint_params=A.KeypointParams(format="xy", remove_invisible=False),
    strict=True,
)

augmented = kp_tf(image=image, keypoints=keypoints)
```

### Multiple images or extra targets

If you need identical augmentation decisions across multiple related arrays, use `additional_targets`.

```python
import albumentations as A

stereo_tf = A.Compose(
    [
        A.RandomCrop(height=256, width=256),
        A.HorizontalFlip(p=0.5),
    ],
    additional_targets={"right_image": "image", "depth": "mask"},
    strict=True,
)

augmented = stereo_tf(
    image=left_image,
    right_image=right_image,
    depth=depth_map,
)
```

Map each extra input to the correct target type:

- use `"image"` for data that should receive image transforms
- use `"mask"` for label-like arrays that must stay aligned but should not receive pixel normalization or color jitter

## Configuration Notes

There is no remote configuration or authentication layer. The practical configuration knobs are the pipeline arguments and target metadata:

- `strict=True`: fail fast on unknown inputs or invalid configuration
- `bbox_params=...`: required for `bboxes`
- `keypoint_params=...`: required for `keypoints`
- `additional_targets=...`: required for multi-image or multi-mask synchronization
- `seed=...` on `A.Compose(...)`: use when you need reproducible augmentation sequences
- `save_applied_params=True`: use when you need to inspect the transforms and parameters that actually ran

Example with reproducibility and transform inspection:

```python
import albumentations as A

tf = A.Compose(
    [
        A.HorizontalFlip(p=0.5),
        A.RandomBrightnessContrast(p=0.5),
    ],
    seed=137,
    save_applied_params=True,
    strict=True,
)

result = tf(image=image)
print(result["applied_transforms"])
```

## Serialization

Use Albumentations' built-in save/load helpers when you need to persist a pipeline definition:

```python
import albumentations as A

transform = A.Compose(
    [
        A.Resize(height=256, width=256),
        A.HorizontalFlip(p=0.5),
    ],
    strict=True,
)

A.save(transform, "transform.json")
loaded = A.load("transform.json")
```

If you use `Lambda` transforms or other non-serializable callables, you must provide the `nonserializable` mapping when loading or avoid serializing that pipeline.

## Common Pitfalls

- OpenCV loads images as BGR. Convert to RGB before augmentation if the rest of your pipeline expects RGB.
- Albumentations expects NumPy arrays, not PIL images. Convert before calling the transform.
- Keep image, mask, bbox, and keypoint inputs shape-compatible. The FAQ explicitly calls out grayscale and shape-handling edge cases.
- Do not send `bboxes` or `keypoints` without the corresponding `BboxParams` or `KeypointParams`.
- Bounding box transforms only work for transforms that support that target type. Check the Supported Targets table before composing a pipeline.
- Keep `label_fields` in sync with `bboxes`; otherwise label arrays can drift from the boxes that remain after filtering.
- `Normalize` changes the image, not the mask. Do not manually normalize segmentation masks unless your downstream code explicitly needs that.
- Use `additional_targets` when augmenting stereo pairs, restoration input/target pairs, or multiple masks together. Separate transform calls will not stay synchronized.

## Version-Sensitive Notes For 2.0.8

- `2.0.8` is the current `albumentations` package version on PyPI as of March 12, 2026.
- The official GitHub release notes for `2.0.0` document the main breaking changes for all `2.x` users. In particular, `always_apply` was removed; use `p=1` for always-on transforms and `p=0` to disable one.
- Several transforms changed parameter names in `2.0.0`. A common migration issue is `RandomResizedCrop`: old examples may show `height=` and `width=`, while the `2.0.0` release notes say the `2.x` API uses `size=(height, width)`.
- `GaussNoise` changed from `var_limit` and `mean` to `std_range` and `mean_range`.
- Several transforms changed default `border_mode` values to `cv2.BORDER_CONSTANT` in `2.0.0`. If your training data depended on old border behavior, set `border_mode` and fill values explicitly.
- `2.0.8` added transforms such as `Mosaic` and `OverlayElements`, plus bug fixes around probabilistic composition and replay behavior. If you see examples from early `2.0.x`, prefer the `2.0.8` release notes and current API reference over third-party blog posts.
- The repository was archived by the maintainers in July 2025. Treat unofficial community examples published before or during the transition carefully and prefer the official docs root, PyPI metadata, and official release notes.

## Official Links

- Docs root: https://albumentations.ai/docs/
- FAQ: https://albumentations.ai/docs/faq/
- Pipelines core concept: https://albumentations.ai/docs/2-core-concepts/pipelines/
- Bounding boxes guide: https://albumentations.ai/docs/3-basic-usage/bounding-boxes-augmentations/
- Segmentation guide: https://albumentations.ai/docs/3-basic-usage/semantic-segmentation/
- Additional targets guide: https://albumentations.ai/docs/4-advanced-guides/additional-targets/
- Serialization example: https://albumentations.ai/docs/examples/serialization/
- PyPI package: https://pypi.org/project/albumentations/
- GitHub releases: https://github.com/albumentations-team/albumentations/releases
