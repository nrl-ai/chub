---
name: mypy-boto3-appsync
description: "Type annotations for boto3 AppSync in Python, covering typed clients, paginators, literals, and TypedDict request shapes"
metadata:
  languages: "python"
  versions: "1.42.6"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,appsync,boto3,type-stubs,mypy,pyright,autocomplete,graphql"
---

# mypy-boto3-appsync Python Package Guide

## Golden Rule

`mypy-boto3-appsync` is a stubs-only package for static typing, not the runtime SDK. Install `boto3` for real AWS AppSync calls, then choose one typing mode:

- Use `boto3-stubs[appsync]` when you want `Session().client("appsync")` overloads to infer the client type automatically in editors and type checkers.
- Use `mypy-boto3-appsync` when you want only the AppSync stubs package and are willing to annotate `AppSyncClient`, paginator objects, literals, and `TypedDict` request shapes explicitly.
- Use `boto3-stubs-lite[appsync]` when the full overload set is too heavy for PyCharm or low-memory environments. The lite package drops `session.client(...)` overload inference, so explicit annotations matter more.

## Install

If you want automatic boto3 overload discovery in editors and type checkers:

```bash
python -m pip install "boto3==1.42.6" "boto3-stubs[appsync]==1.42.6"
```

If you want the standalone service-specific stubs package:

```bash
python -m pip install "boto3==1.42.6" "mypy-boto3-appsync==1.42.6"
```

If PyCharm becomes slow on literal-heavy overloads:

```bash
python -m pip install "boto3==1.42.6" "boto3-stubs-lite[appsync]==1.42.6"
```

Common alternatives:

```bash
uv add "boto3==1.42.6" "boto3-stubs[appsync]==1.42.6"
poetry add "boto3==1.42.6" "boto3-stubs[appsync]==1.42.6"
```

Notes:

- Keep the stubs version aligned with the boto3 version you pin. The maintainer metadata says `mypy-boto3-appsync` uses the related boto3 version number.
- `mypy-boto3-appsync` does not replace `boto3`. Installing only the stubs package gives type checking but no runtime client implementation.

## Authentication And Runtime Setup

Typing comes from `mypy-boto3-appsync`, but runtime behavior still comes from `boto3` and `botocore`. AWS documents that Boto3 needs both credentials and a region to make requests, and that it searches for credentials in a standard order including explicit parameters, environment variables, assume-role providers, shared credentials files, config files, container credentials, and EC2 instance metadata.

Recommended local setup:

```bash
aws configure
```

Useful environment variables:

```bash
export AWS_PROFILE="dev"
export AWS_DEFAULT_REGION="us-east-1"
export AWS_RETRY_MODE="standard"
export AWS_MAX_ATTEMPTS="10"
```

Prefer an explicit session so the profile and region are obvious:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_appsync import AppSyncClient

session = Session(profile_name="dev", region_name="us-east-1")
config = Config(
    retries={
        "mode": "standard",
        "max_attempts": 10,
    }
)

client: AppSyncClient = session.client("appsync", config=config)
```

AppSync is a client-only boto3 surface in this release line. The AWS boto3 AppSync reference documents a low-level `AppSync.Client` plus paginators; there is no boto3 resource interface or waiter section to rely on here.

## Core Usage

### Typed client

```python
from boto3.session import Session
from mypy_boto3_appsync import AppSyncClient

def get_appsync_client() -> AppSyncClient:
    return Session(region_name="us-east-1").client("appsync")

client = get_appsync_client()
response = client.list_graphql_apis(maxResults=25)

for api in response.get("graphqlApis", []):
    print(api["apiId"], api["name"])
```

Use explicit annotations like this when you install the standalone package or the lite package, because only the full `boto3-stubs[appsync]` extra gives overload-based auto-discovery for `session.client("appsync")`.

### Typed paginator

AppSync supports typed paginator overloads for list operations such as `list_graphql_apis`, `list_data_sources`, and `list_types`:

```python
from boto3.session import Session
from mypy_boto3_appsync import AppSyncClient
from mypy_boto3_appsync.paginator import ListGraphqlApisPaginator

