---
name: botocore
description: "botocore Python package guide for low-level AWS clients, shared credential/config loading, retries, paginators, and stubbing"
metadata:
  languages: "python"
  versions: "1.42.66"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,botocore,python,sdk,credentials,retries,paginators,testing"
---

# botocore Python Package Guide

## What This Package Is For

`botocore` is the low-level AWS SDK foundation used by `boto3` and the AWS CLI.

Use it when your code needs to:

- create low-level AWS service clients directly with `botocore.session.Session`
- control request behavior with `botocore.config.Config`
- stub AWS responses in tests with `botocore.stub.Stubber`
- work close to generated AWS service models instead of the higher-level `boto3` resource layer

If you want higher-level service resources for normal application code, prefer `boto3`. If you need the common AWS credential chain, retry configuration, endpoint behavior, or service model-driven clients, `botocore` is the actual implementation layer.

## Version-Sensitive Notes

- This entry keeps `1.42.66` in frontmatter because that was the version used here provided for review.
- Public upstream sources checked on 2026-03-12 did not verify a public `1.42.66` release. The PyPI project page currently shows `1.42.63`.
- The AWS `latest` botocore docs are a rolling docs tree, not a strict patch snapshot. Different pages in the same tree currently show different `1.42.x` patch labels.
- Treat the AWS docs as the canonical behavior reference and PyPI as the package-version source of truth before pinning an exact release.
- If your project also installs `boto3`, keep `boto3` and `botocore` on compatible release lines instead of upgrading one independently.

## Install

Install from PyPI:

```bash
python -m pip install botocore
```

If you need the last public version verified during this session:

```bash
python -m pip install "botocore==1.42.63"
```

If your internal mirror or lockfile already uses the version used here, verify that `1.42.66` is actually available before copying that exact pin.

## Initialize A reviewnd Client

The core botocore workflow is:

1. create a session
2. create a low-level client for one AWS service
3. call service operations with plain Python dictionaries

```python
import botocore.session
from botocore.exceptions import ClientError

session = botocore.session.get_session()
sts = session.create_client("sts", region_name="us-west-2")

try:
    identity = sts.get_caller_identity()
    print(identity["Account"])
    print(identity["Arn"])
except ClientError as err:
    print(err.response["Error"]["Code"])
    raise
```

Useful discovery helpers:

```python
import botocore.session

session = botocore.session.get_session()

print("s3" in session.get_available_services())
print(session.get_available_regions("s3")[:5])
```

## Credentials And Region Configuration

`botocore` uses the standard AWS SDK credential and settings chain. In practice, the most useful inputs are:

- `AWS_PROFILE`
- `AWS_DEFAULT_REGION`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`
- `AWS_SHARED_CREDENTIALS_FILE`
- `AWS_CONFIG_FILE`
- `AWS_MAX_ATTEMPTS`
- `AWS_RETRY_MODE`
- `AWS_EC2_METADATA_DISABLED`

Local profile-based setup is the safest default for development:

```bash
aws configure --profile dev
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

Then create clients without hardcoding credentials:

```python
import botocore.session

session = botocore.session.get_session()
s3 = session.create_client("s3")
```

If you must override settings for a single client, pass them explicitly:

```python
import botocore.session

session = botocore.session.get_session()
client = session.create_client(
    "sqs",
    region_name="us-east-1",
    aws_access_key_id="...",
    aws_secret_access_key="...",
    aws_session_token="...",
)
```

Prefer explicit credentials only for short-lived scripts, local emulators, or test fixtures.

## Use `Config` For Retries, Timeouts, Proxies, And Pools

`botocore.config.Config` is the main runtime override surface for per-client behavior.

Typical settings worth controlling:

- `region_name`
- `connect_timeout`
- `read_timeout`
- `retries`
- `max_pool_connections`
- `proxies`
- `signature_version`

```python
import botocore.session
from botocore.config import Config

session = botocore.session.get_session()

config = Config(
    region_name="us-west-2",
    connect_timeout=5,
    read_timeout=30,
    max_pool_connections=20,
    retries={
        "mode": "standard",
        "total_max_attempts": 10,
    },
)

client = session.create_client("dynamodb", config=config)
```

Practical guidance:

- prefer `total_max_attempts` in `Config(...)` because it consistently counts the initial request
- prefer `mode: "standard"` for most code unless you have a specific reason to use `adaptive`
- lower timeouts for latency-sensitive code instead of accepting the default 60-second connect/read timeouts
- raise `max_pool_connections` if many threads share clients from the same session

For local emulators such as LocalStack, override the endpoint:

```python
import botocore.session
from botocore.config import Config

session = botocore.session.get_session()
s3 = session.create_client(
    "s3",
    region_name="us-east-1",
    endpoint_url="http://localhost:4566",
    aws_access_key_id="test",
    aws_secret_access_key="test",
    config=Config(signature_version="s3v4"),
)
```

## Core Usage Pattern

Operations are generated from AWS service models. Method names are snake_case, and requests and responses use plain dict/list structures.

