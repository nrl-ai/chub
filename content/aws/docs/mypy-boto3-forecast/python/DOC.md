---
name: mypy-boto3-forecast
description: "mypy-boto3-forecast type stubs for boto3 Amazon Forecast clients, paginators, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,mypy,typing,stubs,forecast,python"
---

# mypy-boto3-forecast Python Package Guide

## What It Is

`mypy-boto3-forecast` is a typing-only companion package for `boto3`'s Amazon Forecast client.

Use it when your code already talks to AWS through `boto3` and you want:

- typed `forecast` client methods
- paginator types for Forecast list operations
- generated `TypedDict` request and response shapes
- better `mypy`, `pyright`, and IDE completion

It does not create runtime clients, does not replace `boto3`, and does not change AWS auth or transport behavior.

## Install

If you want the small service-specific package and you are fine adding explicit client annotations:

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-forecast==1.42.3"
```

If you want `session.client("forecast")` and `boto3.client("forecast")` to resolve more automatically in editors and type checkers:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[forecast]==1.42.3"
```

If you need a lighter typing package and can live with explicit annotations:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[forecast]==1.42.3"
```

Practical rule:

- use `mypy-boto3-forecast` for the smallest Forecast-only typing dependency
- use `boto3-stubs[forecast]` when you want better boto3 overloads across normal client creation
- keep the stub package aligned with the `boto3` version family already pinned by the project

## Initialize And Set Up

`mypy-boto3-forecast` does not change how you create sessions. Start with normal `boto3` setup and then type the client explicitly.

```python
from typing import cast

from boto3.session import Session
from botocore.config import Config
from mypy_boto3_forecast.client import ForecastServiceClient

session = Session(profile_name="dev", region_name="us-west-2")

forecast = cast(
    ForecastServiceClient,
    session.client(
        "forecast",
        config=Config(
            retries={"mode": "standard", "max_attempts": 10},
            connect_timeout=5,
            read_timeout=60,
        ),
    ),
)

response = forecast.list_predictors(MaxResults=20)

for predictor in response.get("Predictors", []):
    print(predictor["PredictorArn"])
```

Why `cast(...)` here:

- it works reliably with the standalone `mypy-boto3-forecast` package
- it makes the intended service type obvious
- it avoids relying on broader boto3 overload packages

If the environment installs `boto3-stubs[forecast]`, you can often skip the `cast` and annotate directly because the boto3 session overloads are available.

## Authentication And Configuration

Auth still comes entirely from boto3 and botocore. Use the normal AWS credential chain:

1. explicit credentials passed to `Session(...)` or `client(...)`
2. environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN`
3. shared AWS config and credentials files
4. IAM role or container credentials in AWS-hosted environments

Typical local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

Then create the client normally:

```python
from boto3.session import Session
from typing import cast

from mypy_boto3_forecast.client import ForecastServiceClient

session = Session(profile_name="dev", region_name="us-west-2")
forecast = cast(ForecastServiceClient, session.client("forecast"))
```

For agent-generated code:

- prefer IAM roles in deployed environments
- prefer named profiles locally
- use `botocore.config.Config(...)` for retries, timeouts, proxies, and retry mode
- do not hardcode long-lived AWS credentials

## Core Typed Usage

### Client Type

The main client type lives at:

```python
from mypy_boto3_forecast.client import ForecastServiceClient
```

That type gives you typed methods for the Forecast service operations exposed by boto3's `forecast` client.

### Paginators

The generated package includes paginator classes for Forecast list operations.

```python
from typing import cast

from boto3.session import Session
from mypy_boto3_forecast.client import ForecastServiceClient
from mypy_boto3_forecast.paginator import ListPredictorsPaginator

session = Session(region_name="us-west-2")
forecast = cast(ForecastServiceClient, session.client("forecast"))

paginator = cast(
    ListPredictorsPaginator,
    forecast.get_paginator("list_predictors"),
)

for page in paginator.paginate():
    for predictor in page.get("Predictors", []):
        print(predictor["PredictorName"])
```

Use typed paginators instead of handwritten `NextToken` loops when the API supports them.

### Literals And Type Definitions

The package also exposes:

- `mypy_boto3_forecast.literals`
- `mypy_boto3_forecast.type_defs`

Use them when request payloads or enum-like string values get large enough that plain `dict[str, object]` becomes error-prone.

### `TYPE_CHECKING` Pattern

If you want runtime environments to depend only on `boto3`, keep the stub package as a dev dependency and import it only for type checking:

```python
from typing import TYPE_CHECKING, Any, cast

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_forecast.client import ForecastServiceClient
else:
    ForecastServiceClient = Any

def get_forecast_client(session: Session) -> ForecastServiceClient:
    return cast(ForecastServiceClient, session.client("forecast"))
```

This is a good default when CI runs `mypy` or `pyright` but production images stay minimal.

## Common Pitfalls

- `mypy-boto3-forecast` is not a runtime SDK. You still need `boto3`.
- The package name uses hyphens, but Python imports use underscores: `mypy_boto3_forecast`.
- `forecast` and `forecastquery` are different boto3 services. Query-time APIs use a separate package: `mypy-boto3-forecastquery`.
- If you only install the service-specific stub package, `session.client("forecast")` usually needs an explicit type annotation or `cast`.
- If `boto3`, `botocore`, and the stub package drift apart, generated signatures and `TypedDict` shapes can stop matching the runtime SDK.
- These stubs improve type checking only. Runtime failures from IAM, region support, service quotas, or invalid resources still come from AWS.

## Version-Sensitive Notes

This entry is intentionally pinned to the version used here `1.42.3`.

Upstream visibility is inconsistent:

- the searchable PyPI result for `mypy-boto3-forecast` still surfaced `1.40.17` during this review
- related official package-family pages, including `mypy-boto3`, `mypy-boto3-forecastquery`, and `types-boto3-forecast`, surfaced `1.42.3`
- the maintainer docs index is generated by `mypy-boto3-builder 8.12.0`, which matches the newer 1.42.x package family rather than the stale `1.40.17` search snippet

Treat that as an indexing or publication-visibility mismatch and re-check the canonical package release against the package page if you need exact release provenance.

For code generation, the practical rule remains: match the stub package to the `boto3` family already pinned in the project.

## Amazon Forecast Availability Note

AWS documents that Amazon Forecast is no longer available to new customers. Existing customers can continue to use it.

That matters because your typed code can still be correct while account setup or enablement fails for business reasons rather than SDK reasons.

## Official Sources

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_forecast/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-forecast/`
- Maintainer repository: `https://github.com/youtype/boto3-stubs`
- Boto3 Forecast client reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/forecast.html`
- Amazon Forecast API reference: `https://docs.aws.amazon.com/forecast/latest/dg/API_Operations_Amazon_Forecast_Service.html`
