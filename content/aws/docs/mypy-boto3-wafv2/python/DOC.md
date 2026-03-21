---
name: mypy-boto3-wafv2
description: "Type stubs for boto3 WAFV2 clients, literals, and TypedDict request/response shapes in Python"
metadata:
  languages: "python"
  versions: "1.42.57"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,wafv2,waf,mypy,pyright,type-stubs,python"
---

# `mypy-boto3-wafv2`

## Golden Rule

Use `mypy-boto3-wafv2` only for typing. Keep `boto3` installed for runtime calls, use the low-level WAFV2 client as the default surface, and set the region deliberately:

- use your workload region for `Scope="REGIONAL"`
- use `region_name="us-east-1"` for all `Scope="CLOUDFRONT"` calls

WAFV2 updates are also lock-token based. Read the object first, then pass the returned `LockToken` into `update_*` or `delete_*` operations.

## Install

Recommended for most editor and CI setups:

```bash
python -m pip install "boto3-stubs[wafv2]"
```

Lower-memory alternative:

```bash
python -m pip install "boto3-stubs-lite[wafv2]"
```

Standalone service stubs with an explicitly pinned runtime SDK:

```bash
python -m pip install "boto3==1.42.57" "mypy-boto3-wafv2==1.42.57"
```

Common alternatives:

```bash
uv add "boto3==1.42.57" "mypy-boto3-wafv2==1.42.57"
poetry add "boto3==1.42.57" "mypy-boto3-wafv2==1.42.57"
```

Notes:

- `boto3-stubs[wafv2]` is the easiest option when you want `Session().client("wafv2")` to infer cleanly in VS Code, Pyright, or mypy.
- `boto3-stubs-lite[wafv2]` is more RAM-friendly, but upstream notes that lite mode does not provide `session.client()` or `session.resource()` overloads, so explicit `WAFV2Client` annotations matter more.
- Installing only `mypy-boto3-wafv2` is not enough for runtime code. This package publishes stubs only.

If you need locally generated stubs matched to a pinned boto3 version, upstream recommends:

```bash
uvx --with "boto3==1.42.57" mypy-boto3-builder
```

## Authentication And Setup

This package does not change AWS authentication. Runtime configuration still comes from `boto3` and the normal credentials provider chain.

AWS documents this lookup order, with the common sources being:

1. Explicit credentials passed to `boto3.client(...)`
2. Explicit credentials passed to `boto3.Session(...)`
3. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN`
4. Shared config and credentials files under `~/.aws/`
5. Container or instance metadata credentials

Typical local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

If your code manages CloudFront-scoped WAF objects, override the region to `us-east-1` for those calls.

Explicit typed session setup:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_wafv2 import WAFV2Client

def make_wafv2_client(scope: str) -> WAFV2Client:
    region = "us-east-1" if scope == "CLOUDFRONT" else "us-west-2"
    session = Session(profile_name="dev", region_name=region)
    return session.client(
        "wafv2",
        config=Config(
            retries={
                "mode": "standard",
                "max_attempts": 10,
            }
        ),
    )
```

Use the client interface as the default. AWS documents that new boto3 features land on clients, not on the higher-level resource interface.

## Core Usage

### Typed client

```python
from boto3.session import Session
from mypy_boto3_wafv2 import WAFV2Client

client: WAFV2Client = Session(region_name="us-west-2").client("wafv2")

response = client.list_web_acls(
    Scope="REGIONAL",
    Limit=20,
)

for acl in response.get("WebACLs", []):
    print(acl["Name"], acl["ARN"])
```

For CloudFront or Amplify-managed WAF, use `Scope="CLOUDFRONT"` and create the client in `us-east-1`.

### Manual pagination with `NextMarker`

WAFV2 list operations expose `NextMarker`. A simple loop is usually clearer than guessing whether a paginator exists for the exact operation you need:

```python
from collections.abc import Iterator
from boto3.session import Session
from mypy_boto3_wafv2 import WAFV2Client

def iter_web_acls(scope: str) -> Iterator[dict]:
    region = "us-east-1" if scope == "CLOUDFRONT" else "us-west-2"
    client: WAFV2Client = Session(region_name=region).client("wafv2")

    marker: str | None = None
    while True:
        kwargs = {"Scope": scope, "Limit": 100}
        if marker:
            kwargs["NextMarker"] = marker

        page = client.list_web_acls(**kwargs)

        for acl in page.get("WebACLs", []):
            yield acl

        marker = page.get("NextMarker")
        if not marker:
            break
```

### Update flow with `LockToken`

WAFV2 uses optimistic locking. Read the current object, keep the returned `LockToken`, then submit the update:

```python
from boto3.session import Session
from mypy_boto3_wafv2 import WAFV2Client

client: WAFV2Client = Session(region_name="us-west-2").client("wafv2")

current = client.get_ip_set(
    Name="office-allowlist",
    Scope="REGIONAL",
    Id="11111111-2222-3333-4444-555555555555",
)

ip_set = current["IPSet"]

result = client.update_ip_set(
    Name=ip_set["Name"],
    Scope="REGIONAL",
    Id=ip_set["Id"],
    Description=ip_set.get("Description", ""),
    Addresses=[
        *ip_set["Addresses"],
        "203.0.113.18/32",
    ],
    LockToken=current["LockToken"],
)

next_lock_token = result["NextLockToken"]
print(next_lock_token)
```

If AWS returns `WAFOptimisticLockException`, fetch the object again and retry with the new token.

### Type-only imports for production images

If production installs omit stub packages, keep imports behind `TYPE_CHECKING`:

```python
from __future__ import annotations

from typing import TYPE_CHECKING
from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_wafv2 import WAFV2Client
else:
    WAFV2Client = object

def build_client() -> "WAFV2Client":
    return Session(region_name="us-east-1").client("wafv2")
```

### Literals and TypedDicts

The generated helper modules are most useful when you validate fixed string values and dictionary shapes before the boto3 call site:

```python
from mypy_boto3_wafv2.literals import ActionValueType
from mypy_boto3_wafv2.type_defs import APIKeySummaryTypeDef

action: ActionValueType = "ALLOW"

def summarize_api_key(item: APIKeySummaryTypeDef) -> tuple[str, int]:
    return (item["TokenDomains"][0], item["Version"])
```

Reach for `type_defs` when helper functions build request dictionaries or when fixtures mock WAFV2 responses.

## Practical Notes For Agents

- Separate typing questions from runtime behavior. Retry policy, permissions, ARNs, rate limits, and service exceptions all come from AWS WAFV2 and boto3, not from the stub package.
- Prefer explicit client factories that encode the correct region for each scope. That removes a common class of CloudFront failures.
- Treat `LockToken` as mandatory mutable-state bookkeeping. Cache it only briefly.
- When associating or inspecting WAF on CloudFront, remember that `get_web_acl_for_resource` is not the CloudFront path. AWS directs CloudFront callers to `GetDistributionConfig` instead.

## Common Pitfalls

- Installing `mypy-boto3-wafv2` without `boto3`. The package is typing-only.
- Expecting `boto3-stubs-lite[wafv2]` to infer `Session().client("wafv2")` automatically. Add an explicit `WAFV2Client` annotation in lite mode.
- Using the wrong region for `Scope="CLOUDFRONT"`. AWS requires `us-east-1` for API and SDK calls on CloudFront scope.
- Calling `get_web_acl_for_resource` for CloudFront distributions. AWS explicitly says not to use that operation for CloudFront.
- Reusing a stale `LockToken` after another process or deployment modified the same Web ACL, IP set, or rule group.
- Assuming WAFV2 has a resource-oriented boto3 surface worth preferring. Stay on the client API for predictability and newer feature coverage.

## Version-Sensitive Notes

- This entry is pinned to version used here `1.42.57`.
- On `2026-03-12`, PyPI listed `mypy-boto3-wafv2 1.42.57`, released on `2026-02-25`, and the maintainer package page described it as type annotations for `boto3 WAFV2 1.42.57` generated with `mypy-boto3-builder 8.12.0`.
- The AWS boto3 docs are rolling `latest` pages and may display neighboring `1.42.5x` doc builds. Use those pages for runtime semantics, but use the PyPI release page as the source of truth for the pinned package version.
- Keep the runtime `boto3` version and the WAFV2 stubs line aligned. If you upgrade boto3 for new WAFV2 operations or shapes, verify that the matching stubs release exists before pinning both.

## Official Sources

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_wafv2/`
- Maintainer client docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_wafv2/client/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-wafv2/`
- Upstream builder repository: `https://github.com/youtype/mypy_boto3_builder`
- AWS boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- AWS boto3 resources guide: `https://docs.aws.amazon.com/boto3/latest/guide/resources.html`
- AWS WAFV2 `list_web_acls`: `https://docs.aws.amazon.com/boto3/latest/reference/services/wafv2/client/list_web_acls.html`
- AWS WAFV2 `get_ip_set`: `https://docs.aws.amazon.com/boto3/latest/reference/services/wafv2/client/get_ip_set.html`
- AWS WAFV2 `update_ip_set`: `https://docs.aws.amazon.com/boto3/latest/reference/services/wafv2/client/update_ip_set.html`
- AWS WAFV2 `get_web_acl_for_resource`: `https://docs.aws.amazon.com/boto3/latest/reference/services/wafv2/client/get_web_acl_for_resource.html`
