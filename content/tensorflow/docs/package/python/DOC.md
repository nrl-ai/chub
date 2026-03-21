---
name: package
description: "TensorFlow Python package guide for installing, configuring, training, and exporting models with TensorFlow 2.21.0"
metadata:
  languages: "python"
  versions: "2.21.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "tensorflow,machine-learning,deep-learning,keras,tf.data,gpu"
---

# TensorFlow Python Package Guide

## Golden Rule

Use the `tensorflow` PyPI package for TensorFlow 2.x work, install from PyPI with `pip`, and verify platform support before assuming GPU behavior.

For package version `2.21.0`, PyPI is the best version anchor. The official TensorFlow install and API pages are useful, but parts of the site still show older version labels.

## Installation

TensorFlow officially recommends installing from PyPI with `pip`, not `conda`.

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install tensorflow==2.21.0
```

### Optional extras

Install the base package unless you specifically need an extra:

```bash
python -m pip install "tensorflow[and-cuda]==2.21.0"
python -m pip install "tensorflow[gcs-filesystem]==2.21.0"
```

- `and-cuda`: CUDA-enabled Linux install path published by TensorFlow.
- `gcs-filesystem`: add the Google Cloud Storage filesystem integration when your input pipeline reads from GCS.

### Verify the install

```bash
python - <<'PY'
import tensorflow as tf
print(tf.__version__)
print(tf.config.list_physical_devices("GPU"))
PY
```

## Platform And Version Notes

- PyPI for `2.21.0` says `Requires: Python >=3.10` and publishes wheels for Python `3.10` through `3.13`.
- The TensorFlow install page explicitly warns that Python `3.9` is not supported as of TensorFlow `2.21`.
- Native Windows GPU support ended at TensorFlow `2.10`. For `2.21.0`, use WSL2 if you need GPU acceleration on Windows.
- macOS has no official GPU support on the TensorFlow install page. Treat macOS as CPU-only unless you have a separate Apple-specific stack you already know you need.
- On Linux ARM64, installing `tensorflow` can resolve to AWS-maintained CPU wheels (`tensorflow-cpu-aws`), so do not assume the same wheel provenance as x86 Linux.
- The public TensorFlow Python API site currently renders as `v2.16.1`, even though PyPI has `2.21.0`. Use PyPI and the install matrix as the version source of truth for packaging decisions.

## Initialization And Basic Setup

Start with a plain import and device check:

```python
import tensorflow as tf

print(tf.__version__)
print(tf.config.list_physical_devices("CPU"))
print(tf.config.list_physical_devices("GPU"))
```

If you want to limit TensorFlow to one GPU, or enable memory growth, do it before the runtime initializes the devices:

```python
import tensorflow as tf

gpus = tf.config.list_physical_devices("GPU")
if gpus:
    tf.config.set_visible_devices(gpus[0], "GPU")
    tf.config.experimental.set_memory_growth(gpus[0], True)
```

If you call `set_visible_devices()` or `set_memory_growth()` after TensorFlow has already initialized the GPU, TensorFlow raises a runtime error.

## Core Usage

### Tensors, variables, and autodiff

Use `tf.Tensor` for values and `tf.Variable` for mutable state such as trainable weights.

```python
import tensorflow as tf

x = tf.constant([[1.0, 2.0], [3.0, 4.0]])
w = tf.Variable([[0.5], [1.0]])

with tf.GradientTape() as tape:
    y = tf.matmul(x, w)
    loss = tf.reduce_sum(y)

grads = tape.gradient(loss, [w])
print(y)
print(grads[0])
```

Use `GradientTape` when you need a custom training step or want direct control over differentiation.

### Build datasets with `tf.data`

Use `tf.data.Dataset` as the default input pipeline abstraction.

```python
import tensorflow as tf

features = tf.random.normal((1000, 10))
labels = tf.random.uniform((1000,), maxval=2, dtype=tf.int32)

dataset = tf.data.Dataset.from_tensor_slices((features, labels))
dataset = dataset.shuffle(1000).batch(32).prefetch(tf.data.AUTOTUNE)

for batch_x, batch_y in dataset.take(1):
    print(batch_x.shape, batch_y.shape)
```

Useful defaults for training pipelines:

- use `from_tensor_slices()` for in-memory arrays and tensors
- use `map()` for preprocessing
- use `batch()` before `fit()`
- end the pipeline with `prefetch(tf.data.AUTOTUNE)`
- add `num_parallel_calls=tf.data.AUTOTUNE` to expensive `map()` or `interleave()` stages when input work is CPU- or I/O-bound

### Train a Keras model

For standard supervised training, `tf.keras` is the fastest path from data to a working model.

```python
import tensorflow as tf

