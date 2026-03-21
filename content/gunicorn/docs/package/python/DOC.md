---
name: package
description: "Gunicorn package guide for Python - production WSGI server setup, configuration, and deployment notes"
metadata:
  languages: "python"
  versions: "25.1.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "gunicorn,python,wsgi,server,deployment"
---

# Gunicorn Python Package Guide

## What It Is

`gunicorn` is the standard production process manager and HTTP server for Python WSGI apps on Unix-like systems. Use it to run Flask, Django, Pyramid, or any other WSGI callable behind a reverse proxy such as Nginx.

Important source note: older docs links may point to `https://docs.gunicorn.org/en/stable/`, but current official docs are published at `https://gunicorn.org/`. The old `docs.gunicorn.org` stable pages still render `23.0.0` content, while the new `gunicorn.org` site documents current `25.x` features. Prefer `gunicorn.org` for active work and treat the old stable site as legacy reference material.

## Install

Python compatibility needs a quick check before rollout because current upstream sources do not agree exactly:

- the official install page currently says Python `3.12+`
- PyPI classifiers for `25.1.0` list Python `3.10` through `3.13`

For agent work, do not hard-code an older minimum without checking the target deployment environment.

```bash
pip install gunicorn==25.1.0
```

If you need optional worker implementations or utilities, install the matching extras instead of assuming they are bundled:

```bash
pip install "gunicorn[gevent]==25.1.0"
pip install "gunicorn[eventlet]==25.1.0"
pip install "gunicorn[gthread]==25.1.0"
pip install "gunicorn[tornado]==25.1.0"
pip install "gunicorn[setproctitle]==25.1.0"
```

Current install docs mark `eventlet` as deprecated and scheduled for removal in `26.0`.

## Quick Start

Minimal WSGI app:

```python
def app(environ, start_response):
    body = b"ok"
    start_response("200 OK", [
        ("Content-Type", "text/plain"),
        ("Content-Length", str(len(body))),
    ])
    return [body]
```

Run it:

```bash
gunicorn app:app
```

Common forms:

```bash
# Bind explicitly instead of relying on the default 127.0.0.1:8000
gunicorn --bind 0.0.0.0:8000 app:app

# Django
gunicorn mysite.wsgi

# ASGI on current Gunicorn docs
gunicorn app:app --worker-class asgi
```

The current quickstart shows `gunicorn app:app` for WSGI and `gunicorn app:app --worker-class asgi` for ASGI.

## Recommended Production Setup

Use a reverse proxy in front of Gunicorn for TLS termination, request buffering, static files, and connection handling. The official deployment guide strongly recommends Nginx or a similar proxy in front of Gunicorn.

Typical startup command:

```bash
gunicorn \
  --bind 127.0.0.1:8000 \
  --workers 3 \
  --access-logfile - \
  --error-logfile - \
  mysite.wsgi:application
```

Worker count is workload-dependent. The official quickstart suggests starting around `(2 x num_cores) + 1` and then measuring under real load.

## Configuration

Current official configuration order is, from lower to higher precedence:

- environment variables for supported settings
- framework-specific config (mainly Paste Deploy)
- a Python config file, commonly `gunicorn.conf.py`
- `GUNICORN_CMD_ARGS`
- CLI flags such as `--bind`, `--workers`, `--worker-class`

Example `gunicorn.conf.py`:

```python
bind = "127.0.0.1:8000"
workers = 3
worker_class = "sync"
accesslog = "-"
errorlog = "-"
graceful_timeout = 30
timeout = 30
keepalive = 2
```

Useful startup forms:

```bash
gunicorn -c gunicorn.conf.py mysite.wsgi:application
GUNICORN_CMD_ARGS="--bind=0.0.0.0:8000 --workers=4" gunicorn mysite.wsgi:application
gunicorn --check-config mysite.wsgi:application
gunicorn --print-config mysite.wsgi:application
```

High-signal settings to know:

- `bind`: listen address. Default is `127.0.0.1:8000`, or `0.0.0.0:$PORT` if `PORT` is set.
- `workers`: process count. More workers increase concurrency but also memory use.
- `worker_class`: `sync` by default. Async or threaded classes require the right extra/dependency set.
- `threads`: if you set `threads > 1` with `sync`, Gunicorn switches to `gthread`.
- `timeout`: hard-kill quiet workers after N seconds.
- `graceful_timeout`: how long a worker gets to finish during restart before forced exit.
- `max_requests` and `max_requests_jitter`: recycle workers to mitigate slow memory growth.
- `preload_app`: loads the app before forking. Good for copy-on-write memory savings, but unsafe if your app opens connections or performs per-process setup at import time.
- `reload`: development-only auto-reload. Do not combine it with `preload_app` for production assumptions.
- `worker_connections`: important for async and ASGI workers; use this instead of `threads` when the worker model is connection-oriented.

