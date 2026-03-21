---
name: package
description: "Loguru Python package guide for structured, file, async, and application-aware logging"
metadata:
  languages: "python"
  versions: "0.7.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "loguru,python,logging,structured-logging,observability"
---

# Loguru Python Package Guide

## Golden Rule

Use Loguru through the single shared `logger` object from `from loguru import logger`, configure handlers once at your application's entry point, remove the default stderr handler before adding your own sinks, and avoid `diagnose=True` in production because exception rendering can expose local variable values.

If you are writing a library, do not call `logger.add()` at import time. Disable your package namespace instead and let the application decide how logging should be configured.

## Install

Pin the version if your project depends on current `0.7.x` behavior:

```bash
python -m pip install "loguru==0.7.3"
```

Common alternatives:

```bash
uv add "loguru==0.7.3"
poetry add "loguru==0.7.3"
```

Loguru is pure Python and PyPI classifiers for `0.7.3` include CPython and PyPy across Python 3.x versions.

## Initialize And Configure

Loguru ships with one pre-configured handler that logs to `sys.stderr`. That is convenient for scripts, but most real applications should replace it explicitly.

### Minimal application setup

```python
import sys
from loguru import logger

def configure_logging() -> None:
    logger.remove()
    logger.add(
        sys.stderr,
        level="INFO",
        format="{time:YYYY-MM-DD HH:mm:ss} | {level} | {name}:{function}:{line} | {message}",
        backtrace=False,
        diagnose=False,
    )

if __name__ == "__main__":
    configure_logging()
    logger.info("Application started")
```

Notes:

- Call `logger.remove()` first or you will usually get duplicate console logs.
- Configure handlers once, ideally inside the entry point guarded by `if __name__ == "__main__":`.
- Reconfiguring in imported modules or worker startup paths commonly causes duplicate output and file conflicts.

### Add a rotating file sink

```python
from loguru import logger

logger.add(
    "app.log",
    level="DEBUG",
    rotation="100 MB",
    retention="14 days",
    compression="gz",
    enqueue=True,
    backtrace=False,
    diagnose=False,
)
```

Use `enqueue=True` when multiple threads or processes can write concurrently. Call `logger.complete()` before child processes exit so queued messages flush cleanly.

### Configure from structured data

`logger.configure()` is the Loguru replacement for `logging.basicConfig()` and `logging.dictConfig()`:

```python
import sys
from loguru import logger

logger.configure(
    handlers=[
        {
            "sink": sys.stderr,
            "level": "INFO",
            "format": "{time} | {level} | {message}",
        }
    ],
    extra={"service": "billing-api"},
)
```

## Core Usage

### Basic logging

Loguru uses `{}` formatting, not `%s` formatting:

```python
from loguru import logger

user_id = 42
logger.info("User {} signed in", user_id)
logger.warning("Quota for {user} is almost exhausted", user="alice")
```

Do not pre-format with f-strings when values may contain braces. Let Loguru do the formatting.

### Exceptions

```python
from loguru import logger

try:
    1 / 0
except ZeroDivisionError:
    logger.exception("Computation failed")
```

For automatic exception logging around a function or block:

```python
from loguru import logger

@logger.catch
def run_job() -> None:
    raise RuntimeError("boom")
```

`logger.catch()` is useful, but keep `diagnose=False` on production sinks unless you explicitly want local variable values in traces.

### Structured context with `bind()` and `contextualize()`

```python
from loguru import logger

logger.remove()
logger.add(
    lambda msg: print(msg, end=""),
    format="{time} | {extra[request_id]} | {message}",
)

request_logger = logger.bind(request_id="req-123")
request_logger.info("Request accepted")

with logger.contextualize(request_id="req-456"):
    logger.info("Context-local request")
```

Use:

- `bind()` for a derived logger carrying stable per-object or per-request metadata
- `contextualize()` for temporary context scoped to a block
- `patch()` when you need to inject dynamic values into every record before sinks receive it

Example `patch()` usage:

```python
from datetime import datetime, timezone
from loguru import logger

patched_logger = logger.patch(
    lambda record: record["extra"].update(utc=datetime.now(timezone.utc).isoformat())
)
```

### JSON logs

```python
import sys
from loguru import logger

logger.remove()
logger.add(sys.stdout, serialize=True, backtrace=False, diagnose=False)
logger.bind(request_id="req-123").info("Created order")
```

`serialize=True` is the simplest path when you want machine-readable logs for ingestion systems.

### Lazy debug values

If computing log values is expensive, use `opt(lazy=True)`:

```python
from loguru import logger

def expensive_value() -> str:
    return "computed"

logger.opt(lazy=True).debug("Debug payload: {}", expensive_value)
```

The callable is only evaluated if a configured sink would actually emit the message.

## Configuration And Environment

There is no authentication model for Loguru. Configuration is local to your process and controlled through handlers, formats, levels, and optional environment variables.

Useful environment variables:

- `LOGURU_AUTOINIT=False` disables the pre-configured default stderr sink
- `LOGURU_FORMAT=...` changes the default sink format
- `LOGURU_DIAGNOSE=NO` disables verbose exception variable capture for the default sink
- `LOGURU_LEVEL=INFO` changes the default sink level

Environment variables are most useful for scripts and temporary overrides. For applications, explicit code configuration is easier to audit.

## Library Vs Application Usage

For an application:

- configure handlers in the entry file
- import `logger` everywhere else
- avoid repeated `add()` calls across modules

For a library:

```python
from loguru import logger

logger.disable("my_library")
```

Keep this in your package initialization path and do not add handlers there. Applications that want your library logs can opt in with `logger.enable("my_library")`.

## Common Pitfalls

### Duplicate logs

The usual causes are:

- adding handlers without first calling `logger.remove()`
- configuring logging in more than one module
- configuring logging in code that is re-executed by `multiprocessing`, Gunicorn, or Uvicorn workers

Guard application setup with `if __name__ == "__main__":` and configure once.

### Curly braces and f-strings

Loguru treats log messages like `str.format()`. These can fail:

```python
data = {"foo": 42}
logger.info(f"Processing {data}", data=data)
```

Prefer:

```python
logger.info("Processing {data}", data=data)
logger.bind(data=data).info("Processing payload")
```

If you need literal braces, escape them as `{{` and `}}`.

### Missing `extra[...]` keys in formats

This format will fail if `request_id` is absent:

```python
"{time} | {extra[request_id]} | {message}"
```

Either ensure the key is always bound or provide a patcher/default `extra` configuration so every record contains the expected fields.

### Deadlocks and unsafe sink behavior

Do not log from inside your own sink function, a signal handler, or cleanup paths that can re-enter Loguru. The logger is not re-entrant and can raise `RuntimeError` to avoid deadlocks.

### Multiprocessing on Windows

The default stderr sink is not picklable. For spawned processes on Windows:

- remove the default sink first
- add file or queue-backed sinks with `enqueue=True`
- call `logger.complete()` before the child exits

If you use a non-default multiprocessing context, pass it through `logger.add(..., context=context)`.

### Production exception leakage

`diagnose=True` is convenient in development, but it can print local variables and sensitive values. Prefer:

```python
logger.add("app.log", backtrace=False, diagnose=False)
```

## Interop With Standard `logging`

If you are migrating from the standard library:

- replace `logging.getLogger(__name__)` with `from loguru import logger`
- replace `basicConfig()` and `dictConfig()` with `logger.configure()`
- replace `%s`-style formatting with `{}` formatting
- replace `LoggerAdapter` or `extra=` patterns with `bind()`
- replace `isEnabledFor()` guard code with `logger.opt(lazy=True)`

`logger.add()` can also accept a built-in `logging.Handler` sink when you need to bridge into an existing `logging`-based stack.

## Version-Sensitive Notes

- `0.7.3` is the current upstream release in the stable docs and PyPI as of 2026-03-12.
- The `0.7.3` changelog includes fixes for Cython stack-frame incompatibility, a thread-safety issue affecting `logger.remove()`, Python 3.13 `diagnose=True` formatting, and standard-library `logging.Formatter()` compatibility for non-standard levels.
- `0.7.1` added the `context=` argument to `logger.add()` for multiprocessing context control. If you see examples using custom multiprocessing contexts, they require at least `0.7.1`.
- `0.7.0` made `patch()` cumulative instead of overriding prior patchers. Older blog posts may describe pre-`0.7.0` behavior.

## Official Sources

- Stable docs: `https://loguru.readthedocs.io/en/stable/`
- API reference: `https://loguru.readthedocs.io/en/stable/api/logger.html`
- Migration guide: `https://loguru.readthedocs.io/en/stable/resources/migration.html`
- Troubleshooting: `https://loguru.readthedocs.io/en/stable/resources/troubleshooting.html`
- Recipes: `https://loguru.readthedocs.io/en/stable/resources/recipes.html`
- Changelog: `https://loguru.readthedocs.io/en/stable/project/changelog.html`
- PyPI: `https://pypi.org/project/loguru/`