```python
import botocore.session
from botocore.exceptions import ClientError

session = botocore.session.get_session()
ec2 = session.create_client("ec2", region_name="us-west-2")

try:
    response = ec2.describe_instances(
        Filters=[
            {
                "Name": "instance-state-name",
                "Values": ["running"],
            }
        ]
    )
except ClientError as err:
    code = err.response["Error"]["Code"]
    if code == "UnauthorizedOperation":
        raise PermissionError("Missing EC2 permissions") from err
    raise

for reservation in response["Reservations"]:
    for instance in reservation["Instances"]:
        print(instance["InstanceId"])
```

Inspect service capabilities dynamically when needed:

```python
import botocore.session

session = botocore.session.get_session()
client = session.create_client("lambda", region_name="us-west-2")

print(client.meta.service_model.service_name)
print(client.meta.service_model.operation_names[:10])
```

## Paginators

Many list operations return partial results. Use paginators instead of manually looping on service-specific continuation tokens when the operation supports them.

```python
import botocore.session

session = botocore.session.get_session()
iam = session.create_client("iam")

if iam.can_paginate("list_users"):
    paginator = iam.get_paginator("list_users")
    for page in paginator.paginate(
        PaginationConfig={
            "MaxItems": 200,
            "PageSize": 50,
        }
    ):
        for user in page["Users"]:
            print(user["UserName"])
```

Notes:

- use `client.can_paginate("operation_name")` before assuming a paginator exists
- `PaginationConfig` commonly accepts `MaxItems`, `PageSize`, and `StartingToken`
- paginator page shapes are still raw response dictionaries from the underlying service

## Testing With `Stubber`

Use `botocore.stub.Stubber` for unit tests that should not make network calls.

```python
import botocore.session
from botocore.stub import Stubber

session = botocore.session.get_session()
sts = session.create_client("sts", region_name="us-east-1")

with Stubber(sts) as stubber:
    stubber.add_response(
        "get_caller_identity",
        {
            "UserId": "AIDATEST",
            "Account": "123456789012",
            "Arn": "arn:aws:iam::123456789012:user/test",
        },
        {},
    )

    response = sts.get_caller_identity()
    assert response["Account"] == "123456789012"
```

To test failure paths:

```python
import botocore.session
from botocore.exceptions import ClientError
from botocore.stub import Stubber

session = botocore.session.get_session()
s3 = session.create_client("s3", region_name="us-east-1")

with Stubber(s3) as stubber:
    stubber.add_client_error(
        "head_bucket",
        service_error_code="404",
        service_message="Not Found",
        http_status_code=404,
    )

    try:
        s3.head_bucket(Bucket="missing-bucket")
    except ClientError as err:
        assert err.response["ResponseMetadata"]["HTTPStatusCode"] == 404
```

`Stubber` validates both the called operation name and the expected parameters, which makes it much better than patching random methods with loose mocks.

## Error Handling

Most AWS service failures surface as `botocore.exceptions.ClientError`.

```python
from botocore.exceptions import ClientError, NoCredentialsError

try:
    response = client.some_operation(...)
except NoCredentialsError:
    raise RuntimeError("AWS credentials are not configured")
except ClientError as err:
    code = err.response["Error"]["Code"]
    message = err.response["Error"]["Message"]
    raise RuntimeError(f"AWS error {code}: {message}") from err
```

Practical rules:

- branch on `err.response["Error"]["Code"]`, not on message text
- handle missing credentials separately from service-side errors
- keep retry logic aligned with idempotency and the configured retry mode
- do not suppress `ClientError` broadly without logging the service error code

## Common Pitfalls

- Do not treat `botocore` like `boto3` resources. It gives you low-level clients and raw response dicts.
- Do not hardcode long-lived credentials in source files. Use the shared AWS credential chain.
- Do not assume every list operation has a paginator. Check `can_paginate(...)`.
- Do not assume the AWS `latest` docs tree is patch-pinned to your installed wheel.
- Do not upgrade `botocore` independently from `boto3` without checking compatibility.
- Use `Config(...)` for retry, timeout, proxy, and pool overrides instead of scattering ad hoc environment tweaks across scripts.
- In tests, use `Stubber` instead of live AWS calls or overly loose mocks.

## Official Sources Used For This Entry

- `https://docs.aws.amazon.com/botocore/latest/`
- `https://docs.aws.amazon.com/botocore/latest/tutorial/`
- `https://docs.aws.amazon.com/botocore/latest/reference/config.html`
- `https://docs.aws.amazon.com/botocore/latest/reference/stubber.html`
- `https://docs.aws.amazon.com/botocore/latest/topics/paginators.html`
- `https://docs.aws.amazon.com/sdkref/latest/guide/settings-reference.html`
- `https://docs.aws.amazon.com/boto3/latest/guide/configuration.html`
- `https://docs.aws.amazon.com/boto3/latest/guide/retries.html`
- `https://pypi.org/project/botocore/`
