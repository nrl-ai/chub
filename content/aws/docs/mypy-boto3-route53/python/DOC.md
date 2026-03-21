---
name: mypy-boto3-route53
description: "mypy-boto3-route53 package guide for typed boto3 Route 53 clients, paginators, waiters, literals, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.6"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,route53,dns,mypy,pyright,stubs,python"
---

# mypy-boto3-route53 Python Package Guide

## Golden Rule

Use `boto3` for real Route 53 API calls and `mypy-boto3-route53` only for typing. If you want `Session.client("route53")` to infer automatically, install `boto3-stubs[route53]`; if you install only the standalone or lite package, annotate `Route53Client` explicitly.

## Version-Sensitive Notes

- The version used here `1.42.6` matches the official PyPI release and maintainer docs checked on `2026-03-12`.
- The maintainer project states that `mypy-boto3-route53` uses the same version as the related `boto3` release. Keep the stub package close to your runtime `boto3` line so method signatures and request/response shapes do not drift.
- The docs root is a stable package URL, not a release-pinned docs URL. Pin the package in your environment when exact patch parity matters.
- The published Route 53 docs expose `client`, `paginator`, `waiter`, `literals`, and `type_defs`. I did not find a dedicated Route 53 `service_resource` module in the published `1.42.6` docs, so plan around typed clients.
- The project recommends local generation with `mypy-boto3-builder` when you need stubs that exactly match the installed `boto3` build in your environment.

## Install

Choose one install mode based on how much boto3 typing support you want.

### Best inference: full boto3 stubs

Use this when you want automatic type discovery for `Session.client("route53")`, `client.get_paginator(...)`, and `client.get_waiter(...)`:

```bash
python -m pip install "boto3-stubs[route53]"
```

### Service-specific package only

Use this when you want only the Route 53 typing package and are willing to add explicit annotations:

```bash
python -m pip install "boto3" "mypy-boto3-route53==1.42.6"
```

### Lite aggregate package

Use this when the full stubs package is too heavy for your IDE or environment:

```bash
python -m pip install "boto3-stubs-lite[route53]"
```

The lite package is more RAM-friendly, but the maintainer docs note that it does not provide `session.client/resource` overloads, so explicit annotations become more important.

## Initialization And Setup

`mypy-boto3-route53` does not add its own auth or config layer. All runtime behavior still comes from normal boto3 setup:

- credentials from environment variables, shared AWS config files, IAM Identity Center, assume-role config, container credentials, or runtime IAM roles
- optional profile selection through `AWS_PROFILE` or `boto3.Session(profile_name=...)`
- any retry, timeout, proxy, or endpoint behavior from `botocore.config.Config`

Typical local setup:

```bash
export AWS_PROFILE=dev
```

Then create the real boto3 client and annotate it:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_route53.client import Route53Client

session = boto3.Session(profile_name="dev")
route53: "Route53Client" = session.client("route53")
```

Route 53 is a global service, so examples commonly use `Session().client("route53")` without service-specific region handling. Still keep your broader AWS profile and signing configuration explicit in the session your application uses.

## Core Usage

### Typed Route 53 client

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_route53.client import Route53Client

route53: "Route53Client" = boto3.Session(profile_name="dev").client("route53")

response = route53.list_hosted_zones(MaxItems="10")

for zone in response.get("HostedZones", []):
    print(zone["Name"], zone["Id"])
```

### Typed paginator

Use paginator types instead of hand-rolled marker loops when scanning many hosted zones or record sets:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_route53.client import Route53Client
    from mypy_boto3_route53.paginator import ListResourceRecordSetsPaginator

route53: "Route53Client" = boto3.Session().client("route53")

paginator: "ListResourceRecordSetsPaginator" = route53.get_paginator(
    "list_resource_record_sets"
)

for page in paginator.paginate(HostedZoneId="Z0123456789ABCDEFG"):
    for record in page.get("ResourceRecordSets", []):
        print(record["Name"], record["Type"])
