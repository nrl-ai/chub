---
name: package
description: "pdpyras package guide for Python projects using PagerDuty REST, Events, and Change Events APIs"
metadata:
  languages: "python"
  versions: "5.4.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pagerduty,api,incidents,events,python,requests"
---

# pdpyras Python Package Guide

## Golden Rule

`pdpyras` `5.4.1` is upstream-maintained but officially deprecated. Use it when you are maintaining an existing codebase that already depends on `pdpyras`; for new PagerDuty integrations, PagerDuty directs you to `python-pagerduty` and its migration guide.

Pin the version if you need behavior that matches this guide:

```bash
pip install "pdpyras==5.4.1"
```

## Install

### pip

```bash
pip install pdpyras
```

### Pin the documented version

```bash
pip install "pdpyras==5.4.1"
```

### Poetry

```bash
poetry add "pdpyras==5.4.1"
```

### uv

```bash
uv add "pdpyras==5.4.1"
```

## Authentication And Setup

### REST API v2 session

Use `APISession` for PagerDuty REST API v2 work with a token from the PagerDuty web UI or OAuth flow:

```python
import os
import pdpyras

session = pdpyras.APISession(os.environ["PAGERDUTY_API_TOKEN"])
```

For an OAuth access token, set `auth_type="oauth2"`:

```python
import os
import pdpyras

session = pdpyras.APISession(
    os.environ["PAGERDUTY_OAUTH_TOKEN"],
    auth_type="oauth2",
)
```

### `From` header for account-level API tokens

PagerDuty requires a user identity for some mutating actions when you use an account-level API token. Set `default_from` once on the session instead of repeating headers per request:

```python
import os
import pdpyras

session = pdpyras.APISession(os.environ["PAGERDUTY_API_TOKEN"])
session.default_from = os.environ["PAGERDUTY_FROM_EMAIL"]
```

If you skip this when the endpoint requires it, PagerDuty will reject the request even though the token itself is valid.

### EU service region

`APISession` defaults to the US API host. If your PagerDuty account is in the EU service region, change the base URL before making requests:

```python
import os
import pdpyras

session = pdpyras.APISession(os.environ["PAGERDUTY_API_TOKEN"])
session.url = "https://api.eu.pagerduty.com"
```

### Events API v2

Use `EventsAPISession` for trigger, acknowledge, and resolve events:

```python
import os
import pdpyras

events = pdpyras.EventsAPISession(os.environ["PAGERDUTY_EVENTS_ROUTING_KEY"])
```

### Change Events API

Use `ChangeEventsAPISession` for deployment or release annotations:

```python
import os
import pdpyras

changes = pdpyras.ChangeEventsAPISession(
    os.environ["PAGERDUTY_CHANGE_EVENTS_ROUTING_KEY"]
)
```

## Core Usage

### Read a REST resource

The `r*` helpers return decoded entity payloads instead of raw `requests.Response` objects:

```python
import os
import pdpyras

session = pdpyras.APISession(os.environ["PAGERDUTY_API_TOKEN"])

user = session.rget("users/me")
print(user["name"])
print(user["email"])
```

Use the non-`r` methods such as `get()` if you specifically need the raw HTTP response.

### Find one object by attribute

`find()` is the convenience path when you want a single match from an index endpoint:

```python
import os
import pdpyras

session = pdpyras.APISession(os.environ["PAGERDUTY_API_TOKEN"])

service = session.find("services", "Primary API", attribute="name")
if service is None:
    raise RuntimeError("Service not found")

print(service["id"])
```

### Iterate paginated collections

Use `iter_all()` for endpoints that support classic list pagination:

```python
import os
import pdpyras

session = pdpyras.APISession(os.environ["PAGERDUTY_API_TOKEN"])

for service in session.iter_all("services"):
    print(service["id"], service["name"])
```

If you actually need a materialized list, `list_all()` collects the iterator into memory.

### Create or update an object idempotently

`persist()` is useful when your automation should create a resource if it does not exist yet:

```python
import os
import pdpyras

session = pdpyras.APISession(os.environ["PAGERDUTY_API_TOKEN"])
session.default_from = os.environ["PAGERDUTY_FROM_EMAIL"]

payload = {
    "type": "service",
    "name": "Automation Demo Service",
    "escalation_policy": {
        "id": "P12345",
        "type": "escalation_policy_reference",
    },
}

service = session.persist(
    "services",
    "name",
    payload,
)

print(service["id"])
```

### Update an existing REST object

```python
import os
import pdpyras

session = pdpyras.APISession(os.environ["PAGERDUTY_API_TOKEN"])
session.default_from = os.environ["PAGERDUTY_FROM_EMAIL"]

updated = session.rput(
    "users/PABC123",
    json={
        "user": {
            "type": "user",
            "job_title": "On-call Engineer",
        }
    },
)

print(updated["job_title"])
```

### Trigger, acknowledge, and resolve incidents with the Events API

