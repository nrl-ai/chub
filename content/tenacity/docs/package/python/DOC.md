---
name: package
description: "Tenacity retrying library for Python with decorator, Retrying, and AsyncRetrying patterns"
metadata:
  languages: "python"
  versions: "9.1.4"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "tenacity,retry,retries,backoff,async,resilience"
---

# Tenacity Python Package Guide

## Golden Rule

Use `tenacity` with an explicit retry policy. A bare `@retry` retries forever with no wait between attempts, which is usually the wrong production default. In practice, specify `stop=...`, `wait=...`, and usually `reraise=True`.

## Install

Pin the version your project expects:

```bash
python -m pip install "tenacity==9.1.4"
```

Common alternatives:

```bash
uv add "tenacity==9.1.4"
poetry add "tenacity==9.1.4"
```

PyPI also publishes `doc` and `test` extras, but normal application use does not need them.

## Core Usage

### Retry flaky exceptions with bounded attempts and backoff

```python
from tenacity import retry, retry_if_exception_type, stop_after_attempt, wait_random_exponential

class TransientError(Exception):
    pass

@retry(
    retry=retry_if_exception_type(TransientError),
    stop=stop_after_attempt(5),
    wait=wait_random_exponential(multiplier=1, max=30),
    reraise=True,
)
def fetch_data() -> str:
    value = call_remote_service()
    if value is None:
        raise TransientError("empty response")
    return value
```

Use this pattern for network or service-call retries. `reraise=True` makes the final failure raise the original exception instead of `RetryError`.

### Retry on returned results, not only exceptions

```python
from tenacity import retry, retry_if_result, stop_after_attempt, wait_fixed

@retry(
    retry=retry_if_result(lambda value: value is None),
    stop=stop_after_attempt(3),
    wait=wait_fixed(1),
    reraise=True,
)
def read_from_cache_or_peer() -> str | None:
    return maybe_get_value()
```

Use `retry_if_result(...)` when the function succeeds technically but returns an unacceptable value such as `None`, `False`, or an incomplete payload.

### Trigger a retry explicitly with `TryAgain`

```python
from tenacity import TryAgain, retry, stop_after_attempt, wait_fixed

@retry(stop=stop_after_attempt(4), wait=wait_fixed(0.5), reraise=True)
def poll_until_ready() -> dict:
    payload = get_status()
    if payload["state"] != "ready":
        raise TryAgain
    return payload
```

`TryAgain` is useful when you want retry behavior without manufacturing a fake exception type.

### Retry a shared code block with `Retrying`

Use `Retrying(...)` when you do not want to wrap the whole function:

```python
from tenacity import Retrying, stop_after_attempt, wait_fixed

for attempt in Retrying(stop=stop_after_attempt(3), wait=wait_fixed(1), reraise=True):
    with attempt:
        response = call_remote_service()
        validate_response(response)
```

This is useful inside larger functions where only one block should be retried.

### Async retries with `AsyncRetrying`

```python
from tenacity import AsyncRetrying, retry_if_exception_type, stop_after_attempt, wait_fixed

class TemporaryAPIError(Exception):
    pass

async def fetch_json() -> dict:
    async for attempt in AsyncRetrying(
        retry=retry_if_exception_type(TemporaryAPIError),
        stop=stop_after_attempt(5),
        wait=wait_fixed(1),
        reraise=True,
    ):
        with attempt:
            return await call_async_api()

    raise RuntimeError("unreachable")
```

For async block retries, use `async for attempt in AsyncRetrying(...)`. Do not use the synchronous `for` pattern in async code.

## Policy Building Blocks

Tenacity policies are built from a few composable pieces:

- `stop=...`: when to give up, such as `stop_after_attempt(5)`, `stop_after_delay(30)`, or `stop_before_delay(30)`
- `wait=...`: how long to sleep, such as `wait_fixed(1)`, `wait_exponential(...)`, or `wait_random_exponential(...)`
- `retry=...`: what should trigger another attempt, such as `retry_if_exception_type(...)` or `retry_if_result(...)`
- `before=...`, `after=...`, `before_sleep=...`: hooks for logging or instrumentation
- `retry_error_callback=...`: return a fallback value after final failure instead of raising
- `retry_with(...)`: override a decorated function's retry settings per call

