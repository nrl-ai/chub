---
name: package
description: "responses package guide for mocking requests-based HTTP calls in Python tests"
metadata:
  languages: "python"
  versions: "0.26.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "responses,requests,http,mocking,testing,pytest"
---

# responses Python Package Guide

## Golden Rule

Use `responses` only for code that goes through the `requests` stack. It monkey-patches `requests`, so it is the right tool for unit tests around `requests.Session`, retry adapters, redirect handling, and request validation, but it will not mock `httpx`, `aiohttp`, or raw sockets.

## Install

```bash
python -m pip install "responses==0.26.0"
```

Common project setups:

```bash
uv add --dev "responses==0.26.0"
poetry add --group test "responses==0.26.0"
```

If you want a pytest fixture instead of decorators or context managers, upstream points to the separate `pytest-responses` package:

```bash
python -m pip install pytest-responses
```

## Core Usage

### Decorator-based test

`responses.activate` is the fastest way to wrap a single test:

```python
import requests
import responses

@responses.activate
def test_fetch_user():
    responses.get(
        "https://api.example.com/users/1",
        json={"id": 1, "name": "Ada"},
        status=200,
    )

    resp = requests.get("https://api.example.com/users/1")

    assert resp.status_code == 200
    assert resp.json()["name"] == "Ada"
    assert len(responses.calls) == 1
    assert responses.calls[0].request.url == "https://api.example.com/users/1"
```

If a request does not match a registered response, `responses` raises `requests.exceptions.ConnectionError`.

### Context manager

Use `RequestsMock` when you need scoped setup or per-test configuration:

```python
import requests
import responses

def test_list_items():
    with responses.RequestsMock(assert_all_requests_are_fired=True) as rsps:
        rsps.add(
            responses.GET,
            "https://api.example.com/items",
            json=[{"id": 1}, {"id": 2}],
            status=200,
        )

        resp = requests.get("https://api.example.com/items")

    assert resp.json() == [{"id": 1}, {"id": 2}]
```

Outside the decorator or context manager, requests go to the real network.

### Match request bodies and query params

Prefer matchers over deprecated URL/query-string flags:

```python
import requests
import responses
from responses import matchers

@responses.activate
def test_create_widget():
    responses.post(
        "https://api.example.com/widgets",
        json={"ok": True},
        match=[
            matchers.json_params_matcher({"name": "demo"}),
            matchers.header_matcher({"Authorization": "Bearer test-token"}),
        ],
    )

    resp = requests.post(
        "https://api.example.com/widgets",
        json={"name": "demo"},
        headers={"Authorization": "Bearer test-token"},
    )

    assert resp.json() == {"ok": True}

@responses.activate
def test_search():
    responses.get(
        "https://api.example.com/search",
        body="ok",
        match=[matchers.query_param_matcher({"q": "test", "page": "1"})],
    )

    resp = requests.get(
        "https://api.example.com/search",
        params={"q": "test", "page": "1"},
    )

    assert resp.text == "ok"
```

### Dynamic callbacks

Use callbacks when the response depends on request input:

```python
import json
import requests
import responses

@responses.activate
def test_sum_api():
    def callback(request):
        payload = json.loads(request.body)
        total = sum(payload["numbers"])
        return (200, {"Content-Type": "application/json"}, json.dumps({"total": total}))

    responses.add_callback(
        responses.POST,
        "https://calc.example.com/sum",
        callback=callback,
        content_type="application/json",
    )

    resp = requests.post(
        "https://calc.example.com/sum",
        json={"numbers": [1, 2, 3]},
    )

    assert resp.json() == {"total": 6}
```

## Configuration And Test Setup

There is no auth configuration for `responses` itself. The main setup surface is how you configure the mock registry.

Useful options and helpers:

- `responses.RequestsMock(assert_all_requests_are_fired=True)` to fail if a declared mock is never used
- `responses.RequestsMock(assert_all_requests_are_fired=False)` when a test intentionally leaves some mocks unused
- `@responses.activate(registry=responses.registries.OrderedRegistry)` when the same URL should return different results on successive calls
- `responses.add_passthru("https://real-host.example")` to let unmatched requests for that prefix hit the real server
- `responses.calls` to inspect captured requests and responses after execution
- `responses.replace(...)`, `responses.upsert(...)`, and `responses.remove(...)` for mutating the active registry during a test

Ordered retries example:

```python
import requests
import responses
from responses import registries

@responses.activate(registry=registries.OrderedRegistry)
def test_ordered_responses():
    responses.get("https://api.example.com/data", status=500)
    responses.get("https://api.example.com/data", status=500)
    responses.get("https://api.example.com/data", json={"ok": True}, status=200)

    first = requests.get("https://api.example.com/data")
    second = requests.get("https://api.example.com/data")
    third = requests.get("https://api.example.com/data")

    assert [first.status_code, second.status_code, third.status_code] == [500, 500, 200]
    assert third.json() == {"ok": True}
```

If your production code uses a configured `requests.Session` with retry adapters, create that session in the test and let `responses` serve the sequence you need.

## Common Pitfalls

- `responses` only intercepts `requests`. If the code under test switched to `httpx` or `aiohttp`, your mocks will silently stop applying.
- Unmatched requests raise `ConnectionError`. That usually means the method, URL, query params, body matcher, or headers do not match exactly.
- Do not keep query parameters embedded in the URL when using request matchers. Upstream explicitly recommends `matchers.query_param_matcher(...)` or `matchers.query_string_matcher(...)` instead of the deprecated `match_querystring`.
- Header matching is loose by default because `requests` adds its own standard headers. If you set `strict_match=True`, use a prepared request or expect failures from extra headers.
- When multiple responses match the same request, the default registry returns the first match and removes it. Use `OrderedRegistry` when request order is the point of the test.
- A pass-through prefix only affects requests that are otherwise unmatched. A fully matched registered response still wins.
- Upstream says coroutine and multithreading support exists, but access is locked on the `RequestMock` object so only one thread can use it at a time.

## Version-Sensitive Notes For 0.26.0

- PyPI and the upstream docs agree on `responses 0.26.0` as the current release as of March 12, 2026.
- Current documented minimums are Python 3.8+ and `requests >= 2.30.0`.
- Deprecated APIs still called out upstream:
  - `responses.json_params_matcher` -> `responses.matchers.json_params_matcher`
  - `responses.urlencoded_params_matcher` -> `responses.matchers.urlencoded_params_matcher`
  - `stream=` on `Response` or `CallbackResponse` -> pass `stream` in the request instead
  - `match_querystring=` -> `responses.matchers.query_param_matcher` or `query_string_matcher`
  - module-level `responses.assert_all_requests_are_fired`, `responses.passthru_prefixes`, and `responses.target` -> use `responses.mock.*`
- Recorder and replay helpers under `responses._recorder` and `responses._add_from_file(...)` are still documented as beta. Do not build stable internal tooling around them without pinning and verifying behavior first.

## Official Sources

- PyPI project: `https://pypi.org/project/responses/`
- PyPI JSON metadata: `https://pypi.org/pypi/responses/json`
- Maintainer repository: `https://github.com/getsentry/responses`
- Changelog link from PyPI: `https://github.com/getsentry/responses/blob/master/CHANGES`
