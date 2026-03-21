---
name: mypy-boto3-pinpoint
description: "Typed boto3 Amazon Pinpoint stubs for Python with PinpointClient annotations, literals, and TypedDict request and response shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,pinpoint,typing,type-stubs,mypy,pyright,python"
---

# mypy-boto3-pinpoint Python Package Guide

## What It Is

`mypy-boto3-pinpoint` is the generated typing package for the Amazon Pinpoint `boto3` client.

Use it when you want:

- a typed `PinpointClient`
- generated `Literal` aliases for constrained string fields
- generated `TypedDict` request and response shapes from `type_defs`
- better editor completion and static analysis in `mypy`, Pyright, Pylance, and similar tools

It does not replace `boto3` at runtime. Real AWS calls, credentials, retries, regions, and endpoints still come from normal `boto3` and botocore configuration.

The maintainer docs for this package expose `client`, `literals`, and `type_defs`. They do not expose a `service_resource`, paginator, or waiter module for Pinpoint, so treat this as a client-first stubs package.

## Golden Rules

- Install `boto3` for runtime behavior and `mypy-boto3-pinpoint` for typing.
- Keep `boto3`, `botocore`, and the stubs on the same release line when you care about exact request and response shapes.
- Prefer explicit `PinpointClient` annotations when you use the standalone package or `boto3-stubs-lite[pinpoint]`.
- Configure credentials, profiles, regions, retries, and endpoints through standard boto3 settings, not through the stubs package.
- Amazon Pinpoint reaches end of support on `2026-10-30`. Keep using the stubs only for existing Pinpoint workloads, and check AWS End User Messaging for the unaffected SMS, voice, push, OTP, and phone number validate APIs.

## Install

Recommended for pinned environments:

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-pinpoint==1.42.3"
```

Maintainer-supported alternatives:

```bash
python -m pip install "boto3-stubs[pinpoint]==1.42.3"
python -m pip install "boto3-stubs-lite[pinpoint]==1.42.3"
python -m pip install "mypy-boto3-pinpoint==1.42.3"
```

Practical choice:

- use `mypy-boto3-pinpoint` when you want only Pinpoint typings
- use `boto3-stubs[pinpoint]` when you also want `Session.client("pinpoint")` overload inference
- use `boto3-stubs-lite[pinpoint]` when full overloads slow down PyCharm or use too much memory

If you use other package managers:

```bash
uv add "boto3==1.42.3" "mypy-boto3-pinpoint==1.42.3"
poetry add "boto3==1.42.3" "mypy-boto3-pinpoint==1.42.3"
```

If you need stubs generated against the exact `boto3` build in your environment, the maintainer docs show a local generation flow:

```bash
uvx --with "boto3==1.42.3" mypy-boto3-builder
```

## Initialize Type Checking

For the standalone package, annotate boto3 clients explicitly:

```python
from boto3.session import Session
from mypy_boto3_pinpoint.client import PinpointClient

session = Session(profile_name="marketing", region_name="us-east-1")
pinpoint: PinpointClient = session.client("pinpoint")
```

If your production environment excludes dev dependencies, keep the import behind `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_pinpoint.client import PinpointClient

session = Session(profile_name="marketing", region_name="us-east-1")
pinpoint: "PinpointClient" = session.client("pinpoint")
```

If you use `boto3-stubs[pinpoint]`, explicit type annotations are often unnecessary because the session overloads are included.

## Auth And Configuration

`mypy-boto3-pinpoint` has no auth layer of its own. AWS requests still require valid credentials and a region through the normal boto3 credential chain.

Typical local setup:

```bash
aws configure
export AWS_PROFILE=marketing
export AWS_DEFAULT_REGION=us-east-1
```

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_pinpoint.client import PinpointClient

session = Session(profile_name="marketing", region_name="us-east-1")
pinpoint: PinpointClient = session.client(
    "pinpoint",
    config=Config(retries={"mode": "standard", "max_attempts": 5}),
)
```

Use the same boto3 patterns you would use without stubs for:

- shared config and credentials files
- IAM roles, IAM Identity Center, and assumed roles
- custom endpoints for tests
- retry, timeout, and proxy settings via botocore `Config`

## Core Usage

### Typed client for application metadata

```python
from boto3.session import Session
from mypy_boto3_pinpoint.client import PinpointClient
from mypy_boto3_pinpoint.type_defs import GetAppResponseTypeDef

pinpoint: PinpointClient = Session(region_name="us-east-1").client("pinpoint")
response: GetAppResponseTypeDef = pinpoint.get_app(ApplicationId="your-pinpoint-project-id")

print(response["ApplicationResponse"]["Id"])
print(response["ApplicationResponse"]["Name"])
```

