---
name: mypy-boto3-cognito-identity
description: "mypy-boto3-cognito-identity package guide for typed boto3 Cognito Identity clients, paginators, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,cognito,cognito-identity,type-stubs,mypy,pyright,python"
---

# mypy-boto3-cognito-identity Python Package Guide

## What It Is

`mypy-boto3-cognito-identity` is the maintainer-generated type stub package for the Cognito Identity part of `boto3`.

Use it when you want:

- a typed `CognitoIdentityClient`
- paginator annotations such as `ListIdentityPoolsPaginator`
- generated literal unions from the Cognito Identity service model
- generated `TypedDict` request and response shapes under `type_defs`

This package is `Typing :: Stubs Only`. It does not create AWS sessions, sign requests, or fetch credentials. Real API calls still come from `boto3`.

## Golden Rule

- Install `boto3-stubs[cognito-identity]` when you want the best automatic type inference for `Session().client("cognito-identity")`.
- Install `mypy-boto3-cognito-identity` when you only need the Cognito Identity stub package and are willing to annotate the client explicitly.
- Keep runtime AWS setup in normal `boto3` configuration. The stubs package does not change auth, region, retry, or endpoint behavior.

## Install

Pick one mode and keep `boto3` installed in the environment.

### Recommended for most projects

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[cognito-identity]==1.42.3"
```

Use this when you want `Session.client("cognito-identity")` overloads and the least manual annotation work.

### Service-only stubs

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-cognito-identity==1.42.3"
```

Use this when you only want the Cognito Identity stub package.

### Lite package

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[cognito-identity]==1.42.3"
```

The maintainer docs note that the lite package is more RAM-friendly, but it does not provide the same `session.client(...)` overloads. Add explicit annotations when you choose this option.

### Exact-match local generation

```bash
uvx --with "boto3==1.42.3" mypy-boto3-builder
```

Use this when your lockfile must match a specific `boto3` line exactly and you do not want to rely on the latest published stub wheel.

## Initialize And Setup

Create the real client with `boto3`, then add the generated client type:

```python
from boto3.session import Session
from mypy_boto3_cognito_identity.client import CognitoIdentityClient

session = Session(profile_name="dev", region_name="us-east-1")
cognito: CognitoIdentityClient = session.client("cognito-identity")
```

Practical setup choices for local development:

- `profile_name` plus `~/.aws/config` and `~/.aws/credentials`
- environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_DEFAULT_REGION`
- role-based credentials in ECS, EKS, Lambda, or EC2

The boto3 credentials guide says boto3 searches for credentials in a fixed order including explicit client parameters, session parameters, environment variables, assume-role providers, IAM Identity Center, shared credential files, shared config files, container credentials, and EC2 instance metadata.

## Core Usage

### Typed client calls

```python
from boto3.session import Session
from mypy_boto3_cognito_identity.client import CognitoIdentityClient

session = Session(region_name="us-east-1")
cognito: CognitoIdentityClient = session.client("cognito-identity")

response = cognito.list_identity_pools(MaxResults=10)

for pool in response.get("IdentityPools", []):
    print(pool["IdentityPoolName"], pool["IdentityPoolId"])
```

### `TYPE_CHECKING` pattern

Use this when stubs are installed only in development or CI:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_cognito_identity.client import CognitoIdentityClient
else:
    CognitoIdentityClient = object

cognito: CognitoIdentityClient = boto3.client("cognito-identity", region_name="us-east-1")
```

### Typed request and response shapes

The maintainer docs expose request and response aliases such as `GetIdInputTypeDef`, `GetIdResponseTypeDef`, `GetCredentialsForIdentityInputTypeDef`, and `GetCredentialsForIdentityResponseTypeDef`.

```python
from boto3.session import Session
from mypy_boto3_cognito_identity.client import CognitoIdentityClient
from mypy_boto3_cognito_identity.type_defs import (
    GetCredentialsForIdentityInputTypeDef,
    GetCredentialsForIdentityResponseTypeDef,
    GetIdInputTypeDef,
    GetIdResponseTypeDef,
)

