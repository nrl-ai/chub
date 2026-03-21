---
name: package
description: "onnxruntime package guide for Python inference with InferenceSession, execution providers, SessionOptions, OrtValue, and I/O binding"
metadata:
  languages: "python"
  versions: "1.24.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "onnxruntime,onnx,inference,machine-learning,execution-providers,cpu,gpu"
---

# onnxruntime Python Package Guide

## What It Does

`onnxruntime` runs inference for models in ONNX or ORT format from Python. Use it when a model has already been exported and you need a fast runtime with CPU execution by default and optional hardware acceleration through execution providers.

It is an inference runtime, not a training framework and not the exporter itself.

## Version-Sensitive Notes

- The version used here and the latest PyPI release both showed `1.24.3` on `2026-03-12`.
- PyPI metadata for `1.24.3` requires Python `>=3.10`.
- `onnxruntime` is the CPU release. CUDA inference uses the separate `onnxruntime-gpu` package.
- The official Python guide says only one of `onnxruntime` and `onnxruntime-gpu` should be installed in the same environment at a time.
- Since ONNX Runtime `1.10`, the docs require explicit provider selection for non-CPU targets. CPU is the only case where omitting `providers=` is acceptable.
- The `1.24.3` release is a patch release with bug fixes, security fixes, performance work, and execution-provider updates. The release notes also call out a Python map-input refcount fix and Python 3.14 CI enablement.

## Install

Pin the runtime when you need reproducible behavior:

```bash
python -m pip install "onnxruntime==1.24.3"
```

With `uv`:

```bash
uv add "onnxruntime==1.24.3"
```

With Poetry:

```bash
poetry add "onnxruntime==1.24.3"
```

If you need CUDA, install `onnxruntime-gpu` in a separate environment instead of layering it on top of `onnxruntime`.

If you also need to export or validate models, install the exporter stack separately:

- PyTorch export: `torch`
- TensorFlow export: `tf2onnx`
- scikit-learn export: `skl2onnx`
- ONNX model inspection/checking: `onnx`

## Minimal Inference Flow

Start by loading the model, reading the real input and output names from the session, and feeding NumPy arrays that match the model types and shapes.

```python
from pathlib import Path

import numpy as np
import onnxruntime as ort

model_path = Path("model.onnx")

session = ort.InferenceSession(
    model_path.as_posix(),
    providers=["CPUExecutionProvider"],
)

input_meta = session.get_inputs()[0]
output_meta = session.get_outputs()[0]

x = np.random.rand(1, 4).astype(np.float32)

y = session.run(
    [output_meta.name],
    {input_meta.name: x},
)[0]

print(y.shape)
```

For quick scripts, `session.run(None, feed_dict)` returns all outputs. For production code, requesting only the outputs you need is usually clearer.

## Choosing Execution Providers

Execution providers determine where kernels run. Provider order is priority order, and ONNX Runtime will use a lower-priority provider when a higher-priority provider cannot execute a node.

Check the installed wheel first:

```python
import onnxruntime as ort

print(ort.get_version_string())
print(ort.get_available_providers())
```

Safe provider selection pattern:

```python
import onnxruntime as ort

available = ort.get_available_providers()

if "CUDAExecutionProvider" in available:
    providers = ["CUDAExecutionProvider", "CPUExecutionProvider"]
else:
    providers = ["CPUExecutionProvider"]

session = ort.InferenceSession("model.onnx", providers=providers)
print(session.get_providers())
```

Provider options can be passed inline as tuples:

```python
import onnxruntime as ort

session = ort.InferenceSession(
    "model.onnx",
    providers=[
        ("CUDAExecutionProvider", {"device_id": 0}),
        "CPUExecutionProvider",
    ],
)
```

Use `onnxruntime` when you want CPU execution. Use `onnxruntime-gpu` when you actually need CUDA/TensorRT-backed inference.

## Session Configuration

There is no authentication layer. Configuration is local process configuration:

- which model file to load
- which providers to register
- provider-specific options
- `SessionOptions` and optional per-run options

Useful `SessionOptions` in practice:

- `graph_optimization_level`
- `intra_op_num_threads`
- `inter_op_num_threads`
- `enable_profiling`
- `use_deterministic_compute`
- `optimized_model_filepath`
- `register_custom_ops_library(...)` for custom kernels

Example:

```python
import numpy as np
import onnxruntime as ort

so = ort.SessionOptions()
so.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
so.intra_op_num_threads = 1
so.inter_op_num_threads = 1
so.enable_profiling = True
so.use_deterministic_compute = True
so.optimized_model_filepath = "model.optimized.ort"
so.add_session_config_entry("session.save_model_format", "ORT")

session = ort.InferenceSession(
    "model.onnx",
    sess_options=so,
    providers=["CPUExecutionProvider"],
)

input_name = session.get_inputs()[0].name
session.run(None, {input_name: np.zeros((1, 4), dtype=np.float32)})
profile_path = session.end_profiling()
print(profile_path)
```

