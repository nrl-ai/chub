---
name: mypy-boto3-apigateway
description: "mypy-boto3-apigateway typing guide for boto3 API Gateway clients, paginators, literals, and TypedDict request/response shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,apigateway,typing,mypy,pyright"
---

# mypy-boto3-apigateway Python Package Guide

## What This Package Is For

`mypy-boto3-apigateway` is a typing-only package for the `boto3` API Gateway client.

Use it when you want:

- typed `Session.client("apigateway")` clients
- typed paginator objects
- literal unions and `TypedDict` request/response shapes for API Gateway operations
- better completion and static analysis in mypy, pyright, or Pylance

It does not replace `boto3`, and it does not change AWS authentication or runtime behavior. Your code still runs through the real `boto3` client.

## Version-Sensitive Notes

- The maintainer docs state that the package version matches the related `boto3` version.
- Keep `mypy-boto3-apigateway` aligned with the `boto3` version your project actually installs, or you can end up with missing methods or stale request/response shapes in type checking.
- The current package line is generated for the classic API Gateway control-plane client, whose boto3 service name is `"apigateway"`.
- Do not confuse this package with the separate AWS services `apigatewayv2` or `apigatewaymanagementapi`; those need different stub packages.

## Install

Choose one of these installation patterns.

### 1. Recommended: `boto3-stubs` extra for implicit client typing

This is the easiest setup for editor completion because `session.client("apigateway")` gets the precise client type automatically.

```bash
python -m pip install "boto3-stubs[apigateway]"
```

### 2. Standalone service package for explicit annotations

Use this when you only want the API Gateway stubs.

```bash
python -m pip install "mypy-boto3-apigateway==1.42.3"
```

### 3. Lite variant when full `boto3-stubs` is too heavy

The maintainer docs note that `boto3-stubs-lite[apigateway]` is more memory-friendly, but you must annotate the client explicitly.

```bash
python -m pip install "boto3-stubs-lite[apigateway]"
```

Runtime code still needs `boto3` itself:

```bash
python -m pip install "boto3"
```

## Authentication And Setup

This package does not add any auth layer. Credentials, profiles, regions, retries, and endpoints all come from `boto3` and the AWS SDK configuration chain.

Practical defaults:

- local development: `AWS_PROFILE` plus `AWS_DEFAULT_REGION`
- CI or cloud runtimes: IAM role, workload identity, or environment-injected temporary credentials
- explicit `Session(profile_name=..., region_name=...)` when you want deterministic behavior

Typed setup with an explicit session:

```python
from boto3.session import Session

session = Session(profile_name="dev", region_name="us-east-1")
client = session.client("apigateway")
```

## Core Usage

### Explicitly type the client

Use the generated client type when you install `mypy-boto3-apigateway` directly or the lite package:

```python
from boto3.session import Session
from mypy_boto3_apigateway.client import APIGatewayClient

session = Session(profile_name="dev", region_name="us-east-1")
client: APIGatewayClient = session.client("apigateway")
```

### Call API Gateway operations with typed requests and responses

The package exposes `TypedDict` request and response shapes under `type_defs`.

```python
from boto3.session import Session
from mypy_boto3_apigateway.client import APIGatewayClient
from mypy_boto3_apigateway.type_defs import CreateRestApiRequestTypeDef

session = Session(region_name="us-east-1")
client: APIGatewayClient = session.client("apigateway")

request: CreateRestApiRequestTypeDef = {
    "name": "orders-api",
    "endpointConfiguration": {
        "types": ["REGIONAL"],
    },
}

result = client.create_rest_api(**request)
print(result["id"])
```

### Use typed paginators

The generated docs include paginator classes such as `GetRestApisPaginator`, `GetResourcesPaginator`, and `GetApiKeysPaginator`.

```python
from boto3.session import Session
from mypy_boto3_apigateway.client import APIGatewayClient
from mypy_boto3_apigateway.paginator import GetRestApisPaginator

session = Session(region_name="us-east-1")
client: APIGatewayClient = session.client("apigateway")

paginator: GetRestApisPaginator = client.get_paginator("get_rest_apis")

for page in paginator.paginate():
    for api in page.get("items", []):
        print(api["id"], api["name"])
```

### Use literals for constrained string values

The `literals` module gives you type-checked string unions for many AWS enum-like parameters.

```python
from mypy_boto3_apigateway.literals import EndpointTypeType

def endpoint_config(endpoint_type: EndpointTypeType) -> dict[str, list[str]]:
    return {"types": [endpoint_type]}
```

## Configuration Notes

- The service name is `"apigateway"`, so the typed client comes from `Session.client("apigateway")`.
- Region still matters for control-plane requests. Set `region_name` explicitly unless your environment already does it reliably.
- If you need custom retries, timeouts, or endpoints, configure those through `botocore.config.Config` on the real boto3 client. The stub package only describes types.
- For local testing against emulators, type checking still works as long as the runtime client is created normally with `endpoint_url=...`.

Example with explicit config:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_apigateway.client import APIGatewayClient

session = Session(profile_name="dev", region_name="us-east-1")
config = Config(retries={"mode": "standard", "max_attempts": 10})

client: APIGatewayClient = session.client("apigateway", config=config)
```

## Common Pitfalls

- Installing `mypy-boto3-apigateway` alone does not install or replace `boto3`. You still need the runtime SDK.
- Only `boto3-stubs[apigateway]` gives implicit typing for `session.client("apigateway")`. The standalone and lite packages require explicit annotations.
- The stubs do not validate data at runtime. A typed `TypedDict` request can still fail with a real AWS `ClientError`.
- Use `"apigateway"` for the classic REST API control plane. If your code targets HTTP APIs, WebSocket APIs, or callback management APIs, check whether you actually need `apigatewayv2` or `apigatewaymanagementapi`.
- Keep stub and `boto3` versions aligned. Generated stubs are version-specific.

## When To Reach For The Docs

Use the maintainer docs when you need:

- the exact `APIGatewayClient` method surface
- paginator class names and overloads
- available literal types
- request/response `TypedDict` names for specific operations

Use the AWS boto3 docs when you need:

- auth and credential-chain behavior
- runtime operation semantics and AWS-side errors
- endpoint, region, retry, and transport configuration