session = Session(region_name="us-east-1")
cognito: CognitoIdentityClient = session.client("cognito-identity")

logins = {
    "cognito-idp.us-east-1.amazonaws.com/us-east-1_example": "USER_POOL_JWT",
}

get_id_request: GetIdInputTypeDef = {
    "IdentityPoolId": "us-east-1:11111111-2222-3333-4444-555555555555",
    "Logins": logins,
}

identity: GetIdResponseTypeDef = cognito.get_id(**get_id_request)

credentials_request: GetCredentialsForIdentityInputTypeDef = {
    "IdentityId": identity["IdentityId"],
    "Logins": logins,
}

credentials: GetCredentialsForIdentityResponseTypeDef = (
    cognito.get_credentials_for_identity(**credentials_request)
)

print(credentials["Credentials"]["AccessKeyId"])
```

### Typed paginator

The boto3 Cognito Identity reference shows one paginator for this service: `ListIdentityPools`.

```python
from boto3.session import Session
from mypy_boto3_cognito_identity.client import CognitoIdentityClient
from mypy_boto3_cognito_identity.paginator import ListIdentityPoolsPaginator

session = Session(region_name="us-east-1")
cognito: CognitoIdentityClient = session.client("cognito-identity")

paginator: ListIdentityPoolsPaginator = cognito.get_paginator("list_identity_pools")

for page in paginator.paginate(MaxResults=60):
    for pool in page.get("IdentityPools", []):
        print(pool["IdentityPoolId"])
```

### Literals and service-name-safe annotations

```python
from mypy_boto3_cognito_identity.literals import (
    AmbiguousRoleResolutionTypeType,
    CognitoIdentityServiceName,
)

resolution: AmbiguousRoleResolutionTypeType = "AuthenticatedRole"
service_name: CognitoIdentityServiceName = "cognito-identity"
```

## Configuration And Authentication

`mypy-boto3-cognito-identity` adds no package-specific auth layer. All configuration still comes from `boto3` and AWS.

Typical local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Or create a session explicitly:

```python
from boto3.session import Session
from mypy_boto3_cognito_identity.client import CognitoIdentityClient

session = Session(profile_name="dev", region_name="us-east-1")
cognito: CognitoIdentityClient = session.client("cognito-identity")
```

Important Cognito Identity detail:

- AWS credentials authorize the caller to use the Cognito Identity API.
- The `Logins` map is separate from AWS credentials and carries end-user identity-provider tokens.
- The client region should match the target identity pool region.

If code type-checks but calls fail, debug `boto3` credentials, region, IAM policy, or token inputs rather than the stubs package.

## Common Pitfalls

- Installing stubs without `boto3`. These packages add typing only.
- Using the wrong service name. `"cognito-identity"` and `"cognito-idp"` are different services and different stub packages.
- Expecting automatic inference from `mypy-boto3-cognito-identity` or `boto3-stubs-lite[cognito-identity]`. Use explicit `CognitoIdentityClient` annotations in those modes.
- Treating `Logins` as AWS access keys. They are identity-provider tokens used in Cognito Identity flows.
- Assuming the docs root is a frozen per-release snapshot. It is a generated latest-style page, so verify symbols against the installed version when a new alias matters to shared code.
- Expecting extra typed helpers by analogy with other AWS services. The official docs for this package focus on the client, paginator, literals, and `TypedDict` definitions.

## Version-Sensitive Notes

- This guide is pinned to the version used here `1.42.3`.
- The official PyPI project page shows `mypy-boto3-cognito-identity 1.42.3`, generated with `mypy-boto3-builder 8.12.0`, published on `2025-12-04`.
- The official PyPI page also shows Python requirement `>=3.9`.
- The maintainer docs and PyPI project page both state that package versioning follows the related `boto3` version.
- If runtime `boto3` is newer than the installed stubs, newly added operations, paginator names, literals, or `TypeDef` fields can be missing from type checking even when runtime code still executes.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_cognito_identity/`
- PyPI project page: `https://pypi.org/project/mypy-boto3-cognito-identity/`
- AWS boto3 Cognito Identity reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/cognito-identity.html`
- AWS boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
