---
name: package
description: "nbconvert Python package guide for converting, executing, and templating Jupyter notebooks"
metadata:
  languages: "python"
  versions: "7.17.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "nbconvert,jupyter,notebook,export,html,pdf,cli"
---

# nbconvert Python Package Guide

## Golden Rule

Use `nbconvert` when you need to convert or execute Jupyter notebooks from Python or the `jupyter nbconvert` CLI. Install the extra system dependencies that match the export target before debugging template or exporter code: Pandoc for some markup conversions, XeLaTeX for `--to pdf`, and Playwright/Chromium for `--to webpdf`.

## Install

Pin the package version your project expects:

```bash
python -m pip install "nbconvert==7.17.0"
```

Common alternatives:

```bash
uv add "nbconvert==7.17.0"
poetry add "nbconvert==7.17.0"
conda install nbconvert
```

For WebPDF output, upstream documents the `webpdf` extra:

```bash
python -m pip install "nbconvert[webpdf]==7.17.0"
playwright install chromium
```

Important dependency matrix:

- `--to html`, `--to markdown`, `--to script`, `--to notebook`: package install is usually enough.
- `--to rst` and some markup conversions: requires Pandoc.
- `--to pdf`: requires a TeX environment with XeLaTeX.
- `--to webpdf`: requires Playwright/Chromium; `--allow-chromium-download` can fetch Chromium if a suitable version is missing.

## Initialize And Check The CLI

The package integrates into Jupyter and exposes `jupyter nbconvert`:

```bash
jupyter nbconvert --help
jupyter nbconvert --show-config
```

Generate a default config file when you need stable project-level settings:

```bash
jupyter nbconvert --generate-config
```

Upstream documents the config file path as:

```text
~/.jupyter/jupyter_nbconvert_config.py
```

## Core CLI Usage

### Convert a notebook to HTML

```bash
jupyter nbconvert --to html notebook.ipynb
```

Notes:

- HTML uses the `lab` template by default in current docs.
- The output is written in the current working directory with the notebook base name.
- Supporting assets go into a sibling `*_files/` directory when needed.

### Send simple output to stdout

```bash
jupyter nbconvert --to markdown notebook.ipynb --stdout
```

### Execute a notebook and save the executed copy

```bash
jupyter nbconvert --to notebook --execute report.ipynb
```

Behavior to expect:

- The default output file is `report.nbconvert.ipynb`.
- `--inplace` overwrites the original notebook.
- Execution aborts on the first exception unless you also pass `--allow-errors`.

### Convert multiple notebooks

```bash
jupyter nbconvert --to html notebook1.ipynb notebook2.ipynb
jupyter nbconvert --to html notebook*.ipynb
```

### Pick a specific template

```bash
jupyter nbconvert --to html notebook.ipynb --template classic
jupyter nbconvert --to html notebook.ipynb --template basic
```

## Execute Notebooks From Python

Use `ExecutePreprocessor` when you need deterministic notebook execution inside tests, CI, report pipelines, or dataset refresh jobs.

```python
from pathlib import Path

import nbformat
from nbconvert.preprocessors import ExecutePreprocessor

notebook_path = Path("reports/daily.ipynb")

with notebook_path.open(encoding="utf-8") as f:
    nb = nbformat.read(f, as_version=4)

ep = ExecutePreprocessor(timeout=600, kernel_name="python3")
ep.preprocess(nb, {"metadata": {"path": str(notebook_path.parent)}})

with Path("reports/daily.executed.ipynb").open("w", encoding="utf-8") as f:
    nbformat.write(nb, f)
```

Execution details that matter:

- Upstream documents a default per-cell timeout of `30` seconds.
- Set `timeout=None` or `timeout=-1` to remove the timeout.
- `kernel_name` overrides notebook metadata when you must force a specific kernel.
- The execution `path` controls the working directory for relative file access.

If you want execution to continue after cell failures:

```python
from nbconvert.preprocessors import ExecutePreprocessor

ep = ExecutePreprocessor(timeout=600, kernel_name="python3", allow_errors=True)
```

If you want to stop on the first error but still persist the partially executed notebook, catch `CellExecutionError` and write the notebook in `finally`.

## Convert Notebooks From Python

Use exporters directly when you want conversion in-process instead of spawning the CLI.

```python
import nbformat
from nbconvert import HTMLExporter

with open("notebook.ipynb", encoding="utf-8") as f:
    nb = nbformat.read(f, as_version=4)

exporter = HTMLExporter(template_name="classic")
body, resources = exporter.from_notebook_node(nb)

with open("notebook.html", "w", encoding="utf-8") as f:
    f.write(body)
```

Exporter behavior to remember:

- Exporters are effectively stateless and can be reused across multiple notebooks.
- Exporters expose `from_notebook_node`, `from_file`, and `from_filename`.
- The return value is `(body, resources)`.
- `resources` contains output metadata and, for some exporters or preprocessors, extracted assets.
- Writing files is your job unless you use a writer such as `FilesWriter`.

When you need extracted figures or other preprocessor-driven outputs, configure the exporter with `traitlets.Config`:

```python
from traitlets.config import Config
from nbconvert import HTMLExporter

c = Config()
c.HTMLExporter.preprocessors = ["nbconvert.preprocessors.ExtractOutputPreprocessor"]

exporter = HTMLExporter(config=c)
body, resources = exporter.from_filename("notebook.ipynb")

print(sorted(resources.get("outputs", {}).keys()))
```

## Configuration And Templates

There is no service authentication layer in `nbconvert`. Configuration is about exporters, templates, preprocessors, output paths, and execution behavior.