Use this pattern when you are validating that the project exists and that your credentials can see it before attempting message sends or endpoint updates.

### Typed message payloads

AWS publishes Pinpoint examples around `send_messages`. Use `type_defs` so nested request bodies stay typed instead of degrading into `dict[str, Any]`.

```python
from boto3.session import Session
from mypy_boto3_pinpoint.client import PinpointClient
from mypy_boto3_pinpoint.type_defs import MessageRequestTypeDef, SendMessagesResponseTypeDef

app_id = "your-pinpoint-project-id"
recipient = "user@example.com"

pinpoint: PinpointClient = Session(region_name="us-east-1").client("pinpoint")

message_request: MessageRequestTypeDef = {
    "Addresses": {
        recipient: {
            "ChannelType": "EMAIL",
        }
    },
    "MessageConfiguration": {
        "EmailMessage": {
            "FromAddress": "sender@example.com",
            "SimpleEmail": {
                "Subject": {"Charset": "UTF-8", "Data": "Hello"},
                "HtmlPart": {"Charset": "UTF-8", "Data": "<h1>Hello</h1>"},
            },
        }
    },
}

response: SendMessagesResponseTypeDef = pinpoint.send_messages(
    ApplicationId=app_id,
    MessageRequest=message_request,
)

result = response["MessageResponse"]["Result"][recipient]
print(result.get("MessageId"))
```

This is safer than constructing the request inline because the type checker can catch missing nested keys and invalid enum-like fields earlier.

### Literal aliases

Use generated literals when your helpers accept constrained string values:

```python
from mypy_boto3_pinpoint.literals import ActionType, ChannelTypeType

channel: ChannelTypeType = "EMAIL"
action: ActionType = "DEEP_LINK"
```

That keeps wrapper utilities and config objects aligned with the generated service model.

## Common Pitfalls

- The package name is `mypy-boto3-pinpoint`, but imports use `mypy_boto3_pinpoint`.
- This is a stubs package, not an AWS SDK. If `boto3` or credentials are missing, type checking may still pass while runtime calls fail.
- `boto3-stubs-lite[pinpoint]` omits the same session overload inference you get from full `boto3-stubs[pinpoint]`. Add explicit annotations instead of assuming IDE inference will work.
- Pinpoint is client-oriented in the generated docs. Do not expect `service_resource`, named paginator, or waiter modules for this package.
- Version skew between `boto3`, `botocore`, and the generated stubs can show up as missing methods, wrong literal values, or stale `TypedDict` fields.
- Guard stub imports with `TYPE_CHECKING` if deployment environments omit dev dependencies.
- `ApplicationId` is the Pinpoint project identifier. Many operations fail because code has the right AWS account and region but the wrong application id.
- Channel setup still matters at runtime. For example, email sends need a configured sender identity, and SMS sends need supported origination settings, even when the request is perfectly typed.

## Version-Sensitive Notes

- The version used here `1.42.3` matches the current PyPI package page on `2026-03-12`.
- The maintainer docs describe these packages as versioned to match the related `boto3` release line, so pin the exact versions you test.
- The hosted docs root is a generated latest site, not a version-pinned archive. On `2026-03-12`, it still showed the local generation example with `boto3==1.41.5`, so use PyPI as the source of truth for the exact installable wheel version.
- The boto3 Pinpoint reference page itself is rendered from a different boto3 patch line than this package. Use the AWS docs for runtime behavior and the exact stubs wheel for the typed symbol surface in your environment.
- AWS states that Amazon Pinpoint end support is `2026-10-30`, and that SMS, voice, mobile push, OTP, and phone number validate APIs are not affected because they are supported by AWS End User Messaging.

## Official Sources

- PyPI package page: `https://pypi.org/project/mypy-boto3-pinpoint/`
- Exact release page: `https://pypi.org/project/mypy-boto3-pinpoint/1.42.3/`
- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_pinpoint/`
- Maintainer versioning guide: `https://youtype.github.io/boto3_stubs_docs/#versioning`
- AWS boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- AWS boto3 configuration guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/configuration.html`
- AWS boto3 Pinpoint service reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/pinpoint.html`
- AWS Pinpoint Python code examples: `https://docs.aws.amazon.com/code-library/latest/ug/python_3_pinpoint_code_examples.html`
- AWS Amazon Pinpoint end-of-support notice: `https://docs.aws.amazon.com/pinpoint/latest/developerguide/migrate.html`
