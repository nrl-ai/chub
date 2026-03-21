---
name: mypy-boto3-proton
description: "mypy-boto3-proton package guide for typed boto3 Proton clients, paginators, waiters, and generated request/response shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,proton,boto3,mypy-boto3-proton,boto3-stubs,typing,type-checking"
---

# mypy-boto3-proton Python Package Guide

## Golden Rule

Use `boto3` for runtime AWS calls and use `mypy-boto3-proton` only for typing.

If you want `Session().client("proton")` to infer the right type automatically, install `boto3-stubs[proton]`. If you install `mypy-boto3-proton` or `boto3-stubs-lite[proton]`, expect to add explicit type annotations for clients, paginators, and waiters.

## Install

### Recommended for normal boto3 code

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[proton]==1.42.3"
```

Use this when you want the easiest editor and type-checker experience. The maintainer docs say the full `boto3-stubs` service extra gives auto-discovery for `session.client(...)` and `client.get_paginator(...)` calls, so ordinary boto3 code usually needs no extra annotations.

### Lower-memory option

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[proton]==1.42.3"
```

Use this when editor performance matters more than automatic overload discovery. The maintainer docs note that the lite variant does not provide the `session.client()` and `session.resource()` overloads, so explicit annotations become the normal workflow.

### Standalone Proton stubs

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-proton==1.42.3"
```

Use the standalone package when you only want Proton typings or when you prefer keeping stub imports behind `TYPE_CHECKING`.

## Setup And Authentication

`mypy-boto3-proton` does not authenticate to AWS and does not create clients by itself. Credentials, region, retries, and endpoint selection still come from `boto3`.

Practical setup:

1. Create a `boto3.Session(...)` with an explicit profile and region when the environment is not obvious.
2. Let boto3 load credentials from the normal AWS chain unless you are intentionally passing temporary credentials.
3. Keep auth and runtime config on the boto3 session or client, not in the stubs package.

Minimal setup:

```python
from boto3.session import Session

session = Session(profile_name="dev", region_name="us-west-2")
client = session.client("proton")
```

Important credential sources from the boto3 docs:

- environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, `AWS_PROFILE`, and `AWS_DEFAULT_REGION`
- shared config in `~/.aws/credentials` and `~/.aws/config`
- assume-role, web-identity, and IAM Identity Center profiles
- container or EC2 instance role credentials

For most local development, use a profile instead of hardcoding keys:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

## Core Usage

### Zero-annotation workflow with `boto3-stubs[proton]`

With the full service extra installed, type discovery should work on normal boto3 code:

```python
from boto3.session import Session

session = Session(region_name="us-west-2")
client = session.client("proton")

response = client.list_services()
for service in response.get("services", []):
    print(service["name"])
```

This is the simplest setup for agents generating application code.

### Explicit client annotations

Use explicit annotations when you installed the standalone package or the lite package:

```python
from boto3.session import Session
from mypy_boto3_proton import ProtonClient

session = Session(region_name="us-west-2")
client: ProtonClient = session.client("proton")

response = client.list_services()
print(response.get("services", []))
```

### Explicit paginator annotations

The maintainer docs expose many typed Proton paginators, including `ListServicesPaginator`, `ListServiceInstancesPaginator`, `ListEnvironmentsPaginator`, and `ListComponentProvisionedResourcesPaginator`.

```python
from boto3.session import Session
from mypy_boto3_proton import ProtonClient
from mypy_boto3_proton.paginator import ListServicesPaginator

session = Session(region_name="us-west-2")
client: ProtonClient = session.client("proton")
paginator: ListServicesPaginator = client.get_paginator("list_services")

for page in paginator.paginate():
    for service in page.get("services", []):
        print(service["name"])
```

### Typed waiters, literals, and `type_defs`

The package also provides:

- typed waiter classes under `mypy_boto3_proton.waiter`
- generated literal unions under `mypy_boto3_proton.literals`
- generated `TypedDict` request and response shapes under `mypy_boto3_proton.type_defs`

Use those modules when helper functions need stricter typing than a raw `dict[str, Any]` or string literal can provide.

## `TYPE_CHECKING` Pattern

The maintainer docs explicitly say it is safe to keep this dependency out of production by importing it only for type checking.

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_proton import ProtonClient
    from mypy_boto3_proton.paginator import ListServicesPaginator
else:
    ProtonClient = object
    ListServicesPaginator = object

session = Session(region_name="us-west-2")
client: "ProtonClient" = session.client("proton")
paginator: "ListServicesPaginator" = client.get_paginator("list_services")
```

Use this pattern when production should ship only `boto3`. The `object` fallback also avoids the pylint undefined-name problem called out in the upstream docs.

## Configuration Notes

- `mypy-boto3-proton` has no package-specific runtime configuration surface. If auth, region selection, retries, or endpoints are wrong, fix the underlying boto3 client configuration.
- Proton is an AWS service client, so successful type checking does not prove the region, account, IAM permissions, or service resources are valid at runtime.
- Use `botocore.config.Config(...)` with `Session().client("proton", config=...)` when you need explicit retry mode, timeouts, or other client settings.
- Use `endpoint_url=...` on the boto3 client for tests or custom endpoints. The stubs package does not change endpoint behavior.

## Common Pitfalls

- Installing only `mypy-boto3-proton` and expecting real AWS calls to work without `boto3`.
- Importing the package with hyphens in Python code. The import root is `mypy_boto3_proton`.
- Expecting automatic `Session().client("proton")` inference when you installed only `mypy-boto3-proton` or `boto3-stubs-lite[proton]`. Use explicit annotations in those setups.
- Letting the `boto3` and stub versions drift apart. The maintainer docs say the stub package version matches the related `boto3` version.
- Treating successful type checking as proof that credentials, region choice, IAM permissions, or Proton resource names are correct.
- Using old blog posts or copied code that predates the current generated package line. Always verify the current class names and paginator names in the maintainer docs.

## Version-Sensitive Notes For `1.42.3`

- PyPI lists `1.42.3` as the exact `mypy-boto3-proton` release for this entry, uploaded on `2025-12-04`.
- The package description says it provides type annotations for `boto3 Proton 1.42.3`, and the maintainer docs say the package version follows the related `boto3` version.
- The maintainer docs site is rolling generated output, so use the exact PyPI release page and your lockfile when exact package compatibility matters.
- `boto3-stubs[proton]`, `boto3-stubs-lite[proton]`, and standalone `mypy-boto3-proton` are all valid install paths, but only the non-lite bundle is meant to avoid most explicit annotations.
- If your project needs an exact local regeneration for a pinned boto3 line, the maintainer docs recommend `uvx --with 'boto3==1.42.3' mypy-boto3-builder`.

## Official Sources

- Maintainer docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_proton/
- Proton paginators docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_proton/paginators/
- PyPI project: https://pypi.org/project/mypy-boto3-proton/
- PyPI exact release context: https://pypi.org/project/mypy-boto3-proton/1.42.3/
- AWS boto3 Proton reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/proton.html
- AWS boto3 credentials guide: https://docs.aws.amazon.com/boto3/latest/guide/credentials.html
- AWS boto3 configuration guide: https://docs.aws.amazon.com/boto3/latest/guide/configuration.html
