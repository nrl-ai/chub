---
name: mypy-boto3-cloudwatch
description: "mypy-boto3-cloudwatch type stubs for typed boto3 CloudWatch clients, paginators, waiters, resources, literals, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.56"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,cloudwatch,mypy,pyright,type-stubs,python"
---

# mypy-boto3-cloudwatch Python Package Guide

## What It Is

`mypy-boto3-cloudwatch` is the maintainer-generated type stub package for the CloudWatch part of `boto3`.

Use it when you want:

- a typed `CloudWatchClient` for `Session.client("cloudwatch")`
- typed paginator and waiter objects
- typed CloudWatch resource objects and collections
- generated `Literal` unions for CloudWatch string enums
- generated `TypedDict` request and response shapes under `type_defs`

It does not replace `boto3`, sign AWS requests, or load credentials by itself.

## Install

Keep `boto3` installed in the same environment as the stubs.

### Recommended for most projects

Use the full stub package when you want automatic `Session.client("cloudwatch")` inference:

```bash
python -m pip install boto3 "boto3-stubs[cloudwatch]"
```

### Standalone CloudWatch stubs

Use this when you only need the CloudWatch typing package and are fine with explicit annotations:

```bash
python -m pip install boto3 mypy-boto3-cloudwatch
```

### Lower-memory IDE fallback

The lite package is more memory-friendly, but upstream documents that it does not provide `session.client()` or `session.resource()` overloads:

```bash
python -m pip install boto3 "boto3-stubs-lite[cloudwatch]"
```

### Exact-match local generation

The maintainer docs recommend local generation when exact parity with your pinned boto3 version matters:

```bash
uvx --with "boto3==1.42.56" mypy-boto3-builder
```

Then select `boto3-stubs` and the `CloudWatch` service.

## Authentication And Setup

AWS documents the main credential lookup order as:

1. explicit credentials passed to `boto3.client(...)`
2. explicit credentials passed to `boto3.Session(...)`
3. environment variables
4. assume-role providers
5. shared credentials and config files
6. container and EC2 metadata providers

Typical local setup:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Explicit session setup keeps typing and runtime configuration clear:

```python
from boto3.session import Session
from mypy_boto3_cloudwatch.client import CloudWatchClient

session = Session(profile_name="dev", region_name="us-east-1")
cloudwatch: CloudWatchClient = session.client("cloudwatch")
```

`Session` also accepts explicit credentials, but do not hardcode access keys in source code. Prefer profiles, environment variables, IAM Identity Center, or runtime IAM roles.

## Core Usage

### Typed client annotation

```python
from boto3.session import Session
from mypy_boto3_cloudwatch.client import CloudWatchClient

cloudwatch: CloudWatchClient = Session(region_name="us-east-1").client(
    "cloudwatch"
)

alarms = cloudwatch.describe_alarms(AlarmNames=["high-error-rate"])
print(alarms["MetricAlarms"])
```

AWS documents clients as the low-level interface whose methods map closely to service APIs. Prefer typed clients for new code.

### Typed request shapes for `put_metric_data`

```python
from boto3.session import Session
from mypy_boto3_cloudwatch.client import CloudWatchClient
from mypy_boto3_cloudwatch.literals import StandardUnitType
from mypy_boto3_cloudwatch.type_defs import MetricDatumTypeDef, PutMetricDataInputTypeDef

cloudwatch: CloudWatchClient = Session(region_name="us-east-1").client("cloudwatch")

unit: StandardUnitType = "Count"

datum: MetricDatumTypeDef = {
    "MetricName": "JobsProcessed",
    "Dimensions": [{"Name": "Service", "Value": "billing-worker"}],
    "Unit": unit,
    "Value": 1.0,
}

payload: PutMetricDataInputTypeDef = {
    "Namespace": "MyApp/Workers",
    "MetricData": [datum],
}

cloudwatch.put_metric_data(**payload)
```

`MetricDatumTypeDef` and `PutMetricDataInputTypeDef` are useful when you want type checking for nested CloudWatch request dictionaries instead of passing untyped dict literals around.

### Typed paginator usage

CloudWatch has generated paginator types for operations including `describe_alarm_history`, `describe_alarms`, `describe_anomaly_detectors`, `get_metric_data`, `list_alarm_mute_rules`, `list_dashboards`, and `list_metrics`.