```

### Typed waiter for DNS changes

Route 53 DNS changes are asynchronous. Use the typed waiter after `change_resource_record_sets` when later steps depend on the change reaching `INSYNC`:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_route53.client import Route53Client
    from mypy_boto3_route53.waiter import ResourceRecordSetsChangedWaiter

route53: "Route53Client" = boto3.Session().client("route53")

response = route53.change_resource_record_sets(
    HostedZoneId="Z0123456789ABCDEFG",
    ChangeBatch={
        "Comment": "Update app record",
        "Changes": [
            {
                "Action": "UPSERT",
                "ResourceRecordSet": {
                    "Name": "app.example.com.",
                    "Type": "A",
                    "TTL": 60,
                    "ResourceRecords": [{"Value": "203.0.113.10"}],
                },
            }
        ],
    },
)

waiter: "ResourceRecordSetsChangedWaiter" = route53.get_waiter(
    "resource_record_sets_changed"
)
waiter.wait(Id=response["ChangeInfo"]["Id"])
```

### Typed request dictionaries and literals

Generated request `TypedDict` shapes and literal aliases are useful when helper functions build Route 53 arguments before the client call:

```python
from mypy_boto3_route53.literals import RRTypeType
from mypy_boto3_route53.type_defs import ListResourceRecordSetsRequestTypeDef

record_type: RRTypeType = "A"

params: ListResourceRecordSetsRequestTypeDef = {
    "HostedZoneId": "Z0123456789ABCDEFG",
    "StartRecordName": "app.example.com.",
    "StartRecordType": record_type,
}
```

The generated client docs also expose request shapes such as `ListHostedZonesRequestTypeDef`, `GetChangeRequestTypeDef`, and many Route 53 response `TypedDict` definitions.

## Runtime-Safe Typing Pattern

If your production image does not install stub packages, keep the imports inside `TYPE_CHECKING` so the type checker sees them without turning the stubs into a runtime dependency:

```python
from __future__ import annotations

from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_route53.client import Route53Client

client: "Route53Client" = boto3.Session().client("route53")
```

The maintainer docs also note a `pylint` workaround: if `pylint` complains about names imported only under `TYPE_CHECKING`, assign those names to `object` in the `else` branch.

## Route 53-Specific Notes

- The boto3 service name is `route53`, not `mypy-boto3-route53`.
- Hosted zone IDs and change IDs come from Route 53 responses; do not guess their format.
- Record names are easiest to reason about when you keep them fully qualified, including the trailing dot.
- `change_resource_record_sets` accepts change batches. Prefer `UPSERT` when you want idempotent create-or-replace behavior.
- Wait for `INSYNC` when later automation depends on the change. A successful write call only means Route 53 accepted the change request.
- Route 53 DNS management is separate from services such as Route 53 Domains and Route 53 Resolver. Do not mix service names or docs across those packages.

## Common Pitfalls

- The PyPI package name uses hyphens, but Python imports use underscores: `mypy_boto3_route53`.
- This package is typing-only. Real AWS requests still require `boto3`.
- If you choose `mypy-boto3-route53` or `boto3-stubs-lite[route53]`, do not expect all `Session.client("route53")` calls to infer automatically. Add explicit `Route53Client` annotations.
- Keep the stub package close to the installed `boto3` line. Version drift can hide newly added parameters or leave stale type information in the editor.
- Route 53 authorization, credentials, retries, and profile resolution are boto3 concerns, not stub-package concerns.

## Official Sources

- Docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_route53/`
- Route53 client reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_route53/client/`
- Waiter reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_route53/waiters/`
- Versioning guide: `https://youtype.github.io/boto3_stubs_docs/#versioning`
- PyPI package page: `https://pypi.org/project/mypy-boto3-route53/`
- Boto3 Route 53 service reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/route53.html`
