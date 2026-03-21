---
name: package
description: "structlog package guide for Python projects using structured logging with structlog 25.5.0"
metadata:
  languages: "python"
  versions: "25.5.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "structlog,logging,structured-logging,json,observability,python"
---

# structlog Python Package Guide

## Golden Rule

Use `structlog` to build structured event dictionaries and configure it explicitly during application startup. Pick one rendering strategy per environment: colorful `ConsoleRenderer` for local development, or JSON / stdlib integration for production. Do not treat the zero-config defaults as a stable production contract.

## Install

Pin the version your project expects:

```bash
python -m pip install "structlog==25.5.0"
```

Common alternatives:

```bash
uv add "structlog==25.5.0"
poetry add "structlog==25.5.0"
```

Optional development helpers from the official docs:

```bash
python -m pip install rich
python -m pip install better-exceptions
```

If you want colored console output on Windows, also install:

```bash
python -m pip install colorama
```

## Authentication And Environment

`structlog` is a logging library, so there is no authentication setup. The practical environment choices are:

- where logs are written: stdout, stderr, files, or stdlib handlers
- how logs are rendered: console, JSON, or stdlib formatter output
- whether request-scoped context is carried using `contextvars`

## Quick Start

The default configuration is intentionally convenient:

```python
import structlog

log = structlog.get_logger()
log.info("user_login", user_id=42, plan="pro")
```

That is useful for quick experiments, but for real applications you should configure processors, log level filtering, timestamps, exception rendering, and the output path yourself.

## Recommended Explicit Setup

This is a good production baseline when you want JSON logs directly from structlog:

```python
import logging

import structlog

structlog.configure(
    cache_logger_on_first_use=True,
    wrapper_class=structlog.make_filtering_bound_logger(logging.INFO),
    processors=[
        structlog.contextvars.merge_contextvars,
        structlog.processors.add_log_level,
        structlog.processors.format_exc_info,
        structlog.processors.TimeStamper(fmt="iso", utc=True),
        structlog.processors.JSONRenderer(),
    ],
    logger_factory=structlog.WriteLoggerFactory(),
)

log = structlog.get_logger().bind(service="billing-api")
log.info("startup_complete", version="2026.03.12")
```

What this gives you:

- per-request context support via `contextvars`
- cheap level filtering before expensive rendering work
- ISO 8601 UTC timestamps
- exceptions serialized into an `exception` field
- JSON log lines written directly by structlog

If you switch `JSONRenderer` to a serializer that returns `bytes` such as `orjson.dumps`, use `structlog.BytesLoggerFactory()` instead of `WriteLoggerFactory()`.

## Core Usage

### Bind context once and reuse the logger

```python
import structlog

log = structlog.get_logger().bind(service="api", region="us-west-2")

def create_user(user_id: str) -> None:
    user_log = log.bind(user_id=user_id)
    user_log.info("create_user_started")
    user_log.info("create_user_finished")
```

`bind()` returns a new logger with extra context. Keep the returned logger if you want those fields on later log lines.

### Log exceptions

```python
import structlog

log = structlog.get_logger()

try:
    1 / 0
except ZeroDivisionError:
    log.exception("calculation_failed", operation="divide")
```

For machine-readable output, keep `structlog.processors.format_exc_info` or `structlog.processors.dict_tracebacks` in the processor chain. Use `dict_tracebacks` when downstream systems want structured traceback data instead of a flat string.

### Async logging methods

```python
import asyncio
import structlog

log = structlog.get_logger()

async def main() -> None:
    await log.ainfo("job_started", job_id="sync-users")

asyncio.run(main())
```

The `a...` methods offload log processing to a thread pool so your event loop does not block on formatting, but they add overhead. Use them deliberately, not everywhere by default.

## Request Context With `contextvars`

For web apps and async services, follow the official `contextvars` flow:

```python
import structlog
from structlog.contextvars import bind_contextvars, clear_contextvars

log = structlog.get_logger()

def handle_request(request_id: str) -> None:
    clear_contextvars()
    bind_contextvars(request_id=request_id)

    log.info("request_started")
```

Important behavior from the official docs:

- `structlog.contextvars.merge_contextvars` should be the first processor
- clear context at the start of each request or job
- prefer `bind_contextvars()` / `unbind_contextvars()` for global request context
- sync and async contexts are isolated in hybrid apps, so values bound in one may not appear in the other

## Standard Library Integration

If the application already depends on `logging`, the quickest official path is `structlog.stdlib.recreate_defaults()`:

```python
import logging
import sys

import structlog

logging.basicConfig(
    format="%(message)s",
    stream=sys.stdout,
    level=logging.INFO,
)

structlog.stdlib.recreate_defaults()

log = structlog.get_logger(__name__)
log.info("service_ready", port=8080)
```

For more control, build a stdlib-specific processor chain and end it with `structlog.stdlib.render_to_log_kwargs`. When you do that, the stdlib formatter must render the extra fields, otherwise you will only see the event text and it will look like structlog "lost" your context.

Use `structlog.stdlib.get_logger()` when you want stdlib-compatible type hints on the returned logger.

## Console Output For Development

`structlog.dev.ConsoleRenderer` is the default development renderer. In `25.5.0`, the active renderer became easier to inspect and mutate:

```python
import structlog

cr = structlog.dev.ConsoleRenderer.get_active()
cr.exception_formatter = structlog.dev.plain_traceback
```

Use this only for local development ergonomics. If you want pretty exceptions with Rich or `better-exceptions`, do not keep `structlog.processors.format_exc_info` in the processor chain, because `ConsoleRenderer` expects to format `exc_info` itself.

## Common Pitfalls

- Do not rely on the default configuration for long-lived application behavior. The maintainers explicitly reserve the right to improve defaults and `structlog.dev`.
- `bind()` returns a new logger. If you ignore the return value, your new context is not retained.
- If you use `cache_logger_on_first_use=True`, later `structlog.configure()` calls do not affect already-cached loggers.
- Cached loggers are not pickleable, so avoid that option if you pass loggers through `multiprocessing`.
- `structlog.threadlocal` is deprecated. Prefer `structlog.contextvars` for new code.
- In stdlib integration, `render_to_log_kwargs` only prepares `extra`; your `logging` formatter still has to output it.
- Pretty console exception formatting and `format_exc_info` conflict. Use one strategy or the other.
- Async `a...` logging methods prevent event-loop blocking but cost more per log entry.

## Version-Sensitive Notes For `25.5.0`

- `25.5.0` adds the newer `ConsoleRenderer.get_active()` ergonomics and mutable console-renderer configuration for development workflows.
- The `ConsoleRenderer(width=-1)` convention is deprecated in `25.5.0`; use `None` instead.
- `structlog.stdlib.AsyncBoundLogger` remains deprecated. Prefer the `await logger.ainfo(...)` style async methods on normal bound loggers.
- `structlog.threadlocal` has been deprecated since `22.1.0` in favor of `contextvars`, but the module still exists in `25.5.0` as a compatibility fallback.

## Official References

- Stable docs: `https://www.structlog.org/en/stable/`
- Getting started: `https://www.structlog.org/en/stable/getting-started.html`
- Standard library logging: `https://www.structlog.org/en/stable/standard-library.html`
- Context variables: `https://www.structlog.org/en/stable/contextvars.html`
- Console output: `https://www.structlog.org/en/stable/console-output.html`
- Exceptions: `https://www.structlog.org/en/stable/exceptions.html`
- Performance: `https://www.structlog.org/en/stable/performance.html`
- Deprecated thread-local API: `https://www.structlog.org/en/stable/thread-local.html`
- PyPI: `https://pypi.org/project/structlog/`