Useful CLI flags and aliases from the upstream config reference:

- `--execute`
- `--allow-errors`
- `--stdout`
- `--inplace`
- `--output`
- `--output-dir`
- `--template`
- `--template-file`
- `--no-input`
- `--allow-chromium-download`
- `--disable-chromium-sandbox`

Template configuration highlights:

- Templates live under Jupyter data paths and typically include a `conf.json`.
- `conf.json` declares the base template, supported mimetypes, and preprocessors to register.
- Template inheritance is layered; derived templates can override files like `index.html.j2`, `base.html.j2`, and static assets.
- If your custom template directory is outside the notebook directory, add `--TemplateExporter.extra_template_basedirs=/path/to/parent`.

Example: hide code input across templates without inventing a custom exporter:

```python
from traitlets.config import Config
from nbconvert import HTMLExporter

c = Config()
c.TemplateExporter.exclude_input = True

exporter = HTMLExporter(config=c)
body, resources = exporter.from_filename("notebook.ipynb")
```

## Remove Cells, Inputs, Or Outputs

For report-style output, prefer metadata-driven preprocessing over post-processing the generated HTML.

```python
from traitlets.config import Config
from nbconvert.exporters import HTMLExporter
from nbconvert.preprocessors import TagRemovePreprocessor

c = Config()
c.TagRemovePreprocessor.enabled = True
c.TagRemovePreprocessor.remove_cell_tags = ("remove_cell",)
c.TagRemovePreprocessor.remove_input_tags = ("remove_input",)
c.TagRemovePreprocessor.remove_all_outputs_tags = ("remove_output",)
c.HTMLExporter.preprocessors = ["nbconvert.preprocessors.TagRemovePreprocessor"]

exporter = HTMLExporter(config=c)
exporter.register_preprocessor(TagRemovePreprocessor(config=c), True)

body, resources = exporter.from_filename("notebook.ipynb")
```

CLI equivalent:

```bash
jupyter nbconvert notebook.ipynb \
  --to html \
  --TagRemovePreprocessor.enabled=True \
  --TagRemovePreprocessor.remove_cell_tags remove_cell
```

Use `RegexRemovePreprocessor` when the decision should be based on cell content instead of tags.

## Custom Exporters

Reach for a custom exporter only when templates and built-in preprocessors are not enough.

Upstream extension model:

- Custom exporters are importable Python classes.
- Packages can register them via the `nbconvert.exporters` entry-point group.
- After registration, users can call them with `jupyter nbconvert --to your-exporter-name notebook.ipynb`.
- Without entry points, users can still pass the fully qualified class name to `--to`.

This matters for agent-generated code because many examples online subclass an exporter when a template or preprocessor would be simpler and more stable.

## Common Pitfalls

- Missing system dependencies are the most common failure mode. `nbconvert` may be installed correctly while PDF or RST export still fails because Pandoc, XeLaTeX, or Chromium is missing.
- `--to notebook --execute` writes `*.nbconvert.ipynb` by default, not back to the source file. Use `--inplace` only when overwriting is intentional.
- Relative paths during execution depend on `metadata.path` in `ExecutePreprocessor.preprocess(...)`, not on the notebook file location automatically.
- The default execution timeout is only `30` seconds per cell. Long-running notebooks need an explicit timeout override.
- `allow_errors=True` preserves failing outputs but can hide broken notebooks if your pipeline forgets to inspect the resulting file.
- `TemplateExporter.exclude_input` and related flags affect rendered output, not the original notebook content.
- `--disable-chromium-sandbox` is often required in containers for WebPDF, but upstream explicitly warns it can enable server-side code execution from notebook JavaScript in some circumstances.
- Custom exporter packages should keep CLI semantics aligned with built-ins; otherwise agent-generated flags may stop working.

## Version-Sensitive Notes For 7.17.0

- The upstream docs site and PyPI both reflect `7.17.0` as the current package version on that date.
- The `7.17.0` changelog adds support for arbitrary browser arguments on `WebPDFExporter`, which is relevant when Chromium needs extra launch flags in CI or locked-down environments.
- The same release fixes `QtPNGExporter` returning empty bytes on macOS.
- The `7.17.0` changelog also notes a fix for `CVE-2025-53000` around secure Inkscape path handling on Windows.
- There is mild upstream version drift in Python support messaging: the install page still says tested Python `3.9-3.12`, while the `7.17.0` changelog notes test updates for Python `3.13`, `3.14`, and dropping tests on `3.9`; PyPI metadata still says `Requires: Python >=3.9`. Prefer the PyPI floor for installation gating and treat the tested-version wording as potentially lagging docs text.

## Official Sources

- Docs root: https://nbconvert.readthedocs.io/en/latest/
- Installation: https://nbconvert.readthedocs.io/en/latest/install.html
- CLI usage: https://nbconvert.readthedocs.io/en/latest/usage.html
- Execution API: https://nbconvert.readthedocs.io/en/latest/execute_api.html
- Library API guide: https://nbconvert.readthedocs.io/en/latest/nbconvert_library.html
- Config reference: https://nbconvert.readthedocs.io/en/latest/config_options.html
- Template customization: https://nbconvert.readthedocs.io/en/latest/customizing.html
- Exporter customization: https://nbconvert.readthedocs.io/en/latest/external_exporters.html
- Cell removal: https://nbconvert.readthedocs.io/en/latest/removing_cells.html
- Changelog: https://nbconvert.readthedocs.io/en/latest/changelog.html
- PyPI metadata: https://pypi.org/project/nbconvert/
