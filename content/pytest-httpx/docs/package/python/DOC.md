---
name: package
description: "pytest-httpx package guide for mocking HTTPX requests in Python tests"
metadata:
  languages: "python"
  versions: "0.36.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest-httpx,pytest,httpx,testing,mocking,python"
---

# pytest-httpx Python Package Guide

## Golden Rule

Use `pytest-httpx` when your code under test uses `httpx`. The plugin installs a `httpx_mock` fixture that intercepts HTTPX requests and fails fast when requests or registered responses do not match your expectations.

As of `0.36.0`, the published package targets:

- Python `>=3.10`
- `pytest==9.*`
- `httpx==0.28.*`

## Install

Pin it alongside compatible `pytest` and `httpx` versions:

```bash
python -m pip install "pytest-httpx==0.36.0" "pytest==9.*" "httpx==0.28.*"
```

Common project-manager variants:

```bash
uv add "pytest-httpx==0.36.0" "pytest==9.*" "httpx==0.28.*"
poetry add --group test "pytest-httpx==0.36.0" "pytest==9.*" "httpx==0.28.*"
```

If you test `async` code, also install your async test runner, commonly:

```bash
python -m pip install "pytest-asyncio>=1,<2"
```

## Setup In Tests

There is no auth or environment-variable setup for `pytest-httpx`. After installation, pytest loads the plugin automatically and exposes the `httpx_mock` fixture.

```python
import httpx

def test_sync_client(httpx_mock):
    httpx_mock.add_response(json={"ok": True})

    with httpx.Client() as client:
        response = client.get("https://api.example.com/health")

    assert response.json() == {"ok": True}
```

Async tests work the same way:

```python
import httpx
import pytest

@pytest.mark.asyncio
async def test_async_client(httpx_mock):
    httpx_mock.add_response(status_code=202, json={"queued": True})

    async with httpx.AsyncClient() as client:
        response = await client.post("https://api.example.com/jobs")

    assert response.status_code == 202
    assert response.json() == {"queued": True}
```

Default response behavior if you do not pass arguments to `add_response()`:

- status: `200`
- protocol: `HTTP/1.1`
- body: empty

## Core Workflows

### Register a specific response

```python
def test_post_json(httpx_mock):
    httpx_mock.add_response(
        method="POST",
        url="https://api.example.com/items",
        status_code=201,
        json={"id": "item_123"},
        headers={"x-test": "pytest-httpx"},
    )
```

Matching is done against the full request. If several registrations match, the first unmatched one is used.

### Match query parameters with `match_params`

`0.36.0` adds `match_params`, which is cleaner than encoding every query-string variant into the URL:

```python
from unittest.mock import ANY

def test_partial_query_match(httpx_mock):
    httpx_mock.add_response(
        url="https://api.example.com/search",
        match_params={"q": "books", "page": ANY},
        json={"items": []},
    )
```

Use `match_params` only when the `url` does not already include query parameters.

### Match headers, body, JSON, files, or HTTPX extensions

```python
def test_match_request_shape(httpx_mock):
    httpx_mock.add_response(
        url="https://api.example.com/items",
        method="POST",
        match_headers={"Authorization": "Bearer test-token"},
        match_json={"name": "widget"},
        json={"created": True},
    )
```

Important exclusivity rules:

- `match_json` cannot be combined with `match_content` or `match_files`
- `match_files` cannot be combined with `match_content` or `match_json`

### Return streamed or dynamic responses

```python
import httpx
from pytest_httpx import IteratorStream

def test_streaming(httpx_mock):
    httpx_mock.add_response(stream=IteratorStream([b"part-1", b"part-2"]))

    with httpx.Client() as client:
        with client.stream("GET", "https://api.example.com/stream") as response:
            assert list(response.iter_raw()) == [b"part-1", b"part-2"]
```

Callbacks let the mocked response depend on the incoming request:

```python
import httpx

def test_dynamic_response(httpx_mock):
    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(status_code=200, json={"url": str(request.url)})

    httpx_mock.add_callback(handler)

    with httpx.Client() as client:
        response = client.get("https://api.example.com/test")

    assert response.json() == {"url": "https://api.example.com/test"}
```

### Raise transport errors

Use `add_exception()` when the code path should handle HTTPX failures:

```python
import httpx
import pytest

def test_timeout(httpx_mock):
    httpx_mock.add_exception(httpx.ReadTimeout("read timed out"))

    with httpx.Client() as client:
        with pytest.raises(httpx.ReadTimeout):
            client.get("https://api.example.com/slow")
```

If no registered response matches, `pytest-httpx` raises `httpx.TimeoutException` immediately and includes mismatch details in the error message.

## Inspect Captured Requests

Prefer strict request matching up front, but you can also inspect captured requests:

```python
def test_capture_request(httpx_mock):
    httpx_mock.add_response()

    with httpx.Client() as client:
        client.get("https://api.example.com/items", headers={"x-trace": "123"})

    request = httpx_mock.get_request()
    assert request.headers["x-trace"] == "123"
```

Use:

- `httpx_mock.get_request()` for a single request
- `httpx_mock.get_requests()` when multiple requests are expected

The same filtering rules used for response selection also apply when retrieving captured requests.

## Configuration

`pytest-httpx` does not have an auth model of its own. Configure fixture behavior with the `@pytest.mark.httpx_mock(...)` marker.

### Per-test configuration

```python
import pytest

@pytest.mark.httpx_mock(assert_all_responses_were_requested=False)
def test_optional_response(httpx_mock):
    httpx_mock.add_response()
```

### Main options

- `assert_all_responses_were_requested=False`: allow unused registered responses
- `assert_all_requests_were_expected=False`: allow unmatched outgoing requests at teardown
- `can_send_already_matched_responses=True`: reuse the last matching response after it has already been consumed
- `should_mock=lambda request: ...`: let selected requests pass through for partial integration tests

Prefer response-level controls when possible:

- `is_optional=True` is narrower than disabling `assert_all_responses_were_requested`
- `is_reusable=True` is narrower than enabling `can_send_already_matched_responses`

Example partial pass-through:

```python
import pytest

@pytest.mark.httpx_mock(
    should_mock=lambda request: request.url.host != "localhost"
)
def test_local_server_and_remote_mock(httpx_mock):
    httpx_mock.add_response(url="https://api.example.com/data", json={"ok": True})
```

## Common Pitfalls

- Do not use `pytest-httpx` for code that uses `requests`, `aiohttp`, or another HTTP client. It only intercepts `httpx`.
- The plugin matches the full URL by default, including query parameters.
- Query parameter order does not matter, but repeated-value order does.
- If you use `match_params`, do not also include query parameters in `url`.
- Unused registered responses fail the test at teardown unless you mark them optional or relax fixture settings.
- Unexpected outgoing requests still raise `httpx.TimeoutException` even if `assert_all_requests_were_expected=False`; that option only prevents teardown failure for unmatched requests.
- Suite-wide marker injection through `pytest_collection_modifyitems` is affected by a pytest marker-ordering bug documented upstream; module and class markers may be applied before the suite-level marker.

## Version-Sensitive Notes

- `0.36.0` adds `match_params`, adds Python `3.14` support, requires `pytest==9.*`, and drops Python `3.9` / pytest `8`.
- `0.35.0` moved to `httpx==0.28.*`. Older test code pinned to HTTPX `0.27.*` needs an older `pytest-httpx`.
- `0.34.0` added `is_optional` and `is_reusable`; before that, you had to rely more heavily on global marker options.

## Official Sources

- Docs: `https://colin-b.github.io/pytest_httpx/`
- PyPI project: `https://pypi.org/project/pytest-httpx/`
- PyPI version metadata: `https://pypi.org/pypi/pytest-httpx/0.36.0/json`
- Changelog: `https://github.com/Colin-b/pytest_httpx/blob/master/CHANGELOG.md`
- Repository: `https://github.com/Colin-b/pytest_httpx`
