---
name: mypy-boto3-secretsmanager
description: "mypy-boto3-secretsmanager package guide for typed boto3 Secrets Manager clients, paginators, literals, and TypedDict responses"
metadata:
  languages: "python"
  versions: "1.42.8"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,secretsmanager,typing,mypy,pyright,python"
---

# mypy-boto3-secretsmanager Python Package Guide

## What It Is

`mypy-boto3-secretsmanager` is a stubs-only package for `boto3` Secrets Manager clients. Use it when you want typed `client(...)` calls, paginator overloads, service literals, and `TypedDict` request and response shapes while still running the normal AWS SDK for Python at runtime.

It does not replace `boto3`, add its own authentication layer, or change Secrets Manager behavior.

## Install

If you are pinning to the version used here, keep `boto3` and the stub package aligned:

```bash
python -m pip install "boto3==1.42.8" "mypy-boto3-secretsmanager==1.42.8"
```

If you want automatic type discovery for `boto3.client("secretsmanager")` and `Session.client("secretsmanager")`, install the extra-based bundle instead:

```bash
python -m pip install "boto3-stubs[secretsmanager]==1.42.8"
```

If editor performance matters more than overload-based auto-discovery, the maintainer docs also provide a lighter option:

```bash
python -m pip install "boto3-stubs-lite[secretsmanager]==1.42.8"
```

For an exact local match without depending on the floating docs site, the maintainer recommends generating stubs locally:

```bash
uvx --with 'boto3==1.42.8' mypy-boto3-builder
```

## Setup And Auth

All configuration still comes from `boto3` and `botocore`.

Typical setup:

- local development: `AWS_PROFILE=dev`
- region selection: `AWS_DEFAULT_REGION=us-east-1` or `region_name="us-east-1"`
- deployed workloads: IAM role, web identity, or other normal AWS credential sources

Example:

```python
import boto3

session = boto3.Session(profile_name="dev", region_name="us-east-1")
```

## Initialize A Typed Client

With the standalone service package, annotate the client explicitly:

```python
from boto3.session import Session
from mypy_boto3_secretsmanager import SecretsManagerClient

session = Session(profile_name="dev", region_name="us-east-1")
client: SecretsManagerClient = session.client("secretsmanager")
```

If you installed `boto3-stubs[secretsmanager]`, the type checker can usually infer the client type without the explicit annotation.

## Core Usage

### Read A Secret Value

```python
from boto3.session import Session
from mypy_boto3_secretsmanager import SecretsManagerClient
from mypy_boto3_secretsmanager.type_defs import GetSecretValueResponseTypeDef

session = Session(region_name="us-east-1")
client: SecretsManagerClient = session.client("secretsmanager")

response: GetSecretValueResponseTypeDef = client.get_secret_value(
    SecretId="my/app/config",
)

secret_string = response.get("SecretString")
secret_binary = response.get("SecretBinary")

if secret_string is not None:
    print(secret_string)
elif secret_binary is not None:
    print(secret_binary.decode("utf-8"))
```

Notes:

- AWS returns the `AWSCURRENT` version when you do not pass `VersionId` or `VersionStage`.
- `SecretString` and `SecretBinary` are mutually exclusive in normal use; handle both if you are writing reusable helpers.

### Write A New Secret Version

Use `create_secret` for the initial secret and `put_secret_value` for later versions.

```python
from boto3.session import Session
from mypy_boto3_secretsmanager import SecretsManagerClient
from mypy_boto3_secretsmanager.type_defs import PutSecretValueRequestTypeDef

session = Session(region_name="us-east-1")
client: SecretsManagerClient = session.client("secretsmanager")

client.create_secret(
    Name="my/app/config",
    SecretString='{"token":"initial"}',
)

put_request: PutSecretValueRequestTypeDef = {
    "SecretId": "my/app/config",
    "SecretString": '{"token":"rotated"}',
}
client.put_secret_value(**put_request)
```

Notes:

- `put_secret_value` creates a new version; it does not mutate an existing version in place.
- If you omit `VersionStages`, AWS moves `AWSCURRENT` to the new version automatically.

