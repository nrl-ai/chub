---
name: mypy-boto3-config
description: "mypy-boto3-config type stubs for typed boto3 AWS Config clients, paginators, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.32"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,config,boto3,python,typing,stubs,mypy,pyright"
---

# mypy-boto3-config Python Package Guide

## Golden Rule

`mypy-boto3-config` is only the typing layer for AWS Config. Keep `boto3` installed for runtime calls, create the real client with `Session().client("config")`, and handle credentials, region, retries, and endpoints through normal boto3 and AWS configuration.

For new AWS Config code, prefer the client interface over boto3 resources. AWS documents clients as the low-level interface that supports all service operations, and boto3 resources are feature-frozen.

## Install

### Recommended for most projects

```bash
python -m pip install boto3 "boto3-stubs[config]"
```

This is the simplest path when you want normal boto3 runtime behavior plus automatic type inference for `Session.client("config")` in editors and type checkers.

### Standalone service stubs

```bash
python -m pip install boto3 "mypy-boto3-config==1.42.32"
```

Use this when you only want the Config service stub package. In this setup, explicit client annotations are usually the clearest option.

### Lower-memory editor fallback

```bash
python -m pip install boto3 "boto3-stubs-lite[config]"
```

Upstream notes that the lite package does not provide `session.client()` and `session.resource()` overloads, so expect to annotate clients explicitly.

### Generate for an exact boto3 line

If your project is pinned to a different boto3 patch line and you want locally generated matching stubs:

```bash
uvx --with "boto3==1.42.32" mypy-boto3-builder
```

## Authentication And Setup

This package does not add its own configuration surface. boto3 still resolves credentials and region through the normal AWS provider chain.

The boto3 credentials guide says boto3 searches, in order:

1. Credentials passed directly to `boto3.client(...)`
2. Credentials passed when creating a `Session`
3. Environment variables
4. Role-based and other AWS providers later in the chain

Useful environment variables:

- `AWS_PROFILE`
- `AWS_DEFAULT_REGION`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`
- `AWS_MAX_ATTEMPTS`
- `AWS_RETRY_MODE`

Typical local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Then create an explicitly typed client:

```python
from boto3.session import Session
from mypy_boto3_config.client import ConfigServiceClient

session = Session(profile_name="dev", region_name="us-east-1")
config_client: ConfigServiceClient = session.client("config")
```

## Core Usage

### Typed AWS Config client

`ConfigServiceClient` is the main type to use for `boto3.client("config")`.

```python
from boto3.session import Session
from mypy_boto3_config.client import ConfigServiceClient

client: ConfigServiceClient = Session(region_name="us-east-1").client("config")

response = client.describe_config_rules(ConfigRuleNames=["required-tags"])

for rule in response.get("ConfigRules", []):
    print(rule["ConfigRuleName"], rule["ConfigRuleState"])
```

### Typed paginator

The generated stubs include paginator types such as `ListConfigurationRecordersPaginator` and `GetComplianceDetailsByResourcePaginator`.

```python
from boto3.session import Session
from mypy_boto3_config.client import ConfigServiceClient
from mypy_boto3_config.paginator import ListConfigurationRecordersPaginator

client: ConfigServiceClient = Session(region_name="us-east-1").client("config")
paginator: ListConfigurationRecordersPaginator = client.get_paginator(
    "list_configuration_recorders"
)

for page in paginator.paginate():
    for recorder in page.get("ConfigurationRecorders", []):
        print(recorder["name"], recorder["roleARN"])
```

### `TYPE_CHECKING` and `cast(...)` pattern

Use this when stubs are installed only in development or CI:

```python
from typing import TYPE_CHECKING, cast

import boto3

if TYPE_CHECKING:
    from mypy_boto3_config.client import ConfigServiceClient

session = boto3.Session(region_name="us-east-1")
client = cast("ConfigServiceClient", session.client("config"))
```

This keeps runtime imports clean while preserving static typing.

### Literals and `type_defs`

Use generated literal unions and `TypedDict` shapes when helper code builds request objects or validates response structure before the boto3 call site.

```python
from mypy_boto3_config.literals import ComplianceTypeType
from mypy_boto3_config.type_defs import ConfigRuleTypeDef

desired_compliance: ComplianceTypeType = "COMPLIANT"

rule: ConfigRuleTypeDef = {
    "ConfigRuleName": "required-tags",
    "Source": {
        "Owner": "AWS",
        "SourceIdentifier": "REQUIRED_TAGS",
    },
}
```

The generated docs currently list Config-specific shapes such as `ConfigurationRecorderTypeDef`, `ConfigRuleTypeDef`, and `SelectResourceConfigResponseTypeDef`.

## Practical Notes For Agents

- Use the boto3 service name string `config`, not `configservice`, when constructing the runtime client.
- Treat this package as typing-only. Successful type checking does not prove that AWS Config is enabled, a recorder exists, or the caller has the right IAM permissions.
- The AWS Config service reference exposes many read and write operations, including `describe_config_rules`, `get_compliance_details_by_resource`, `put_config_rule`, and `put_configuration_recorder`. Prefer read-only calls in diagnostics and automation unless the task explicitly requires mutation.
- If you already install `boto3-stubs[config]`, many IDEs can infer the client type without explicit annotations. Keep explicit annotations in reusable helpers and libraries where clarity matters more than editor magic.

## Common Pitfalls

- Installing `mypy-boto3-config` without `boto3` and expecting runtime AWS calls to work.
- Importing the package with hyphens in Python code. The import root is `mypy_boto3_config`.
- Expecting automatic `Session.client("config")` inference when you installed only `mypy-boto3-config` or `boto3-stubs-lite[config]`.
- Treating generated `TypedDict` definitions as runtime validation. boto3 still returns ordinary dictionaries.
- Assuming the stubs package changes authentication, region resolution, retry behavior, or endpoints. Those all remain boto3 concerns.
- Using boto3 resources for new Config code. AWS says new boto3 features land on clients, not resources.
- Letting `boto3`, `botocore`, and the stubs drift too far apart. Runtime calls may still work while your typings become stale.

## Version-Sensitive Notes

- This entry is pinned to version used here `1.42.32`.
- On 2026-03-12, PyPI lists `mypy-boto3-config 1.42.32`, marks it as `Stubs Only`, and says it was generated with `mypy-boto3-builder 8.12.0`.
- PyPI also says `mypy-boto3-config` follows the related `boto3` version, so matching the stub version to the boto3 line is the safest default.
- The maintainer docs site is a rolling generated build. On 2026-03-12, its local-generation example already showed `boto3==1.42.62` even though PyPI for this package still published `1.42.32`. Use PyPI for exact release pinning and use the docs site for import names and generated module layout.
- The current generated docs surface client, paginator, literal, and `type_defs` modules for Config. They do not present a boto3 resource entry point for this service, which fits AWS guidance to prefer clients.

## Official Sources

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_config/`
- Maintainer client reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_config/client/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-config/`
- PyPI release JSON: `https://pypi.org/pypi/mypy-boto3-config/1.42.32/json`
- AWS boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- AWS boto3 clients guide: `https://docs.aws.amazon.com/boto3/latest/guide/clients.html`
- AWS boto3 resources guide: `https://docs.aws.amazon.com/boto3/latest/guide/resources.html`
- AWS Config boto3 service reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/config.html`
