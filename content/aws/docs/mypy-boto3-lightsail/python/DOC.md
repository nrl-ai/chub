---
name: mypy-boto3-lightsail
description: "Type annotations for boto3 Lightsail in Python, covering typed clients, paginators, literals, and TypedDict request and response shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,lightsail,type-stubs,mypy,pyright,autocomplete,python"
---

# mypy-boto3-lightsail Python Package Guide

## Golden Rule

`mypy-boto3-lightsail` is a typing package, not the runtime SDK. Install `boto3` for real Lightsail API calls, then choose one typing mode:

- Use `boto3-stubs[lightsail]` when you want `Session().client("lightsail")` to infer `LightsailClient` automatically in editors and type checkers.
- Use `mypy-boto3-lightsail` when you want only the service-specific stubs package and are willing to annotate `LightsailClient`, paginator types, literals, and `type_defs` explicitly.
- Use `boto3-stubs-lite[lightsail]` if full overload support is too heavy for PyCharm or constrained environments. The lite package drops `session.client/resource` overloads, so explicit annotations become more important.

## Version-Sensitive Notes

- The version used here `1.42.3` matches the official PyPI package version checked on `2026-03-12`.
- The maintainer docs state that these generated stubs track the related `boto3` version. Pin `boto3` and `mypy-boto3-lightsail` together when you want method signatures and typed request shapes to stay aligned.
- The published docs root is a live generated documentation surface, not a version-pinned archive. When the hosted docs drift, prefer the package version you actually installed.
- The published Lightsail stub docs expose `client`, `paginator`, `literals`, and `type_defs`. I did not find published `service_resource` or `waiter` modules for Lightsail in the current docs, so plan around typed clients and paginators.
- The maintainer docs recommend local generation with `mypy-boto3-builder` when you need stubs that exactly match a different installed `boto3` build.

## Install

Choose one install mode based on how much inference support you want.

### Best inference: full boto3 stubs

Use this when you want `session.client("lightsail")` to infer without extra annotations:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[lightsail]==1.42.3"
```

### Service-specific package only

Use this when you want only the Lightsail stubs package and are fine with explicit annotations:

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-lightsail==1.42.3"
```

### Lite aggregate package

Use this when IDE performance matters more than automatic overload inference:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[lightsail]==1.42.3"
```

Common alternatives:

```bash
uv add "boto3==1.42.3" "boto3-stubs[lightsail]==1.42.3"
poetry add "boto3==1.42.3" "boto3-stubs[lightsail]==1.42.3"
```

## Authentication And Runtime Setup

`mypy-boto3-lightsail` adds typing only. Credentials, region resolution, retries, endpoints, and actual API behavior still come from `boto3` and `botocore`.

AWS documents that boto3 searches for credentials in the standard provider chain, including:

1. Explicit credentials passed in code
2. Environment variables
3. Shared AWS config and credentials files
4. Container or instance role credentials

Typical local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Typed client setup:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_lightsail.client import LightsailClient

session = Session(profile_name="dev", region_name="us-east-1")

lightsail: LightsailClient = session.client(
    "lightsail",
    config=Config(
        retries={
            "mode": "standard",
            "max_attempts": 10,
        }
    ),
)
```

Keep the region explicit. Lightsail resources are regional, and missing or mismatched region configuration is an easy way to get `NoRegionError` or to query the wrong account region.

## Core Usage

### Typed Lightsail client

Use the generated client type for normal boto3 calls:

```python
from boto3.session import Session
from mypy_boto3_lightsail.client import LightsailClient

def get_lightsail_client() -> LightsailClient:
    return Session(region_name="us-east-1").client("lightsail")

client = get_lightsail_client()
response = client.get_instances()

for instance in response.get("instances", []):
    print(instance["name"])
```

### Typed paginator

The stub docs publish overloads for many paginated operations, including `get_instances`, `get_operations`, `get_key_pairs`, `get_domains`, and `get_relational_databases`:

```python
from boto3.session import Session
from mypy_boto3_lightsail.client import LightsailClient
from mypy_boto3_lightsail.paginator import GetInstancesPaginator

client: LightsailClient = Session(region_name="us-east-1").client("lightsail")
paginator: GetInstancesPaginator = client.get_paginator("get_instances")

for page in paginator.paginate():
    for instance in page.get("instances", []):
        print(instance["name"])
```

### Typed request dictionaries

Use generated request `TypedDict` shapes when helper functions assemble kwargs before the client call:

```python
from boto3.session import Session
from mypy_boto3_lightsail.client import LightsailClient
from mypy_boto3_lightsail.type_defs import GetInstanceRequestTypeDef

client: LightsailClient = Session(region_name="us-east-1").client("lightsail")

params: GetInstanceRequestTypeDef = {
    "instanceName": "web-1",
}

response = client.get_instance(**params)
print(response["instance"]["name"])
```

### Typed response dictionaries and literals

Use `type_defs` for response handling and `literals` for constrained string values:

```python
from boto3.session import Session
from mypy_boto3_lightsail.client import LightsailClient
from mypy_boto3_lightsail.literals import InstanceAccessProtocolType
from mypy_boto3_lightsail.type_defs import GetInstanceAccessDetailsResultTypeDef

client: LightsailClient = Session(region_name="us-east-1").client("lightsail")
protocol: InstanceAccessProtocolType = "ssh"

result: GetInstanceAccessDetailsResultTypeDef = client.get_instance_access_details(
    instanceName="web-1",
    protocol=protocol,
)

print(result["accessDetails"]["username"])
```

### `TYPE_CHECKING` guard for runtime-safe imports

If production installs omit stub packages, keep the imports behind `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_lightsail.client import LightsailClient
else:
    LightsailClient = object

client: "LightsailClient" = boto3.Session(region_name="us-east-1").client("lightsail")
```

The maintainer docs call this safe and specifically note it as a workaround for a common `pylint` complaint about names defined only under `TYPE_CHECKING`.

## Tooling Notes

- `boto3-stubs[lightsail]` is the easiest choice for VSCode, Pylance, mypy, and pyright because it provides automatic boto3 client overloads.
- The maintainer PyPI docs warn that PyCharm can become slow on `Literal` overloads. If that happens, switch to `boto3-stubs-lite[lightsail]` or disable PyCharm's built-in type checker and run `mypy` or `pyright` separately.
- The standalone `mypy-boto3-lightsail` package is useful when you want only Lightsail typings installed, but you should annotate factory return types explicitly instead of relying on boto3 overload inference.
- If you need exact parity with a newer or older boto3 lockfile, generate service stubs locally with `mypy-boto3-builder` rather than trusting the hosted docs snapshot.

## Common Pitfalls

- Do not treat `mypy-boto3-lightsail` as the runtime SDK. Real AWS calls still require `boto3`.
- Do not confuse the package name and the import root: install `mypy-boto3-lightsail`, import `mypy_boto3_lightsail`.
- Do not expect `boto3-stubs-lite[lightsail]` or the standalone service package to infer `Session().client("lightsail")` automatically in all tooling. Add explicit `LightsailClient` annotations.
- Do not assume Lightsail has typed resource or waiter modules just because other AWS services do. The current published stub docs expose typed clients and paginators, not a Lightsail resource layer.
- Do not forget region selection. Lightsail resources such as instances, buckets, databases, and distributions live in specific regions.
- Do not attribute auth or retry behavior to the stubs package. Credential resolution, endpoint selection, and retry configuration are still boto3 concerns.

## Official Sources

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_lightsail/`
- Maintainer client reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_lightsail/client/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-lightsail/`
- AWS boto3 Lightsail reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/lightsail.html`
- AWS boto3 credential guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- AWS boto3 clients guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/clients.html`