model = tf.keras.Sequential(
    [
        tf.keras.layers.Input(shape=(10,)),
        tf.keras.layers.Dense(64, activation="relu"),
        tf.keras.layers.Dense(1, activation="sigmoid"),
    ]
)

model.compile(
    optimizer="adam",
    loss="binary_crossentropy",
    metrics=["accuracy"],
)

features = tf.random.normal((1000, 10))
labels = tf.cast(tf.random.uniform((1000,), maxval=2, dtype=tf.int32), tf.float32)
dataset = (
    tf.data.Dataset.from_tensor_slices((features, labels))
    .shuffle(1000)
    .batch(32)
    .prefetch(tf.data.AUTOTUNE)
)

history = model.fit(dataset, epochs=3)
```

Use `Sequential` only for a plain stack of single-input, single-output layers. Switch to the Functional API or subclassing for multiple inputs, multiple outputs, shared layers, or non-linear topology.

### Compile hot paths with `tf.function`

Wrap stable tensor-heavy code in `@tf.function` when you need graph execution and reduced Python overhead.

```python
import tensorflow as tf

@tf.function(reduce_retracing=True)
def score_batch(x):
    return tf.reduce_sum(x, axis=-1)

print(score_batch(tf.ones((4, 8))))
```

`tf.function` can retrace when shapes, dtypes, or Python arguments vary. Prefer tensor inputs, consistent shapes, and `reduce_retracing=True` when you expect some input variation.

### Save and export models

For Keras models, prefer the high-level `.keras` format for normal save/load workflows:

```python
model.save("classifier.keras")
reloaded = tf.keras.models.load_model("classifier.keras")
```

Use `SavedModel` export when you need a serving artifact:

```python
model.export("exported_model")
```

If you are working with lower-level TensorFlow modules instead of Keras models:

```python
import tensorflow as tf

class Doubler(tf.Module):
    @tf.function
    def __call__(self, x):
        return x * 2

module = Doubler()
tf.saved_model.save(module, "saved_module")
loaded = tf.saved_model.load("saved_module")
print(loaded(tf.constant([1, 2, 3])))
```

## Configuration And External Access

TensorFlow itself does not require package-level authentication.

- Local tensors, local files, and local training require no auth setup.
- Authentication only matters when your pipeline touches external systems such as cloud storage, model registries, or hosted serving systems.
- If you need Google Cloud Storage access from TensorFlow file APIs, install the `gcs-filesystem` extra and configure your cloud credentials outside TensorFlow using the normal provider flow for that environment.

## Common Pitfalls

- Do not assume the docs site version label matches the PyPI package version. For this package, the API site is still labeled `v2.16.1` while PyPI has `2.21.0`.
- Do not use `conda` as the primary install path for current stable TensorFlow. TensorFlow explicitly recommends `pip` because official releases land on PyPI.
- Do not configure visible GPUs or memory growth after the first TensorFlow GPU call.
- Do not use `tf.Tensor` when you need mutation. Use `tf.Variable` for trainable or mutable state.
- Do not overuse `@tf.function` on code with highly variable Python arguments; retracing can erase the expected performance benefit.
- Do not use `Sequential` for branching or multi-input models.
- Do not assume native Windows GPU support exists for TensorFlow `2.21.0`; it does not.
- Do not default to `SavedModel` for ordinary Keras checkpointing. TensorFlow’s SavedModel guide recommends the newer `.keras` format plus `tf.keras.Model.export()` for Keras objects.

## Version-Sensitive Notes For `2.21.0`

- PyPI shows `2.21.0` released on `2026-03-06`.
- PyPI metadata for `2.21.0` requires Python `>=3.10`.
- The install matrix on TensorFlow.org shows `2.21.0` wheels for Python `3.10` to `3.13`.
- The public API reference currently still renders under TensorFlow `v2.16.1`, so newer symbols or packaging details may lag there.
- The install page warns that Python `3.9` is no longer supported as of TensorFlow `2.21`.

## Official Sources Used

- TensorFlow install guide: https://www.tensorflow.org/install/pip
- TensorFlow Python API root: https://www.tensorflow.org/api_docs/python/
- TensorFlow basics guide: https://www.tensorflow.org/guide/basics
- Keras Sequential guide: https://www.tensorflow.org/guide/keras/sequential_model
- `tf.data` guide: https://www.tensorflow.org/guide/data
- `tf.data` performance guide: https://www.tensorflow.org/guide/data_performance
- GPU guide: https://www.tensorflow.org/guide/gpu
- SavedModel guide: https://www.tensorflow.org/guide/saved_model
- PyPI release page: https://pypi.org/project/tensorflow/2.21.0/
