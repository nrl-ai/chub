---
name: package
description: "modal Python package guide for building, running, and deploying Python functions, jobs, and web endpoints on Modal"
metadata:
  languages: "python"
  versions: "1.3.5"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "modal,python,serverless,cloud,gpu,jobs"
---

# modal Python Package Guide

## What This Package Is For

`modal` is the Python SDK and CLI for defining Python functions, container images, jobs, web endpoints, scheduled tasks, and stateful resources that run on Modal's managed infrastructure.

Use it when you need to:

- run Python functions remotely with CPU or GPU resources
- package dependencies into a reproducible image
- deploy long-lived app objects and web endpoints
- attach secrets, volumes, queues, or dictionaries to remote code
- keep local orchestration code in Python instead of writing Dockerfiles and cloud-control code by hand

The normal import is:

```python
import modal
```

## Version-Sensitive Notes

- Official sources checked on 2026-03-12 align on `1.3.5` as the current package version for this guide.
- Modal's docs site is a rolling docs site, not a version-pinned archive. Confirm exact method availability against your installed package if you are copying older examples or upgrading from a pre-1.x codebase.
- PyPI currently declares Python `>=3.9,<3.15`. If your runtime is older than Python 3.9, install will fail before you reach any Modal code.
- Prefer current docs that use `modal.App(...)` and `@app.function(...)`. Older examples from blogs or issue threads may use older naming or decorators.

## Install

Install the package and CLI from PyPI:

```bash
python -m pip install modal
```

If you need exact parity with this entry:

```bash
python -m pip install "modal==1.3.5"
```

Common alternatives:

```bash
uv add modal
poetry add modal
```

Confirm the CLI is available:

```bash
modal version
```

## Authentication And Configuration

For local development, authenticate once with the CLI:

```bash
modal setup
```

This opens a browser flow and stores a token for the active profile.

For CI or other non-interactive environments, configure a token explicitly:

```bash
export MODAL_TOKEN_ID="..."
export MODAL_TOKEN_SECRET="..."
```

Practical rules:

- use `modal setup` on a developer machine
- use token environment variables in CI
- do not hardcode credentials in source files
- if you use multiple profiles, inspect the active config with `modal config show`

## Core Workflow

### 1. Define an app and image

```python
import modal

app = modal.App("hello-modal")

image = (
    modal.Image.debian_slim(python_version="3.11")
    .pip_install("httpx", "pandas")
)
```

Use `modal.Image` to declare the runtime environment for remote execution. If your code depends on a local package that is not installed from PyPI inside the image, add it explicitly with `add_local_python_source(...)`.

### 2. Add a remote function

```python
import modal

app = modal.App("hello-modal")
image = modal.Image.debian_slim().pip_install("httpx")

@app.function(image=image, timeout=300)
def fetch_status(url: str) -> int:
    import httpx

    response = httpx.get(url, timeout=30.0)
    response.raise_for_status()
    return response.status_code
```

Call it remotely from a local entrypoint:

```python
@app.local_entrypoint()
def main():
    status = fetch_status.remote("https://modal.com")
    print(status)
```

Run it from your machine:

```bash
modal run app.py
```

Key call styles from the current API reference:

- `.remote(...)`: execute remotely and wait for the result
- `.spawn(...)`: start work asynchronously and get a function call handle
- `.map(...)`: fan out one function over many inputs
- `.local(...)`: run the wrapped function locally for debugging when supported by your workflow

### 3. Deploy when you want a stable app

```bash
modal deploy app.py
```

Use `modal deploy` for persistent app objects such as scheduled jobs, web endpoints, or named functions that other systems call later. Use `modal run` for ad hoc local orchestration and testing.

## Packaging Local Code And Assets

Remote containers only see what you put into the image or what you mount explicitly.

Package a local Python module into the image:

```python
image = (
    modal.Image.debian_slim()
    .pip_install("pydantic")
    .add_local_python_source("my_package")
)
```

Bundle a local data directory:

```python
image = modal.Image.debian_slim().add_local_dir("prompts", remote_path="/root/prompts")
```

Use this whenever your app imports local modules, prompt templates, model files, or other assets that are not downloaded inside the container build itself.

## Secrets And Environment Variables

Attach a named Modal secret to a function:

```python
secret = modal.Secret.from_name("openai-api-key")

@app.function(secrets=[secret])
def use_secret() -> None:
    import os

    print(bool(os.environ["OPENAI_API_KEY"]))
```

Create an ephemeral secret from local values when needed:

