---
name: mypy-boto3-ec2
description: "mypy-boto3-ec2 type stubs for boto3 EC2 clients, resources, paginators, waiters, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.62"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,ec2,boto3,type-stubs,mypy,pyright,python"
---

# mypy-boto3-ec2 Python Package Guide

## Golden Rule

`mypy-boto3-ec2` is a typing package for `boto3` EC2 code, not a runtime AWS SDK.

Use it in one of these modes:

- Install `boto3-stubs[ec2]` for the best editor and type-checker experience with automatic `Session.client("ec2")` and `Session.resource("ec2")` overloads.
- Install `mypy-boto3-ec2` when you only want the standalone EC2 stubs and are willing to add explicit annotations.
- Install `boto3-stubs-lite[ec2]` when IDE memory use matters more than automatic overload inference.

Runtime credentials, regions, retries, endpoints, and API behavior still come from `boto3` and botocore.

## Install

### Recommended for most projects

```bash
python -m pip install boto3 'boto3-stubs[ec2]'
```

This is the maintainer-recommended path when you want autocomplete and overload-based type inference without annotating every EC2 variable yourself.

### Standalone EC2 stubs

```bash
python -m pip install boto3 mypy-boto3-ec2
```

Use this when you want only the EC2 stubs package. In this mode, explicit annotations are usually necessary.

### Lower-memory IDE fallback

```bash
python -m pip install boto3 'boto3-stubs-lite[ec2]'
```

The lite package is more memory-friendly, especially for PyCharm, but it does not provide `session.client()` and `session.resource()` overloads. Add explicit annotations if you use it.

### Conda

```bash
conda install mypy-boto3-ec2
```

### Generate locally for an exact boto3 pin

If your project is pinned to a specific `boto3` version and you need the closest possible type alignment, the maintainer recommends local generation:

```bash
uvx --with 'boto3==1.42.62' mypy-boto3-builder
```

## Runtime Setup And AWS Auth

`mypy-boto3-ec2` has no package-specific initialization. All runtime behavior still comes from `boto3`.

AWS documents that Boto3 searches a credential chain that starts with explicit client or session parameters, then environment variables, then role or profile-based providers, then shared config files, container credentials, and finally EC2 instance metadata.

Common setup for local development:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

Or create an explicit session in code:

```python
from boto3.session import Session

session = Session(profile_name="dev", region_name="us-west-2")
```

Useful environment variables:

- `AWS_PROFILE`
- `AWS_DEFAULT_REGION`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`

Do not hardcode credentials in source code. The stub package does not change auth behavior.

## Core Usage

### Typed client

Use clients for full EC2 API coverage. AWS documents that clients map closely to service APIs and support all service operations.

```python
from boto3.session import Session
from mypy_boto3_ec2.client import EC2Client

session = Session(profile_name="dev", region_name="us-east-1")
ec2: EC2Client = session.client("ec2")

response = ec2.describe_instances(MaxResults=5)

for reservation in response.get("Reservations", []):
    for instance in reservation.get("Instances", []):
        print(instance["InstanceId"])
```

### Explicit annotations for standalone or lite installs

With standalone `mypy-boto3-ec2` or `boto3-stubs-lite[ec2]`, annotate clients and resources explicitly:

```python
from boto3.session import Session
from mypy_boto3_ec2.client import EC2Client

client: EC2Client = Session(region_name="us-east-1").client("ec2")
```

### Typed paginator

```python
from boto3.session import Session
from mypy_boto3_ec2.client import EC2Client
from mypy_boto3_ec2.paginator import DescribeInstancesPaginator

client: EC2Client = Session(region_name="us-east-1").client("ec2")
paginator: DescribeInstancesPaginator = client.get_paginator("describe_instances")

for page in paginator.paginate(PaginationConfig={"MaxItems": 25}):
    for reservation in page.get("Reservations", []):
        for instance in reservation.get("Instances", []):
            print(instance["InstanceId"])
