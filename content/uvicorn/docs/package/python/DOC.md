---
name: package
description: "Uvicorn 0.41.0 package guide for serving ASGI apps in Python."
metadata:
  languages: "python"
  versions: "0.41.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "uvicorn,asgi,python,server,fastapi,starlette,websockets"
---

# `uvicorn` Python package

Use `uvicorn` when you need to run an ASGI app such as FastAPI, Starlette, or a plain ASGI callable. For `0.41.0`, treat it as the HTTP/WebSocket server layer, not the application framework.

## Install

`0.41.0` requires Python `>=3.10`.

Minimal install:

```bash
pip install uvicorn==0.41.0
```

With the commonly useful optional runtime extras:

```bash
pip install 'uvicorn[standard]==0.41.0'
```

With `uv`:

```bash
uv add uvicorn==0.41.0
uv add 'uvicorn[standard]==0.41.0'
```

`uvicorn[standard]` is usually the better default for local development and most production installs because it adds:

- `uvloop` for a faster event loop where supported
- `httptools` for HTTP parsing
- `websockets` for WebSocket handling
- `watchfiles` for better `--reload`
- `python-dotenv` for `--env-file`
- `PyYAML` for YAML log config

## Initialize and run

### Plain ASGI app

```python
async def app(scope, receive, send):
    assert scope["type"] == "http"
    await send(
        {
            "type": "http.response.start",
            "status": 200,
            "headers": [
                (b"content-type", b"text/plain"),
                (b"content-length", b"13"),
            ],
        }
    )
    await send({"type": "http.response.body", "body": b"Hello, world!"})
```

Run it:

```bash
uvicorn main:app
```

### Common development command

```bash
uvicorn main:app --reload --host 0.0.0.0 --port 8000
```

Use `main:app` style import strings for CLI runs and for any setup that needs reload or multiple workers.

### Programmatic startup

```python
import uvicorn

if __name__ == "__main__":
    uvicorn.run("main:app", host="127.0.0.1", port=8000, log_level="info")
```

If you need direct lifecycle control:

```python
import asyncio
import uvicorn

async def main():
    config = uvicorn.Config("main:app", host="127.0.0.1", port=8000, log_level="info")
    server = uvicorn.Server(config)
    await server.serve()

if __name__ == "__main__":
    asyncio.run(main())
```

### Application factory

```python
def create_app():
    app = ...
    return app
```

```bash
uvicorn --factory main:create_app
```

Use `--factory` when your app is created at runtime instead of exported as a module-level object.

## Configuration model

Uvicorn supports three configuration paths, in this precedence order:

1. CLI flags
2. `uvicorn.run(...)` keyword arguments
3. `UVICORN_*` environment variables

Typical examples:

```bash
export UVICORN_HOST=0.0.0.0
export UVICORN_PORT=8000
uvicorn main:app
```

Useful options for agents:

- `--host` and `--port` for bind address
- `--reload` for local development
- `--workers` for multiple processes
- `--loop auto|asyncio|uvloop`
- `--http auto|h11|httptools`
- `--ws auto|websockets|websockets-sansio|wsproto`
- `--lifespan auto|on|off`
- `--proxy-headers` and `--forwarded-allow-ips`
- `--root-path` when mounted behind a path prefix
- `--env-file` for app environment variables
- `--log-config`, `--log-level`, `--no-access-log`
- `--ssl-keyfile` and `--ssl-certfile` for direct TLS termination

## FastAPI and Starlette usage

FastAPI and Starlette projects typically expose an `app` object and run the same way:

```python
from fastapi import FastAPI

app = FastAPI()

@app.get("/health")
async def health():
    return {"ok": True}
```

```bash
uvicorn main:app --reload
```

If the module is not in the current working directory, add `--app-dir` or run from the correct project root.

## Proxies, TLS, and auth boundaries

Uvicorn can read forwarded headers and can terminate TLS itself, but many deployments place it behind Nginx, a load balancer, or a CDN.

Common reverse-proxy setup:

```bash
uvicorn main:app --proxy-headers --forwarded-allow-ips='127.0.0.1'
```

Adjust `--forwarded-allow-ips` to the actual trusted proxy addresses. Do not set `'*'` unless you intentionally trust every upstream hop.

For self-hosted production, common patterns are:

- Uvicorn behind Nginx using a UNIX socket or local TCP port
- Uvicorn behind a platform load balancer
- Uvicorn managed by a process manager or Gunicorn-compatible worker setup

Auth is not a Uvicorn feature. Handle authentication, authorization, sessions, and CSRF in the ASGI app or in upstream infrastructure. This is an inference from Uvicorn's role as an ASGI server rather than an app framework.

## Production patterns

Single-process:

```bash
uvicorn main:app --host 0.0.0.0 --port 8000
```

Built-in multiprocess manager:

```bash
uvicorn main:app --workers 4 --host 0.0.0.0 --port 8000
```

Relevant production controls:

- `--workers` defaults from `WEB_CONCURRENCY` when set
- `--limit-concurrency` can shed load with HTTP 503 responses
- `--limit-max-requests` can recycle workers after N requests
- `--limit-max-requests-jitter` was added in `0.41.0` to stagger those restarts
- `--timeout-graceful-shutdown` controls shutdown wait time
- `--timeout-worker-healthcheck` controls worker healthcheck timeout

Uvicorn's built-in process manager uses `spawn`, not pre-fork. That keeps multiprocess mode usable on Windows, but it also means your startup code must be import-safe.

## Common pitfalls

- `--reload` and `--workers` are mutually exclusive.
- Passing an app object directly to `uvicorn.run(app, ...)` only works when you are not using reload or multiprocessing. Prefer `"module:app"` import strings for reusable startup code.
- Put `uvicorn.run(...)` inside `if __name__ == "__main__":` when using reload or workers.
- `--env-file` is for the application environment. It does not configure Uvicorn itself, and `UVICORN_*` variables are not loaded from that file.
- `--reload-include` and `--reload-exclude` only take effect when `watchfiles` is installed. Without it, reload falls back to polling Python files.
- `--proxy-headers` is not enough for every proxy chain. Complex multi-proxy setups may need explicit ASGI middleware to derive client IP and scheme correctly.
- If imports fail on startup, verify the `APP` import string and use `--app-dir` when the module root is not on `PYTHONPATH`.

## Version-sensitive notes for `0.41.0`

- PyPI marks `0.41.0` as requiring Python `>=3.10`.
- Current docs expose `--ws websockets-sansio`; that protocol option was added in `0.35.0`, so older blog posts may not match current WebSocket configuration.
- `0.41.0` adds `--limit-max-requests-jitter` for staggered worker recycling.
- The `uvicorn.workers` module is deprecated in the official docs and scheduled for removal in a future release. For Gunicorn-managed deployments, the docs now recommend the separate `uvicorn-worker` package.
- The official docs site is current, not version-pinned. When behavior matters, prefer examples that still match the `0.41.0` CLI and release notes.

## Official references

- Docs root: https://www.uvicorn.org/
- Canonical docs link advertised by upstream: https://uvicorn.dev/
- Installation: https://www.uvicorn.org/installation/
- Settings and CLI options: https://www.uvicorn.org/settings/
- Deployment: https://www.uvicorn.org/deployment/
- Release notes: https://www.uvicorn.org/release-notes/
- PyPI package: https://pypi.org/project/uvicorn/
