---
name: package
description: "backoff retry decorators for synchronous and asyncio Python code"
metadata:
  languages: "python"
  versions: "2.2.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "backoff,retry,retries,python,asyncio,http,resilience"
---

# backoff Python Package Guide

## Golden Rule

Use `backoff` to wrap your own I/O calls with retry decorators, not as an HTTP client. In `2.2.1`, the package is stable but dormant: PyPI still lists `2.2.1` as the latest release, and the upstream GitHub repository was archived and made read-only on August 8, 2025.

## Install

Pin the version your project expects:

```bash
python -m pip install "backoff==2.2.1"
```

Common alternatives:

```bash
uv add "backoff==2.2.1"
poetry add "backoff==2.2.1"
```

`backoff` has no auth or service configuration of its own. It only controls retry timing around your function calls.

## Core Model

The library exposes decorators that retry based on either exceptions or return values:

- `backoff.on_exception(...)`: retry when the wrapped function raises one of the specified exceptions
- `backoff.on_predicate(...)`: retry when a predicate matches the return value
- Wait generators such as `backoff.expo`, `backoff.fibo`, `backoff.constant`, and `backoff.runtime`

The decorators work for both normal functions and `async def` coroutines.

## Basic Exception Retry

Use `on_exception` for network and API calls that should retry on transient failures:

```python
import requests
import backoff

def fatal_http_error(exc: requests.exceptions.RequestException) -> bool:
    response = exc.response
    return response is not None and 400 <= response.status_code < 500

@backoff.on_exception(
    backoff.expo,
    requests.exceptions.RequestException,
    max_time=60,
    giveup=fatal_http_error,
)
def fetch_json(url: str) -> dict:
    response = requests.get(url, timeout=10)
    response.raise_for_status()
    return response.json()
```

Use this pattern when you want exponential backoff for timeouts, connection failures, and retryable HTTP errors, but want to stop immediately on permanent `4xx` responses.

## Limit Retries By Attempt Count

Use `max_tries` when you want a bounded number of attempts instead of a time budget:

```python
import requests
import backoff

@backoff.on_exception(
    backoff.expo,
    (requests.exceptions.Timeout, requests.exceptions.ConnectionError),
    max_tries=5,
    jitter=None,
)
def fetch_text(url: str) -> str:
    return requests.get(url, timeout=5).text
```

Set `jitter=None` when you need deterministic delays in tests or fixed-interval behavior for local tooling.

## Poll Until A Return Value Changes

Use `on_predicate` when the call succeeds technically but the result is not ready yet:

```python
import backoff

@backoff.on_predicate(backoff.constant, interval=1, jitter=None)
def poll_job_status(client, job_id: str):
    job = client.get_job(job_id)
    if job["state"] == "done":
        return job
    return None
```

Important default: if you omit `predicate=...`, `on_predicate` retries on any falsey return value. Be explicit when `0`, `False`, `[]`, or `{}` are valid successful results in your application.

## Honor `Retry-After` With `backoff.runtime`

`backoff.runtime` lets the wait duration come from the function result or exception details. This is the most useful `2.x` feature when wrapping rate-limited APIs:

```python
import requests
import backoff

@backoff.on_predicate(
    backoff.runtime,
    predicate=lambda response: response.status_code == 429,
    value=lambda response: int(response.headers.get("Retry-After", "1")),
    jitter=None,
)
def get_with_retry_after(url: str) -> requests.Response:
    return requests.get(url, timeout=10)
```

Use `runtime` when the server tells you exactly how long to wait.

## Async Usage

Decorators also work on coroutines:

```python
import aiohttp
import backoff

@backoff.on_exception(backoff.expo, aiohttp.ClientError, max_time=60)
async def fetch_text(url: str) -> str:
    async with aiohttp.ClientSession(raise_for_status=True) as session:
        async with session.get(url) as response:
            return await response.text()
```

You can also supply async handler functions for `on_success`, `on_backoff`, and `on_giveup`.

## Runtime Configuration

Decorator arguments are usually evaluated when the module is imported. If retry settings come from runtime config, pass callables instead of constants:

```python
import backoff

def current_max_time() -> int:
    return settings.BACKOFF_MAX_TIME

@backoff.on_exception(backoff.expo, ValueError, max_time=current_max_time)
def flaky_operation() -> str:
    ...
```

On `2.2.1`, callable `max_time` and `max_tries` are safe to use; `2.1.1` fixed a bug in that area.

## Logging And Event Handlers

`backoff` logs to the `backoff` logger, which uses a `NullHandler` by default. You will not see retry logs unless you configure a handler or pass a custom logger.

Basic logging setup:

```python
import logging

logging.getLogger("backoff").addHandler(logging.StreamHandler())
logging.getLogger("backoff").setLevel(logging.INFO)
```

For metrics or custom observability, use handlers:

```python
import backoff
import requests

def on_backoff(details: dict) -> None:
    print(
        f"retrying {details['target'].__name__} "
        f"after {details['tries']} tries; waiting {details['wait']:.1f}s"
    )

@backoff.on_exception(
    backoff.expo,
    requests.exceptions.RequestException,
    on_backoff=on_backoff,
)
def fetch(url: str):
    return requests.get(url, timeout=10)
```

Handler detail dictionaries can include `target`, `args`, `kwargs`, `tries`, `elapsed`, `wait`, `value`, and, for exception retries, `exception`.

## Common Pitfalls

- `raise_on_giveup` defaults to `True` for `on_exception`. If retries are exhausted, the last exception is re-raised.
- If you set `raise_on_giveup=False`, the decorated function returns `None` on give-up. Do not assume your original return type still applies.
- `on_predicate` defaults to retrying falsey values. This can accidentally treat empty collections or `0` as failures.
- Since version `1.2`, the default jitter is `backoff.full_jitter`, so actual sleep times vary unless you set `jitter=None`.
- Extra keyword arguments such as `interval` or `max_value` are passed to the wait generator, not to your wrapped function.
- Multiple decorators can be stacked, but the order affects behavior. Keep each decorator tied to one clear retry condition.
- The project is archived. Do not expect fixes for new Python/runtime edge cases unless your team vendors or replaces it.

## Version-Sensitive Notes

- `2.2.1` is the latest PyPI release as of March 12, 2026.
- `2.2.1` only changed type hints for wait generators; it did not change retry semantics.
- `2.0.0` added `raise_on_giveup`, `backoff.runtime`, Python 3.10 support, and dropped Python 3.6 support.
- Upstream metadata for `2.2.1` declares Python `>=3.7, <4.0` and classifiers through Python 3.10. That does not guarantee testing on newer interpreters.

## Official Sources

- GitHub repository and README: `https://github.com/litl/backoff`
- PyPI project page: `https://pypi.org/project/backoff/`
- GitHub releases: `https://github.com/litl/backoff/releases`
