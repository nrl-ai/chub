---
name: mypy-boto3-kms
description: "Typed boto3 KMS stubs for Python: KMSClient annotations, paginators, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.50"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "mypy-boto3-kms,boto3,kms,aws,python,type-stubs,typing"
---

# mypy-boto3-kms Python Package Guide

## What It Is

`mypy-boto3-kms` is the generated typing package for AWS KMS in `boto3`.

Use it when you need:

- `KMSClient` annotations for `boto3.client("kms")`
- generated paginator types for KMS list operations
- `Literal` aliases for enum-like parameters
- generated `TypedDict` request and response shapes in `type_defs`

It does not replace `boto3` at runtime. You still create real clients with `boto3`; this package only supplies typing information.

## Version Covered

- Package: `mypy-boto3-kms`
- Ecosystem: `pypi`
- Version covered: `1.42.50`
- Registry: `https://pypi.org/project/mypy-boto3-kms/`
- Docs root used for this entry: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_kms/`

The package docs site is generated separately from PyPI and can lag by a few patch releases. As of 2026-03-12, PyPI publishes `1.42.50`, while the maintainer docs still show a `generate-services 1.42.44 kms` example on the landing page. Pin to the version you actually install.

## Install

Install `boto3` plus the KMS stubs:

```bash
python -m pip install boto3 mypy-boto3-kms==1.42.50
```

The maintainer docs also expose bundle installs:

```bash
python -m pip install 'boto3-stubs[kms]'
python -m pip install 'boto3-stubs-lite[kms]'
python -m pip install mypy-boto3-kms
```

Practical choice:

- use `mypy-boto3-kms` when you want only the KMS typing package
- use `boto3-stubs[kms]` when you also want `Session.client(...)` overloads and broader boto3 typing
- use `boto3-stubs-lite[kms]` when full stubs slow down PyCharm or consume too much memory

Python requirement on PyPI is currently `>=3.9`.

## Core Setup

Create the runtime client with `boto3`, then annotate it with `KMSClient`.

```python
from boto3.session import Session
from mypy_boto3_kms.client import KMSClient

session = Session(region_name="us-west-2")
kms: KMSClient = session.client("kms")

aliases = kms.list_aliases(Limit=10)
for alias in aliases.get("Aliases", []):
    print(alias.get("AliasName"))
```

The generated package is organized into a few modules:

```python
from mypy_boto3_kms.client import KMSClient
from mypy_boto3_kms.literals import AlgorithmSpecType
from mypy_boto3_kms.type_defs import AliasListEntryTypeDef
```

Use:

- `client` for the typed KMS client
- `paginator` types when you paginate KMS list operations
- `literals` for constrained string parameters
- `type_defs` for request and response dictionaries

## AWS Auth And Configuration

Authentication still comes from the normal boto3 and AWS SDK credential chain. The stub package adds no auth behavior of its own.

Preferred local setup:

```bash
aws configure
```

Common environment variables:

```bash
export AWS_PROFILE=default
export AWS_DEFAULT_REGION=us-west-2
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
export AWS_SESSION_TOKEN=...
```

The official boto3 credentials guide documents the usual search order, including:

1. explicit credentials passed to `client(...)`
2. explicit credentials passed to `Session(...)`
3. environment variables
4. assume-role providers
5. shared credentials and config files
6. container and EC2 instance metadata

For agent-written code, prefer shared config or short-lived credentials over hard-coded secrets.

## Common Typed Usage Patterns

### Explicit Client Annotation

This is the safest pattern when you install only `mypy-boto3-kms` or use the lite bundle.

```python
from boto3.session import Session
from mypy_boto3_kms.client import KMSClient

session = Session(region_name="us-east-1")
kms: KMSClient = session.client("kms")

metadata = kms.describe_key(KeyId="alias/my-key")
print(metadata["KeyMetadata"]["Arn"])
```

### Paginators

Use paginators for inventory-style operations instead of assuming a single response contains every key or alias.

```python
from boto3.session import Session
from mypy_boto3_kms.client import KMSClient

session = Session(region_name="us-east-1")
kms: KMSClient = session.client("kms")

paginator = kms.get_paginator("list_keys")
for page in paginator.paginate(PaginationConfig={"MaxItems": 100, "PageSize": 25}):
    for item in page.get("Keys", []):
        print(item["KeyId"])
```

### Literal Types And TypedDict Shapes

Literal aliases help keep parameter values valid, and `type_defs` make helper functions easier to type-check.

```python
from mypy_boto3_kms.client import KMSClient
from mypy_boto3_kms.literals import AlgorithmSpecType
from mypy_boto3_kms.type_defs import AliasListEntryTypeDef

def alias_name(alias: AliasListEntryTypeDef) -> str | None:
    return alias.get("AliasName")

def encrypt_blob(kms: KMSClient, key_id: str, plaintext: bytes) -> bytes:
    algorithm: AlgorithmSpecType = "SYMMETRIC_DEFAULT"
    response = kms.encrypt(
        KeyId=key_id,
        Plaintext=plaintext,
        EncryptionAlgorithm=algorithm,
    )
    return response["CiphertextBlob"]
```

### Runtime-Optional Imports

If stubs are installed only in development, keep the import inside `TYPE_CHECKING` and cast the runtime client.

```python
from typing import TYPE_CHECKING, cast
import boto3

if TYPE_CHECKING:
    from mypy_boto3_kms.client import KMSClient
else:
    KMSClient = object

kms = cast(KMSClient, boto3.client("kms", region_name="us-east-1"))
```

## Common Pitfalls

- Install `boto3` as well. The stubs package does not include runtime KMS client code.
- Import names differ from the package name: install `mypy-boto3-kms`, import `mypy_boto3_kms`.
- `boto3-stubs-lite[kms]` intentionally omits `Session.client(...)` and `Session.resource(...)` overloads, so keep explicit client annotations.
- Full `boto3-stubs[kms]` can hurt PyCharm performance because of many generated overloads and literals; switch to the lite bundle if the IDE becomes sluggish.
- The stub package does not configure AWS credentials, region, retries, or endpoints. All of that still comes from boto3 configuration.
- KMS is region-scoped. Make sure `region_name` or `AWS_DEFAULT_REGION` matches the key you are trying to use.

## Version-Sensitive Notes

- PyPI currently publishes `1.42.50`, and this doc is pinned to that version.
- The maintainer docs root is rolling generated output. The landing page still references `generate-services 1.42.44 kms`, so do not copy patch numbers from that snippet blindly.
- When you pin versions, keep `boto3` and `mypy-boto3-kms` on the same `1.42.x` line unless you have verified compatibility another way.
- If you need broader auto-inference from `session.client("kms")`, install the matching `boto3-stubs[kms]` bundle instead of only the standalone package.

## Official Sources

- PyPI project: https://pypi.org/project/mypy-boto3-kms/
- Official generated docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_kms/
- Boto3 credentials guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html
