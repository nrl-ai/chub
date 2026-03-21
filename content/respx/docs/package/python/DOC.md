---
name: package
description: "RESPX package guide for Python tests using HTTPX request mocking and route assertions"
metadata:
  languages: "python"
  versions: "0.22.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "respx,httpx,testing,mocking,async,pytest"
---

# RESPX Python Package Guide

## Golden Rule

Use `respx` to mock `httpx` traffic in tests, and keep routes specific. Prefer a configured router or the `respx_mock` pytest fixture over mutating the global default router with loose patterns.

As of March 12, 2026, PyPI still lists `0.22.0` as the current release, which matches the version used here. The 0.22.0 release also adds HTTPX 0.28 support and drops Python 3.7.

## Install

Install RESPX together with the test tools you already use:

```bash
python -m pip install "respx==0.22.0" pytest
```

Common alternatives:

```bash
uv add --dev "respx==0.22.0"
poetry add --group test "respx==0.22.0"
```

RESPX is for mocking `httpx`, not `requests`. For this version, the maintained compatibility target is HTTPX 0.25+, and the 0.22.0 release notes explicitly mention HTTPX 0.28 support.

## Quick Start With `pytest`

RESPX ships a `pytest` fixture named `respx_mock`. Add routes, then call your code that uses `httpx`.

```python
import httpx

def fetch_user(user_id: str) -> dict:
    response = httpx.get(f"https://api.example.com/users/{user_id}")
    response.raise_for_status()
    return response.json()

def test_fetch_user(respx_mock):
    user_route = respx_mock.get("https://api.example.com/users/42").mock(
        return_value=httpx.Response(200, json={"id": "42", "name": "Ada"})
    )

    result = fetch_user("42")

    assert result["name"] == "Ada"
    assert user_route.called
    assert user_route.call_count == 1
```

If you want unused routes to fail the test, configure the router explicitly:

```python
import httpx
import pytest

@pytest.mark.respx(assert_all_called=True, base_url="https://api.example.com")
def test_list_users(respx_mock):
    respx_mock.get("/users").mock(
        return_value=httpx.Response(200, json=[{"id": "42"}])
    )

    response = httpx.get("https://api.example.com/users")

    assert response.status_code == 200
```

## Setup Patterns

### Decorator or context manager

Use `respx.mock(...)` when patching global `httpx` traffic is fine for the whole test body:

```python
import httpx
import respx

@respx.mock(base_url="https://api.example.com", assert_all_called=True)
def test_create_user():
    create_route = respx.post("/users").mock(
        return_value=httpx.Response(201, json={"id": "123"})
    )

    response = httpx.post("https://api.example.com/users", json={"name": "Ada"})

    assert response.status_code == 201
    assert create_route.called
```

Equivalent context-manager style:

```python
import httpx
import respx

with respx.mock(base_url="https://api.example.com", assert_all_called=True) as router:
    router.get("/health").respond(200, json={"ok": True})
    response = httpx.get("https://api.example.com/health")
    assert response.json() == {"ok": True}
```

### Async clients

RESPX works with `httpx.AsyncClient` too:

```python
import httpx
import pytest
import respx

@pytest.mark.asyncio
@respx.mock
async def test_async_lookup():
    respx.get("https://api.example.com/items/1").mock(
        return_value=httpx.Response(200, json={"id": 1})
    )

    async with httpx.AsyncClient() as client:
        response = await client.get("https://api.example.com/items/1")

    assert response.json() == {"id": 1}
```

### Mock without patching all HTTPX usage

If your code already accepts a transport or client, use RESPX transport classes instead of patching globally:

```python
import httpx
import respx

mock_transport = respx.SyncMockTransport(assert_all_called=True)
user_route = mock_transport.get("https://api.example.com/users/42").respond(
    200, json={"id": "42"}
)

with httpx.Client(transport=mock_transport) as client:
    response = client.get("https://api.example.com/users/42")

assert response.status_code == 200
assert user_route.called
```

This pattern is safer when tests run in parallel or when only one injected client should be mocked.

## Matching Requests

