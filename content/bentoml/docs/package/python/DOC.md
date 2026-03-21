---
name: package
description: "BentoML package guide for Python model serving, packaging, and BentoCloud deployment workflows"
metadata:
  languages: "python"
  versions: "1.4.36"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "bentoml,python,serving,inference,mlops,deployment"
---

# BentoML Python Package Guide

## What It Is

`bentoml` is a Python framework for turning model inference code into production services. The current upstream docs center on class-based services defined with `@bentoml.service`, HTTP APIs defined with `@bentoml.api`, packaging those services into Bentos, and then serving or containerizing them for deployment.

## Installation

Pin the package when you need agent output to match the version used here:

```bash
python -m pip install "bentoml==1.4.36"
```

With Poetry:

```bash
poetry add "bentoml==1.4.36"
```

With uv:

```bash
uv add bentoml==1.4.36
```

If the service depends on model-framework packages, add those separately. BentoML packaging can also install extra Python packages declared in the service config.

## Initialize A Service

Create a module such as `service.py` and define a class service:

```python
import bentoml

@bentoml.service(
    traffic={"timeout": 30},
    workers=1,
)
class Summarizer:
    def __init__(self) -> None:
        self.prefix = "summary:"

    @bentoml.api
    def summarize(self, text: str) -> str:
        return f"{self.prefix} {text[:80]}"
```

Run it locally:

```bash
bentoml serve service:Summarizer
```

The `service:Summarizer` target is `<module>:<service class>`. For local iteration, the docs also show `bentoml serve --reload ...` patterns.

## Core Usage

### Define HTTP APIs

Methods decorated with `@bentoml.api` become HTTP endpoints. The endpoint path defaults to the method name.

```python
import bentoml
from pydantic import BaseModel

class PredictRequest(BaseModel):
    prompt: str

class PredictResponse(BaseModel):
    output: str

@bentoml.service
class TextService:
    @bentoml.api
    def health(self) -> dict[str, str]:
        return {"status": "ok"}

    @bentoml.api
    def predict(self, request: PredictRequest) -> PredictResponse:
        return PredictResponse(output=request.prompt.upper())
```

Use plain Python types for simple JSON endpoints. Reach for Pydantic models or explicit IO descriptors when request or response shapes need validation or when you are serving files, images, or other non-JSON content.

### Package A Deployable Bento

Once the service works locally, build a Bento:

```bash
bentoml build
```

This produces a versioned Bento artifact containing the service code and its runtime configuration. You can then containerize it:

```bash
bentoml containerize my_service:latest
```

On Apple Silicon, the official docs recommend passing `--platform=linux/amd64` when the deployment target expects x86 containers.

### Bundle Python Dependencies

BentoML supports package installation from service configuration. A minimal example in `bentofile.yaml`:

```yaml
service: "service:Summarizer"
include:
  - "*.py"
python:
  packages:
    - "bentoml==1.4.36"
    - "transformers"
    - "torch"
```

Use this to make builds reproducible. If your project already has a lockfile, keep BentoML packaging aligned with it instead of duplicating drifting dependency pins.

## Config And Auth

### Local Configuration

- `BENTOML_HOME` controls where Bentos, models, environments, and temporary files are stored.
- The default home directory is `~/bentoml`.
- The docs recommend setting `BENTOML_HOME` explicitly when you want isolated local state in CI, tests, or per-project environments.

Example:

```bash
export BENTOML_HOME="$PWD/.bentoml"
```

Service-level runtime settings can also be declared on `@bentoml.service`, including traffic limits, worker count, and GPU/CPU resource requirements:

```python
@bentoml.service(
    workers=2,
    resources={"gpu": 1, "gpu_type": "nvidia-l4"},
    traffic={"timeout": 60, "concurrency": 16},
)
class GPUService:
    ...
```

### BentoCloud Auth

For BentoCloud deployment, authenticate the CLI before pushing or deploying:

```bash
bentoml cloud login
```

After login, choose the correct context before operating on deployments:

```bash
bentoml cloud current-context
```

If your automation runs non-interactively, prefer the official BentoCloud token-based setup rather than embedding credentials in code.

## Common Pitfalls

- Old service style vs current docs: for `1.4.x`, prefer class-based `@bentoml.service` services. Older blog posts may still show `svc = bentoml.Service(...)`.
- Packaging only `bentoml`: model frameworks such as `torch`, `transformers`, or `diffusers` are not implied. Add them explicitly to your environment or `bentofile.yaml`.
- Mutable local state in tests: BentoML writes to `~/bentoml` by default. Set `BENTOML_HOME` for isolated test runs and CI.
- Container target mismatch: on Apple Silicon, local container builds may not match Linux x86 production unless you pass `--platform=linux/amd64`.
- Hidden endpoint assumptions: method names become routes by default. Renaming a Python method changes the HTTP path unless you pin the route explicitly.
- Resource declarations are deployment inputs, not magic autoscaling. If you mark `resources={"gpu": 1}` on a machine without GPUs, local runs still need a compatible runtime.
- Docs-root drift: the official docs URL is `/en/latest/`, not a frozen `1.4.36` snapshot. Verify any CLI flag or deployment behavior that looks newer than your installed package.

## Version-Sensitive Notes

- `1.4.36` is the package version covered here and was released on PyPI on `2026-03-10`.
- The official docs currently document the modern class-based service API. The service page explicitly notes that, starting with BentoML `1.2`, services should be defined as a class decorated with `@bentoml.service`.
- The service docs also note behavior added in BentoML `1.3.20` for accessing deployment context from inside a service. If you copy advanced deployment examples, confirm they are compatible with `1.4.36`.
- Inference from the official docs structure: because the docs root is `en/latest`, examples can move ahead of the pinned package version. Treat the live docs as canonical for current patterns, but pin installs and packaging config to the version your project actually uses.

## Practical Guidance For Agents

1. Start with one small class-based service and one or two `@bentoml.api` methods before adding model loading, batching, or GPU configuration.
2. Keep runtime dependencies explicit in `bentofile.yaml` or your environment manager so Bento builds are reproducible.
3. Set `BENTOML_HOME` in CI and tests to avoid leaking state across jobs.
4. Use `bentoml serve module:ServiceClass` for local verification before building or deploying.
5. Add BentoCloud login and context checks only when the task actually includes cloud deployment; they are not needed for local serving.

## Official Sources

- Documentation root: `https://docs.bentoml.com/en/latest/`
- Create services: `https://docs.bentoml.com/en/latest/build-with-bentoml/services.html`
- Package dependencies and Bento builds: `https://docs.bentoml.com/en/latest/get-started/packaging-for-deployment.html`
- Runtime config and `BENTOML_HOME`: `https://docs.bentoml.com/en/latest/reference/bentoml/configurations.html`
- BentoCloud CLI auth and context: `https://docs.bentoml.com/en/latest/reference/bentoml/bento-cloud.html`
- PyPI package page: `https://pypi.org/project/bentoml/`
- PyPI release history for `1.4.36`: `https://pypi.org/project/bentoml/#history`
