---
name: mypy-boto3-transfer
description: "mypy-boto3-transfer package guide for typed boto3 AWS Transfer Family clients, paginators, waiters, literals, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.45"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,transfer,transfer-family,boto3,mypy-boto3-transfer,boto3-stubs,typing,type-checking"
---

# mypy-boto3-transfer Python Package Guide

## Golden Rule

Use `boto3` for real AWS Transfer Family API calls and use `mypy-boto3-transfer` only for typing. If you want `Session.client("transfer")` to infer automatically, install `boto3-stubs[transfer]`; if you install only the standalone or lite package, annotate `TransferClient` explicitly.

## Version-Sensitive Notes

- On `2026-03-12`, the version used here `1.42.45` matches the current official PyPI release for `mypy-boto3-transfer`.
- The PyPI release page shows `1.42.45` was published on `2026-02-09` and generated with `mypy-boto3-builder 8.12.0`.
- Upstream states that the package version follows the related `boto3` version. Keep your stub package close to the `boto3` line you actually run so request/response shapes do not drift.
- The maintainer docs root is a stable package URL, not a release-pinned docs snapshot. Pin the package in your environment when exact patch parity matters.
- The published docs expose `client`, `paginator`, `waiter`, `literals`, `type_defs`, and examples. I did not find a dedicated `service_resource` module for Transfer, so plan around typed clients.
- The release history skips some patch numbers. Do not assume every `boto3` patch has a matching standalone `mypy-boto3-transfer` upload.
- If you need stubs that exactly match the `boto3` build in your environment, the maintainer docs recommend generating them locally with `mypy-boto3-builder`.

## Install

Choose one install mode based on how much boto3 typing support you want.

### Best inference: full boto3 stubs

Use this when you want automatic type discovery for `Session.client("transfer")`, `client.get_paginator(...)`, and `client.get_waiter(...)`:

```bash
python -m pip install "boto3" "boto3-stubs[transfer]"
```

### Service-specific package only

Use this when you want only the Transfer Family typing package and are willing to add explicit annotations:

```bash
python -m pip install "boto3==1.42.45" "mypy-boto3-transfer==1.42.45"
```

### Lite aggregate package

Use this when the full stubs package is too heavy for your IDE or environment:

```bash
python -m pip install "boto3" "boto3-stubs-lite[transfer]"
```

The lite package is more RAM-friendly, but the maintainer docs note that it does not provide `session.client/resource` overloads, so explicit annotations become more important.

## Initialization And Setup

`mypy-boto3-transfer` does not add its own auth or config layer. All runtime behavior still comes from normal boto3 setup:

- credentials from environment variables, shared AWS config files, IAM Identity Center, assume-role config, container credentials, or runtime IAM roles
- optional profile selection through `AWS_PROFILE` or `boto3.Session(profile_name=...)`
- optional `botocore.config.Config` settings for region, retries, timeouts, proxies, and endpoint behavior

Typical setup:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Then create the real boto3 client and annotate it:

```python
from typing import TYPE_CHECKING

import boto3
from botocore.config import Config

if TYPE_CHECKING:
    from mypy_boto3_transfer.client import TransferClient

config = Config(
    region_name="us-east-1",
    retries={"mode": "standard", "max_attempts": 10},
)

session = boto3.Session(profile_name="dev")
transfer: "TransferClient" = session.client("transfer", config=config)
```

If you target LocalStack or a private endpoint for tests, set `endpoint_url=...` on the real boto3 client. The stub package does not change endpoint resolution.

## Core Usage

### Typed Transfer client

Use the typed client for ordinary server, user, connector, and workflow calls:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_transfer.client import TransferClient

transfer: "TransferClient" = boto3.Session().client("transfer")

response = transfer.list_servers(MaxResults=10)

for server in response.get("Servers", []):
    print(server["ServerId"], server["State"], server["Protocols"])
```

### Typed paginator

Use paginators instead of hand-rolled token loops when listing users or servers:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_transfer.client import TransferClient
    from mypy_boto3_transfer.paginator import ListUsersPaginator

transfer: "TransferClient" = boto3.Session().client("transfer")

paginator: "ListUsersPaginator" = transfer.get_paginator("list_users")

for page in paginator.paginate(ServerId="s-0123456789abcdef0", MaxResults=50):
    for user in page.get("Users", []):
        print(user["UserName"], user["Role"])
```