Notes:

- `inter_op_num_threads` controls parallelism across graph nodes.
- `intra_op_num_threads` controls parallelism within nodes.
- `optimized_model_filepath` only writes an optimized model if you set it.
- If your model depends on custom ops, load the custom op shared library before running inference.

## Inspect Inputs And Outputs Instead Of Hardcoding Names

Use the model metadata APIs before building feeds:

```python
import onnxruntime as ort

session = ort.InferenceSession("model.onnx", providers=["CPUExecutionProvider"])

for arg in session.get_inputs():
    print(arg.name, arg.shape, arg.type)

for arg in session.get_outputs():
    print(arg.name, arg.shape, arg.type)
```

This avoids the common mistake of guessing names like `"input"` or `"output"` when the exporter generated something else.

## Working With NumPy, OrtValue, And Device Memory

The common path is NumPy on CPU:

```python
import numpy as np
import onnxruntime as ort

x = np.ones((1, 3, 224, 224), dtype=np.float32)
session = ort.InferenceSession("model.onnx", providers=["CPUExecutionProvider"])
outputs = session.run(None, {"input": x})
```

For more control, create `OrtValue` objects explicitly:

```python
import numpy as np
import onnxruntime as ort

x = np.ones((1, 4), dtype=np.float32)
ort_value = ort.OrtValue.ortvalue_from_numpy(x)

session = ort.InferenceSession("model.onnx", providers=["CPUExecutionProvider"])
result = session.run(None, {"input": ort_value})
```

The API docs note that some ONNX data types are not supported by NumPy, including bfloat16 and some float8 types. For those cases, the documented path is direct device binding, for example with Torch tensors on GPU.

## I/O Binding For Device-Resident Data

By default, ONNX Runtime places inputs and outputs on CPU. If your pipeline already has tensors on an accelerator, use I/O binding to avoid extra device-to-CPU copies.

Input on CPU, output copied back to CPU:

```python
import numpy as np
import onnxruntime as ort

x = np.ones((1, 4), dtype=np.float32)
session = ort.InferenceSession(
    "model.onnx",
    providers=["CUDAExecutionProvider", "CPUExecutionProvider"],
)

binding = session.io_binding()
binding.bind_cpu_input("input", x)
binding.bind_output("output")

session.run_with_iobinding(binding)
y = binding.copy_outputs_to_cpu()[0]
```

For fully device-resident workflows, bind `OrtValue` inputs and outputs on the target device and call `run_with_iobinding(...)`.

## Async Execution

If you need asynchronous execution from Python, `run_async(...)` executes inference in a separate C++ thread and invokes a callback with the results.

```python
import numpy as np
import onnxruntime as ort

session = ort.InferenceSession("model.onnx", providers=["CPUExecutionProvider"])
input_name = session.get_inputs()[0].name

def on_done(results, user_data, error):
    if error:
        raise RuntimeError(error)
    print(results[0].shape)

session.run_async(
    [session.get_outputs()[0].name],
    {input_name: np.ones((1, 4), dtype=np.float32)},
    on_done,
    None,
)
```

Use this only when you actually need callback-style execution. Most code is simpler with synchronous `run(...)`.

## Common Pitfalls

- Do not install both `onnxruntime` and `onnxruntime-gpu` in one environment.
- Do not assume the model input name is `"input"` or the output name is `"output"`. Read them from `get_inputs()` and `get_outputs()`.
- Match dtypes exactly. A large share of examples require `np.float32`, not Python floats or float64 NumPy arrays.
- Match shapes exactly, including batch dimensions. Exported models often expect `NCHW` image tensors or fixed feature counts.
- Do not assume the installed wheel actually exposes CUDA or another accelerator. Check `ort.get_available_providers()`.
- For non-CPU execution, pass `providers=` explicitly. The docs call this out as required starting with ORT `1.10`.
- By default, outputs land on CPU. If your surrounding pipeline is on GPU, use I/O binding to avoid unnecessary copies.
- If your model uses custom operators, register the custom ops library before creating the session.
- If you serialize optimized models, remember that `.ort` implies ORT model format, and you can also force it with `session.save_model_format=ORT`.

## Official Sources Used

- Python API docs: `https://onnxruntime.ai/docs/api/python/api_summary.html`
- Python getting started: `https://onnxruntime.ai/docs/get-started/with-python.html`
- Install guide: `https://onnxruntime.ai/docs/install/`
- Execution providers overview: `https://onnxruntime.ai/docs/execution-providers/`
- PyPI package page: `https://pypi.org/project/onnxruntime/`
- `1.24.3` release notes: `https://github.com/Microsoft/onnxruntime/releases/tag/v1.24.3`
