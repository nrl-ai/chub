---
name: mypy-boto3-inspector2
description: "Typed boto3 stubs for Amazon Inspector2 clients, paginators, literals, and TypedDict request and response shapes"
metadata:
  languages: "python"
  versions: "1.42.49"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,inspector2,type-stubs,mypy,pyright,python"
---

# mypy-boto3-inspector2 Python Package Guide

## Golden Rule

- Use `mypy-boto3-inspector2` only for static typing of Amazon Inspector code that runs through `boto3`.
- Real AWS requests, credentials, retries, endpoints, and IAM behavior still come from `boto3` and `botocore`.
- Keep the stub package on the same release line as your runtime `boto3` when you want reliable request and response typing.

## Install

Install the runtime SDK and the matching stubs together:

```bash
python -m pip install "boto3==1.42.49" "mypy-boto3-inspector2==1.42.49"
```

Upstream also documents the umbrella extras packages:

```bash
python -m pip install "boto3-stubs[inspector2]==1.42.49"
python -m pip install "boto3-stubs-lite[inspector2]==1.42.49"
```

Common alternatives:

```bash
uv add boto3==1.42.49
uv add --dev mypy-boto3-inspector2==1.42.49
poetry add boto3==1.42.49
poetry add --group dev mypy-boto3-inspector2==1.42.49
```

Use `boto3-stubs[inspector2]` when you want overloads for `boto3.client("inspector2")` and `Session.client("inspector2")`. Use `mypy-boto3-inspector2` when you want only the Inspector2 type package and are willing to annotate the client and paginator types explicitly. Use `boto3-stubs-lite[inspector2]` if the full overload set is too heavy for your editor; the lite variant omits those boto3 helper overloads.

## Initialize And Authenticate

Authentication is standard boto3 authentication. Prefer the normal AWS credential chain:

1. `aws configure` or named profiles in `~/.aws/config` and `~/.aws/credentials`
2. Environment variables such as `AWS_PROFILE`, `AWS_DEFAULT_REGION`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN`
3. IAM roles or workload credentials in AWS runtimes

Typical setup:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_inspector2.client import Inspector2Client

session = Session(profile_name="dev", region_name="us-east-1")

client: Inspector2Client = session.client(
    "inspector2",
    config=Config(
        retries={"mode": "standard", "max_attempts": 10},
    ),
)
```

If you use `boto3-stubs-lite[inspector2]`, add an explicit cast because the overloaded client helpers are not installed:

```python
from typing import cast
from boto3.session import Session
from mypy_boto3_inspector2.client import Inspector2Client

session = Session(region_name="us-east-1")
client = cast(Inspector2Client, session.client("inspector2"))
```

## Core Usage

### Typed client helpers

Annotate helper signatures with the generated client type instead of `Any`:

```python
from mypy_boto3_inspector2.client import Inspector2Client

def list_finding_arns(client: Inspector2Client) -> list[str]:
    response = client.list_findings(maxResults=25)
    return [finding["findingArn"] for finding in response.get("findings", [])]
```

### Typed response shapes

Use generated `type_defs` when you want request and response dictionaries to stay explicit:

```python
from mypy_boto3_inspector2.client import Inspector2Client
from mypy_boto3_inspector2.type_defs import ListFindingsResponseTypeDef

def get_critical_titles(client: Inspector2Client) -> list[str]:
    response: ListFindingsResponseTypeDef = client.list_findings(
        filterCriteria={
            "severity": [{"comparison": "EQUALS", "value": "CRITICAL"}],
        },
        maxResults=25,
    )
    return [finding["title"] for finding in response.get("findings", [])]
```

### Typed paginators

The generated paginator surface is useful for account-wide or large-result scans:

```python
from boto3.session import Session
from mypy_boto3_inspector2.client import Inspector2Client
from mypy_boto3_inspector2.paginator import ListFindingsPaginator

client: Inspector2Client = Session(region_name="us-east-1").client("inspector2")
paginator: ListFindingsPaginator = client.get_paginator("list_findings")

for page in paginator.paginate(PaginationConfig={"PageSize": 100}):
    for finding in page.get("findings", []):
        print(finding["severity"], finding["findingArn"])
```

### Typed request bodies

For update-style APIs, define the payload with a generated request shape before splatting it into the boto3 call:

```python
from mypy_boto3_inspector2.client import Inspector2Client
from mypy_boto3_inspector2.type_defs import (
    UpdateEc2DeepInspectionConfigurationRequestTypeDef,
)

def enable_deep_inspection(client: Inspector2Client) -> None:
    request: UpdateEc2DeepInspectionConfigurationRequestTypeDef = {
        "activateDeepInspection": True,
        "packagePaths": ["/usr/lib", "/usr/local/lib"],
    }
    client.update_ec2_deep_inspection_configuration(**request)
```

### `TYPE_CHECKING` guard for dev-only stubs

If the stubs are only installed in development, guard the imports and keep runtime imports clean:

```python
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from mypy_boto3_inspector2.client import Inspector2Client

def uses_client(client: "Inspector2Client") -> str:
    return client.meta.service_model.service_name
```

## Config And Tooling Notes

- This package does not load credentials, regions, or retry configuration by itself.
- Put region, timeouts, endpoints, and retry behavior on the boto3 session or client.
- Install the stubs in the same environment your IDE, `mypy`, or `pyright` analyzes.
- The PyPI description recommends the `TYPE_CHECKING` guard pattern when the package is not available in production installs.
- The maintainer docs call out PyCharm performance problems with `Literal`-heavy overloads; switch to `boto3-stubs-lite[inspector2]` or run `mypy`/`pyright` outside PyCharm if that becomes an issue.

## Common Pitfalls

- Install name and import name differ: install `mypy-boto3-inspector2`, import `mypy_boto3_inspector2`.
- This package does not replace `boto3`; without `boto3`, your code cannot make Inspector runtime calls.
- The boto3 service name is `inspector2`. Do not guess older names like `inspector`.
- If `boto3` and the stubs drift apart, your type hints can stop matching the actual request and response shapes.
- Missing region, credentials, service enablement, or IAM permissions are still runtime failures; static typing does not prevent them.
- The rolling maintainer docs can drift ahead of the exact package version published on PyPI, so use PyPI for exact pinning.

## Version-Sensitive Notes

- On `2026-03-12`, PyPI published `mypy-boto3-inspector2 1.42.49`, which matches the version used here for this session.
- The PyPI project description says the stub package version follows the related `boto3` version line, so align your pins when you need accurate types.
- The maintainer docs site is a rolling generated reference. Treat it as the best source for the available typed modules and boto3 method coverage, and treat PyPI as the source of truth for exact package version pinning.
- PyPI requires Python `>=3.9` for the current release line.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_inspector2/`
- Client docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_inspector2/client/`
- Paginators docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_inspector2/paginators/`
- Package page: `https://pypi.org/project/mypy-boto3-inspector2/`
- boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- boto3 Inspector2 reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/inspector2.html`
