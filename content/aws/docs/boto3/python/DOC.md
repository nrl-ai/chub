---
name: boto3
description: "AWS SDK for Python (boto3) package guide with sessions, clients, credentials, retries, paginators, waiters, and concurrency rules"
metadata:
  languages: "python"
  versions: "1.42.66"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,botocore,python,sdk,cloud"
---

# boto3 Python Package Guide

## Golden Rule

- Use an explicit `boto3.Session(...)` and create service clients from that session.
- Prefer clients for new code. AWS documents the resource interface as feature-frozen.
- Do not hardcode AWS credentials in source code. Let boto3 load them from profiles, environment variables, IAM Identity Center, assume-role configuration, or runtime IAM roles.

## Version-Sensitive Notes

- This entry is pinned to the version used here `1.42.66`.
- On March 12, 2026, PyPI lists `boto3 1.42.66`. The AWS `latest` docs tree is rolling and still shows mixed patch numbers across pages, so use PyPI for exact package pinning and the AWS guide pages for behavior and examples.
- `boto3` on PyPI requires Python `>=3.9`.
- AWS documents three retry modes: `legacy`, `standard`, and `adaptive`. For new code, prefer `standard` unless you intentionally want adaptive client-side rate limiting.

## Install

Pin the package version when you need reproducible behavior:

```bash
python -m pip install "boto3==1.42.66"
```

If you need the optional CRT-backed integrations:

```bash
python -m pip install "boto3[crt]==1.42.66"
```

## Recommended Setup

Start with an explicit session, set the region deliberately, and verify the caller identity before making service-specific calls.

```python
import boto3
from botocore.config import Config
from botocore.exceptions import ClientError

session = boto3.Session(profile_name="dev", region_name="us-west-2")

config = Config(
    retries={
        "mode": "standard",
        "max_attempts": 10,
    }
)

sts = session.client("sts", config=config)

try:
    identity = sts.get_caller_identity()
    print(identity["Account"])
    print(identity["Arn"])
except ClientError as err:
    print(err.response["Error"]["Code"])
    raise
```

## Credentials And Region Resolution

The AWS credentials guide documents a longer provider chain than most agents remember. In practice, the important sources are:

1. Explicit credentials passed to `boto3.client(...)`
2. Explicit credentials passed to `boto3.Session(...)`
3. Environment variables
4. Assume-role configuration
5. Assume-role-with-web-identity configuration
6. IAM Identity Center credentials
7. Shared credentials file
8. AWS config file
9. Container credentials
10. EC2 instance metadata

For region selection, boto3 can use:

- `region_name=` on `Session(...)` or `client(...)`
- `AWS_DEFAULT_REGION`
- the selected profile in `~/.aws/config`
- `Config(region_name=...)`

Useful environment variables:

- `AWS_PROFILE`
- `AWS_DEFAULT_REGION`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`
- `AWS_MAX_ATTEMPTS`
- `AWS_RETRY_MODE`

For local development, a named profile is usually the cleanest default:

```python
import boto3

session = boto3.Session(profile_name="dev", region_name="us-east-1")
s3 = session.client("s3")
```

For cross-account access, assume a role and then build a new session from the temporary credentials:

```python
import boto3

base_session = boto3.Session(profile_name="admin", region_name="us-east-1")
sts = base_session.client("sts")

assumed = sts.assume_role(
    RoleArn="arn:aws:iam::123456789012:role/app-reader",
    RoleSessionName="context-hub-example",
)

creds = assumed["Credentials"]

session = boto3.Session(
    aws_access_key_id=creds["AccessKeyId"],
    aws_secret_access_key=creds["SecretAccessKey"],
    aws_session_token=creds["SessionToken"],
    region_name="us-east-1",
)
```

## Clients Vs Resources

### Default Choice: Clients

Clients are the low-level interface and track AWS service APIs most closely.

```python
import boto3

session = boto3.Session(region_name="us-east-1")
s3 = session.client("s3")
ddb = session.client("dynamodb")

print(s3.list_buckets())
print(ddb.list_tables(Limit=10))
```

Use clients when you need:

- the newest service features
- generated request and response dictionaries
- paginators and waiters
- direct parity with AWS API reference examples

### Use Resources Only For Existing Convenience Workflows

AWS states it does not intend to add new features to the resource interface. Keep using resources only when you already depend on their object model.

```python
import boto3

session = boto3.Session(region_name="us-east-1")
s3 = session.resource("s3")

for bucket in s3.buckets.all():
    print(bucket.name)