RESPX routes can be created with shorthand helpers like `respx.get(...)` or with `route(...)` plus lookups.

```python
import httpx
import respx

@respx.mock
def test_match_path_and_query():
    route = respx.route(
        method="GET",
        host="api.example.com",
        path__regex=r"^/users/\d+$",
        params__contains={"expand": "teams"},
    ).mock(return_value=httpx.Response(200, json={"ok": True}))

    response = httpx.get(
        "https://api.example.com/users/42",
        params={"expand": "teams", "page": "1"},
    )

    assert response.json() == {"ok": True}
    assert route.called
```

Useful matching strategies:

- Use `base_url="https://api.example.com"` plus relative paths like `"/users"`.
- Match on `method`, `host`, `path`, `params`, `headers`, `cookies`, `content`, `json`, or regex-based variants such as `path__regex`.
- Name routes when you want later lookup or assertions: `router.get(..., name="users_list")`.

## Returning Responses And Side Effects

### Static response

```python
import httpx
import respx

@respx.mock
def test_static_response():
    respx.get("https://api.example.com/ping").respond(204)

    response = httpx.get("https://api.example.com/ping")

    assert response.status_code == 204
```

### Dynamic side effect

Use a callback when the response depends on the incoming request:

```python
import httpx
import respx

def echo_name(request: httpx.Request) -> httpx.Response:
    payload = request.read().decode()
    return httpx.Response(200, json={"raw": payload})

@respx.mock
def test_dynamic_response():
    respx.post("https://api.example.com/echo").mock(side_effect=echo_name)

    response = httpx.post("https://api.example.com/echo", content="Ada")

    assert response.json() == {"raw": "Ada"}
```

### Sequential side effects

You can model retries or rate limits with an iterable:

```python
import httpx
import respx

@respx.mock
def test_retry_flow():
    respx.get("https://api.example.com/data").mock(
        side_effect=[
            httpx.Response(429),
            httpx.Response(200, json={"ok": True}),
        ]
    )

    first = httpx.get("https://api.example.com/data")
    second = httpx.get("https://api.example.com/data")

    assert first.status_code == 429
    assert second.json() == {"ok": True}
```

## Configuration And Auth Notes

RESPX itself has no credentials or auth configuration. The main configuration surface is the router:

- `base_url` to avoid repeating the host for every route
- `assert_all_mocked` to fail on unmatched requests
- `assert_all_called` to fail when declared routes were never hit

If your application sends auth headers, tokens, or cookies through `httpx`, match them explicitly when they matter:

```python
import httpx
import respx

@respx.mock(base_url="https://api.example.com")
def test_bearer_token():
    respx.get(
        "/me",
        headers__contains={"authorization": "Bearer test-token"},
    ).mock(return_value=httpx.Response(200, json={"id": "me"}))

    response = httpx.get(
        "https://api.example.com/me",
        headers={"authorization": "Bearer test-token"},
    )

    assert response.json() == {"id": "me"}
```

## Common Pitfalls

- `respx` mocks `httpx`, not `requests`. If the code under test uses `requests`, RESPX will never intercept it.
- The API reference notes that the plain default router `respx.mock` has `assert_all_called` disabled unless you configure it. Set it explicitly when unused routes should fail the test.
- If you disable `assert_all_mocked`, unmatched requests get an auto-generated `200 OK` response. That can hide missing mocks.
- Very generic routes can shadow more specific ones. Add specific matches first and keep regexes tight.
- Direct transports like `SyncMockTransport` or `AsyncMockTransport` do not give you the decorator/context-manager teardown around the whole test body, so assert on the returned route objects explicitly.
- `pass_through()` sends the request to the real network. Only use it when an external call is intentional in the test.
- RESPX call history is router state. Reset or recreate routers between tests instead of sharing mutable global routes.

## Version-Sensitive Notes For `0.22.0`

- `0.22.0` adds support for `httpx 0.28.0`.
- `0.22.0` drops Python 3.7 support. Use Python 3.8+.
- The docs site is mostly versionless. If a project is pinned to older RESPX or older HTTPX, check the GitHub release history before copying examples from the current docs.
