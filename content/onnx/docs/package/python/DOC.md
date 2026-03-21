---
name: package
description: "onnx package guide for Python covering model creation, load/save, checking, shape inference, external data, and model hub usage"
metadata:
  languages: "python"
  versions: "1.20.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "onnx,python,ml,model-format,protobuf,shape-inference"
---

# onnx Python Package Guide

## What It Is

`onnx` is the core Python package for working with ONNX model files and protobuf objects.

Use it when you need to:

- load or save `.onnx` models
- inspect graphs, nodes, tensors, and metadata
- validate models with `onnx.checker`
- build or rewrite graphs with `onnx.helper`
- infer shapes with `onnx.shape_inference`
- work with large models that use external tensor data
- download sample models from the ONNX Model Hub

Common imports:

- `import onnx`
- `from onnx import TensorProto, checker, helper, hub, numpy_helper, shape_inference`

Practical boundary: `onnx` is primarily for model representation and tooling. If you need production inference, verify the runtime or backend separately.

## Version-Sensitive Notes

- This entry is pinned to the version used here `1.20.1`.
- PyPI has an exact release page for `1.20.1`, but the versionless ONNX docs root may move ahead of `1.20.1`. Treat examples from `onnx.ai/onnx/` as current-maintainer guidance, then verify anything version-sensitive against the `1.20.x` release line.
- The official `v1.20.1` release notes describe this as a patch release with important bug fixes and build-system changes rather than a large Python API shift.
- PyPI publishes an optional `reference` extra: `pip install "onnx[reference]==1.20.1"`.

## Install

Pin the package when you need reproducible tooling behavior:

```bash
python -m pip install "onnx==1.20.1"
```

If you need the optional reference implementation dependencies:

```bash
python -m pip install "onnx[reference]==1.20.1"
```

Verify the installed version:

```bash
python -c "import onnx; print(onnx.__version__)"
```

## Quick Start: Load, Inspect, Validate

Start by loading a model, checking it, and inspecting the graph shape before making changes.

```python
import onnx
from onnx import checker

model = onnx.load("model.onnx")
checker.check_model(model)

print("ir_version:", model.ir_version)
print("producer:", model.producer_name)
print("graph name:", model.graph.name)
print("inputs:", [value.name for value in model.graph.input])
print("outputs:", [value.name for value in model.graph.output])
print("nodes:", len(model.graph.node))
```

Use this pattern before writing graph-rewrite code. It catches malformed models early and gives you the key metadata you usually need for debugging.

## Create And Save A Model

The Python API overview uses `onnx.helper` to build graphs and models. Be explicit about tensor types, shapes, and opset imports.

```python
import onnx
from onnx import TensorProto, checker, helper

input_info = helper.make_tensor_value_info("X", TensorProto.FLOAT, [1, 2])
output_info = helper.make_tensor_value_info("Y", TensorProto.FLOAT, [1, 2])

const_tensor = helper.make_tensor(
    name="bias",
    data_type=TensorProto.FLOAT,
    dims=[1, 2],
    vals=[1.0, 1.0],
)

add_node = helper.make_node("Add", inputs=["X", "bias"], outputs=["Y"])

graph = helper.make_graph(
    nodes=[add_node],
    name="add_bias_graph",
    inputs=[input_info],
    outputs=[output_info],
    initializer=[const_tensor],
)

model = helper.make_model(
    graph,
    opset_imports=[helper.make_opsetid("", 18)],
    producer_name="context-hub-example",
)

checker.check_model(model)
onnx.save_model(model, "add_bias.onnx")
```

Practical rules:

- Set `opset_imports` deliberately instead of relying on whatever default your environment provides.
- Run `checker.check_model(...)` before saving generated models.
- Use `onnx.save_model(...)` for file output and `model.SerializeToString()` when another API needs raw bytes.

## Convert Between NumPy Arrays And ONNX Tensors

`onnx.numpy_helper` is the practical bridge when you need to inspect or inject tensor values.

```python
import numpy as np
from onnx import numpy_helper

array = np.array([[1.0, 2.0]], dtype=np.float32)
tensor = numpy_helper.from_array(array, name="weights")
round_trip = numpy_helper.to_array(tensor)

print(tensor.name)
print(round_trip.shape)
```

Use this for initializers, test fixtures, and model-inspection tools.

## Shape Inference

Run shape inference after graph edits so downstream code can rely on richer type information.

```python
import onnx
from onnx import shape_inference

model = onnx.load("add_bias.onnx")
inferred = shape_inference.infer_shapes(model)

for value in inferred.graph.value_info:
    print(value.name)
```

For very large models or models stored with external data, prefer path-based APIs:

```python
from onnx import checker, shape_inference

checker.check_model("model.onnx")
shape_inference.infer_shapes_path("model.onnx", "model-inferred.onnx")
```

That avoids loading the entire protobuf into memory when you only need file-based validation or shape updates.

## External Data For Large Models

ONNX supports storing tensor data outside the main `.onnx` file. This matters for large weights and for models near or above protobuf size limits.

```python
import onnx

model = onnx.load("model.onnx")

onnx.save_model(
    model,
    "model-with-external-data.onnx",
    save_as_external_data=True,
    all_tensors_to_one_file=True,
    location="model.weights.bin",
    size_threshold=1024,
)
```

Useful behaviors from the official docs:

- `onnx.load(...)` loads external data by default.
- If you need only the structure first, pass `load_external_data=False`.
- After loading with `load_external_data=False`, call `onnx.load_external_data_for_model(model, base_dir)` when you are ready to resolve the tensor files.
- For models larger than 2 GB, use the path-based checker and shape-inference APIs rather than object-based ones.

## Model Hub Usage

`onnx.hub` can download models from the ONNX Model Hub.

```python
from onnx import hub

model = hub.load("mnist")
print(model.graph.name)
```

Treat remote model loading as a trusted-source operation. The official hub docs explicitly place responsibility on the user to verify that the source is trusted and safe to extract or execute.

## Configuration And Environment

There is no service authentication layer in `onnx`. The main configuration surface is about model compatibility and file handling:

- Python version must satisfy the PyPI requirement for the package line you install.
- Opset version matters when you create or transform graphs.
- External-data paths matter when models store weights outside the main file.
- Relative paths are resolved from the model location or the `base_dir` you pass when loading external tensor files.

Useful checks in scripts and CI:

```python
import onnx

print("onnx version:", onnx.__version__)
```

```bash
python -m pip show onnx
```

## Common Pitfalls

- Do not assume the versionless docs root is patch-pinned to `1.20.1`.
- Do not skip `checker.check_model(...)` after graph rewrites or generated-model output.
- Do not rely on implicit opset defaults when you create models; set `opset_imports` explicitly.
- Do not forget external tensor files when moving or packaging a model saved with `save_as_external_data=True`.
- Do not load untrusted Hub content into sensitive environments without reviewing the source.
- Do not assume `onnx` itself is the runtime you will use for production inference.

## Official URLs Used For This Entry

- API root: `https://onnx.ai/onnx/api/`
- Python API overview: `https://onnx.ai/onnx/repo-docs/PythonAPIOverview.html`
- Python intro: `https://onnx.ai/onnx/intro/python.html`
- Model hub API: `https://onnx.ai/onnx/api/hub.html`
- PyPI project: `https://pypi.org/project/onnx/`
- Exact PyPI release: `https://pypi.org/project/onnx/1.20.1/`
- Release notes: `https://github.com/onnx/onnx/releases/tag/v1.20.1`
