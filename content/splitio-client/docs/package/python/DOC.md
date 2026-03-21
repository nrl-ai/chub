---
name: package
description: "Split Python SDK (`splitio-client`) guide for feature flags, treatments, and event tracking"
metadata:
  languages: "python"
  versions: "10.6.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "split,harness,feature-flags,experimentation,python,sdk"
---

# Split Python SDK Package Guide

## Golden Rule

Use `splitio-client` for server-side Split Feature Management and Experimentation in Python, initialize it once per process with a server-side SDK key, wait for readiness before evaluating flags, and always destroy the factory on shutdown so impressions, events, and telemetry can flush cleanly.

The docs URL (`https://help.split.io/hc/en-us/articles/360020359652`) now redirects to Harness's canonical Python SDK page. Use the Harness page for current behavior and config details.

## Install

Pin the package version your project expects:

```bash
python -m pip install "splitio-client==10.6.0"
```

The official docs commonly recommend the `cpphash` extra:

```bash
python -m pip install "splitio-client[cpphash]==10.6.0"
```

Extras from the official package metadata:

```bash
python -m pip install "splitio-client[asyncio]==10.6.0"
python -m pip install "splitio-client[redis]==10.6.0"
```

Use the extra that matches your runtime:

- `cpphash`: preferred default from the official docs
- `asyncio`: required if you use the SDK's asyncio support
- `redis`: required for Redis-backed synchronization in multi-process deployments

## Authentication And Initialization

The SDK uses a Split server-side SDK key, not an end-user credential. Put it in an environment variable and construct the factory once per process:

```bash
export SPLIT_SDK_KEY="your-server-side-sdk-key"
```

```python
import os
from splitio import get_factory

sdk_key = os.environ["SPLIT_SDK_KEY"]

factory = get_factory(
    sdk_key,
    config={
        "impressionsMode": "OPTIMIZED",
        "labelsEnabled": False,
    },
)

ready = factory.block_until_ready(timeout=5)
if not ready:
    raise RuntimeError("Split SDK did not become ready in time")

client = factory.client()
manager = factory.manager()
```

Notes:

- `factory.block_until_ready()` is the safe gate before evaluating treatments.
- Keep a singleton factory instead of rebuilding clients per request.
- Call `factory.destroy()` during process shutdown to flush pending work and stop SDK threads.

## Core Usage

### Evaluate a treatment

Use a stable matching key. The simplest server-side form is a string key:

```python
treatment = client.get_treatment("customer-123", "checkout_redesign")

if treatment == "on":
    enable_new_checkout()
else:
    enable_legacy_checkout()
```

### Evaluate with attributes

Pass attributes when your targeting rules depend on them:

```python
treatment = client.get_treatment(
    "customer-123",
    "checkout_redesign",
    attributes={
        "plan": "pro",
        "account_age_days": 420,
        "region": "us",
    },
)
```

### Read treatment config

If the flag uses dynamic config, fetch both the treatment and the config payload:

```python
treatment, config = client.get_treatment_with_config(
    "customer-123",
    "checkout_redesign",
)

if treatment == "on" and config:
    apply_checkout_config(config)
```

### Track an event

Use `track()` for conversion or business events tied to a key and traffic type:

```python
ok = client.track(
    "customer-123",
    "user",
    "checkout_completed",
    value=149.0,
    properties={
        "currency": "USD",
        "plan": "pro",
    },
)

if not ok:
    raise RuntimeError("Split track() returned False")
```

### Inspect flag metadata

The manager exposes split definitions and metadata without evaluating treatments:

```python
split_view = manager.split("checkout_redesign")
if split_view is None:
    raise LookupError("Unknown split")

print(split_view.name)
print(split_view.killed)
print(split_view.change_number)
```

### Shutdown cleanly

```python
try:
    run_application(client)
finally:
    factory.destroy()
```

## Configuration Notes

The Python SDK accepts a `config` dictionary when you create the factory. Common settings agents tend to need first:

- `impressionsMode`: choose the impression collection mode explicitly if your org has guidance for optimized vs debug-style visibility
- `labelsEnabled`: disable labels when you do not need them, especially if you want less verbose impression data
- `localhost`: use local override files for offline development or tests
- Redis-backed settings: use these only for supported multi-process deployments and install the `redis` extra first

Practical setup guidance from the official docs:

- Use one SDK instance per process.
- Keep the SDK warm in long-lived processes instead of recreating it per request or job.
- In web apps and workers, own initialization and teardown at process lifespan boundaries.
- For async runtimes, install the `asyncio` extra and follow the SDK's async-specific setup instead of mixing sync examples into an async event loop.

## Localhost Mode

For local development or tests, the SDK supports localhost mode with file-based flag definitions. This is useful when you want deterministic flag behavior without a live Split environment.

Use localhost mode when:

- CI should not depend on live flag state
- local development needs fixed treatments
- you want predictable unit or integration tests around flag branches

Treat localhost mode as a local override, not as a substitute for validating real targeting rules in a live environment.

## Multi-Process And Async Notes

The official docs call out deployment-specific behavior that matters in Python:

- Django and other pre-fork or multi-process servers need multi-process-aware setup; do not assume a single in-memory SDK instance is shared across workers.
- Redis consumer mode requires the `redis` extra and a supported Redis-backed architecture.
- Async support was added in the `10.x` line and requires Python `3.7.16+` plus the `asyncio` extra.
- If you are on `uWSGI`, follow the Python SDK's dedicated guidance instead of a generic WSGI startup pattern.

## Common Pitfalls

- Do not use the SDK before readiness. If you skip `block_until_ready()`, early evaluations can return fallback or `control` behavior.
- Do not recreate the factory on every request. The SDK maintains background synchronization and telemetry workers.
- Do not forget `factory.destroy()`. Losing shutdown flushes can drop impressions or events.
- Keep matching keys stable and deterministic. Random or per-request keys make treatments inconsistent.
- Use the correct traffic type in `track()`. The event can fail or become misleading if the traffic type does not match your Split definitions.
- Prefer the canonical Harness docs over older `help.split.io` links; the old Help Center URL is a redirect, not the maintained source.
- For offline testing, use localhost mode rather than monkey-patching SDK internals.

## Version-Sensitive Notes For `10.6.0`

- PyPI and the current docs both align on `splitio-client 10.6.0` as of 2026-03-12.
- `8.0.0` moved `block_until_ready` to the factory, so older examples that call readiness methods elsewhere are stale.
- `10.0.0` introduced asyncio support; async projects should prefer the async-specific installation and initialization guidance from the official docs.
- The maintained documentation is now under Harness Developer Hub, even though the package name and many examples still use the older Split branding.

## Official Sources

- Harness Python SDK docs: `https://developer.harness.io/docs/feature-management-experimentation/sdks-and-infrastructure/client-side-sdks/python-sdk/`
- Redirecting docs URL: `https://help.split.io/hc/en-us/articles/360020359652`
- PyPI package page: `https://pypi.org/project/splitio-client/`