client: AppSyncClient = Session(region_name="us-east-1").client("appsync")
paginator: ListGraphqlApisPaginator = client.get_paginator("list_graphql_apis")

for page in paginator.paginate(PaginationConfig={"PageSize": 25}):
    for api in page.get("graphqlApis", []):
        print(api["apiId"], api["name"])
```

### Literals and TypedDict request shapes

Use `literals` for constrained string values and `type_defs` for nested request structures you build in user code:

```python
from mypy_boto3_appsync.literals import ApiCacheStatusType
from mypy_boto3_appsync.type_defs import CognitoUserPoolConfigTypeDef

status: ApiCacheStatusType = "AVAILABLE"

user_pool_config: CognitoUserPoolConfigTypeDef = {
    "awsRegion": "us-east-1",
    "userPoolId": "us-east-1_example",
    "appIdClientRegex": "my-client-id",
}
```

Build nested request fragments as typed variables first, then pass them into the boto3 client call that needs them. That catches misspelled keys before runtime and keeps large AppSync payloads readable.

### `TYPE_CHECKING` guard for dev-only stubs

If your production image omits typing dependencies, guard imports so runtime environments do not need the stub package:

```python
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from mypy_boto3_appsync import AppSyncClient
else:
    AppSyncClient = object
```

## Tooling Notes

- `boto3-stubs[appsync]` is the easiest option for VSCode plus Pylance, mypy, and pyright because it supports automatic type discovery for `boto3.client(...)` and `session.client(...)`.
- The maintainer PyPI page specifically warns that PyCharm can become slow on `Literal` overloads. If that happens, switch to `boto3-stubs-lite[appsync]` or disable the PyCharm type checker and run `mypy` or `pyright` separately.
- The standalone `mypy-boto3-appsync` package is useful when you only want AppSync typing installed, but your client factories should return explicit `AppSyncClient` types instead of relying on overload inference.

## Common Pitfalls

- Do not treat `mypy-boto3-appsync` as the runtime SDK. Real AWS calls still require `boto3`.
- Do not mix unrelated boto3 and stubs versions casually. This package tracks boto3 versions, so pin them together when you want predictable method signatures.
- Do not expect `boto3-stubs-lite[appsync]` to infer `session.client("appsync")` return types. Add explicit `AppSyncClient` annotations.
- Do not forget region configuration. AppSync endpoints are regional, so `AWS_DEFAULT_REGION` or `region_name` needs to match the target API.
- Do not assume every AppSync list operation is paginated the same way. Use `client.get_paginator(...)` only for operations documented with paginator support.
- Do not hard-code AWS credentials in source. AWS recommends environment variables, shared config files, assume-role flows, or runtime identity providers instead.

## Version-Sensitive Notes

- PyPI lists `mypy-boto3-appsync 1.42.6` as the current release for this package on 2026-03-12, and the release was uploaded on 2025-12-09.
- The package version is intended to match the related boto3 AppSync release line. If you upgrade `boto3`, upgrade the matching AppSync stubs package in the same change.
- The AppSync client surface in current boto3 docs includes both older `*_graphql_api` operations and newer `create_api` or channel-namespace operations. Type hints follow the installed boto3 service model, so confirm the exact method you need against the boto3 AppSync reference for your pinned version.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_appsync/`
- PyPI release metadata: `https://pypi.org/project/mypy-boto3-appsync/1.42.6/`
- PyPI JSON metadata: `https://pypi.org/pypi/mypy-boto3-appsync/json`
- AWS boto3 AppSync reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/appsync.html`
- AWS boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- AWS boto3 configuration guide: `https://docs.aws.amazon.com/boto3/latest/guide/configuration.html`
