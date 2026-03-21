---
name: package
description: "Official Sentry Python SDK for error monitoring, tracing, profiling, structured logs, and framework integrations"
metadata:
  languages: "python"
  versions: "2.54.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "sentry,observability,error-monitoring,tracing,profiling,logging"
---

# sentry-sdk Python Package Guide

## Golden Rule

Initialize `sentry-sdk` as early as possible in process startup, set a real DSN, and turn on tracing, profiling, and logs deliberately instead of copying `1.0` sample rates from quickstarts into production. Sentry auto-enables many integrations when their libraries are installed, so be explicit if you need to disable or override defaults.

## Install

Pin the stable 2.x release your project expects:

```bash
python -m pip install "sentry-sdk==2.54.0"
```

Common alternatives:

```bash
uv add "sentry-sdk==2.54.0"
poetry add "sentry-sdk==2.54.0"
```

Install an extra when you want Sentry's optional integration dependencies packaged with the SDK:

```bash
python -m pip install "sentry-sdk[django]==2.54.0"
python -m pip install "sentry-sdk[fastapi]==2.54.0"
python -m pip install "sentry-sdk[flask]==2.54.0"
python -m pip install "sentry-sdk[sqlalchemy]==2.54.0"
python -m pip install "sentry-sdk[openai]==2.54.0"
```

The PyPI release exposes many extras, including `django`, `fastapi`, `flask`, `sqlalchemy`, `httpx`, `aiohttp`, `celery`, `openai`, `anthropic`, `langchain`, `langgraph`, `mcp`, and `loguru`.

## Initialize And Configure

Sentry reads `SENTRY_DSN`, `SENTRY_ENVIRONMENT`, and `SENTRY_RELEASE` automatically, but explicit arguments still win. A practical setup looks like this:

```python
import os
import sentry_sdk

sentry_sdk.init(
    dsn=os.environ["SENTRY_DSN"],
    release=os.getenv("SENTRY_RELEASE"),
    environment=os.getenv("SENTRY_ENVIRONMENT", "development"),
    send_default_pii=False,
    traces_sample_rate=0.1,
    profiles_sample_rate=0.1,
    enable_logs=False,
)
```

Important options to know:

- `dsn`: without it, the SDK initializes but does not send data.
- `release`: set this explicitly so regressions line up with deploys.
- `environment`: defaults to `production`; set it yourself if you use `development`, `staging`, and `production`.
- `send_default_pii`: off by default; only enable it if you actually want integrations to send user/request PII.
- `ignore_errors`: fastest way to suppress known exception classes.
- `before_send`, `before_send_transaction`, `before_send_log`: last-chance hooks to redact or drop outgoing events, transactions, or logs.
- `shutdown_timeout`: important for CLI jobs and other short-lived processes because events are drained from a background queue.

If you need async auto-instrumentation, initialize inside the first async entrypoint instead of at module import time:

```python
import asyncio
import sentry_sdk

async def main() -> None:
    sentry_sdk.init(
        dsn="https://examplePublicKey@o0.ingest.sentry.io/0",
        traces_sample_rate=0.1,
    )

asyncio.run(main())
```

## Core Usage

### Capture exceptions and messages

```python
import sentry_sdk

sentry_sdk.init(dsn="https://examplePublicKey@o0.ingest.sentry.io/0")

try:
    1 / 0
except ZeroDivisionError as exc:
    sentry_sdk.capture_exception(exc)

sentry_sdk.capture_message("cache warmed")
```

### Add custom spans around important work

```python
import sentry_sdk

with sentry_sdk.start_span(op="task", name="sync customer record"):
    sync_customer()
```

Use custom spans for expensive internal work that auto-instrumentation will not see clearly.

### Send structured logs to Sentry

Structured logs require `enable_logs=True`:

```python
import sentry_sdk
from sentry_sdk import logger as sentry_logger

sentry_sdk.init(
    dsn="https://examplePublicKey@o0.ingest.sentry.io/0",
    enable_logs=True,
)

sentry_logger.info(
    "Processed order {order_id}",
    order_id="ord_123",
)

sentry_logger.error("Payment failed")
```

### Filter noisy events before they leave the process

```python
import sentry_sdk

def before_send(event, hint):
    if hint.get("exc_info", [None])[0] == ZeroDivisionError:
        return None
    event.setdefault("extra", {})["service"] = "billing"
    return event

sentry_sdk.init(
    dsn="https://examplePublicKey@o0.ingest.sentry.io/0",
    before_send=before_send,
)
```

For transaction filtering, use `traces_sampler` or `before_send_transaction`. For log filtering, use `before_send_log`.

## Sampling, Tracing, And Profiling

Error event sampling:

- `sample_rate` applies one uniform rate to errors.
- `error_sampler` lets you decide per event and overrides `sample_rate`.

