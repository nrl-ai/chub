---
name: mypy-boto3-sns
description: "mypy-boto3-sns type stubs for boto3 SNS clients, resources, paginators, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,sns,type-stubs,mypy,pyright,python"
---

# mypy-boto3-sns Python Package Guide

## Golden Rule

`mypy-boto3-sns` is a stub-only package for `boto3` SNS usage. It improves static typing, but it does not ship the runtime AWS SDK, does not configure credentials, and does not make AWS calls by itself.

Use one of these patterns:

- Install `boto3-stubs[sns]` if you want automatic typing for `Session().client("sns")` and `Session().resource("sns")`.
- Install `mypy-boto3-sns` if you want the standalone SNS stub package and are willing to annotate types explicitly.
- Keep auth, region, retries, and endpoint behavior on the normal `boto3` side.

## Install

### Recommended: automatic boto3 overloads

Use this when you want the best editor inference for ordinary boto3 session code.

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[sns]==1.42.3"
```

### Standalone SNS stubs

Use this when you only want the SNS service package or want to keep imports explicit.

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-sns==1.42.3"
```

### Lower-memory alternative

Use the lite package if your IDE struggles with the full overload set.

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[sns]==1.42.3"
```

The maintainer docs note that the lite package does not provide `session.client()` and `session.resource()` overloads, so explicit annotations are more important in lite mode.

## Runtime Setup And Auth

`mypy-boto3-sns` has no package-specific configuration. Use the standard boto3 credential and region chain.

Common local setup:

```bash
aws configure --profile dev
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Then create an SNS client from a normal session:

```python
from boto3.session import Session

session = Session(profile_name="dev", region_name="us-east-1")
sns = session.client("sns")
```

AWS documents that boto3 searches several locations for credentials, including explicit parameters, environment variables, assume-role configuration, IAM Identity Center, shared config files, container credentials, and EC2 instance metadata.

## Core Usage

### Typed client

Prefer the client interface for new code. AWS documents the boto3 resource interface as feature-frozen.

```python
from boto3.session import Session
from mypy_boto3_sns import SNSClient

session = Session(profile_name="dev", region_name="us-east-1")
sns: SNSClient = session.client("sns")

response = sns.publish(
    TopicArn="arn:aws:sns:us-east-1:123456789012:orders",
    Subject="order-created",
    Message='{"order_id":"123"}',
)

print(response["MessageId"])
```

### Typed paginator

The generated package exposes paginator types such as `ListTopicsPaginator`.

```python
from boto3.session import Session
from mypy_boto3_sns import SNSClient
from mypy_boto3_sns.paginator import ListTopicsPaginator

sns: SNSClient = Session(region_name="us-east-1").client("sns")
paginator: ListTopicsPaginator = sns.get_paginator("list_topics")

for page in paginator.paginate(PaginationConfig={"PageSize": 100}):
    for topic in page.get("Topics", []):
        print(topic["TopicArn"])
```

### Typed service resource

Use resources only if your codebase already depends on them.

```python
from boto3.session import Session
from mypy_boto3_sns import SNSServiceResource

resource: SNSServiceResource = Session(region_name="us-east-1").resource("sns")

for topic in resource.topics.all():
    print(topic.arn)
```

### Typed request and response shapes

The package also exposes generated `TypedDict` and `Literal` modules for helper code that passes dict-shaped data around separately from the boto3 call site.

```python
from mypy_boto3_sns.literals import LanguageCodeStringType
from mypy_boto3_sns.type_defs import PublishInputTypeDef

language: LanguageCodeStringType = "en-US"

request: PublishInputTypeDef = {
    "TopicArn": "arn:aws:sns:us-east-1:123456789012:orders",
    "Message": "hello",
    "MessageAttributes": {},
}
```

## TYPE_CHECKING Pattern

The package documentation explicitly supports importing stub types behind `TYPE_CHECKING` when you do not want the stub package to matter at runtime.

```python
from __future__ import annotations

from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_sns import SNSClient

def make_client() -> "SNSClient":
    return Session(region_name="us-east-1").client("sns")
```

This is also the cleanest answer when linters complain about typing-only imports.

## Common Pitfalls

- Installing only `mypy-boto3-sns` and expecting runtime AWS calls to work. You still need `boto3`.
- Using the package name as a service name. The boto3 client is still `session.client("sns")`.
- Expecting automatic overload inference from `boto3-stubs-lite[sns]`. Lite mode needs explicit annotations.
- Treating the stub package as an auth or config layer. Credentials, profiles, regions, retries, and endpoints still come from `boto3` and botocore.
- Preferring resources for new code. AWS says resources are not getting new features, so clients are the safer default.
- Pinning only one side of the version pair. The maintainer docs state that stub versions follow the related `boto3` version, so keep them aligned when exact API coverage matters.

## Version-Sensitive Notes

- This entry is pinned to the version used here `1.42.3`, and PyPI currently publishes `mypy-boto3-sns 1.42.3`.
- The PyPI project metadata for `1.42.3` says the package version matches the related `boto3` version. If you pin one, pin both.
- The maintainer docs root is a moving `latest` page. On 2026-03-12 it already showed local generation examples for newer `boto3` `1.42.66`, so treat that site as the current API-shape reference, not as release-specific pinning evidence for `1.42.3`.
- If exact generated names matter, verify against the installed package or the release-specific PyPI README before copying a hardcoded `TypedDict` or `Literal` symbol from newer docs.

## Official Sources

- Maintainer docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_sns/
- PyPI project page: https://pypi.org/project/mypy-boto3-sns/
- PyPI JSON metadata: https://pypi.org/pypi/mypy-boto3-sns/json
- Boto3 credentials guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html
- Boto3 resources guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/resources.html
- SNS paginator reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/sns/paginator/ListTopics.html
