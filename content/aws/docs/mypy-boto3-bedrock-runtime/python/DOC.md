---
name: mypy-boto3-bedrock-runtime
description: "mypy-boto3-bedrock-runtime guide for Python type-safe Bedrock Runtime clients, literals, paginators, and TypedDict request shapes"
metadata:
  languages: "python"
  versions: "1.42.42"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "python,aws,boto3,bedrock,bedrock-runtime,mypy,typing,stubs"
---

# mypy-boto3-bedrock-runtime Python Package Guide

## What It Is

`mypy-boto3-bedrock-runtime` is a stubs-only package for the `boto3` Bedrock Runtime client.

Use it when you want static typing for:

- `boto3.client("bedrock-runtime")` as `BedrockRuntimeClient`
- the `list_async_invokes` paginator
- `literals` such as `AsyncInvokeStatusType`
- `type_defs` such as `ConverseRequestTypeDef`, `ConverseResponseTypeDef`, and `InvokeModelResponseTypeDef`

It does not send requests by itself. You still need `boto3` installed and AWS credentials configured.

## Install

Install the runtime SDK and the stubs together if you want reproducible typing:

```bash
python -m pip install "boto3==1.42.42" "mypy-boto3-bedrock-runtime==1.42.42"
```

If you want boto3 overload auto-discovery in editors and type checkers, the maintainer docs recommend the umbrella extra:

```bash
python -m pip install "boto3-stubs[bedrock-runtime]"
```

If memory use in the editor matters more than auto-discovery, the docs also provide a lite variant:

```bash
python -m pip install "boto3-stubs-lite[bedrock-runtime]"
```

`boto3-stubs-lite` is more RAM-friendly, but it does not provide `session.client()` and `session.resource()` overloads, so explicit annotations become more important.

## Initialize A Typed Client

Use the normal boto3 session flow and annotate the returned client.

```python
from boto3.session import Session

from mypy_boto3_bedrock_runtime import BedrockRuntimeClient

session = Session(profile_name="bedrock-dev", region_name="us-east-1")
client: BedrockRuntimeClient = session.client("bedrock-runtime")
```

Use `"bedrock-runtime"` for inference calls. `"bedrock"` is a different control-plane service.

## Core Usage

### Typed `converse` requests

Use `type_defs` when you want request and response dictionaries checked before the API call.

```python
from boto3.session import Session

from mypy_boto3_bedrock_runtime import BedrockRuntimeClient
from mypy_boto3_bedrock_runtime.type_defs import (
    ConverseRequestTypeDef,
    ConverseResponseTypeDef,
    InferenceConfigurationTypeDef,
)

session = Session(region_name="us-east-1")
client: BedrockRuntimeClient = session.client("bedrock-runtime")

inference_config: InferenceConfigurationTypeDef = {
    "maxTokens": 256,
    "temperature": 0.2,
}

request: ConverseRequestTypeDef = {
    "modelId": "your-model-id",
    "messages": [
        {
            "role": "user",
            "content": [{"text": "Summarize why typed boto3 clients help."}],
        }
    ],
    "inferenceConfig": inference_config,
}

response: ConverseResponseTypeDef = client.converse(**request)
print(response["output"]["message"]["content"])
```

The Bedrock Runtime reference documents `modelId` and `messages` as the core `converse` inputs. The stubs package gives you the matching request and response shapes.

### Typed `invoke_model` responses

Use `invoke_model` when you need the raw model-specific JSON body instead of the higher-level `converse` interface.

```python
import json
from boto3.session import Session

from mypy_boto3_bedrock_runtime import BedrockRuntimeClient
from mypy_boto3_bedrock_runtime.type_defs import InvokeModelResponseTypeDef

session = Session(region_name="us-east-1")
client: BedrockRuntimeClient = session.client("bedrock-runtime")

payload = {
    "inputText": "Explain the difference between boto3 and its stubs.",
}

response: InvokeModelResponseTypeDef = client.invoke_model(
    body=json.dumps(payload).encode("utf-8"),
    contentType="application/json",
    accept="application/json",
    modelId="your-model-id",
)

body = json.loads(response["body"].read())
print(body)
```