```python
import os
import pdpyras

events = pdpyras.EventsAPISession(os.environ["PAGERDUTY_EVENTS_ROUTING_KEY"])

dedup_key = "checkout-api-prod"

dedup_key = events.trigger(
    "Checkout API latency is above threshold",
    "checkout-api-prod",
    dedup_key=dedup_key,
    severity="critical",
    custom_details={
        "component": "checkout-api",
        "group": "payments",
        "class": "latency",
    },
)

events.acknowledge(dedup_key)
events.resolve(dedup_key)

print(dedup_key)
```

Keep the same `dedup_key` across trigger, acknowledge, and resolve calls so PagerDuty correlates them into one incident lifecycle.

### Submit a change event

```python
import os
import pdpyras

changes = pdpyras.ChangeEventsAPISession(
    os.environ["PAGERDUTY_CHANGE_EVENTS_ROUTING_KEY"]
)

changes.submit(
    "Deployed checkout-api 2026.03.12",
    source="github-actions",
    custom_details={
        "service": "checkout-api",
        "environment": "prod",
        "release": "2026.03.12",
    },
)
```

`submit()` also accepts a `timestamp` parameter. Upstream added that in the `5.1.0` line, which matters if you need the event to reflect the actual deployment time rather than submission time.

## Configuration And HTTP Behavior

### Timeouts and retries

`pdpyras` includes retry logic on top of `requests`. The module reference exposes defaults such as:

- `TIMEOUT = 60`
- `max_http_attempts = 10`
- unlimited retries on `429` rate-limit responses by default

```python
import os
import pdpyras

session = pdpyras.APISession(os.environ["PAGERDUTY_API_TOKEN"])
session.timeout = 30
session.max_http_attempts = 5
session.retry[500] = 2
```

The same retry-oriented defaults were extended to `EventsAPISession` and `ChangeEventsAPISession` in the `5.1.2` line.

### Query parameters

Pass PagerDuty query parameters as normal Python dict keys:

```python
import os
import pdpyras

session = pdpyras.APISession(os.environ["PAGERDUTY_API_TOKEN"])

incidents = session.list_all(
    "incidents",
    params={
        "statuses": ["triggered", "acknowledged"],
        "limit": 25,
    },
)
```

Upstream notes that list-valued parameters are automatically normalized to the `param[]=` form PagerDuty expects.

### Raw response vs wrapped entity

The convenience methods come in three common shapes:

- `get`, `post`, `put`, `delete`: raw `requests.Response`
- `jget`, `jpost`, `jput`: decoded JSON document
- `rget`, `rpost`, `rput`: unwrapped PagerDuty entity payload

Choose the helper that matches the response shape you actually need.

## Common Pitfalls

- Do not start new greenfield integrations on `pdpyras`. PagerDuty marks it deprecated and recommends `python-pagerduty`.
- `APISession` defaults to `https://api.pagerduty.com`. Set `session.url = "https://api.eu.pagerduty.com"` for EU-region accounts before you make requests.
- Account-level API tokens often need a `From` header. Set `session.default_from` for mutating operations.
- `iter_all()` paginates live collections. Upstream explicitly warns against mutating the same collection while you are iterating it.
- Keep `dedup_key` stable across Events API lifecycle calls or you will create separate incidents instead of acknowledging or resolving the original one.
- Use `r*` helpers only when the endpoint returns a wrapped entity like `{"user": {...}}`. If you need headers, status code, or an unwrapped body, use `get()` or `jget()` instead.
- Retry behavior is helpful for scripts, but the default unlimited retry on `429` can stall short-lived jobs longer than you expect.

## Version-Sensitive Notes For 5.4.1

- PyPI currently lists `5.4.1` as the latest `pdpyras` release.
- The official docs landing page for `5.4.1` now opens with a deprecation notice and links to the `python-pagerduty` migration guide.
- The upstream changelog marks deprecation in the `5.4.0` line, so `5.4.1` should be treated as a maintenance-era release rather than the start of a new feature line.
- The `5.1.x` line added important behavior that many older blog posts will miss: `ChangeEventsAPISession.submit()` gained a `timestamp` parameter in `5.1.0`, and retry defaults were added to the Events and Change Events sessions in `5.1.2`.
- If you are reading older examples, remember that `pdpyras` has three session types with different auth tokens and base URLs. Do not pass a REST API token to `EventsAPISession` or a routing key to `APISession`.

## Official Sources

- Docs root: https://pagerduty.github.io/pdpyras/
- User guide: https://pagerduty.github.io/pdpyras/user_guide.html
- Module reference: https://pagerduty.github.io/pdpyras/module_reference.html
- Changelog: https://pagerduty.github.io/pdpyras/changelog.html
- Migration guide: https://github.com/PagerDuty/pdpyras/blob/master/migration_guide.md
- PyPI: https://pypi.org/project/pdpyras/
