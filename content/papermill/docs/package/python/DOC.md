---
name: package
description: "papermill for parameterizing, executing, and saving Jupyter notebooks in Python pipelines"
metadata:
  languages: "python"
  versions: "2.7.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "papermill,jupyter,notebooks,pipeline,parameters,automation"
---

# papermill Python Package Guide

## What It Is

`papermill` is a notebook runner for parameterized Jupyter workflows. Use it when a `.ipynb` file is part of a repeatable job and you need to inject inputs, execute the notebook, and keep the executed output notebook as an artifact.

## Install

Pin the version your project expects:

```bash
python -m pip install "papermill==2.7.0"
```

Common alternatives:

```bash
uv add "papermill==2.7.0"
poetry add "papermill==2.7.0"
```

Optional extras are important when your notebooks live outside the local filesystem or you want optional helpers:

```bash
python -m pip install "papermill[s3]==2.7.0"
python -m pip install "papermill[azure]==2.7.0"
python -m pip install "papermill[gcs]==2.7.0"
python -m pip install "papermill[github]==2.7.0"
python -m pip install "papermill[hdfs]==2.7.0"
python -m pip install "papermill[black]==2.7.0"
python -m pip install "papermill[all]==2.7.0"
```

Execution prerequisites agents often miss:

- The notebook's kernel must exist on the machine where `papermill` runs.
- The kernel environment must also contain the notebook's runtime dependencies.
- `papermill` itself runs in one environment, but the executed notebook runs through the selected Jupyter kernel.

## Prepare A Notebook For Parameters

Papermill expects a cell tagged `parameters`. Treat that cell as defaults only.

Example notebook cell:

```python
# tag this cell as: parameters
run_date = "2026-03-12"
limit = 100
include_archived = False
```

When executed, papermill inserts an `injected-parameters` cell immediately after the tagged cell and writes only the overridden values there.

If no cell is tagged `parameters`, papermill inserts the `injected-parameters` cell at the top of the notebook instead.

## Core Usage

### Execute A Notebook From Python

`execute_notebook(...)` is the main API. It returns the executed `NotebookNode` and, unless `output_path=None`, writes the executed notebook to disk or another supported backend.

```python
import papermill as pm

nb = pm.execute_notebook(
    "notebooks/daily_report.ipynb",
    "artifacts/daily_report-2026-03-12.ipynb",
    parameters={
        "run_date": "2026-03-12",
        "limit": 100,
        "include_archived": False,
    },
    kernel_name="python3",
    cwd="notebooks",
    log_output=True,
    progress_bar=False,
    autosave_cell_every=60,
)

print(nb.metadata.papermill["status"])
```

Useful execution arguments for coding agents:

- `kernel_name`: set explicitly when notebook metadata is missing or unreliable.
- `cwd`: run the notebook relative to the repo or notebook directory instead of the process cwd.
- `prepare_only=True`: inject parameters and metadata without executing cells.
- `start_timeout`: increase this when kernels are slow to start.
- `execution_timeout`: fail long-running cells instead of waiting forever.
- `report_mode=True`: hide input cells for report-style outputs.
- `output_path=None`: execute and return the notebook object without saving a file.

### Execute From The CLI

Basic execution:

```bash
papermill notebooks/daily_report.ipynb artifacts/daily_report.ipynb \
  -p run_date 2026-03-12 \
  -p limit 100 \
  -p include_archived false
```

Parameter input modes:

```bash
papermill in.ipynb out.ipynb -p limit 100
papermill in.ipynb out.ipynb -r version 1.0
papermill in.ipynb out.ipynb -f parameters.yaml
papermill in.ipynb out.ipynb -y $'items:\n  - a\n  - b'
papermill in.ipynb out.ipynb -b YWxwaGE6IDAuNgpsaW1pdDogMTAwCg==
```

Use `-p` when you want scalar values to be parsed as numbers or booleans. Use `-r` when the value must remain a string.

### Inspect Notebook Parameters Before Running

For a notebook you did not author, inspect first:

```python
import papermill as pm

params = pm.inspect_notebook("notebooks/daily_report.ipynb")
print(params)
```

CLI equivalent:

```bash
papermill --help-notebook notebooks/daily_report.ipynb
```

This is the fastest way to confirm parameter names, inferred types, and default values before generating automation code.

### Prepare Without Execution

Use this when you want a parameterized notebook artifact for review or a later execution stage:

```python
import papermill as pm

pm.execute_notebook(
    "notebooks/template.ipynb",
    "artifacts/prepared.ipynb",
    parameters={"run_date": "2026-03-12"},
    prepare_only=True,
)
```

CLI equivalent:

```bash
papermill notebooks/template.ipynb artifacts/prepared.ipynb \
  -p run_date 2026-03-12 \
  --prepare-only
```

### Use Remote Storage Paths

Papermill supports local files plus named handlers for remote paths. The upstream docs explicitly call out:

- local filesystem
- `http://` and `https://`
- `s3://`
- `adl://`
- `abs://`
- `gs://`

Example with S3 output:

```bash
AWS_PROFILE=dev papermill local/input.ipynb s3://my-bucket/output.ipynb \
  -p run_date 2026-03-12
```

The handler must be installed through extras and its underlying SDK credentials must already work in the environment.

## Configuration And Auth

Papermill itself has no service-specific auth config. Authentication is delegated to the storage backend libraries used by each handler.

### Local notebooks

No extra auth is required. The two settings that matter most are:

- `cwd` for relative file access
- `kernel_name` for consistent execution in automated jobs

### S3

Install:

```bash
python -m pip install "papermill[s3]==2.7.0"
```

Auth uses the normal boto3 credential chain. In CLI workflows, `AWS_PROFILE` is the most useful knob when multiple accounts are configured.

### Google Cloud Storage

Install:

```bash
python -m pip install "papermill[gcs]==2.7.0"
```

Auth is handled by `gcsfs`. In practice, use the credential flow you already use for Google Cloud tooling, typically Application Default Credentials in local or CI environments.

### Azure Blob Storage / Data Lake

Install:

```bash
python -m pip install "papermill[azure]==2.7.0"
```

Auth is handled by the Azure storage and identity libraries. Keep credential setup outside notebook code and validate it independently before assuming papermill path access works.

## Common Pitfalls

- Missing `parameters` tag: papermill will still inject parameters, but it inserts the cell at the top of the notebook, which is easy to misread during debugging.
- Inter-dependent defaults in the `parameters` cell do not re-evaluate. If the defaults are `a = 1` and `twice = a * 2`, then running with `-p a 9` still leaves `twice = 2`. Put derived values in a later cell.
- Wrong parameter type from the CLI: `-p` parses scalars, while `-r` keeps values as strings. Use YAML-based inputs for lists and nested objects.
- `NoSuchKernel` errors with conda-managed notebooks: install `jupyter` or at least `ipykernel` in the target environment, expose that environment as a Jupyter kernel, or pass `-k <kernel-name>` explicitly.
- Output notebook not written: `execute_notebook(..., output_path=None)` is valid and returns a notebook object, but no artifact is saved anywhere.
- Long-running cells on ephemeral infrastructure: tune `request_save_on_cell_execute`, `autosave_cell_every`, `stdout_file`, `stderr_file`, and `log_output` so partial progress is preserved outside the notebook UI.
- Remote paths without extras: `s3://`, `gs://`, `adl://`, and `abs://` handlers do not work from a bare `pip install papermill`.

## Version-Sensitive Notes For 2.7.0

- `2.7.0` was released on February 27, 2026.
- `2.7.0` supports Python `3.10+` only. The release dropped Python 3.8 and 3.9 support and added Python 3.13 support.
- The 2.7.0 release removed the `ansicolors` dependency and migrated packaging to `pyproject.toml`.
- The changelog also notes that parameter inspection now raises the same missing-kernel and missing-language errors as the other execution pathways.
- The previous release on PyPI was `2.6.0` on April 26, 2024, so older blog posts may assume pre-2.7.0 Python support or dependency behavior.

## Official Sources

- Docs root: `https://papermill.readthedocs.io/en/latest/`
- Installation: `https://papermill.readthedocs.io/en/latest/installation.html`
- Parameterization: `https://papermill.readthedocs.io/en/latest/usage-parameterize.html`
- Inspect: `https://papermill.readthedocs.io/en/latest/usage-inspect.html`
- Execute: `https://papermill.readthedocs.io/en/latest/usage-execute.html`
- CLI: `https://papermill.readthedocs.io/en/latest/usage-cli.html`
- Troubleshooting: `https://papermill.readthedocs.io/en/latest/troubleshooting.html`
- Workflow reference: `https://papermill.readthedocs.io/en/latest/reference/papermill-workflow.html`
- Input/output reference: `https://papermill.readthedocs.io/en/latest/reference/papermill-io.html`
- Package registry: `https://pypi.org/project/papermill/`