Tracing:

- Set `traces_sample_rate` for a uniform transaction sample rate.
- Use `traces_sampler` when you need route-aware or tenant-aware sampling.
- `traces_sampler` and `traces_sample_rate` are mutually exclusive.
- `trace_propagation_targets` controls which outgoing HTTP requests receive `sentry-trace` and `baggage` headers.
- `traces_sample_rate=0` still allows continuation of inbound distributed traces; use `None` if you want tracing fully off.

Profiling:

- Profiling depends on tracing being enabled.
- Current Python docs expose both `profiles_sample_rate` / `profiles_sampler` in options and `profile_session_sample_rate` with `profile_lifecycle="trace"` in the quick start.
- Stay consistent with one style inside a codebase and prefer the current official docs over blog posts when choosing profiling knobs.

Example with dynamic tracing and trace-linked profiling:

```python
import sentry_sdk
from sentry_sdk.types import SamplingContext

def traces_sampler(ctx: SamplingContext) -> float:
    parent = ctx.get("parent_sampled")
    if parent is not None:
        return float(parent)

    transaction_context = ctx.get("transaction_context") or {}
    name = transaction_context.get("name", "")
    if name.endswith("/healthcheck"):
        return 0.0
    return 0.1

sentry_sdk.init(
    dsn="https://examplePublicKey@o0.ingest.sentry.io/0",
    traces_sampler=traces_sampler,
    profiles_sample_rate=0.5,
)
```

## Integrations

Sentry has broad Python integration coverage and many integrations are auto-enabled when the matching library is installed. The official integrations index marks common frameworks like Django, Flask, FastAPI, AIOHTTP, Quart, Sanic, Starlette, SQLAlchemy, Redis, MongoDB, and asyncpg as auto-enabled.

Useful control switches:

- `integrations=[...]`: add or override integration config explicitly.
- `disabled_integrations=[...]`: turn off specific auto-enabled integrations.
- `auto_enabling_integrations=False`: stop library-detected integrations from activating automatically.
- `default_integrations=False`: disables both default integrations and auto-enabling integrations unless you add them back manually.

AI-specific note:

- The OpenAI integration is auto-enabled when `openai` is installed.
- For streaming OpenAI token accounting, Sentry docs call out an extra `tiktoken` dependency.

## Common Pitfalls

- Initializing too late means you miss startup exceptions, early breadcrumbs, and some framework instrumentation.
- Copying `traces_sample_rate=1.0` or profiling sample rates from quickstarts into production usually creates unnecessary volume.
- Enabling `send_default_pii=True` without reviewing data scrubbing is risky for user IDs, request headers, IPs, and request bodies.
- Changing scope state inside `before_send` is too late; modify the event payload directly there.
- Setting `default_integrations=False` turns off more than most users expect, including auto-enabled framework hooks.
- Short-lived scripts may exit before events flush; raise `shutdown_timeout` or flush explicitly when reliability matters.
- `enable_logs=True` is required for Sentry structured logs and for automatic capture from supported logging integrations.
- If you do not want any tracing at all, do not rely on `traces_sample_rate=0`; set tracing options to `None`.

## Version-Sensitive Notes For 2.54.0

- On March 12, 2026, the version used here `2.54.0` matched the live latest stable PyPI release.
- PyPI already shows `3.0.0` alpha releases. If your project is pinned to `2.54.0`, avoid mixing prerelease `3.x` examples into this code path.
- Structured logs are only supported in stable form from SDK `2.35.0` onward; older blog posts may still describe them as experimental.
- The default `max_value_length` increased in `2.34.0` from `1024` to `100000`, which changes truncation behavior for large payload fields.
- The `2.54.0` release added top-level `set_attribute` and `remove_attribute` helpers for logs and metrics telemetry.
- The same `2.54.0` release includes OpenAI streaming fixes, so if you rely on Sentry's OpenAI integration, prefer `2.54.0` or newer in the `2.x` line.

## Official Sources

- Docs root: `https://docs.sentry.io/platforms/python/`
- Options: `https://docs.sentry.io/platforms/python/configuration/options/`
- Filtering: `https://docs.sentry.io/platforms/python/configuration/filtering/`
- Logs: `https://docs.sentry.io/platforms/python/logs/`
- Tracing: `https://docs.sentry.io/platforms/python/tracing/`
- Integrations: `https://docs.sentry.io/platforms/python/integrations/`
- OpenAI integration: `https://docs.sentry.io/platforms/python/integrations/openai/`
- PyPI project: `https://pypi.org/project/sentry-sdk/`
- Exact PyPI release: `https://pypi.org/project/sentry-sdk/2.54.0/`
- Release notes: `https://github.com/getsentry/sentry-python/releases/tag/2.54.0`