```python
runtime_secret = modal.Secret.from_dict(
    {
        "OPENAI_API_KEY": "sk-...",
        "APP_ENV": "dev",
    }
)
```

Practical rule: if code needs an environment variable inside the remote container, pass it through a Modal secret or another explicit Modal config mechanism. Your shell environment is not automatically mirrored into remote execution.

## Persistent Storage And Shared State

Use a `Volume` when multiple runs or containers need shared files:

```python
volume = modal.Volume.from_name("model-cache", create_if_missing=True)

@app.function(volumes={"/cache": volume})
def write_file() -> None:
    path = "/cache/result.txt"
    with open(path, "w", encoding="utf-8") as f:
        f.write("hello from modal\n")
    volume.commit()
```

Important behavior from the reference docs:

- `commit()` persists new or changed files from the running container into the volume
- `reload()` refreshes a mounted volume view from the latest committed state
- use a `Volume` for files, not for secrets

If you need keyed shared state instead of files, consider Modal's named objects such as `modal.Dict` or `modal.Queue`.

## Common Patterns

### Batch fan-out

```python
@app.function()
def square(x: int) -> int:
    return x * x

@app.local_entrypoint()
def main():
    for result in square.map(range(5)):
        print(result)
```

### GPU-backed function

```python
@app.function(gpu="A10G", image=modal.Image.debian_slim().pip_install("torch"))
def train() -> str:
    return "started"
```

Only request a GPU when the workload actually needs one. Modal allocates infrastructure from the decorator configuration, so resource choices affect startup behavior and cost.

### Web entrypoints

Use `modal deploy` plus the current web-serving decorators from the Modal guide when you need HTTP entrypoints. Keep the local `@app.local_entrypoint()` for smoke tests and administrative workflows, not for the production request path.

## Common Pitfalls

### Calling the function the wrong way

`@app.function` creates a Modal function object. For remote execution, call `.remote(...)`, `.spawn(...)`, or `.map(...)` on that object instead of assuming a plain Python call will execute in the cloud.

### Forgetting to package local imports

If the remote function imports a local package, add it to the image with `add_local_python_source(...)` or install it during the image build. Your local virtualenv does not get copied automatically.

### Depending on undeclared Python packages

Every third-party dependency used inside the remote function must be installed in the image with `pip_install(...)`, `uv_pip_install(...)`, or another image build step.

### Assuming local environment variables exist remotely

Remote containers only receive what you pass in through Modal-managed configuration such as secrets. Missing environment variables usually mean the function was configured locally but not attached remotely.

### Forgetting volume synchronization

If one container writes files and another needs to read them later, commit the changes and reload where needed. A mounted `Volume` is shared state, but visibility still depends on the documented `commit()` and `reload()` flow.

### Mixing local orchestration and remote imports carelessly

Keep orchestration code inside `@app.local_entrypoint()` or normal top-level Python, and keep cloud-executed code inside decorated functions and classes. This makes it obvious what runs locally and what runs in Modal's containers.

## Minimal End-To-End Example

```python
import modal

app = modal.App("fetch-example")

image = (
    modal.Image.debian_slim(python_version="3.11")
    .pip_install("httpx")
)

api_secret = modal.Secret.from_name("api-token")
cache = modal.Volume.from_name("fetch-cache", create_if_missing=True)

@app.function(image=image, secrets=[api_secret], volumes={"/cache": cache})
def fetch(url: str) -> str:
    import httpx

    body = httpx.get(url, timeout=30.0).text
    with open("/cache/last-response.html", "w", encoding="utf-8") as f:
        f.write(body)
    cache.commit()
    return body[:200]

@app.local_entrypoint()
def main():
    print(fetch.remote("https://modal.com"))
```

Run locally:

```bash
modal run app.py
```

Deploy when you want a stable app object:

```bash
modal deploy app.py
```

## Official Sources Used For This Entry

- Modal guide: `https://modal.com/docs/guide`
- Modal API reference: `https://modal.com/docs/reference`
- `modal.App` reference: `https://modal.com/docs/reference/modal.App`
- `modal.Function` reference: `https://modal.com/docs/reference/modal.Function`
- `modal.Image` reference: `https://modal.com/docs/reference/modal.Image`
- `modal.Secret` reference: `https://modal.com/docs/reference/modal.Secret`
- `modal.Volume` reference: `https://modal.com/docs/reference/modal.Volume`
- Local data guide: `https://modal.com/docs/guide/local-data`
- Global objects guide: `https://modal.com/docs/guide/global-objects`
- Changelog: `https://modal.com/docs/reference/changelog`
- PyPI registry page: `https://pypi.org/project/modal/`