```python
from datetime import UTC, datetime, timedelta

from boto3.session import Session
from mypy_boto3_cloudwatch.client import CloudWatchClient
from mypy_boto3_cloudwatch.paginator import GetMetricDataPaginator
from mypy_boto3_cloudwatch.type_defs import MetricDataQueryTypeDef

cloudwatch: CloudWatchClient = Session(region_name="us-east-1").client("cloudwatch")

query: MetricDataQueryTypeDef = {
    "Id": "cpu",
    "MetricStat": {
        "Metric": {
            "Namespace": "AWS/EC2",
            "MetricName": "CPUUtilization",
            "Dimensions": [{"Name": "InstanceId", "Value": "i-0123456789abcdef0"}],
        },
        "Period": 300,
        "Stat": "Average",
    },
    "ReturnData": True,
}

paginator: GetMetricDataPaginator = cloudwatch.get_paginator("get_metric_data")

for page in paginator.paginate(
    MetricDataQueries=[query],
    StartTime=datetime.now(UTC) - timedelta(hours=1),
    EndTime=datetime.now(UTC),
):
    for result in page["MetricDataResults"]:
        print(result["Id"], result.get("Values", []))
```

### Typed waiter usage

The maintainer docs publish waiter types for `alarm_exists`, `alarm_mute_rule_exists`, and `composite_alarm_exists`.

```python
from boto3.session import Session
from mypy_boto3_cloudwatch.client import CloudWatchClient
from mypy_boto3_cloudwatch.waiter import AlarmExistsWaiter

cloudwatch: CloudWatchClient = Session(region_name="us-east-1").client("cloudwatch")
waiter: AlarmExistsWaiter = cloudwatch.get_waiter("alarm_exists")

waiter.wait(
    AlarmNames=["high-error-rate"],
    WaiterConfig={"Delay": 10, "MaxAttempts": 12},
)
```

### Typed resources

CloudWatch still has typed resource classes and collections, but AWS documents the resource interface as feature-frozen. Use resources when you already depend on them; prefer clients for new features.

```python
from boto3.session import Session
from mypy_boto3_cloudwatch.service_resource import (
    Alarm,
    CloudWatchServiceResource,
    ServiceResourceMetricsCollection,
)

resource: CloudWatchServiceResource = Session(region_name="us-east-1").resource(
    "cloudwatch"
)

alarm: Alarm = resource.Alarm("high-error-rate")
metrics: ServiceResourceMetricsCollection = resource.metrics

print(alarm.name)
print(metrics)
```

## Type-Checking Patterns

### `TYPE_CHECKING` imports for dev-only stubs

If production images do not install stub packages, keep the imports behind `TYPE_CHECKING` so runtime imports stay clean:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_cloudwatch.client import CloudWatchClient
else:
    CloudWatchClient = object

def make_client() -> "CloudWatchClient":
    return Session(region_name="us-east-1").client("cloudwatch")
```

This is also the upstream workaround for `pylint` complaints about typing-only imports.

### Use literals when an API accepts a fixed string set

```python
from mypy_boto3_cloudwatch.literals import StandardUnitType

unit: StandardUnitType = "Percent"
```

Literal aliases help catch invalid enum-like strings before runtime.

## Common Pitfalls

- Installing `mypy-boto3-cloudwatch` without `boto3`. PyPI classifies this package as `Typing :: Stubs Only`.
- Using `boto3-stubs-lite[cloudwatch]` and expecting overload-based inference from `Session.client("cloudwatch")`. Add explicit annotations in lite mode.
- Treating `TypedDict` coverage as runtime validation. Boto3 responses are still normal dictionaries and AWS can still omit optional keys.
- Debugging auth or region issues inside the stub package. Fix the `boto3.Session(...)` setup first.
- Preferring resources by default. AWS explicitly says newer features should be accessed through the client interface.
- Assuming the docs site, PyPI wheel, and AWS `latest` docs are always on the same boto3 patch. They drift.

## Version-Sensitive Notes

- This entry is pinned to version used here `1.42.56`.
- On March 12, 2026, PyPI lists `mypy-boto3-cloudwatch 1.42.56`, released on `2026-02-24`, and describes it as generated with `mypy-boto3-builder 8.12.0`.
- On the same date, the maintainer docs root for this package shows local-generation commands against `boto3==1.42.66`, so the docs site is useful for current type surfaces but not an exact wheel-version pin source.
- On the same date, AWS `docs.aws.amazon.com/boto3/latest/...` pages resolve to mixed boto3 patch numbers such as `1.42.51`, `1.42.52`, and `1.42.54`. Use AWS docs for runtime behavior and service semantics, but use PyPI for exact package pinning.
- If your application pins a different boto3 or botocore patch and you need exact typing parity, upstream recommends local generation instead of assuming the published wheel matches your lockfile exactly.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_cloudwatch/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-cloudwatch/`
- Upstream builder repository: `https://github.com/youtype/mypy_boto3_builder`
- Boto3 CloudWatch reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/cloudwatch.html`
- Boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- Boto3 session reference: `https://docs.aws.amazon.com/boto3/latest/reference/core/session.html`
- Boto3 clients guide: `https://docs.aws.amazon.com/boto3/latest/guide/clients.html`
- Boto3 resources guide: `https://docs.aws.amazon.com/boto3/latest/guide/resources.html`
