---
name: mypy-boto3-sagemaker
description: "mypy-boto3-sagemaker package guide for typed boto3 SageMaker clients, paginators, waiters, and static analysis"
metadata:
  languages: "python"
  versions: "1.42.66"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "python,pypi,aws,sagemaker,boto3,mypy,typing,stubs"
---

# mypy-boto3-sagemaker Python Package Guide

## What It Is

`mypy-boto3-sagemaker` is the generated type-stub package for the SageMaker part of `boto3`. It gives you typed client methods, paginators, waiters, literals, and request/response definitions for static analysis and editor completion.

It is not a runtime AWS SDK. Your code still runs through normal `boto3` sessions and clients.

## Golden Rules

- Install `boto3` for runtime behavior and `mypy-boto3-sagemaker` for typing.
- Annotate the client explicitly when you want reliable `mypy` or `pyright` inference.
- Keep the stub version close to the installed `boto3` version.
- Configure credentials, region, retries, and endpoints through `boto3` and botocore, not through the stub package.
- Treat the package as development-time support. It improves correctness before runtime, but it does not fix IAM, region, or network problems.

## Install

Pin the runtime SDK and the matching stubs together when you want reproducible typing:

```bash
python -m pip install "boto3==1.42.66" "mypy-boto3-sagemaker==1.42.66"
```

The upstream docs also treat this standalone package as equivalent to the service extra on `boto3-stubs`:

```bash
python -m pip install "boto3-stubs[sagemaker]==1.42.66"
```

If you want the `boto3` runtime installed through the umbrella package too, use:

```bash
python -m pip install "boto3-stubs[boto3,sagemaker]==1.42.66"
```

## Initialize A Typed SageMaker Client

```python
from boto3.session import Session
from mypy_boto3_sagemaker.client import SageMakerClient

session = Session(profile_name="dev", region_name="us-west-2")
sm: SageMakerClient = session.client("sagemaker")
```

For botocore retry settings, proxies, or timeouts, keep using `Config`:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_sagemaker.client import SageMakerClient

config = Config(retries={"max_attempts": 10, "mode": "standard"})
session = Session(region_name="us-west-2")
sm: SageMakerClient = session.client("sagemaker", config=config)
```

## Core Usage

### Typed Client Calls

```python
from mypy_boto3_sagemaker.client import SageMakerClient

def training_job_status(sm: SageMakerClient, job_name: str) -> str:
    response = sm.describe_training_job(TrainingJobName=job_name)
    return response["TrainingJobStatus"]
```

### Typed Paginators

Use the generated paginator classes when you want explicit annotations:

```python
from mypy_boto3_sagemaker.client import SageMakerClient
from mypy_boto3_sagemaker.paginator import ListTrainingJobsPaginator

def recent_training_jobs(sm: SageMakerClient) -> list[str]:
    paginator: ListTrainingJobsPaginator = sm.get_paginator("list_training_jobs")
    names: list[str] = []

    for page in paginator.paginate(SortBy="CreationTime", SortOrder="Descending"):
        names.extend(item["TrainingJobName"] for item in page["TrainingJobSummaries"])

    return names
```

### Typed Waiters

The generated waiters let type checkers understand the waiter name and request shape:

```python
from mypy_boto3_sagemaker.client import SageMakerClient
from mypy_boto3_sagemaker.waiter import TrainingJobCompletedOrStoppedWaiter

def wait_for_training_job(sm: SageMakerClient, job_name: str) -> None:
    waiter: TrainingJobCompletedOrStoppedWaiter = sm.get_waiter(
        "training_job_completed_or_stopped"
    )
    waiter.wait(TrainingJobName=job_name)
```

### TYPE_CHECKING For Dev-Only Stub Installs

If the stubs are installed only in development or CI, keep the type imports behind `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_sagemaker.client import SageMakerClient

session = Session(profile_name="dev", region_name="us-west-2")
sm: "SageMakerClient" = session.client("sagemaker")
```

This pattern is useful if production images ship `boto3` without the stub package.

## AWS Config And Authentication

`mypy-boto3-sagemaker` does not change how AWS authentication works. Use standard `boto3` configuration:

- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`
- `AWS_PROFILE`
- `AWS_DEFAULT_REGION`
- `~/.aws/credentials`
- `~/.aws/config`
- IAM roles or task roles when running on AWS

Typical local setup:

```python
from boto3.session import Session
from mypy_boto3_sagemaker.client import SageMakerClient

session = Session(profile_name="ml-dev", region_name="us-west-2")
sm: SageMakerClient = session.client("sagemaker")

print(sm.list_domains()["Domains"])
```

If the client is typed correctly but calls still fail, debug the runtime AWS setup first: credentials, region, permissions, VPC/network access, and endpoint configuration.

## Common Pitfalls

- Installing only `mypy-boto3-sagemaker` and expecting SageMaker calls to work without `boto3`.
- Letting `boto3` and the stubs drift too far apart, which can make typed method signatures or shape definitions stale.
- Using `session.client("sagemaker")` without an explicit annotation and then assuming the editor will infer the precise client type.
- Treating successful type checking as proof that the request is valid for the target account or region.
- Importing stub modules at runtime in environments where the stubs are installed only as development dependencies.
- Assuming older blog posts match current generated waiter names or paginator names. Re-check against the current docs when copying examples.

## Version-Sensitive Notes

- This entry is pinned to the version used here `1.42.66`.
- The official PyPI release page for `mypy-boto3-sagemaker 1.42.66` shows a release date of `2026-03-11`.
- The official `boto3-stubs` package page also lists `1.42.66` as current and describes the standalone service packages as equivalent to service extras on `boto3-stubs`.
- The generated docs are tied closely to `boto3` service definitions. If your project pins an older `boto3`, pin the matching `mypy-boto3-sagemaker` version too.
- The upstream docs also note that `boto3-stubs-lite` does not provide the same `session.client(...)` and `session.resource(...)` overloads, so explicit annotations matter even more if you use the lite variant elsewhere in the family.

## Official Sources

- PyPI release page: https://pypi.org/project/mypy-boto3-sagemaker/1.42.66/
- Package docs root: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_sagemaker/
- SageMaker client reference: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_sagemaker/client/
- SageMaker examples: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_sagemaker/client/#usage-examples
- boto3-stubs package page: https://pypi.org/project/boto3-stubs/1.42.66/
- Boto3 credentials guide: https://docs.aws.amazon.com/boto3/latest/guide/credentials.html
- Boto3 configuration guide: https://docs.aws.amazon.com/boto3/latest/guide/configuration.html