### Typed waiter for server state

Transfer Family server start and stop operations are asynchronous. Use the waiter that matches the state you need before moving to later automation:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_transfer.client import TransferClient
    from mypy_boto3_transfer.waiter import ServerOnlineWaiter

transfer: "TransferClient" = boto3.Session().client("transfer")
server_id = "s-0123456789abcdef0"

transfer.start_server(ServerId=server_id)

waiter: "ServerOnlineWaiter" = transfer.get_waiter("server_online")
waiter.wait(ServerId=server_id)
```

Use `server_offline` after `stop_server(...)` when a workflow depends on the server reaching `OFFLINE`.

### Typed request dictionaries and literals

The generated docs also expose Transfer-specific `TypedDict` request and response shapes plus literal aliases. This is useful when helper functions assemble request dictionaries before the client call:

```python
from mypy_boto3_transfer.literals import AgreementStatusTypeType
from mypy_boto3_transfer.type_defs import As2AsyncMdnConnectorConfigOutputTypeDef

status: AgreementStatusTypeType = "ACTIVE"

connector_config: As2AsyncMdnConnectorConfigOutputTypeDef = {
    "LocalProfileId": "p-0123456789abcdef0",
    "PartnerProfileId": "p-abcdef0123456789",
    "MdnSigningAlgorithm": "SHA256WITHRSA",
    "MdnSigningCertificate": "arn:aws:acm:us-east-1:123456789012:certificate/example",
}
```

Use the published `type_defs` reference when you need exact shapes for operations such as `create_user`, `describe_server`, connectors, certificates, or workflow detail objects.

## Runtime-Safe Typing Pattern

If your production image does not install stub packages, keep the imports inside `TYPE_CHECKING` so the type checker sees them without turning the stubs into a runtime dependency:

```python
from __future__ import annotations

from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_transfer.client import TransferClient

client: "TransferClient" = boto3.Session().client("transfer")
```

## Transfer-Family-Specific Notes

- The boto3 service name is `transfer`, not `mypy-boto3-transfer`.
- `create_user` applies to Transfer Family servers that use the `SERVICE_MANAGED` identity provider. It is not the universal user-management path for every identity setup.
- If you use logical directories, the server and user configuration must agree on `HomeDirectoryType="LOGICAL"` and the corresponding directory mappings.
- Waiters tell you when the control-plane server state changes. They do not confirm that individual file transfers or workflows finished successfully.
- Use the real AWS Transfer Family docs for runtime semantics such as connector behavior, protocol support, identity-provider requirements, and billing details.

## Common Pitfalls

- The PyPI package name uses hyphens, but Python imports use underscores: `mypy_boto3_transfer`.
- This package is typing-only. Real AWS requests still require `boto3`.
- If you choose `mypy-boto3-transfer` or `boto3-stubs-lite[transfer]`, do not expect all `Session.client("transfer")` calls to infer automatically. Add explicit `TransferClient` annotations.
- Keep `boto3`, `botocore`, and `mypy-boto3-transfer` reasonably aligned. New Transfer Family fields can appear in runtime models before matching stubs land.
- PyCharm can be slow with large `Literal` overload sets. If IDE responsiveness matters more than overload inference, `boto3-stubs-lite` is the safer default.
- Do not mix up package docs with runtime docs. The maintainer docs explain the typing surface; the AWS Transfer Family docs explain service behavior and API semantics.

## Official Sources

- Docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_transfer/`
- Client reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_transfer/client/`
- Waiters reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_transfer/waiters/`
- Type defs reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_transfer/type_defs/`
- Versioning guide: `https://youtype.github.io/boto3_stubs_docs/#versioning`
- PyPI package page: `https://pypi.org/project/mypy-boto3-transfer/`
- PyPI exact release: `https://pypi.org/project/mypy-boto3-transfer/1.42.45/`
- Maintainer repository: `https://github.com/youtype/types-boto3`
- Boto3 Transfer service reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/transfer.html`
- Boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- Boto3 configuration guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/configuration.html`
