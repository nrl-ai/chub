---
name: mypy-boto3-sts
description: "Type annotations for boto3 STS clients, literals, and TypedDicts in Python projects"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,sts,boto3,mypy,pyright,type-stubs,python"
---

# `mypy-boto3-sts` Python Package Guide

## Golden Rule

`mypy-boto3-sts` is a typing package for `boto3`, not a runtime AWS SDK. Use it to improve editor completion and static checking for STS clients, keep it aligned with your `boto3` release line, and let `boto3` handle credentials, sessions, and actual API calls.

## Install

Choose one installation path. Do not install all three.

### Option 1: full `boto3-stubs` with STS overloads

Best when you want `Session().client("sts")` to infer the client type automatically.

```bash
python -m pip install "boto3-stubs[sts]==1.42.3"
```

### Option 2: `boto3-stubs-lite` for lower IDE and type-checker overhead

Use this when full overload support is too heavy for your editor. The maintainer docs note that the lite build does not provide `session.client()` or `session.resource()` overloads, so you need explicit annotations.

```bash
python -m pip install "boto3-stubs-lite[sts]==1.42.3"
python -m pip install "boto3==1.42.3"
```

### Option 3: standalone `mypy-boto3-sts`

Use this when you only want the STS typing package. You still need `boto3` at runtime.

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-sts==1.42.3"
```

## Client Setup

For actual AWS calls, create the client through `boto3`:

```python
import boto3
from mypy_boto3_sts import STSClient

session = boto3.Session(profile_name="dev", region_name="us-east-1")
sts: STSClient = session.client("sts")

identity = sts.get_caller_identity()
print(identity["Account"])
print(identity["Arn"])
```

If you installed full `boto3-stubs[sts]`, your type checker can usually infer the return type from `session.client("sts")`. If you installed `boto3-stubs-lite[sts]` or standalone `mypy-boto3-sts`, keep the explicit `STSClient` annotation.

## Authentication And Configuration

This package does not define its own auth behavior. `boto3` still resolves credentials and region settings from the normal AWS chain.

The Boto3 credentials guide is the source of truth for configuration order. The sources that matter most in practice are:

1. Explicit credentials passed to `Session(...)` or `client(...)`
2. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, and `AWS_DEFAULT_REGION`
3. Shared AWS config and credentials files, usually `~/.aws/config` and `~/.aws/credentials`
4. Assume-role, web identity, IAM Identity Center, container, or instance-role providers

Typical local setup:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Then construct the typed client normally:

```python
import boto3
from mypy_boto3_sts import STSClient

sts: STSClient = boto3.Session().client("sts")
```

## Core Usage

### Type an STS client

`STSClient` is the main type for `boto3.client("sts")`.

```python
from boto3.session import Session
from mypy_boto3_sts.client import STSClient

def get_client() -> STSClient:
    return Session(region_name="us-east-1").client("sts")
```

### Type `get_caller_identity()` responses

Use the generated response `TypedDict` when you want stricter checking around returned fields.

```python
import boto3
from mypy_boto3_sts import STSClient
from mypy_boto3_sts.type_defs import GetCallerIdentityResponseTypeDef

sts: STSClient = boto3.Session(region_name="us-east-1").client("sts")
identity: GetCallerIdentityResponseTypeDef = sts.get_caller_identity()

account_id = identity["Account"]
arn = identity["Arn"]
user_id = identity["UserId"]
```

### Type `assume_role()` request and response payloads

The package exposes `TypedDict` request and response shapes for STS operations.

```python
import boto3
from mypy_boto3_sts import STSClient
from mypy_boto3_sts.type_defs import (
    AssumeRoleRequestTypeDef,
    AssumeRoleResponseTypeDef,
)

sts: STSClient = boto3.Session(profile_name="admin", region_name="us-east-1").client("sts")

request: AssumeRoleRequestTypeDef = {
    "RoleArn": "arn:aws:iam::123456789012:role/CrossAccountReadOnly",
    "RoleSessionName": "reporting-job",
    "DurationSeconds": 3600,
}

response: AssumeRoleResponseTypeDef = sts.assume_role(**request)
creds = response["Credentials"]

print(creds["AccessKeyId"])
print(creds["Expiration"])
```

Useful STS response shapes exposed by the package include:

- `CredentialsTypeDef`
- `GetCallerIdentityResponseTypeDef`
- `AssumeRoleResponseTypeDef`
- `GetSessionTokenResponseTypeDef`
- `GetFederationTokenResponseTypeDef`

### Use literals for stricter helper APIs

The generated literals module includes `STSServiceName`, `ServiceName`, `ResourceServiceName`, and `RegionName`.

```python
from mypy_boto3_sts.literals import STSServiceName

def normalize_service_name(value: STSServiceName) -> str:
    return value

service_name: STSServiceName = "sts"
```

### Keep type-only imports out of production dependencies when needed

The maintainer docs explicitly recommend `TYPE_CHECKING` guards if you do not want stub packages installed in production. They also note that `pylint` may complain unless you provide fallback `object` assignments.

```python
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from mypy_boto3_sts import STSClient
else:
    STSClient = object
```

Use this pattern only when your deployment environment does not include the typing package. If your runtime environment already installs the stubs, plain imports are simpler.

## Common STS Operations Covered By The Stubs

The generated `STSClient` docs currently include typing for the standard STS calls agents usually need:

- `get_caller_identity`
- `assume_role`
- `assume_role_with_web_identity`
- `assume_role_with_saml`
- `get_session_token`
- `get_federation_token`
- `decode_authorization_message`

The current generated docs also expose newer STS operations such as:

- `assume_root`
- `get_delegated_access_token`
- `get_web_identity_token`

Verify that your pinned `boto3` and `botocore` versions actually expose the same operations before copying examples for those newer methods into a locked project.

## Common Pitfalls

- Do not treat `mypy-boto3-sts` as a replacement for `boto3`. It adds types only.
- Do not expect automatic `session.client("sts")` inference from the lite package. The maintainer docs explicitly say lite builds do not include those overloads.
- Keep the stubs on the same release line as `boto3` when you want the best method and shape coverage.
- Pin the version from PyPI, not from the generated docs site. The docs root is a rolling generated site and can be ahead of the published wheel.
- These types help editors and static analysis, but they do not validate IAM policies, trust policies, MFA requirements, or role-session limits at runtime.
- `STSClient` typing does not eliminate ordinary boto3 error handling. AWS service errors still arrive as runtime exceptions from `botocore`.

## Version-Sensitive Notes

- Frontmatter is pinned to the package version used here `1.42.3`.
- PyPI lists `mypy-boto3-sts 1.42.3` as the current published release checked on `2026-03-12`.
- The generated maintainer docs site is not a strict patch snapshot of the published wheel. Treat it as the canonical API-shape reference, but confirm package pins from PyPI before writing install commands.
- PyPI states this package requires Python `>=3.9` and classifies it as `Stubs Only`.
- PyPI and the related `types-boto3-*` project pages describe versioning as tracking the related `boto3` version, so mismatched `boto3` and stub versions are the main source of typing drift.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_sts/`
- STS client reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_sts/client/`
- STS type definitions: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_sts/type_defs/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-sts/`
- AWS Boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- AWS Boto3 STS reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/sts.html`
