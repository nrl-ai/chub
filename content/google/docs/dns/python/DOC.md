---
name: dns
description: "Google Cloud DNS Python client for managed zones, record sets, and transactional DNS changes"
metadata:
  languages: "python"
  versions: "0.36.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,dns,gcp,networking,python"
---

# Google Cloud DNS Python Client

## Golden Rule

Use `google-cloud-dns` with Application Default Credentials (ADC), pass `project=` explicitly when your environment can see more than one Google Cloud project, and treat DNS updates as transactional changes rather than in-place record edits.

## Install

Pin the version your project expects:

```bash
python -m pip install "google-cloud-dns==0.36.0"
```

Common alternatives:

```bash
uv add "google-cloud-dns==0.36.0"
poetry add "google-cloud-dns==0.36.0"
```

## Authentication And Project Setup

This library uses Google Cloud credentials, not API keys.

Preferred auth order:

1. Local development: `gcloud auth application-default login`
2. Google Cloud runtime with an attached service account or workload identity
3. `GOOGLE_APPLICATION_CREDENTIALS` pointing at a service-account key only when the first two are not available

Typical setup for local development:

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
```

Cloud DNS prerequisites the docs call out:

- Enable the Cloud DNS API on the target project.
- Enable billing on the project.
- Grant DNS permissions such as `roles/dns.admin` for write operations.

The product docs also note narrower permissions for record and zone operations, including `dns.managedZones.*`, `dns.resourceRecordSets.*`, and `dns.changes.*`.

## Initialize The Client

Basic client:

```python
from google.cloud import dns

client = dns.Client(project="your-project-id")
```

Explicit service-account credentials:

```python
from google.cloud import dns
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)

client = dns.Client(
    project="your-project-id",
    credentials=credentials,
)
```

Custom endpoint override when you need to set `api_endpoint` through `client_options`:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import dns

client = dns.Client(
    project="your-project-id",
    client_options=ClientOptions(api_endpoint="https://dns.googleapis.com"),
)
```

The `Client` reference documents `project`, `credentials`, and `client_options` as the main public configuration knobs. `_http` exists, but the reference marks it as private.

## Core Usage

### List managed zones

```python
from google.cloud import dns

client = dns.Client(project="your-project-id")

for zone in client.list_zones():
    print(zone.name, zone.dns_name)
```

### Create or bind a managed-zone handle

`client.zone(...)` constructs a `ManagedZone` object bound to the client. If you omit `dns_name`, later `create()` calls fail.

```python
from google.cloud import dns

client = dns.Client(project="your-project-id")

zone = client.zone(
    "example-com",
    dns_name="example.com.",
    description="Public zone for example.com",
)
```

Fetch current server state when you need it:

```python
zone.reload()
print(zone.name_servers)
```

### Create a public managed zone

```python
from google.cloud import dns

client = dns.Client(project="your-project-id")

zone = client.zone(
    "example-com",
    dns_name="example.com.",
    description="Public zone for example.com",
)
zone.create()
```

### List record sets in a zone

```python
from google.cloud import dns

client = dns.Client(project="your-project-id")
zone = client.zone("example-com")

for record_set in zone.list_resource_record_sets():
    print(record_set.name, record_set.record_type, record_set.ttl, record_set.rrdatas)
```

### Add a record with a change set

The client models record updates through `Changes`, which bundle additions and deletions and are submitted with `create()`.

```python
from google.cloud import dns

client = dns.Client(project="your-project-id")
zone = client.zone("example-com")

record = zone.resource_record_set(
    "www.example.com.",
    "A",
    300,
    ["203.0.113.10"],
)

change = zone.changes()
change.add_record_set(record)
change.create()
```

### Replace an existing record

Cloud DNS updates are not patch-style mutations on an existing record object. Replace a record by deleting the old record set and adding the new one in the same change.

```python
from google.cloud import dns

client = dns.Client(project="your-project-id")
zone = client.zone("example-com")

current = zone.resource_record_set(
    "www.example.com.",
    "A",
    300,
    ["203.0.113.10"],
)
replacement = zone.resource_record_set(
    "www.example.com.",
    "A",
    60,
    ["203.0.113.25"],
)

change = zone.changes()
change.delete_record_set(current)
change.add_record_set(replacement)
change.create()
```

### Poll for change completion

```python
import time

from google.cloud import dns

client = dns.Client(project="your-project-id")
zone = client.zone("example-com")

change = zone.changes()
change.add_record_set(
    zone.resource_record_set(
        "api.example.com.",
        "A",
        60,
        ["203.0.113.40"],
    )
)
change.create()

while True:
    change.reload()
    if change.status == "done":
        break
    time.sleep(2)
```

## Configuration Notes

- Prefer passing `project=` explicitly in scripts and automation. The client falls back to the default inferred environment, which is easy to misread in shared ADC setups.
- Use fully qualified DNS names with trailing dots where Cloud DNS expects them, such as `example.com.` or `www.example.com.`.
- `client.list_zones()` and `zone.list_resource_record_sets()` return iterators. Use iterator paging rather than manually carrying `page_token` unless you have a specific reason.
- The reference exposes a small synchronous surface around `Client`, `ManagedZone`, `Changes`, and `ResourceRecordSet`.

## Common Pitfalls

- `client.zone("name")` creates a local zone handle. It does not fetch remote state until you call methods such as `reload()`, `create()`, or listing methods.
- `zone.create()` needs `dns_name`. If you only constructed the handle with a zone name, creation fails.
- Do not assume you can update a record by mutating a local `ResourceRecordSet` object. Submit a `Changes` request.
- Cloud DNS automatically manages apex `NS` and `SOA` records for public zones. The product docs say those records cannot be deleted through the API.
- User ADC against the wrong active project is a common cause of confusing permission or not-found errors.
- For private, forwarding, peering, and other advanced zone types, verify the product docs before assuming the high-level Python client exposes a first-class helper for the exact workflow you need. This is an inference from the current reference coverage.

## Version-Sensitive Notes

- PyPI currently lists `0.36.0` as the latest release for `google-cloud-dns`.
- The Cloud Python docs root at `https://cloud.google.com/python/docs/reference/dns/latest` is the correct canonical reference entry point, but the current module and class pages under `latest` still render as `0.35.1`.
- Because of that upstream docs drift, validate method signatures against the live reference pages before relying on stale blog posts or old snippets.
- If you need package-specific release deltas for `0.36.0`, check the source repository history in addition to the reference pages, because the rendered API docs lag the current PyPI release.

## Official Sources

- Python reference root: https://cloud.google.com/python/docs/reference/dns/latest
- `Client` reference: https://docs.cloud.google.com/python/docs/reference/dns/latest/google.cloud.dns.client.Client
- `ManagedZone` reference: https://docs.cloud.google.com/python/docs/reference/dns/latest/google.cloud.dns.zone.ManagedZone
- `Changes` reference: https://docs.cloud.google.com/python/docs/reference/dns/latest/google.cloud.dns.changes.Changes
- Cloud DNS zone operations: https://docs.cloud.google.com/dns/docs/zones
- Cloud DNS record-set operations: https://docs.cloud.google.com/dns/docs/records
- Cloud DNS IAM roles: https://docs.cloud.google.com/iam/docs/roles-permissions/dns
- PyPI package page: https://pypi.org/project/google-cloud-dns/
- Source repository: https://github.com/googleapis/python-dns