The AWS reference requires `body`, `contentType`, `accept`, and `modelId`, and it specifies JSON request bodies for `invoke_model`.

### Typed paginator

Bedrock Runtime currently exposes one paginator: `list_async_invokes`.

```python
from boto3.session import Session

from mypy_boto3_bedrock_runtime import BedrockRuntimeClient
from mypy_boto3_bedrock_runtime.paginator import ListAsyncInvokesPaginator
from mypy_boto3_bedrock_runtime.type_defs import AsyncInvokeSummaryTypeDef

session = Session(region_name="us-east-1")
client: BedrockRuntimeClient = session.client("bedrock-runtime")

paginator: ListAsyncInvokesPaginator = client.get_paginator("list_async_invokes")

for page in paginator.paginate(statusEquals="Completed", maxResults=25):
    for summary in page.get("asyncInvokeSummaries", []):
        item: AsyncInvokeSummaryTypeDef = summary
        print(item["invocationArn"], item["status"])
```

### Literals for constrained values

Use the generated literal types when a field only accepts a small fixed set of strings.

```python
from mypy_boto3_bedrock_runtime.literals import AsyncInvokeStatusType

status: AsyncInvokeStatusType = "Completed"
```

This is useful for filters like `statusEquals` on async invocation listing and for avoiding string typos in your own wrappers.

## Authentication And Configuration

This package inherits normal boto3 configuration behavior because all runtime calls still go through boto3.

Practical defaults:

- set a Bedrock-supported AWS region explicitly
- use `AWS_PROFILE` for local development when possible
- use IAM roles or workload identity in deployed environments
- avoid hard-coding credentials unless you are handling temporary credentials intentionally

Useful environment variables:

```bash
export AWS_PROFILE=bedrock-dev
export AWS_DEFAULT_REGION=us-east-1
```

According to the boto3 credentials guide, boto3 searches for credentials in this order: explicit client parameters, explicit `Session` parameters, environment variables, assume-role providers, web identity, IAM Identity Center, shared credentials/config files, container credentials, and EC2 instance metadata.

## Common Pitfalls

### This package is types only

`mypy-boto3-bedrock-runtime` does not bundle the runtime client. If `boto3` is missing, your code will still fail at import or runtime even if the type checker passes.

### Use the correct service name

Inference uses:

```python
session.client("bedrock-runtime")
```

Do not switch this to `session.client("bedrock")` unless you are intentionally using the Bedrock control-plane API.

### Keep boto3 and stubs in the same version family

The maintainer docs state that `mypy-boto3-bedrock-runtime` versions track the related `boto3` version. In practice, pinning both to the same `1.42.42` line avoids missing parameters and stale editor hints.

### `boto3-stubs-lite` changes the ergonomics

The lite package is valid, but the docs explicitly warn that it does not provide `session.client/resource` overloads. If you use it, annotate returned clients directly instead of relying on inference.

### Use `TYPE_CHECKING` if you do not want a runtime dependency

The PyPI project description notes that it is safe to gate imports behind `TYPE_CHECKING` when you only want the stubs for development and CI type checks.

## Version-Sensitive Notes

- The version covered here is `1.42.42`.
- PyPI currently shows `1.42.42` as the latest release, published on February 4, 2026.
- The generated maintainer docs are useful for symbols and examples, but the local-generation snippet on that site still references `boto3==1.42.39` as of March 12, 2026.
- The AWS boto3 Bedrock Runtime reference is a moving latest-docs target and currently renders newer boto3 patch documentation than this package version. Treat the AWS method list as authoritative for service behavior, but keep your installed `boto3`, `botocore`, and stubs aligned when you need exact type coverage.

## Official Sources Used

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_bedrock_runtime/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-bedrock-runtime/`
- AWS Bedrock Runtime boto3 reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/bedrock-runtime.html`
- AWS boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