```

### Typed waiter

```python
from boto3.session import Session
from mypy_boto3_ec2.client import EC2Client
from mypy_boto3_ec2.waiter import InstanceRunningWaiter

client: EC2Client = Session(region_name="us-east-1").client("ec2")
waiter: InstanceRunningWaiter = client.get_waiter("instance_running")

waiter.wait(InstanceIds=["i-0123456789abcdef0"])
```

### Typed service resource

AWS documents the resource API as feature-frozen, so prefer clients for new EC2 features. Use resources when you intentionally want the object-oriented interface.

```python
from boto3.session import Session
from mypy_boto3_ec2.service_resource import EC2ServiceResource, Instance

resource: EC2ServiceResource = Session(region_name="us-east-1").resource("ec2")
instance: Instance = resource.Instance("i-0123456789abcdef0")

print(instance.instance_id)
```

### Literals and TypedDicts

Use generated literal and `type_defs` helpers when wrapper code needs stricter typing than plain dictionaries:

```python
from mypy_boto3_ec2.client import EC2Client
from mypy_boto3_ec2.literals import InstanceTypeType
from mypy_boto3_ec2.type_defs import FilterTypeDef
from boto3.session import Session

client: EC2Client = Session(region_name="us-east-1").client("ec2")

instance_type: InstanceTypeType = "t3.micro"
filters: list[FilterTypeDef] = [
    {
        "Name": "instance-state-name",
        "Values": ["running"],
    }
]

client.describe_instances(Filters=filters)
print(instance_type)
```

## Tooling Patterns

### Keep stub imports out of production-only environments

If runtime images do not install stub packages, gate the imports with `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_ec2.client import EC2Client

def make_client() -> "EC2Client":
    return Session(region_name="us-east-1").client("ec2")
```

### Pylint workaround for dev-only stub installs

The maintainer docs call out a `pylint` issue with `TYPE_CHECKING` imports. Use an `object` fallback when needed:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_ec2.client import EC2Client
else:
    EC2Client = object

client: EC2Client = Session(region_name="us-east-1").client("ec2")
```

## Common Pitfalls

- Installing only `mypy-boto3-ec2` and expecting unannotated `session.client("ec2")` or `session.resource("ec2")` calls to become typed automatically. That behavior comes from `boto3-stubs[ec2]`.
- Forgetting to install `boto3`. These packages are stubs, not the runtime SDK.
- Treating the stubs package as AWS auth or config middleware. Credentials, region, retries, endpoints, and permissions still come from normal `boto3` and botocore setup.
- Defaulting to the resource API for every EC2 task. AWS says newer service features land on clients, not resources.
- Using `boto3-stubs-lite[ec2]` and expecting overload-based inference from `session.client("ec2")`. Lite mode needs more explicit annotations.
- Importing stub symbols at runtime when the package is installed only in dev dependencies. Use `TYPE_CHECKING` or keep the stubs installed anywhere those imports execute.
- Assuming generated stubs always match the EC2 surface in your environment. When exact shape coverage matters, pin the stub version with the matching `boto3` version or generate stubs locally.

## Version-Sensitive Notes

- PyPI lists `mypy-boto3-ec2 1.42.62` as the latest version on March 12, 2026, released on March 5, 2026.
- The maintainer states that `mypy-boto3-ec2` uses the same version as the related `boto3` release.
- Practical rule: when exact request and response shapes matter, pin `boto3==1.42.62` with `mypy-boto3-ec2==1.42.62`.
- The builder project version is separate. PyPI shows this package was generated with `mypy-boto3-builder 8.12.0`, but that builder version is not the package version you should pin in application dependencies.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_ec2/`
- PyPI project page: `https://pypi.org/project/mypy-boto3-ec2/`
- Builder and issue tracker: `https://github.com/youtype/mypy_boto3_builder`
- Boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- Boto3 clients guide: `https://docs.aws.amazon.com/boto3/latest/guide/clients.html`
- Boto3 resources guide: `https://docs.aws.amazon.com/boto3/latest/guide/resources.html`
