---
name: package
description: "Keras 3.13.2 package guide for Python deep learning with TensorFlow, JAX, PyTorch, and OpenVINO backends"
metadata:
  languages: "python"
  versions: "3.13.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "keras,python,deep-learning,machine-learning,tensorflow,jax,pytorch,openvino"
---

# Keras Python Package Guide

## What It Is

`keras` is the Keras 3 multi-backend deep learning package for Python. It supports TensorFlow, JAX, and PyTorch as training backends, plus OpenVINO for inference-only workloads.

For new code:

- use `import keras`, not `from tensorflow import keras`
- use the Functional API for most non-trivial models
- use `keras.ops` in custom layers and metrics if you want backend-agnostic code

`keras` does not talk to a remote service and has no API authentication model. The main setup decision is which backend runtime you install and select.

## Install

Install the Keras package itself:

```bash
python -m pip install "keras==3.13.2"
```

If you use `uv`:

```bash
uv add "keras==3.13.2"
```

You must also install at least one backend package. The official Keras package metadata and install docs list these minimum backend versions for the current Keras 3 line:

- TensorFlow: `>=2.16.1`
- JAX: `>=0.4.20`
- PyTorch: `>=2.1.0`
- OpenVINO: `>=2025.3.0` for inference only

Common installs:

```bash
python -m pip install "keras==3.13.2" "tensorflow>=2.16.1"
python -m pip install "keras==3.13.2" "jax>=0.4.20"
python -m pip install "keras==3.13.2" "torch>=2.1.0"
```

If you need legacy Keras 2 behavior, the official package note is that Keras 2 remains available separately as `tf-keras`. Do not assume `keras` is an alias for the old standalone Keras 2 package.

## Backend Selection And Setup

Set the backend before importing `keras`. The official docs are explicit that the backend cannot be changed after import.

```python
import os

os.environ["KERAS_BACKEND"] = "tensorflow"

import keras
```

Supported backend names:

- `"tensorflow"`
- `"jax"`
- `"torch"`
- `"openvino"`

You can also configure the backend through `~/.keras/keras.json`, but the environment variable is the safest approach for scripts, tests, notebooks, and agent-generated code because it is explicit in-process.

OpenVINO is inference-only. Use it for `model.predict(...)`, not for training.

## Core Workflow

For most real projects, start with the Functional API instead of `Sequential`. The official Models API says the Functional API is what most people should be using because it handles arbitrary model graphs, shared layers, and multiple inputs or outputs.

```python
import os

os.environ["KERAS_BACKEND"] = "tensorflow"

import numpy as np
import keras
from keras import layers

inputs = keras.Input(shape=(20,), name="features")
x = layers.Dense(64, activation="relu")(inputs)
x = layers.Dropout(0.2)(x)
outputs = layers.Dense(3, activation="softmax", name="label")(x)

model = keras.Model(inputs=inputs, outputs=outputs)

model.compile(
    optimizer="adam",
    loss="sparse_categorical_crossentropy",
    metrics=["accuracy"],
)

x_train = np.random.random((256, 20)).astype("float32")
y_train = np.random.randint(0, 3, size=(256,))

history = model.fit(
    x_train,
    y_train,
    batch_size=32,
    epochs=5,
    validation_split=0.2,
)

eval_metrics = model.evaluate(x_train, y_train, verbose=0)
predictions = model.predict(x_train[:5], verbose=0)
```

The standard built-in workflow is:

1. Define a model.
2. `compile(...)` with optimizer, loss, and metrics.
3. `fit(...)` on NumPy arrays, `tf.data.Dataset`, or other supported dataset-style inputs.
4. `evaluate(...)` on held-out data.
5. `predict(...)` for inference.

Useful training notes from the official guides:

- `validation_split` only works with NumPy array inputs
- `tf.data.Dataset` can be passed directly to `fit()`, `evaluate()`, and `predict()`
- use callbacks such as `keras.callbacks.EarlyStopping(...)` and `keras.callbacks.ModelCheckpoint(...)` for real training jobs
- `class_weight` and `sample_weight` are first-class `fit(...)` inputs

## When To Use `Sequential`, Functional API, Or Subclassing

- Use `keras.Sequential([...])` only for straight single-input, single-output stacks.
- Use the Functional API for most application code.
- Subclass `keras.Model` only when the model cannot be expressed as a DAG of layers or when you need custom training behavior.

If you expect to save, inspect, plot, or reuse parts of the model graph, the Functional API is usually the best default.

## Backend-Agnostic Custom Code

Keras 3 will run TensorFlow-specific code when the backend is TensorFlow, but cross-backend code should prefer Keras namespaces:

- `keras.layers` for layers
- `keras.ops` instead of raw `tf.*` ops in symbolic model-building code
- `keras.random` for backend-agnostic random operations

Example:

```python
import keras

inputs = keras.layers.Input(shape=(2, 2, 1))
outputs = keras.ops.squeeze(inputs, axis=-1)
model = keras.Model(inputs, outputs)
```

This pattern matters if you want the same layer, metric, or model code to run on JAX, TensorFlow, and PyTorch.

## Saving, Loading, And Export

Keras 3 changed the default persistence story. The official saving guide says the only supported native `model.save()` format in Keras 3 is the `.keras` format.

Save and reload a full model:

```python
model.save("classifier.keras")

restored = keras.models.load_model("classifier.keras")
```

Important Keras 3 rules:

- `model.save("my_model.keras")` is the preferred full-model save path
- legacy `.h5` is still supported, but `.keras` is the current native format
- `model.save("saved_model")` no longer writes a TensorFlow SavedModel directory
- use `model.export("saved_model")` when you specifically need TensorFlow SavedModel output for TF Serving, TFLite, or similar tooling

If you need to consume an existing TensorFlow SavedModel inside Keras 3, `keras.models.load_model()` will not load it. The migration guide says to use `keras.layers.TFSMLayer(...)` for inference-only loading:

```python
layer = keras.layers.TFSMLayer(
    "saved_model",
    call_endpoint="serving_default",
)
```

For custom layers, activations, or models that need robust round-tripping, prefer registering them:

```python
@keras.saving.register_keras_serializable(package="MyProject")
class MyLayer(keras.layers.Layer):
    ...
```

That avoids brittle `custom_objects={...}` loading paths.

## Configuration And Environment

Keras has no auth configuration. Focus on runtime configuration:

- backend choice via `KERAS_BACKEND`
- backend package version compatibility
- CPU vs GPU environment isolation
- checkpoint and model file paths

The upstream install notes recommend a clean Python environment per backend when working with CUDA-enabled stacks to avoid dependency conflicts.

Official install guidance also says Keras 3 is compatible with Linux and macOS, and recommends WSL2 for Windows users.

## Common Pitfalls

- **Importing `tf.keras` in new Keras 3 code:** replace `from tensorflow import keras` and `tf.keras.*` usage with `import keras` and `keras.*`.
- **Forgetting to install a backend:** `pip install keras` alone is not enough for training.
- **Setting `KERAS_BACKEND` too late:** choose the backend before `import keras`.
- **Trying to switch backends mid-process:** the official docs say you cannot change backend after import.
- **Using raw TensorFlow ops on `KerasTensor` objects during Functional API graph construction:** use `keras.ops` equivalents instead.
- **Assuming deep nested input structures still work:** Keras 3 disallows inputs and outputs nested more than one level deep in Functional models.
- **Passing `None` inside nested `call()` tensor arguments or returns:** Keras 3 disallows this; make optional values separate arguments instead.
- **Hitting XLA errors on GPU with TensorFlow backend:** Keras 3 sets `jit_compile=True` by default on GPU. If a custom layer or model uses unsupported ops, compile with `jit_compile=False`.
- **Using `model.save("dir")` expecting a TensorFlow SavedModel:** use `.keras` for native save or `model.export(...)` for SavedModel export.
- **Trying to load a SavedModel with `keras.models.load_model(...)`:** use `keras.layers.TFSMLayer(...)` instead.
- **Expecting per-output losses from `evaluate()` automatically in multi-output models:** provide explicit metrics in `compile(...)`.

## Version-Sensitive Notes

- PyPI currently lists `keras 3.13.2` with Python requirement `>=3.11`.
- The Keras 3 API docs are current-maintainer docs rather than per-patch frozen docs, so examples here are aligned to the Keras 3 line, not a patch-specific rendered manual.
- The official package description says Keras 2 remains available as `tf-keras`. That is the clearest escape hatch if a project is pinned to pre-Keras-3 behavior.
- OpenVINO is supported as an inference-only backend, not a training backend.
- If you are migrating TensorFlow-first code and want cross-backend portability, replacing `tf.*` calls with `keras.ops.*` is one of the highest-value changes.

## Official Sources

- Keras API root: https://keras.io/api/
- Keras getting started: https://keras.io/getting_started/
- Keras Models API: https://keras.io/api/models/
- Keras training guide: https://keras.io/guides/training_with_built_in_methods/
- Keras Functional API guide: https://keras.io/guides/functional_api/
- Keras saving guide: https://keras.io/guides/serialization_and_saving/
- Keras migration guide: https://keras.io/guides/migrating_to_keras_3/
- PyPI package page: https://pypi.org/project/keras/
