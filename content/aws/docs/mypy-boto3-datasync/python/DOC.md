---
name: mypy-boto3-datasync
description: "mypy-boto3-datasync package guide for typed boto3 AWS DataSync clients, paginators, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.9"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,datasync,boto3,python,typing,stubs,mypy,pyright"
---

# mypy-boto3-datasync Python Package Guide

## Golden Rule

- Keep `boto3` as the runtime AWS SDK. `mypy-boto3-datasync` is a typing-only companion for DataSync code.
- Prefer `boto3-stubs[datasync]` when you want `Session.client("datasync")` inference with less annotation work.
- Use standalone `mypy-boto3-datasync` when you only want DataSync stubs and are willing to annotate the client explicitly.
- Treat PyPI as the source of truth for the exact published stub version. The maintainer docs site is a rolling latest view.

## Install

Service-specific stubs only:

```bash
python -m pip install "boto3==1.42.9" "mypy-boto3-datasync==1.42.9"
```

Bundled boto3 stubs with automatic `Session.client("datasync")` overloads:

```bash
python -m pip install "boto3-stubs[datasync]"
```

Lower-memory bundled variant:

```bash
python -m pip install "boto3-stubs-lite[datasync]"
```

Notes:

- `boto3-stubs-lite[datasync]` keeps the generated DataSync types, but the maintainer docs say it omits `session.client(...)` and `session.resource(...)` overloads.
- Install the stubs in the same environment that your editor, `mypy`, or `pyright` analyzes.
- If you need exact parity with a custom `boto3` build, the maintainer project also supports local generation with `mypy-boto3-builder`.

## Initialize And Authenticate

`mypy-boto3-datasync` does not add its own auth, retry, endpoint, or region layer. All runtime behavior still comes from `boto3` and `botocore`.

Common credential sources, following boto3's documented search order:

1. explicit credentials passed to `client(...)` or `Session(...)`
2. environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN`
3. shared config and credentials files, including `AWS_PROFILE`
4. container, instance, or other AWS runtime credentials

Typical local setup:

```bash
export AWS_PROFILE=storage-migrations
export AWS_DEFAULT_REGION=us-west-2
```

Typed client initialization:

```python
from boto3.session import Session
from mypy_boto3_datasync import DataSyncClient

session = Session(profile_name="storage-migrations", region_name="us-west-2")
datasync: DataSyncClient = session.client("datasync")
```

If your production image does not install stub packages, keep the import type-only:

```python
from typing import TYPE_CHECKING, cast

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_datasync import DataSyncClient

session = Session(region_name="us-west-2")
datasync = cast("DataSyncClient", session.client("datasync"))
```

If you create sessions manually, follow boto3's session guidance and create a separate `Session` per thread or process instead of sharing one across concurrent workers.

## Core Usage

### Typed DataSync client

Use the generated client type for normal DataSync API calls:

```python
from boto3.session import Session
from mypy_boto3_datasync import DataSyncClient

datasync: DataSyncClient = Session(region_name="us-west-2").client("datasync")

response = datasync.list_locations(MaxResults=25)

for location in response.get("Locations", []):
    print(location["LocationArn"], location.get("LocationUri"))
```

### Typed paginators

The published package docs list paginator types such as `ListTasksPaginator` and `ListTaskExecutionsPaginator`:

```python
from boto3.session import Session
from mypy_boto3_datasync import DataSyncClient
from mypy_boto3_datasync.paginator import ListTasksPaginator

datasync: DataSyncClient = Session(region_name="us-west-2").client("datasync")
paginator: ListTasksPaginator = datasync.get_paginator("list_tasks")

for page in paginator.paginate():
    for task in page.get("Tasks", []):
        print(task["TaskArn"], task.get("Name"))
```

### Start and inspect a task execution

Runtime call shapes still come from the boto3 DataSync client docs, but the stubs keep request and response shapes typed:

```python
from boto3.session import Session
from mypy_boto3_datasync import DataSyncClient

datasync: DataSyncClient = Session(region_name="us-west-2").client("datasync")

start = datasync.start_task_execution(
    TaskArn="arn:aws:datasync:us-west-2:123456789012:task/task-0123456789abcdef0",
)

task_execution_arn = start["TaskExecutionArn"]

details = datasync.describe_task_execution(TaskExecutionArn=task_execution_arn)
print(details["Status"], details.get("Result"))
```

### Typed literals and request shapes

The generated package docs also expose `literals` and `type_defs`. Use them when helper functions assemble DataSync request dictionaries before the real boto3 call:

```python
from mypy_boto3_datasync.type_defs import PlatformTypeDef

platform: PlatformTypeDef = {
    "Version": "1",
    "Platform": "linux",
}
```

For DataSync-heavy codebases, typed request and response shapes are most useful at module boundaries where raw AWS dictionaries would otherwise degrade to `dict[str, Any]`.

## Type Checking Notes

- Import from `mypy_boto3_datasync`, not `mypy-boto3-datasync`. The PyPI distribution name uses hyphens; the Python module uses underscores.
- Standalone `mypy-boto3-datasync` is the narrow dependency choice, but explicit `DataSyncClient` annotations are more important there.
- `boto3-stubs[datasync]` is the easiest path if you want overload-based inference from `Session.client("datasync")`.
- `boto3-stubs-lite[datasync]` is the memory-friendlier option, but plan on explicit annotations.

## Common Pitfalls

- Do not install only `mypy-boto3-datasync` and expect AWS calls to work. You still need `boto3`.
- Do not pass the package name to boto3. The service name is `"datasync"`, not `"mypy-boto3-datasync"`.
- Do not assume the maintainer docs root is release-pinned. It is a latest-style page and can move ahead of the exact wheel you install.
- Do not treat the stubs as a separate AWS config layer. Credentials, retries, endpoints, IAM permissions, and throttling behavior are still normal boto3 concerns.
- Do not assume all stub install modes infer `Session.client("datasync")` the same way. The full bundled stubs have overloads; lite and standalone installs need more explicit typing.
- Do not share one boto3 `Session` across threads or processes.

## Version-Sensitive Notes For `1.42.9`

- The version used here for this session was `1.42.9`, and the official PyPI project page also reported `1.42.9` on March 12, 2026.
- The maintainer docs for this package describe install paths for `mypy-boto3-datasync`, `boto3-stubs[datasync]`, and `boto3-stubs-lite[datasync]`, plus the typed `DataSyncClient`, paginator classes, literals, and `type_defs`.
- The maintainer project states that stub package versions track the related `boto3` release line. Keep `boto3`, `botocore`, and the stub package close together when exact method and shape parity matters.
- The DataSync boto3 reference is a rolling latest reference, so examples copied from AWS docs may reflect a newer runtime patch line than your pinned stub package. Pin first, then type-check against the installed versions in your environment.

## Official Sources

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_datasync/`
- Maintainer versioning guide: `https://youtype.github.io/boto3_stubs_docs/#versioning`
- PyPI project page: `https://pypi.org/project/mypy-boto3-datasync/`
- Runtime boto3 DataSync reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/datasync.html`
- boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- boto3 session guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/session.html`
- DataSync `start_task_execution`: `https://docs.aws.amazon.com/boto3/latest/reference/services/datasync/client/start_task_execution.html`
- DataSync `describe_task_execution`: `https://docs.aws.amazon.com/boto3/latest/reference/services/datasync/client/describe_task_execution.html`
