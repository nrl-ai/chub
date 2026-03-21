---
name: package
description: "VCR.py package guide for recording and replaying HTTP interactions in Python tests"
metadata:
  languages: "python"
  versions: "8.1.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "vcrpy,python,testing,http,requests,httpx,aiohttp,cassettes"
---

# VCR.py Python Package Guide

## Golden Rule

Use `vcrpy` to make HTTP-heavy tests deterministic, but import it as `vcr` and scrub secrets before you commit cassette files. As of March 12, 2026, PyPI lists `vcrpy 8.1.1`, while the official Read the Docs site still renders as `8.0.0`, so prefer PyPI metadata for version and Python compatibility and use the maintainer docs for behavior and APIs.

## Install

Pin the package in test dependencies so cassette behavior stays stable across CI and local runs:

```bash
python -m pip install "vcrpy==8.1.1"
```

Common alternatives:

```bash
uv add --dev "vcrpy==8.1.1"
poetry add --group test "vcrpy==8.1.1"
```

`vcrpy` works by patching supported HTTP stacks. The official docs cover `requests`, `urllib3`, `urllib`, `httpx`, `httpcore`, `aiohttp`, `boto3`, and Tornado integrations.

## Minimal Setup

Use a shared `VCR` instance instead of sprinkling one-off defaults across tests:

```python
from pathlib import Path

import requests
import vcr

my_vcr = vcr.VCR(
    cassette_library_dir=str(Path("tests/cassettes")),
    path_transformer=vcr.VCR.ensure_suffix(".yaml"),
    record_mode="once",
    match_on=["method", "scheme", "host", "port", "path", "query"],
    filter_headers=[("authorization", "DUMMY")],
    filter_query_parameters=[("api_key", "DUMMY")],
)

def test_list_users():
    with my_vcr.use_cassette("users/list"):
        response = requests.get(
            "https://api.example.com/users",
            headers={"Authorization": "Bearer super-secret-token"},
            params={"api_key": "super-secret-key"},
            timeout=10,
        )
        response.raise_for_status()
        assert response.json()
```

What this setup buys you:

- cassettes stay under `tests/cassettes/`
- every cassette gets a `.yaml` suffix automatically
- `record_mode="once"` reuses existing cassettes and fails loudly when requests drift
- auth headers and query tokens are scrubbed before writing to disk

## One-Off Usage

For a single test module, the top-level helper is enough:

```python
import requests
import vcr

@vcr.use_cassette("tests/cassettes/github.yaml")
def test_github_status():
    response = requests.get("https://api.github.com/rate_limit", timeout=10)
    response.raise_for_status()
    assert "resources" in response.json()
```

When you omit the cassette path, VCR.py can auto-generate one from the test name, but explicit paths are easier to manage in larger suites.

## Core Configuration

### `record_mode`

Pick this deliberately:

- `once`: record once, replay afterwards, and raise if a new unmatched request appears
- `new_episodes`: append new requests to an existing cassette
- `none`: never hit the network; fail if the cassette does not satisfy the request
- `all`: always hit the real network and rewrite the cassette

For CI, `once` or `none` is usually the safest choice.

### Request matching

VCR.py matches on request attributes, not just URL text. The default matcher list includes method, scheme, host, port, path, and query. If your API legitimately varies on headers or body, extend `match_on` rather than weakening assertions globally.

Example for JSON POSTs that should also match on body:

```python
import vcr

api_vcr = vcr.VCR(
    match_on=["method", "scheme", "host", "port", "path", "query", "body"],
)
```

### Cassette storage and naming

Common options:

- `cassette_library_dir`: base directory for cassette files
- `path_transformer=vcr.VCR.ensure_suffix(".yaml")`: normalize file suffixes
- `func_path_generator`: customize auto-generated cassette names for decorators

If you want every test file to keep its own cassette tree, centralize that naming policy in one helper instead of duplicating it per test.

### Filtering sensitive data

VCR.py does not manage credentials for you. It records whatever your application sends unless you filter it.

Use these options aggressively:

- `filter_headers=[("authorization", "DUMMY")]`
- `filter_query_parameters=[("token", "DUMMY")]`
- `filter_post_data_parameters=[("client_secret", "DUMMY")]`
- `before_record_request` for custom request scrubbing
- `before_record_response` for response-body cleanup

Example custom request filter:

```python
import vcr

def drop_healthcheck(request):
    if request.uri.endswith("/health"):
        return None
    return request

filtered_vcr = vcr.VCR(
    before_record_request=drop_healthcheck,
    filter_headers=[("authorization", "DUMMY")],
)
```

Returning `None` from `before_record_request` skips that interaction entirely.

### Compressed responses

Set `decode_compressed_response=True` when you want readable cassette bodies. Leave it off if your test needs to preserve the exact compressed payload behavior.

## Test Framework Integration

### `unittest`

VCR.py ships built-in helpers for `unittest`:

```python
import requests
from vcr.unittest import VCRTestCase

class TestStatusAPI(VCRTestCase):
    def test_status(self):
        response = requests.get("https://api.example.com/status", timeout=10)
        response.raise_for_status()
        self.assertEqual(response.status_code, 200)
```

Use `VCRMixin` if you need to combine VCR.py with another base test class.

### `pytest`

Plain decorators and context managers work fine under `pytest`. If you want markers or fixtures, the official docs point to `pytest-vcr` and `pytest-recording`, but those are separate packages, not built into `vcrpy`.

## Common Pitfalls

- Import `vcr`, not `vcrpy`. The PyPI package name and Python import name differ.
- `record_mode="once"` is the default. When a request changes and the existing cassette no longer matches, VCR.py raises `CannotOverwriteExistingCassetteException`.
- `ignore_localhost=True` and `ignore_hosts=[...]` do not mock those requests. They let them bypass VCR.py and hit the real network.
- If the same recorded interaction must be replayed multiple times in one test, call `cassette.rewind()` or design the test to consume the interaction once.
- Keep cassette files out of public repos unless you have filtered secrets and sensitive payload fields first.
- Re-record cassettes when upstream APIs intentionally change response bodies or header shapes. Trying to loosen matchers everywhere usually creates brittle tests later.

## Version-Sensitive Notes

- PyPI currently publishes `8.1.1`, released on January 4, 2026.
- The official docs root at `https://vcrpy.readthedocs.io/en/latest/` still renders as `8.0.0`. Treat the docs content as the current maintainer guide, but use PyPI metadata for the package version and Python floor.
- The official 8.0.0 changelog notes two important upgrade constraints that still matter for 8.1.1 users:
  - Python 3.9 support was dropped; current PyPI metadata requires Python `>=3.10`.
  - `urllib3 < 2` is no longer supported.
- The same 8.0.0 release rewrote `httpx` support to patch `httpcore`, which can affect existing test suites when upgrading from older VCR.py releases.
- `drop_unused_requests` was added in 8.x. Use it if you want stricter cleanup of stale cassette interactions after test changes.

## Official Sources

- Documentation root: `https://vcrpy.readthedocs.io/en/latest/`
- Configuration and advanced usage: `https://vcrpy.readthedocs.io/en/latest/advanced.html`
- API reference: `https://vcrpy.readthedocs.io/en/latest/api.html`
- Changelog: `https://vcrpy.readthedocs.io/en/latest/changelog.html`
- PyPI metadata: `https://pypi.org/project/vcrpy/`
