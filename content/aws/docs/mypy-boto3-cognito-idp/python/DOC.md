---
name: mypy-boto3-cognito-idp
description: "mypy-boto3-cognito-idp typed boto3 stubs for Cognito Identity Provider clients, paginators, literals, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.59"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,cognito,cognito-idp,python,typing,stubs,mypy"
---

# mypy-boto3-cognito-idp Python Package Guide

## What It Is

`mypy-boto3-cognito-idp` adds static typing for the Cognito Identity Provider part of `boto3`.

Use it when you want type checking and editor completion for:

- `Session.client("cognito-idp")` as `CognitoIdentityProviderClient`
- generated paginator types such as `ListUserPoolsPaginator`
- generated `Literal` aliases such as `AuthFlowTypeType`
- generated `TypedDict` request and response shapes such as `AdminGetUserResponseTypeDef`

This package does not replace `boto3` at runtime. Real AWS calls still go through normal `boto3` clients and normal AWS credentials.

## Version Note

This entry is pinned to the version used here `1.42.59`, and the official PyPI package page for `mypy-boto3-cognito-idp` also reported `1.42.59` on March 12, 2026.

The upstream versioning guide says these packages follow the related `boto3` version. Keep your stub package and runtime `boto3` dependency on the same release family whenever possible.

## Install

Install the runtime SDK plus the service-specific stubs:

```bash
python -m pip install "boto3==1.42.59" "mypy-boto3-cognito-idp==1.42.59"
```

If your team standardizes on the umbrella stubs package, the official docs also support:

```bash
python -m pip install "boto3-stubs[cognito-idp]==1.42.59"
```

If editor memory usage matters more than automatic `Session.client(...)` inference, the upstream docs also list the lighter variant:

```bash
python -m pip install "boto3-stubs-lite[cognito-idp]==1.42.59"
```

`boto3-stubs-lite` keeps the generated Cognito IDP types, but the official docs note that it does not provide the same `session.client(...)` and `session.resource(...)` overloads as full `boto3-stubs`.

## Initialization And Setup

Keep setup boring: configure AWS auth and region the same way you would for any other boto3 client, then annotate the returned client with the generated type.

```python
from boto3.session import Session
from mypy_boto3_cognito_idp import CognitoIdentityProviderClient

session = Session(profile_name="dev", region_name="us-west-2")
cognito: CognitoIdentityProviderClient = session.client("cognito-idp")
```

If the stub package is only installed in development, guard typing-only imports:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_cognito_idp import CognitoIdentityProviderClient

def get_cognito_client() -> "CognitoIdentityProviderClient":
    return Session(region_name="us-west-2").client("cognito-idp")
```

## Config And Authentication

`mypy-boto3-cognito-idp` does not add any auth helpers. Credential lookup and region resolution still come from `boto3` and `botocore`.

Typical local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

Use the same patterns you already trust for boto3:

- shared AWS config and credentials files
- environment variables such as `AWS_PROFILE` and `AWS_DEFAULT_REGION`
- IAM roles or assumed roles in deployed environments
- explicit `Session(...)` configuration when you need to override defaults

Cognito IDP is regional. Make sure the client region matches the user pool region before you debug request shapes or permissions.

## Core Usage

### Typed Cognito IDP Client

```python
from boto3.session import Session
from mypy_boto3_cognito_idp import CognitoIdentityProviderClient

cognito: CognitoIdentityProviderClient = Session(
    region_name="us-west-2",
).client("cognito-idp")

pool = cognito.describe_user_pool(UserPoolId="us-west-2_example")["UserPool"]
print(pool["Id"], pool["Name"])
```

### Typed Response Shapes

```python
from boto3.session import Session
from mypy_boto3_cognito_idp import CognitoIdentityProviderClient
from mypy_boto3_cognito_idp.type_defs import AdminGetUserResponseTypeDef

cognito: CognitoIdentityProviderClient = Session(
    region_name="us-west-2",
).client("cognito-idp")

response: AdminGetUserResponseTypeDef = cognito.admin_get_user(
    UserPoolId="us-west-2_example",
    Username="alice@example.com",
)

for attribute in response.get("UserAttributes", []):
    print(attribute["Name"], attribute.get("Value"))
```

### Typed Paginators

```python
from boto3.session import Session
from mypy_boto3_cognito_idp import CognitoIdentityProviderClient
from mypy_boto3_cognito_idp.paginator import ListUserPoolsPaginator

cognito: CognitoIdentityProviderClient = Session(
    region_name="us-west-2",
).client("cognito-idp")

paginator: ListUserPoolsPaginator = cognito.get_paginator("list_user_pools")

for page in paginator.paginate(MaxResults=60):
    for pool in page.get("UserPools", []):
        print(pool["Id"], pool["Name"])
```

### Literals For Safer Enum-Like Values

```python
from mypy_boto3_cognito_idp.literals import AuthFlowTypeType

auth_flow: AuthFlowTypeType = "USER_PASSWORD_AUTH"
```

Import literal aliases when you want the type checker to reject unsupported Cognito string values before runtime.

## IDE And Type Checker Notes

The official package page explicitly calls out support for:

- `mypy`
- `pyright`
- PyCharm
- VSCode with Pylance

Install the stubs into the same environment that your editor or CI type checker analyzes. If your editor struggles with large overload sets, the upstream docs suggest trying `boto3-stubs-lite`.

## Common Pitfalls

### PyPI Name, Import Name, And Service Name Differ

- install: `mypy-boto3-cognito-idp`
- import: `mypy_boto3_cognito_idp`
- runtime service name: `"cognito-idp"`

Do not substitute one for another.

### These Are Stubs, Not A Runtime SDK

Keep importing and executing `boto3`. The stubs only improve static analysis and autocomplete.

### `boto3-stubs-lite` Needs More Explicit Annotations

If you choose `boto3-stubs-lite[cognito-idp]`, expect to annotate client variables manually because the full session overloads are intentionally omitted upstream.

### Keep Versions Aligned

These types are generated from the boto3 service model. If `boto3`, `botocore`, and `mypy-boto3-cognito-idp` drift too far apart, method signatures and `TypedDict` fields can stop matching what your runtime client actually supports.

### Treat `TYPE_CHECKING` As The Safe Default For Dev-Only Stubs

If production images do not install stub packages, keep imports for annotation-only names behind `TYPE_CHECKING` so runtime imports stay clean.

### Do Not Expect A Separate Cognito Resource Interface Here

The official package docs for `mypy-boto3-cognito-idp` expose client, paginator, literal, and `type_defs` sections. Use typed clients as the default workflow.

## Version-Sensitive Notes

- Official sources checked on March 12, 2026 matched the version used here `1.42.59`.
- The official PyPI page lists `Requires: Python >=3.9`.
- The generated docs root is stable across releases rather than release-specific. When exact parity matters, verify the installed package version on PyPI and pin the dependency explicitly.

## Official Sources

- Docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_cognito_idp/`
- Versioning guide: `https://youtype.github.io/boto3_stubs_docs/#versioning`
- PyPI package page: `https://pypi.org/project/mypy-boto3-cognito-idp/`
- Maintainer repository linked from PyPI: `https://github.com/youtype/mypy_boto3_builder`
