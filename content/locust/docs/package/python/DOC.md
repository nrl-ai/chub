---
name: package
description: "Locust package guide for Python load testing with the official Locust 2.43.3 docs"
metadata:
  languages: "python"
  versions: "2.43.3"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "locust,python,load-testing,performance,http,distributed"
---

# Locust Python Package Guide

## Golden Rule

Use `locust` for protocol-level load testing, not browser automation. Write a `locustfile.py`, run Locust against a known target host, and validate response content explicitly when status code alone is not enough.

## Install

Pin the package version used by the project:

```bash
python -m pip install "locust==2.43.3"
```

Common alternatives:

```bash
uv add "locust==2.43.3"
poetry add "locust==2.43.3"
```

Useful PyPI extras when you need them:

```bash
python -m pip install "locust[mqtt]==2.43.3"
python -m pip install "locust[otel]==2.43.3"
```

Locust also documents `uvx locust` for one-off runs without installing it into the current environment.

## Minimal `locustfile.py`

The common starting point is `HttpUser` plus `task` functions:

```python
from locust import HttpUser, between, task

class WebsiteUser(HttpUser):
    wait_time = between(1, 3)
    host = "https://example.com"

    @task
    def index(self) -> None:
        self.client.get("/")

    @task(3)
    def health(self) -> None:
        self.client.get("/health")
```

Run it with the web UI:

```bash
locust -f locustfile.py
```

By default the UI listens on `http://localhost:8089`.

## Headless Runs And CI

For automation or CI, run headless and set user count, spawn rate, and runtime explicitly:

```bash
locust -f locustfile.py \
  --headless \
  --users 20 \
  --spawn-rate 5 \
  --run-time 2m \
  --host https://staging.example.com
```

Useful report outputs:

```bash
locust -f locustfile.py --headless --users 50 --spawn-rate 10 --run-time 5m --html report.html
locust -f locustfile.py --headless --users 50 --spawn-rate 10 --run-time 5m --csv run_metrics
```

If you want a run to start automatically but keep the web UI available, use `--autostart` instead of `--headless`.

## Core Usage Patterns

### Reuse session state and log in once

`on_start()` is the normal place to establish authenticated state for each simulated user:

```python
from locust import HttpUser, task

class ApiUser(HttpUser):
    host = "https://api.example.com"

    def on_start(self) -> None:
        response = self.client.post(
            "/login",
            json={"username": "demo", "password": "secret"},
        )
        response.raise_for_status()

    @task
    def profile(self) -> None:
        self.client.get("/me")
```

Cookies are persisted by the HTTP client. For token-based auth, store the token from `on_start()` and send it in headers.

### Group dynamic URLs in stats

Use `name=` so IDs or query-heavy routes do not explode the statistics table:

```python
@task
def item(self) -> None:
    item_id = 123
    self.client.get(f"/items/{item_id}", name="/items/[id]")
```

### Validate response content with `catch_response`

When a 200 response can still be logically wrong, mark success or failure yourself:

```python
@task
def search(self) -> None:
    with self.client.get("/search?q=locust", catch_response=True) as response:
        if "locust" not in response.text.lower():
            response.failure("expected search term in body")
```

### Organize or filter tasks

Use task weights for traffic mix and `@tag` when you need selective execution:

```python
from locust import HttpUser, between, tag, task

class TaggedUser(HttpUser):
    wait_time = between(1, 2)

    @tag("smoke")
    @task
    def smoke(self) -> None:
        self.client.get("/health")
```

Then run only tagged tasks:

```bash
locust -f locustfile.py --tags smoke
```

## Configuration

Locust reads configuration in this order: home config, local config, explicit `--config`, environment variables, then CLI flags. In practice, CLI flags override everything else.

Common config file names:

- `~/.locust.conf`
- `./locust.conf`
- `./pyproject.toml`

Example `locust.conf`:

```ini
headless = true
users = 25
spawn-rate = 5
run-time = 3m
host = https://staging.example.com
html = reports/locust.html
```

Useful environment variables:

```bash
export LOCUST_HOST="https://staging.example.com"
export LOCUST_USERS="25"
export LOCUST_SPAWN_RATE="5"
export LOCUST_HEADLESS="true"
```

## Web UI Authentication

There are two separate auth concerns:

1. Application-under-test auth: usually handled in `on_start()` with your app's login flow or bearer tokens.
2. Locust web UI auth: protects the Locust dashboard itself.

For simple web UI protection, enable the built-in login screen:

```bash
locust -f locustfile.py --web-login
```

For custom authentication, register the documented `username_password_callback` hook and use `environment.web_ui.auth_args` to configure the UI. Locust's extension docs show how to integrate custom auth providers and `Flask-Login`.

## Distributed Execution And Higher Load

Single-process Locust runs can become CPU-bound before the target system is saturated. The official guidance is:

- use more workers or `--processes` when one machine has multiple cores
- use distributed mode for larger tests
- switch to `FastHttpUser` when the Python HTTP client becomes the bottleneck

Typical distributed startup:

```bash
# controller
locust -f locustfile.py --master

# worker nodes
locust -f locustfile.py --worker --master-host 192.168.1.10
```

For a quick multi-process run on one machine:

```bash
locust -f locustfile.py --processes -1
```

If the locustfile is only present on the master, workers can fetch it by starting with `-f -`.

## Common Pitfalls

- `HttpUser` is not a real browser. It will not execute JavaScript, parse DOM state, or load assets the way Playwright or Selenium would.
- If you do not set `host` on the class and do not pass `--host`, your test will fail before requests can run.
- Very high `wait_time` values, blocking code, and expensive response parsing can make the generator the bottleneck instead of the target service.
- Group dynamic endpoints with `name=` or stats become noisy and hard to interpret.
- Use `catch_response=True` when correctness depends on response body content, not just status code.
- Headless CI runs should always set `--users`, `--spawn-rate`, and `--run-time`; otherwise runs are easy to misread or leave hanging.
- Distributed mode only helps if each worker can reach the same target environment and any shared test data is coordinated safely.

## Version-Sensitive Notes For 2.43.3

- PyPI currently lists `locust 2.43.3` and `Requires-Python >=3.10`.
- The official changelog for the 2.43.x line notes a `requests>=2.32.5` update in `2.43.0`; avoid copying older environment snapshots that pin older `requests` versions around Locust.
- `2.43.3` is a small patch release in the stable line rather than a major behavior shift, so most current stable docs examples still apply directly to this version.
