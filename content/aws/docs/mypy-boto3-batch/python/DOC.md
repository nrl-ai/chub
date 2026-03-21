---
name: mypy-boto3-batch
description: "Type stubs for boto3 Batch in Python, covering typed clients, paginators, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.59"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,batch,boto3,type-stubs,mypy,pyright,python"
---

# mypy-boto3-batch Python Package Guide

## Golden Rule

`mypy-boto3-batch` is a typing package for AWS Batch code that uses `boto3`. It does not make AWS calls by itself, it does not manage credentials, and it does not replace the runtime SDK.

Use it in one of these modes:

- Install `boto3-stubs[batch]` for the smoothest editor and type-checker experience with typed `Session.client("batch")` overloads.
- Install `mypy-boto3-batch` when you only want the Batch service stubs and are willing to annotate clients explicitly.
- Install `boto3-stubs-lite[batch]` when IDE memory use matters more than automatic overload inference.

Keep runtime setup in normal `boto3` or botocore configuration.

## Install

### Recommended for most projects

```bash
python -m pip install "boto3-stubs[batch]==1.42.59"
```

This is the maintainer-recommended path when you want typed `boto3.client("batch")` and `Session().client("batch")` without annotating every variable yourself.

### Standalone Batch stubs

```bash
python -m pip install "boto3==1.42.59" "mypy-boto3-batch==1.42.59"
```

Use this when you want only the Batch stubs package. In this mode, explicit annotations are the safest default.

### Lower-memory IDE fallback

```bash
python -m pip install "boto3-stubs-lite[batch]==1.42.59"
```

The lite package is more memory-friendly, but the maintainer docs note that it does not provide `Session.client()` overloads. Add explicit annotations or casts if you use it.

### Generate stubs locally for an exact boto3 pin

If your project is pinned to a boto3 version newer than the standalone wheel, generate local stubs against that runtime version:

```bash
uvx --with "boto3==1.42.59" mypy-boto3-builder
```

## Setup And AWS Authentication

`mypy-boto3-batch` has no package-specific initialization. All runtime behavior still comes from `boto3`.

Boto3 uses the standard AWS credential provider chain, including:

1. Explicit credentials passed to a session or client
2. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, and `AWS_DEFAULT_REGION`
3. Shared config and credentials files under `~/.aws/`
4. Role-based providers such as IAM Identity Center, assume-role, container credentials, or EC2 instance metadata

Typical session setup:

```python
from boto3.session import Session

session = Session(
    profile_name="dev",
    region_name="us-east-1",
)
```

If the boto3 session is misconfigured, the stubs will still type-check while the AWS call fails at runtime.

## Core Usage

### Typed Batch client

Use the runtime client from `boto3`, then annotate it with the generated Batch type:

```python
from boto3.session import Session
from mypy_boto3_batch import BatchClient

session = Session(profile_name="dev", region_name="us-east-1")
batch: BatchClient = session.client("batch")

response = batch.describe_job_queues()
for queue in response.get("jobQueues", []):
    print(queue["jobQueueName"], queue["state"])
```

### Explicit annotations for standalone or lite installs

With standalone `mypy-boto3-batch` or `boto3-stubs-lite[batch]`, annotate the client directly instead of relying on inference:

```python
from boto3.session import Session
from mypy_boto3_batch import BatchClient

client: BatchClient = Session(region_name="us-east-1").client("batch")
```

### Typed paginator

The generated docs expose paginator classes for Batch operations such as `describe_job_queues`, `list_jobs`, `list_jobs_by_consumable_resource`, `list_scheduling_policies`, and `list_service_jobs`.

```python
from boto3.session import Session
from mypy_boto3_batch import BatchClient
from mypy_boto3_batch.paginator import ListJobsPaginator

client: BatchClient = Session(region_name="us-east-1").client("batch")
paginator: ListJobsPaginator = client.get_paginator("list_jobs")

for page in paginator.paginate(jobQueue="high-priority", jobStatus="RUNNING"):
    for job in page.get("jobSummaryList", []):
        print(job["jobId"], job["status"])
```

### Submit a job with a typed client

```python
from mypy_boto3_batch import BatchClient

def submit_etl_job(client: BatchClient) -> str:
    response = client.submit_job(
        jobName="daily-etl",
        jobQueue="high-priority",
        jobDefinition="etl-job:1",
        containerOverrides={
            "command": ["python", "main.py"],
            "environment": [
                {"name": "RUN_MODE", "value": "daily"},
            ],
        },
    )
    return response["jobId"]
```

Use the generated `type_defs` module when wrapper code needs stronger typing for nested request and response shapes, and use `literals` when you want enum-like values checked before runtime.

### Keep stub imports out of production-only environments

If the stubs are installed only in development or CI, keep the imports behind `TYPE_CHECKING` and cast the runtime client:

```python
from typing import TYPE_CHECKING, cast

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_batch import BatchClient

client = cast("BatchClient", Session(region_name="us-east-1").client("batch"))
```

This keeps runtime imports clean while preserving editor and mypy support.

## Configuration Notes

- `mypy-boto3-batch` adds typing for the Batch client and paginator surface only. The generated docs page for this service does not expose a Batch service resource module, so client-based code is the default.
- Keep `boto3`, `botocore`, and the stub package aligned when exact request and response shapes matter.
- Region selection still comes from the boto3 session or client. Set `region_name` explicitly in scripts and tests when the environment is ambiguous.
- Use `botocore.config.Config` for retries, timeouts, proxies, and endpoint tuning. The stubs package only helps type-check those calls.

## Common Pitfalls

- Installing only `mypy-boto3-batch` and expecting it to replace `boto3`. It is a stubs package, not the runtime AWS SDK.
- Mixing up the names: install `mypy-boto3-batch`, import `mypy_boto3_batch`, and create the runtime client with service name `"batch"`.
- Expecting `Session.client("batch")` to infer a typed return value when you installed only `mypy-boto3-batch` or `boto3-stubs-lite[batch]`. Add explicit annotations in those modes.
- Importing stub symbols at runtime when the package is only a dev dependency. Use `TYPE_CHECKING` or keep the stubs installed anywhere those imports execute.
- Copying examples from the moving generated docs root without verifying the installable wheel version on PyPI. The docs site can be ahead of the latest standalone package release.
- Treating typing as permission validation. Good type hints do not prevent runtime failures from missing IAM permissions, wrong regions, or incorrect Batch resource names.

## Version-Sensitive Notes

- The version used here `1.42.59` matches the current PyPI project page for `mypy-boto3-batch`.
- PyPI lists `mypy-boto3-batch 1.42.59` as the latest installable release on March 12, 2026, published on February 27, 2026.
- The generated maintainer docs root is ahead of the standalone wheel and currently shows local generation with `boto3==1.42.66`.
- The maintainer documentation states that this package uses the same version as the related `boto3` package.
- Practical rule: if exact type alignment matters, pin `boto3==1.42.59` with `mypy-boto3-batch==1.42.59`, or generate stubs locally against your exact boto3 pin.

## Official Sources

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_batch/`
- Maintainer versioning guide: `https://youtype.github.io/boto3_stubs_docs/#versioning`
- PyPI package page: `https://pypi.org/project/mypy-boto3-batch/`
- Maintainer source repository: `https://github.com/youtype/mypy_boto3_builder`
- Boto3 Batch reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/batch.html`
- Boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
