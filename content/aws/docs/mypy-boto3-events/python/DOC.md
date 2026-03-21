---
name: mypy-boto3-events
description: "Typed stubs for the boto3 EventBridge client, paginators, literals, and TypedDicts in Python"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,eventbridge,boto3,typing,stubs,mypy"
---

# mypy-boto3-events Python Package Guide

## Golden Rule

`mypy-boto3-events` is a typing package for the boto3 EventBridge client. It does not replace `boto3`, it does not send requests by itself, and it does not configure AWS credentials for you.

Use it when you want:

- typed `boto3.client("events")` calls
- typed paginator objects
- generated `TypedDict` request and response shapes
- literal types for constrained EventBridge string values

Keep the runtime and typing story separate:

- Install `boto3` for runtime AWS calls
- Install `mypy-boto3-events` or `boto3-stubs[events]` for type checking and editor support
- Configure auth, region, retries, and endpoints through boto3 and botocore

## Install

Install the direct service stub package when you only need EventBridge typing:

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-events==1.42.3"
```

Upstream also publishes the aggregated extras:

```bash
python -m pip install "boto3-stubs[events]"
python -m pip install "boto3-stubs-lite[events]"
```

Use `boto3-stubs[events]` when you want the standard bundled install flow from the maintainer docs. Use `boto3-stubs-lite[events]` when you want lighter runtime dependencies and are comfortable with more explicit type annotations.

## Initialize And Setup

### Typed client setup

```python
import boto3
from mypy_boto3_events import EventBridgeClient

session = boto3.Session(profile_name="dev", region_name="us-east-1")
events: EventBridgeClient = session.client("events")
```

### If the typing package is dev-only

If production images do not install the stub wheel, keep the type import behind `TYPE_CHECKING` and cast the client:

```python
from typing import TYPE_CHECKING, cast

import boto3

if TYPE_CHECKING:
    from mypy_boto3_events import EventBridgeClient

events = cast("EventBridgeClient", boto3.client("events", region_name="us-east-1"))
```

This keeps runtime imports clean while preserving editor and mypy support.

## Core Usage

### Publish events with typed request entries

The generated `type_defs` module includes request shapes such as `PutEventsRequestEntryTypeDef`.

```python
import json

from mypy_boto3_events import EventBridgeClient
from mypy_boto3_events.type_defs import PutEventsRequestEntryTypeDef

def publish_order_created(events: EventBridgeClient) -> None:
    entries: list[PutEventsRequestEntryTypeDef] = [
        {
            "Source": "com.example.orders",
            "DetailType": "order.created",
            "Detail": json.dumps({"order_id": "123"}),
            "EventBusName": "default",
        }
    ]

    response = events.put_events(Entries=entries)

    if response["FailedEntryCount"]:
        raise RuntimeError(response["Entries"])
```

`put_events` requires `Entries`, and EventBridge expects `Detail` to be a JSON string for application events. Do not pass a raw Python dict.

### Use typed paginators

The maintainer docs expose paginator types such as `ListRuleNamesByTargetPaginator`.

```python
from mypy_boto3_events import EventBridgeClient
from mypy_boto3_events.paginator import ListRuleNamesByTargetPaginator

def list_rule_names_by_target(
    events: EventBridgeClient,
    target_arn: str,
) -> list[str]:
    paginator: ListRuleNamesByTargetPaginator = events.get_paginator(
        "list_rule_names_by_target"
    )
    names: list[str] = []

    for page in paginator.paginate(TargetArn=target_arn):
        names.extend(page.get("RuleNames", []))

    return names
```

### Use literals for constrained values

The package also exports literal unions for enum-like fields:

```python
from mypy_boto3_events.literals import RuleStateType

state: RuleStateType = "ENABLED"
```

This is mainly useful for catching misspelled constants before runtime.

## Auth And Configuration

Authentication comes from boto3. The stubs package does not add a separate auth layer.

Boto3 checks several credential sources, including:

1. explicit parameters passed when creating a client or session
2. environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN`
3. shared config and credentials files under `~/.aws/`
4. assume-role and web-identity configuration
5. container or instance role credentials

Practical setup:

```python
import boto3
from botocore.config import Config
from mypy_boto3_events import EventBridgeClient

config = Config(
    region_name="us-east-1",
    retries={"mode": "standard", "max_attempts": 10},
)

events: EventBridgeClient = boto3.client("events", config=config)
```

Notes:

- Set `region_name` explicitly in local scripts and tests when the environment is ambiguous.
- Keep credential selection on the boto3 session or client, not in your typing layer.
- Use `botocore.config.Config` for retries, timeouts, proxies, and endpoint tuning.

## Common Pitfalls

- `mypy-boto3-events` is typing-only. Without `boto3`, your code will type-check but cannot call AWS.
- The names differ: install `mypy-boto3-events`, import `mypy_boto3_events`, and create the runtime client with service name `"events"`.
- The typed client class is `EventBridgeClient`, not the older `EventsClient` name used in some stale examples.
- `put_events` expects `Detail` as a serialized JSON string for custom event payloads.
- `boto3-stubs-lite[events]` can require more explicit annotations or casts because it is optimized for lighter installs.
- The docs site may be generated from a newer boto3-stubs build than the latest standalone PyPI wheel, so version drift is possible between type docs and the package you can install.

## Version-Sensitive Notes

- The version used here `1.42.3` matches the current PyPI project page for `mypy-boto3-events`.
- PyPI shows `1.42.3` as the latest installable release, published on December 4, 2025.
- As of March 12, 2026, the maintainer docs root is generated from a newer boto3 line and currently brands this module as `1.42.46` documentation.
- The docs root also shows local-generation examples pinned to `boto3==1.42.46`, which is newer than the standalone stub wheel on PyPI.
- Before pinning dependencies, verify the versions of `boto3`, `botocore`, and the installed stub package in the same environment. If the docs site is ahead of the wheel, prefer the installed package metadata over the moving `/latest` docs branding.

## Official Sources

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_events/`
- Maintainer paginator reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_events/paginators/`
- Maintainer type-def reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_events/type_defs/#puteventsrequestentrytypedef`
- PyPI package page: `https://pypi.org/project/mypy-boto3-events/`
- Boto3 EventBridge `put_events` reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/events/client/put_events.html`
- Boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- Boto3 configuration guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/configuration.html`