```

## Core Usage Patterns

### Call Operations With Keyword Arguments

Client methods are generated from the service model. Use keyword arguments, not positional arguments.

```python
import boto3

ec2 = boto3.Session(region_name="us-west-2").client("ec2")

response = ec2.describe_instances(MaxResults=5)
reservations = response.get("Reservations", [])
```

### Use Paginators For List APIs

Many list operations return partial results. Prefer a paginator whenever an operation may span multiple pages.

```python
import boto3

s3 = boto3.Session(region_name="us-east-1").client("s3")
paginator = s3.get_paginator("list_objects_v2")

for page in paginator.paginate(Bucket="my-bucket", Prefix="logs/"):
    for item in page.get("Contents", []):
        print(item["Key"])
```

### Use Waiters Instead Of Hand-Written Sleep Loops

Waiters are only available for operations where AWS publishes waiter models, so check per service.

```python
import boto3

s3 = boto3.Session(region_name="us-east-1").client("s3")
s3.create_bucket(Bucket="example-bucket-123")

waiter = s3.get_waiter("bucket_exists")
waiter.wait(Bucket="example-bucket-123")
```

### Catch `ClientError` And Inspect The AWS Error Code

```python
import boto3
from botocore.exceptions import ClientError

s3 = boto3.Session(region_name="us-east-1").client("s3")

try:
    s3.head_bucket(Bucket="does-not-exist")
except ClientError as err:
    code = err.response["Error"]["Code"]
    if code in {"404", "NoSuchBucket"}:
        print("Bucket is missing")
    else:
        raise
```

## Retry And Client Configuration

Use `botocore.config.Config` when you need to override retry behavior or client settings such as timeouts, proxies, or signature behavior.

```python
import boto3
from botocore.config import Config

config = Config(
    region_name="us-west-2",
    retries={
        "mode": "standard",
        "max_attempts": 10,
    },
)

dynamodb = boto3.Session().client("dynamodb", config=config)
```

Notes that matter in practice:

- `standard` is the safest default for most applications.
- `adaptive` adds client-side rate limiting on top of standard retry behavior.
- Retry settings can also come from `AWS_RETRY_MODE` and `AWS_MAX_ATTEMPTS`.
- AWS documents a subtle difference in how `max_attempts` is interpreted depending on where you configure it. In a `Config` object it counts retries only, while `AWS_MAX_ATTEMPTS` and config-file settings count the initial request too. Use `total_max_attempts` when you want consistent total-request semantics from code.

## Concurrency Rules

The boto3 guides call out an important split:

- Low-level clients are generally thread-safe.
- Clients are not safe to share across processes.
- `Session` objects are not thread-safe.
- Resource objects are not thread-safe and should not be shared across threads or processes.

For threaded code, create one session per thread and then build clients or resources from that session.

## Common Pitfalls

- `boto3.client(...)` and `boto3.resource(...)` use a shared default session if you do not create one explicitly. That is convenient for scripts but a poor default for libraries, tests, and concurrent code.
- Credentials and region are separate concerns. Having valid credentials without a region still produces confusing failures for regional services.
- List-style APIs often paginate even when a small test account returns everything in one call.
- Not every service has waiters, and waiter names differ from operation names.
- The legacy `boto3.amazonaws.com` docs hostname redirects to the canonical `docs.aws.amazon.com` root. Store the canonical root in tooling.
- The `latest` AWS docs tree is rolling. Verify exact package pinning against PyPI before writing version-sensitive automation.

## Official Source URLs

- AWS docs root: `https://docs.aws.amazon.com/boto3/latest/`
- AWS quickstart: `https://docs.aws.amazon.com/boto3/latest/guide/quickstart.html`
- AWS credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- AWS configuration guide: `https://docs.aws.amazon.com/boto3/latest/guide/configuration.html`
- AWS clients guide: `https://docs.aws.amazon.com/boto3/latest/guide/clients.html`
- AWS resources guide: `https://docs.aws.amazon.com/boto3/latest/guide/resources.html`
- AWS paginators guide: `https://docs.aws.amazon.com/boto3/latest/guide/paginators.html`
- AWS retries guide: `https://docs.aws.amazon.com/boto3/latest/guide/retries.html`
- AWS error handling guide: `https://docs.aws.amazon.com/boto3/latest/guide/error-handling.html`
- AWS session guide: `https://docs.aws.amazon.com/boto3/latest/guide/session.html`
- PyPI package page: `https://pypi.org/project/boto3/`