## Worker Class Guidance

Use the simplest worker model that matches the app:

- `sync`: default for normal WSGI apps doing short blocking work.
- `gthread`: straightforward way to increase concurrency for blocking I/O without adopting greenlets.
- `gevent` or `eventlet`: cooperative workers that need compatible libraries and monkey-patching discipline.
- `tornado`: specialized integration if the app stack already depends on Tornado.
- `asgi`: first-class worker in the current official docs for FastAPI, Starlette, Quart, and other ASGI apps.

## Reverse Proxy And Trust Settings

Gunicorn usually sits behind Nginx, HAProxy, or a platform load balancer. In that setup, the important trust boundary is forwarded headers.

- `secure_scheme_headers` defines which proxy headers imply HTTPS to Gunicorn.
- `forwarded_allow_ips` controls which proxy IPs are allowed to set those forwarded headers.
- The deployment guide warns that if you expose Gunicorn directly to the internet, clients can spoof `X-Forwarded-*` headers. Only trust forwarded headers from your actual proxy layer.

Typical proxy header for correct scheme handling:

```nginx
proxy_set_header X-Forwarded-Proto $scheme;
```

If your framework needs help reconstructing the original scheme/host, use its proxy middleware rather than guessing. The official deployment docs explicitly call out Werkzeug's `ProxyFix` for Flask-style stacks.

Gunicorn does not provide application auth. Handle authentication and authorization in the app or at the reverse proxy, not in Gunicorn config.

## Logging And Operations

For containers and systemd, write logs to stdout/stderr:

```bash
gunicorn \
  --access-logfile - \
  --error-logfile - \
  mysite.wsgi:application
```

Other practical settings:

- `capture_output = True` forwards application stdout/stderr into Gunicorn error logs.
- `reuse_port = True` can help certain multi-process deployment patterns, but only enable it when you understand the kernel and load-balancer behavior.
- `worker_tmp_dir` should point at a fast local filesystem if heartbeat blocking becomes an issue; the settings docs warn that disk-backed temp dirs can stall workers.
- `raw_env` or `--env KEY=value` can inject runtime environment variables for the app process.

## Common Pitfalls

- Do not rely on the previous old `docs.gunicorn.org/en/stable/` URL for `25.x` behavior; use `gunicorn.org`.
- Binding to `0.0.0.0` is fine inside a container or private host, but you still usually want a reverse proxy or platform ingress in front.
- Too many workers can reduce throughput by causing CPU contention and excessive memory use.
- `sync` workers ignore persistent connections; the settings reference notes that `keepalive` does not affect `sync` workers.
- `preload_app` can break apps that create DB pools, event loops, or other per-process state during import.
- Greenlet-based worker classes often need compatible drivers and monkey-patching before imports. If you are not already committed to that model, start with `sync` or `gthread`.
- If you use PROXY protocol, also restrict `--proxy-allow-from`; do not accept PROXY headers from arbitrary clients unless the service is isolated behind a trusted load balancer.
- Windows is not the target platform. Gunicorn is designed for Unix-like process semantics.

## Version-Sensitive Notes For `25.1.0`

- PyPI lists `25.1.0` as released on `2026-02-13`.
- Current official docs document first-class ASGI support, HTTP/2 settings, and dirty-arbiter features in `25.x`.
- The old `docs.gunicorn.org/en/stable/` pages are stale relative to `25.1.0`; use them only when you need legacy reference context.
- Current upstream Python-version statements are inconsistent across sources, so verify interpreter support against the deployment target before pinning runtime requirements.

## Official Sources

- Docs root: https://gunicorn.org/
- Install: https://gunicorn.org/install/
- Quickstart: https://gunicorn.org/quickstart/
- Configuration overview: https://gunicorn.org/configure/
- Settings reference: https://gunicorn.org/reference/settings/
- Deployment guidance: https://gunicorn.org/deploy/
- ASGI worker guide: https://gunicorn.org/asgi/
- PyPI package page: https://pypi.org/project/gunicorn/
