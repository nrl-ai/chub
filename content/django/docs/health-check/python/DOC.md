---
name: health-check
description: "django-health-check for Django service probes, machine-readable health endpoints, and custom checks in Python"
metadata:
  languages: "python"
  versions: "4.1.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,health-check,monitoring,ops,readiness,liveness"
---

# django-health-check for Django

## What It Is

`django-health-check` adds a health endpoint and a CLI check runner for Django apps. It is aimed at load balancers, uptime monitors, Prometheus scraping, Docker/Kubernetes probes, and custom service checks.

- Package: `django-health-check`
- Import root: `health_check`
- Version covered: `4.1.2`
- Python requirement: `>=3.10`
- Current PyPI classifiers include Django `5.2` and `6.0`
- Docs root: `https://codingjoe.dev/django-health-check/`
- Registry: `https://pypi.org/project/django-health-check/`

As of March 12, 2026, the upstream GitHub releases page shows `4.1.2` as the latest release, published on March 4, 2026.

## Install

Base install:

```bash
python -m pip install "django-health-check==4.1.2"
```

Common extras:

```bash
python -m pip install "django-health-check[psutil,redis]==4.1.2"
python -m pip install "django-health-check[celery,kafka,rabbitmq,rss]==4.1.2"
```

With `uv`:

```bash
uv add "django-health-check[psutil,redis]==4.1.2"
```

PyPI currently lists these extras: `atlassian`, `celery`, `kafka`, `psutil`, `rabbitmq`, `redis`, `rss`.

## Minimal Setup

Add the core app:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "health_check",
]
```

Wire an explicit endpoint with explicit checks:

```python
# urls.py
from django.urls import path
from health_check.views import HealthCheckView
from redis.asyncio import Redis as RedisClient

urlpatterns = [
    path(
        "health/",
        HealthCheckView.as_view(
            checks=[
                "health_check.Cache",
                "health_check.Database",
                "health_check.Mail",
                "health_check.Storage",
                "health_check.DNS",
                "health_check.contrib.psutil.Disk",
                "health_check.contrib.psutil.Memory",
                (
                    "health_check.contrib.redis.Redis",
                    {
                        "client_factory": lambda: RedisClient.from_url(
                            "redis://localhost:6379"
                        )
                    },
                ),
            ],
        ),
        name="health_check",
    ),
]
```

Notes:

- If you omit `checks=...`, the default is all built-in checks except third-party checks.
- Third-party checks usually require both the extra dependency and an explicit entry in the `checks` list.
- Use `client_factory` for Redis in `4.x`; the older `client=` argument is deprecated.

## Response Formats And Status Codes

The `/health/` endpoint returns:

- HTTP `200` when all checks pass
- HTTP `500` when any check fails

Useful formats:

- Plain text: `Accept: text/plain` or `?format=text`
- JSON: `Accept: application/json` or `?format=json`
- OpenMetrics: `?format=openmetrics`
- RSS/Atom: `?format=rss`, `?format=atom`, or RSS/Atom `Accept` headers

Example:

```bash
curl -H "Accept: application/json" http://localhost:8000/health/
curl http://localhost:8000/health/?format=openmetrics
```

Important: RSS and Atom feeds always return HTTP `200`, even when checks fail. Use the normal endpoint, plain text, or JSON if you need failure to show up as a non-200 HTTP status.

## Built-In Checks That Matter First

These are the main classes to reach for in `4.1.2`:

- `health_check.Cache`: cache probe, supports `alias`, `key_prefix`, and `timeout`
- `health_check.Database`: database probe, supports `alias`
- `health_check.Storage`: storage probe, supports `alias`
- `health_check.DNS`: DNS resolution probe
- `health_check.Mail`: SMTP backend probe
- `health_check.contrib.psutil.Disk`: supports `path` and `max_disk_usage_percent`
- `health_check.contrib.psutil.Memory`: supports `min_gibibytes_available` and `max_memory_usage_percent`
- `health_check.contrib.kafka.Kafka`: requires `bootstrap_servers`
- `health_check.contrib.rabbitmq.RabbitMQ`: requires `amqp_url`
- `health_check.contrib.redis.Redis`: prefer `client_factory`

Example with tuned parameters:

```python
from django.urls import path
from health_check.views import HealthCheckView
from redis.asyncio import Redis as RedisClient

