---
name: package
description: "docling package guide for Python document conversion, OCR, CLI usage, and pipeline configuration"
metadata:
  languages: "python"
  versions: "2.78.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "docling,python,documents,pdf,ocr,conversion,cli"
---

# docling Python Package Guide

## Golden Rule

- Use the official `docling` package for document conversion and start with `DocumentConverter` unless you need pipeline-level customization.
- Pin `docling==2.78.0` when you need reproducible behavior, because the stable docs site can describe newer releases.
- Install optional extras only for the features you actually use (`easyocr`, `tesserocr`, `vlm`, `chunking`, `asr`).
- Keep remote inference opt-in. Upstream examples explicitly require `enable_remote_services=True` for remote VLM usage.

## Version-Sensitive Notes

- This entry is pinned to the version used here `2.78.0`, and that exact version is published on PyPI.
- The repository's tagged `v2.78.0` README is the safest upstream reference for examples that must match this version exactly.
- The official docs site is a moving "stable" target, so examples there can drift ahead of `2.78.0`.
- The `v2.78.0` package metadata requires Python `>=3.10,<4.0`.
- The package exposes optional extras for `chunking`, `easyocr`, `tesserocr`, `vlm`, and `asr`. Do not assume the base install includes every OCR or VLM backend.

## Install

Use a pinned install for repeatable agent behavior:

```bash
python -m pip install "docling==2.78.0"
```

Install only the extras you need:

```bash
python -m pip install "docling[easyocr]==2.78.0"
python -m pip install "docling[vlm]==2.78.0"
python -m pip install "docling[chunking]==2.78.0"
```

For offline or pre-provisioned environments, upstream documents a separate model download helper:

```bash
python -m pip install "docling-tools"
docling-tools models download
```

## Recommended Setup

Start with the base converter, then add format or pipeline options only when the default output is not sufficient.

```python
from docling.document_converter import DocumentConverter

converter = DocumentConverter()
```

`docling` supports common office and web document inputs including PDF, DOCX, XLSX, HTML, image formats, Markdown, AsciiDoc, and CSV. For most coding tasks, the practical output targets are Markdown, structured text, or the internal document model.

## Core Usage

### Convert One Document To Markdown

```python
from docling.document_converter import DocumentConverter

converter = DocumentConverter()
result = converter.convert("report.pdf")

markdown = result.document.export_to_markdown()
print(markdown)
```

### Convert A URL

The upstream README shows that `convert()` accepts remote URLs directly:

```python
from docling.document_converter import DocumentConverter

converter = DocumentConverter()
result = converter.convert("https://arxiv.org/pdf/2206.01062")

print(result.document.export_to_markdown())
```

### Convert Multiple Files

```python
from pathlib import Path

from docling.document_converter import DocumentConverter

converter = DocumentConverter()

for path in Path("incoming").glob("*.pdf"):
    result = converter.convert(path)
    output = path.with_suffix(".md")
    output.write_text(result.document.export_to_markdown(), encoding="utf-8")
```

### Use The CLI

Upstream documents the `docling` CLI for direct conversions:

```bash
docling https://arxiv.org/pdf/2206.01062
docling ./tests/data/2305.03393v1-pg9.pdf
```

Use the CLI when you need a shell-friendly conversion step in scripts or quick inspection during debugging. Use the Python API when you need programmatic control over output handling and pipeline options.

## Pipeline Configuration

Reach for pipeline options when you need OCR, hardware acceleration, or table extraction tuning.

```python
from docling.backend.docling_parse_v2_backend import DoclingParseV2DocumentBackend
from docling.datamodel.base_models import InputFormat
from docling.datamodel.pipeline_options import AcceleratorDevice, AcceleratorOptions
from docling.document_converter import DocumentConverter, PdfFormatOption
from docling.pipeline.standard_pdf_pipeline import StandardPdfPipeline

converter = DocumentConverter(
    format_options={
        InputFormat.PDF: PdfFormatOption(
            pipeline_cls=StandardPdfPipeline,
            backend=DoclingParseV2DocumentBackend,
            pipeline_options=dict(
                accelerator_options=AcceleratorOptions(
                    num_threads=4,
                    device=AcceleratorDevice.AUTO,
                ),
            ),
        )
    }
)
```

Common knobs to check first:

- OCR enablement and OCR backend
- table structure extraction
- accelerator device and thread count
- artifacts or model cache location for offline runs

## Offline Models And Artifacts

If the runtime environment cannot download models on demand, prefetch them and point the pipeline to the local artifacts directory.

```bash
docling-tools models download --output-dir ./docling-models
```

```python
from docling.datamodel.pipeline_options import (
    PdfPipelineOptions,
    RapidOcrOptions,
    TesseractOcrOptions,
)

pipeline_options = PdfPipelineOptions()
pipeline_options.artifacts_path = "docling-models"
pipeline_options.ocr_options = RapidOcrOptions()
```

Swap in `TesseractOcrOptions()` if that is the backend you installed and provisioned.

## Config And Auth

`docling` does not require package-level authentication for local document conversion.

What usually matters instead:

- Network access for downloading model artifacts on first use
- local filesystem paths for cached artifacts and model bundles
- optional third-party credentials only if you deliberately enable remote model or VLM services outside the default local path

For remote VLM scenarios, upstream examples keep remote calls disabled unless you set `enable_remote_services=True`. Treat that as an explicit trust boundary and keep provider secrets in environment variables managed by the backend you configure, not in source code.

## Common Pitfalls

- Python version mismatch: `docling==2.78.0` requires Python 3.10 or newer.
- Missing extras: OCR and VLM features can fail at import or runtime if you installed only the base package.
- Stable docs drift: the docs site may document behavior newer than `2.78.0`; confirm against the `v2.78.0` tagged repo when copying examples.
- First-run downloads: model assets may be fetched lazily, which breaks in CI or air-gapped environments unless you pre-download them.
- Remote inference assumptions: enabling remote services can send document content to external systems; do not enable it by default in sensitive environments.
- Over-customizing early: start with plain `DocumentConverter()` before wiring custom backends or pipeline classes.

## Practical Agent Workflow

1. Install `docling==2.78.0`, plus only the extras required by the task.
2. Start with `DocumentConverter().convert(...)` and export to Markdown.
3. If output quality is poor for scanned PDFs, add explicit OCR pipeline options.
4. If the environment is offline, pre-download models with `docling-tools` and set `artifacts_path`.
5. If the upstream stable docs and installed behavior disagree, trust the tagged `v2.78.0` repository content over newer stable-site examples.

## Official Sources

- Docs root: `https://docling-project.github.io/docling/`
- Installation docs: `https://docling-project.github.io/docling/installation/`
- Usage docs: `https://docling-project.github.io/docling/usage/`
- CLI docs: `https://docling-project.github.io/docling/usage/cli/`
- Repo README for `v2.78.0`: `https://github.com/docling-project/docling/blob/v2.78.0/README.md`
- PyPI package page: `https://pypi.org/project/docling/2.78.0/`
