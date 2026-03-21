---
name: mypy-boto3-sqs
description: "Typed boto3 SQS stubs for Python with install choices, typed clients and resources, paginators, request dicts, and runtime-safe patterns"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,sqs,mypy,typing,stubs,python"
---

# mypy-boto3-sqs Python Package Guide

## Golden Rule

Use `mypy-boto3-sqs` only for static typing and editor support around SQS code. Keep `boto3` as the runtime SDK, keep AWS credentials and region setup in boto3, and annotate clients or resources with the generated SQS types when inference is not enough.

## What This Package Is For

`mypy-boto3-sqs` is the generated type-stub package for the SQS part of boto3. It gives you typed definitions for:

- `SQSClient`
- `SQSServiceResource`
- SQS paginator classes such as `ListQueuesPaginator`
- request and response type definitions such as `SendMessageRequestTypeDef`
- SQS literal unions and typed resource collections

It does not send requests by itself and it does not change boto3 behavior at runtime.

## Install

Pick one of these installation patterns:

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-sqs==1.42.3"
```

If you want boto3 plus generated overloads for `session.client("sqs")` and `session.resource("sqs")`, use the maintained extra package:

```bash
python -m pip install "boto3-stubs[sqs]"
```

If you want lower dependency weight and can live without those overloads, use the lite variant:

```bash
python -m pip install "boto3-stubs-lite[sqs]"
```

Practical guidance:

- Install `boto3` as the runtime dependency either way.
- Use the full `boto3-stubs[sqs]` extra when you want the best IDE inference with the least annotation noise.
- Use `mypy-boto3-sqs` directly when you only need SQS types.
- Use the lite package when build size matters and explicit annotations are acceptable.

## Setup And Initialization

The normal pattern is an ordinary boto3 session plus typed annotations from `mypy_boto3_sqs`.

### Typed client

```python
from boto3.session import Session
from mypy_boto3_sqs import SQSClient

session = Session(profile_name="dev", region_name="us-west-2")
sqs: SQSClient = session.client("sqs")

response = sqs.send_message(
    QueueUrl=queue_url,
    MessageBody="hello from typed boto3",
)

print(response["MessageId"])
```

### Typed resource

```python
from boto3.session import Session
from mypy_boto3_sqs import SQSServiceResource
from mypy_boto3_sqs.service_resource import Queue

session = Session(region_name="us-west-2")
sqs: SQSServiceResource = session.resource("sqs")
queue: Queue = sqs.Queue(queue_url)

result = queue.send_message(MessageBody="hello from the resource API")
print(result.get("MessageId"))
```

### When explicit annotations are optional

With the full `boto3-stubs[sqs]` package, IDEs and type checkers can often infer the SQS client or resource type from the service name. With `mypy-boto3-sqs` alone or with the lite variant, explicit annotations are more useful and sometimes necessary.

## Core Usage Patterns

### Use paginator types for list operations

```python
from boto3.session import Session
from mypy_boto3_sqs import SQSClient
from mypy_boto3_sqs.paginator import ListQueuesPaginator

session = Session(region_name="us-west-2")
sqs: SQSClient = session.client("sqs")
paginator: ListQueuesPaginator = sqs.get_paginator("list_queues")

for page in paginator.paginate(QueueNamePrefix="orders-"):
    for url in page.get("QueueUrls", []):
        print(url)
```

### Use typed request dictionaries in helpers

```python
from boto3.session import Session
from mypy_boto3_sqs import SQSClient
from mypy_boto3_sqs.type_defs import SendMessageRequestTypeDef

session = Session(region_name="us-west-2")
sqs: SQSClient = session.client("sqs")

request: SendMessageRequestTypeDef = {
    "QueueUrl": queue_url,
    "MessageBody": "typed payload",
}

sqs.send_message(**request)
```

### Model long polling explicitly

```python
from boto3.session import Session
from mypy_boto3_sqs import SQSClient
from mypy_boto3_sqs.type_defs import ReceiveMessageRequestTypeDef

session = Session(region_name="us-west-2")
sqs: SQSClient = session.client("sqs")

request: ReceiveMessageRequestTypeDef = {
    "QueueUrl": queue_url,
    "MaxNumberOfMessages": 10,
    "WaitTimeSeconds": 20,
    "MessageAttributeNames": ["All"],
}

response = sqs.receive_message(**request)

for message in response.get("Messages", []):
    print(message["ReceiptHandle"])
```

### Type resource collections when iterating queues

```python
from boto3.session import Session
from mypy_boto3_sqs import SQSServiceResource
from mypy_boto3_sqs.service_resource import Queue
from mypy_boto3_sqs.service_resource import ServiceResourceQueuesCollection

session = Session(region_name="us-west-2")
sqs: SQSServiceResource = session.resource("sqs")
queues: ServiceResourceQueuesCollection = sqs.queues

for queue in queues.all():
    typed_queue: Queue = queue
    print(typed_queue.url)
```

## Runtime-Safe Typing Pattern

If production environments do not install stub packages, keep the imports behind `TYPE_CHECKING` and use forward references:

```python
from __future__ import annotations

from typing import TYPE_CHECKING
from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_sqs import SQSClient

session = Session(region_name="us-west-2")
sqs: "SQSClient" = session.client("sqs")
```

Use this pattern when the package is dev-only and your deployed image only includes `boto3`.

## AWS Credentials And Config

`mypy-boto3-sqs` adds no new auth layer. boto3 still resolves credentials and region from the normal AWS SDK chain:

1. Explicit credentials or region passed to `Session(...)` or `client(...)`
2. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_DEFAULT_REGION`
3. Shared config and credentials files under `~/.aws/`
4. `AWS_PROFILE`
5. IAM roles, IAM Identity Center, or workload identity in AWS-hosted environments

Typical local setup:

```bash
aws configure --profile dev
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

Then create a normal session:

```python
from boto3.session import Session

session = Session(profile_name="dev", region_name="us-west-2")
```

If you need custom retry or timeout behavior, configure it on the boto3 client with `botocore.config.Config`; this stub package only affects types.

## Common Pitfalls

- This package is not a runtime SQS client. You still need `boto3`.
- `boto3-stubs-lite[sqs]` does not provide the session `client` and `resource` overloads from the full variant, so type inference is weaker.
- If you import `mypy_boto3_sqs` directly at runtime but only install it as a dev dependency, production imports will fail. Use `TYPE_CHECKING` when needed.
- SQS request dictionaries are strict about key names. Use the generated `type_defs` instead of untyped dicts when building payloads outside the call site.
- Most SQS APIs use `QueueUrl`, not a queue name. Typed request defs help catch that early.
- FIFO-only keys such as `MessageGroupId` and `MessageDeduplicationId` are valid only for FIFO queues.

## Version-Sensitive Notes

- The version used here for this session was `1.42.3`, and the live PyPI project page also showed `1.42.3` on March 12, 2026.
- The maintainer docs state that service package versions track the related boto3 version line, so keep your stub packages aligned with the boto3 family you actually install.
- The upstream docs site still documents the `boto3-stubs` package family and `mypy_boto3_sqs` imports.
- The upstream repository now positions `types-boto3` as the successor project to `boto3-stubs`. If you are comparing newer examples from that repository, expect equivalent service-specific packages and different import roots.

## Official Sources

- Docs root: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_sqs/
- Type definitions reference: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_sqs/type_defs/
- Service resource reference: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_sqs/service_resource/
- PyPI package page: https://pypi.org/project/mypy-boto3-sqs/
- Upstream repository: https://github.com/youtype/types-boto3