urlpatterns = [
    path(
        "health/",
        HealthCheckView.as_view(
            checks=[
                ("health_check.Cache", {"alias": "default", "key_prefix": "probe"}),
                ("health_check.Database", {"alias": "default"}),
                ("health_check.Storage", {"alias": "default"}),
                (
                    "health_check.contrib.psutil.Disk",
                    {"path": "/app", "max_disk_usage_percent": 85.0},
                ),
                (
                    "health_check.contrib.psutil.Memory",
                    {
                        "min_gibibytes_available": 0.5,
                        "max_memory_usage_percent": 90.0,
                    },
                ),
                (
                    "health_check.contrib.redis.Redis",
                    {
                        "client_factory": lambda: RedisClient.from_url(
                            "redis://redis:6379/0"
                        )
                    },
                ),
            ]
        ),
    ),
]
```

## Container And Kubernetes Probes

For container liveness/readiness, upstream recommends a smaller probe endpoint that avoids external dependencies if those services already expose their own health checks.

```python
# urls.py
from django.urls import path
from health_check.views import HealthCheckView

urlpatterns = [
    path(
        "container/health/",
        HealthCheckView.as_view(
            checks=[
                "health_check.contrib.psutil.Disk",
                "health_check.contrib.psutil.Memory",
            ]
        ),
        name="health_check-container",
    ),
]
```

Run it from the CLI:

```bash
python manage.py health_check health_check-container web:8000 --forwarded-host example.com
```

Important:

- The host used by the command must be allowed by `ALLOWED_HOSTS`.
- The command defaults `X-Forwarded-Proto` to `https`; use `--forwarded-proto http` if SSL is not in play.
- For Kubernetes `httpGet` probes, your app server must bind to `0.0.0.0`, not only `127.0.0.1`.

## Custom Checks

Custom checks subclass `health_check.base.HealthCheck` and implement `run`. The method may be sync or async.

```python
import dataclasses

from health_check.base import HealthCheck, ServiceUnavailable

@dataclasses.dataclass
class ExternalAPIHealthCheck(HealthCheck):
    api_base_url: str = dataclasses.field(repr=False)

    async def run(self) -> None:
        # Replace with your actual probe logic.
        if not self.api_base_url.startswith("https://"):
            raise ServiceUnavailable("API base URL is invalid")
```

Guidance:

- Raise `ServiceWarning` for degraded-but-working states.
- Raise `ServiceUnavailable` for hard failures.
- Keep exception messages human-readable but non-sensitive.
- Sensitive dataclass fields should use `repr=False`, because `__repr__` is used in reports.

## Security And Deployment Notes

- Do not expose a public unauthenticated health endpoint unless the returned information is safe to disclose.
- If you need a shared-secret URL, upstream explicitly says not to reuse Django `SECRET_KEY`. Generate a separate token and put it in the path.
- Prefer network restrictions, reverse-proxy controls, or a dedicated internal route for deeper health endpoints.
- If the endpoint is used behind HTTPS or proxy-aware middleware, make sure forwarded-host and forwarded-proto behavior matches your Django deployment settings.

Example tokenized route:

```python
from django.urls import path
from health_check.views import HealthCheckView

urlpatterns = [
    path("health/your-generated-token/", HealthCheckView.as_view()),
]
```

## Version-Sensitive Notes For 4.x

Do not copy pre-`4.x` setup guides directly.

The `4.x` migration changed the integration model:

- Remove old `health_check.*` sub-apps from `INSTALLED_APPS`; keep only `health_check`
- Remove old `HEALTH_CHECK_*` settings
- Replace `include("health_check.urls")` with `HealthCheckView.as_view(...)`
- `DatabaseHealthCheck` is replaced by `Database`
- old storage-specific health checks are replaced by `Storage`
- `CeleryHealthCheck` is replaced by `Ping`
- `MigrationsHealthCheck` was removed; use Django's built-in check framework instead

If you are upgrading from `3.21+`, the upstream migration guide is the reference to follow before rewriting settings or URLs.

## Common Pitfalls

- Forgetting to install the extra dependency for a third-party check such as `psutil`, `redis`, `kafka`, or `rabbitmq`
- Registering a Redis check with `client=` instead of `client_factory`
- Reusing the old `include("health_check.urls")` pattern from blog posts or old repos
- Using the health command against a host not present in `ALLOWED_HOSTS`
- Making container probes depend on external services and then treating dependency outages as app liveness failures
- Returning sensitive data in exception messages from custom checks

## Official Sources

- Docs: `https://codingjoe.dev/django-health-check/`
- Install guide: `https://codingjoe.dev/django-health-check/install/`
- Usage: `https://codingjoe.dev/django-health-check/usage/`
- Checks reference: `https://codingjoe.dev/django-health-check/checks/`
- Container guide: `https://codingjoe.dev/django-health-check/container/`
- Migration guide: `https://codingjoe.dev/django-health-check/migrate-to-v4/`
- PyPI: `https://pypi.org/project/django-health-check/`
- Releases: `https://github.com/codingjoe/django-health-check/releases/tag/4.1.2`