Example: combine hard bounds with jitter:

```python
from tenacity import retry, stop_after_attempt, stop_after_delay, wait_fixed, wait_random

@retry(
    stop=stop_after_delay(20) | stop_after_attempt(5),
    wait=wait_fixed(1) + wait_random(0, 2),
    reraise=True,
)
def write_with_backoff() -> None:
    store_record()
```

`wait_random_exponential(...)` is usually the better default for distributed services. Fixed waits are more appropriate for short local polling loops.

## Logging, Fallbacks, And Per-Call Overrides

Add logging before each sleep:

```python
import logging

from tenacity import before_sleep_log, retry, stop_after_attempt, wait_fixed

logger = logging.getLogger(__name__)

@retry(
    stop=stop_after_attempt(3),
    wait=wait_fixed(1),
    before_sleep=before_sleep_log(logger, logging.WARNING),
    reraise=True,
)
def call_with_logs() -> None:
    do_work()
```

Return a fallback instead of raising:

```python
from tenacity import retry, retry_if_result, stop_after_attempt

def return_last_value(retry_state):
    return retry_state.outcome.result()

@retry(
    stop=stop_after_attempt(3),
    retry=retry_if_result(lambda value: value is None),
    retry_error_callback=return_last_value,
)
def eventually_returns_none() -> str | None:
    return None
```

Override policy at the call site:

```python
result = fetch_data.retry_with(stop=stop_after_attempt(2))()
```

This is useful in tests or in short-lived CLI commands where you want fewer retries than production.

## Configuration And Auth

Tenacity is a local utility library. There is no auth flow, client initialization, or environment-variable contract that the package requires at runtime.

Configuration lives in Python code:

- choose retry conditions explicitly
- choose backoff and jitter explicitly
- decide whether final failure should raise or return a fallback
- attach logging or metrics callbacks where you need them

For shared application defaults, wrap Tenacity in your own helper decorator or helper function so the retry policy stays consistent across the codebase.

## Common Pitfalls

- Bare `@retry` retries forever with no waiting. Always bound retries in production code.
- If you omit `reraise=True`, final failure usually raises `RetryError` instead of the original exception.
- Do not retry broad exceptions unless you are sure they are transient. Restrict the retry predicate to the failures you actually want to recover from.
- Prefer exponential backoff plus jitter for remote services; constant one-second polling is a thundering-herd footgun.
- When retrying async blocks, use `AsyncRetrying` and `async for`; older snippets that behave synchronously are stale.
- If you use `retry_if_result(...)` with `Retrying` or `AsyncRetrying` code blocks, set the attempt result explicitly with `attempt.retry_state.set_result(result)` after the block succeeds.
- `retry_with(...)` is good for per-call overrides; do not duplicate multiple decorators just to change attempt counts in tests.
- Some older Tenacity examples on the web show a direct `.statistics` attribute on the wrapped function. The 9.0.0 release notes call out an API breakage around statistics, and current official docs surfaces are inconsistent, so verify the access pattern against the installed version before relying on it.

## Version-Sensitive Notes

- PyPI currently lists `9.1.4` as the latest release, published on February 7, 2026.
- PyPI metadata currently requires Python `>=3.10`.
- The Read the Docs changelog page currently only shows up to the `8.2.x` series even though PyPI and GitHub releases include `9.x`. For recent changes, check PyPI release history and GitHub releases in addition to the docs site.
- The `9.0.0` GitHub release explicitly warns about API breakage on the statistics attribute. Re-check any statistics or instrumentation snippets copied from older articles or older Tenacity docs.

## Official Sources

- Docs root: `https://tenacity.readthedocs.io/en/latest/`
- API reference: `https://tenacity.readthedocs.io/en/latest/api.html`
- Changelog: `https://tenacity.readthedocs.io/en/latest/changelog.html`
- PyPI: `https://pypi.org/project/tenacity/`
- Releases: `https://github.com/jd/tenacity/releases`