### Paginate `list_secrets`

```python
from boto3.session import Session
from mypy_boto3_secretsmanager import SecretsManagerClient
from mypy_boto3_secretsmanager.paginator import ListSecretsPaginator

session = Session(region_name="us-east-1")
client: SecretsManagerClient = session.client("secretsmanager")
paginator: ListSecretsPaginator = client.get_paginator("list_secrets")

for page in paginator.paginate(IncludePlannedDeletion=False):
    for item in page.get("SecretList", []):
        print(item["Name"])
```

AWS documents `ListSecrets` as eventually consistent. If you need the freshest state for one secret after a write, call `describe_secret` for that secret instead of trusting a list result immediately.

## Useful Typed Imports

Common imports for this package:

```python
from mypy_boto3_secretsmanager import SecretsManagerClient
from mypy_boto3_secretsmanager.literals import FilterNameStringTypeType
from mypy_boto3_secretsmanager.paginator import ListSecretsPaginator
from mypy_boto3_secretsmanager.type_defs import (
    CreateSecretRequestTypeDef,
    GetSecretValueResponseTypeDef,
    PutSecretValueRequestTypeDef,
)
```

Use `type_defs` when you want typed kwargs or typed return values in helper functions:

```python
from mypy_boto3_secretsmanager.type_defs import CreateSecretRequestTypeDef

request: CreateSecretRequestTypeDef = {
    "Name": "my/app/config",
    "SecretString": '{"token":"initial"}',
}
```

## Keep Runtime Dependencies Optional

Because this package is stubs-only, some projects keep it out of production images and import it only for type checking:

```python
from __future__ import annotations

from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_secretsmanager import SecretsManagerClient

def get_client() -> "SecretsManagerClient":
    return boto3.client("secretsmanager", region_name="us-east-1")
```

If your linter complains about `TYPE_CHECKING`-only names, the maintainer docs recommend using the heavier `boto3-stubs[...]` install or falling back to a non-typing alias in runtime-only branches.

## Common Pitfalls

- The package name uses hyphens, but the import root is `mypy_boto3_secretsmanager`.
- `mypy-boto3-secretsmanager` is not a runtime SDK. You still need `boto3` for actual API calls.
- Keep `boto3`, `botocore`, and the stubs in the same release family. The maintainer states that stub versions track the related `boto3` version.
- The docs root from PyPI is a floating latest snapshot. On 2026-03-12 it was showing generator commands for `boto3==1.42.66`, not the package version covered here `1.42.8`.
- `boto3-stubs-lite` is more memory-friendly but does not provide the `session.client(...)` and `boto3.client(...)` overloads that many IDEs use for automatic inference.
- `put_secret_value` creates a new secret version every time. AWS warns against calling it more often than roughly once every 10 minutes on a sustained basis because you can run into version-count limits.
- `list_secrets` can lag recent writes for several minutes.
- Runtime exceptions still come from `botocore` and AWS service behavior, not from this package.

## Version-Sensitive Notes

- This entry is intentionally pinned to `1.42.8`, the version used here and the current PyPI release dated 2025-12-11.
- The maintainer docs URL is useful for module structure and available typed imports, but it is not version-pinned. Treat it as canonical for package layout, not as proof that every latest-page method exists in `1.42.8`.
- If you need an exact stub surface for the installed `boto3` build, use the maintainer's local-generation workflow with `mypy-boto3-builder`.
- AWS runtime semantics such as staging labels, KMS permissions, caching guidance, and eventual consistency come from the boto3/AWS docs, not the stub package itself.

## Official Sources Used

- `https://pypi.org/project/mypy-boto3-secretsmanager/`
- `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_secretsmanager/`
- `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_secretsmanager/client/`
- `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_secretsmanager/type_defs/`
- `https://docs.aws.amazon.com/boto3/latest/reference/services/secretsmanager/client/get_secret_value.html`
- `https://docs.aws.amazon.com/boto3/latest/reference/services/secretsmanager/client/put_secret_value.html`
- `https://docs.aws.amazon.com/boto3/latest/reference/services/secretsmanager/client/list_secrets.html`
