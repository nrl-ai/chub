---
name: mypy-boto3-logs
description: "mypy-boto3-logs package guide for typed boto3 CloudWatch Logs clients, paginators, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.60"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,cloudwatch-logs,boto3,python,typing,stubs,mypy,pyright"
---

# mypy-boto3-logs Python Package Guide

## Golden Rule

- Keep `boto3` as the runtime AWS SDK. `mypy-boto3-logs` only adds static typing for the CloudWatch Logs client.
- Prefer `boto3-stubs[logs]` when you want `Session.client("logs")` overload inference with minimal annotation work.
- Use standalone `mypy-boto3-logs` when you want only the CloudWatch Logs stubs and are willing to annotate clients explicitly.
- Treat PyPI as the source of truth for the exact published package version. On March 12, 2026, PyPI listed `1.42.60`, while the generated docs page already said the package docs were compatible with `boto3==1.42.63`.

## Install

If you want the narrow standalone service stubs, pin them with the matching boto3 line:

```bash
python -m pip install "boto3==1.42.60" "mypy-boto3-logs==1.42.60"
```

If you want the upstream-recommended overloads for `Session.client("logs")`, install the bundled package instead:

```bash
python -m pip install "boto3-stubs[logs]"
```

Lower-memory bundle:

```bash
python -m pip install "boto3-stubs-lite[logs]"
```

Notes:

- `boto3-stubs-lite[logs]` omits the `Session.client(...)` overloads, so explicit annotations or `cast(...)` become more important.
- For `uv` or Poetry, keep the same rule: install `boto3` for runtime, and keep the stub package in the same environment that `mypy`, `pyright`, or your IDE analyzes.

## Initialize And Authenticate

`mypy-boto3-logs` does not change AWS credentials, region resolution, retries, or endpoints. Those still come from normal `boto3` and `botocore` behavior.

Common credential sources, in boto3's documented search order:

1. explicit credentials passed to `client(...)` or `Session(...)`
2. environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`
3. shared config and credentials files, including `AWS_PROFILE`
4. container or instance credentials in AWS runtimes

Typical local setup:

```bash
export AWS_PROFILE=observability-dev
export AWS_DEFAULT_REGION=us-west-2
```

Typed client initialization:

```python
from boto3.session import Session
from mypy_boto3_logs.client import CloudWatchLogsClient

session = Session(profile_name="observability-dev", region_name="us-west-2")
logs: CloudWatchLogsClient = session.client("logs")
```

If you create sessions manually, keep boto3's session guidance in mind: do not share a `Session` object across threads or processes. Create one session per thread or worker when concurrency matters.

## Core Usage

### Typed CloudWatch Logs client

Use the generated client type for normal CloudWatch Logs operations:

```python
from boto3.session import Session
from mypy_boto3_logs.client import CloudWatchLogsClient

session = Session(profile_name="observability-dev", region_name="us-west-2")
logs: CloudWatchLogsClient = session.client("logs")

response = logs.describe_log_groups(
    logGroupNamePrefix="/aws/lambda/",
    limit=20,
)

for group in response.get("logGroups", []):
    print(group["logGroupName"])
```

This follows the same runtime call shape as the boto3 CloudWatch Logs client docs, but the response and operation names stop collapsing to `Any`.

### Dev-only stubs with `TYPE_CHECKING`

If production images do not install the stub package, keep the imports type-only:

```python
from typing import TYPE_CHECKING, cast

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_logs.client import CloudWatchLogsClient

session = Session(region_name="us-west-2")
logs = cast("CloudWatchLogsClient", session.client("logs"))
```

### Typed paginators

The generated docs expose paginator classes for CloudWatch Logs operations such as `describe_log_groups`:

```python
from boto3.session import Session
from mypy_boto3_logs.client import CloudWatchLogsClient
from mypy_boto3_logs.paginator import DescribeLogGroupsPaginator

logs: CloudWatchLogsClient = Session(region_name="us-west-2").client("logs")
paginator: DescribeLogGroupsPaginator = logs.get_paginator("describe_log_groups")

for page in paginator.paginate(logGroupNamePrefix="/aws/lambda/"):
    for group in page.get("logGroups", []):
        print(group["logGroupName"])
```

### Typed event shapes

Use the generated `type_defs` module when helper functions pass AWS-shaped dictionaries around:

```python
from collections.abc import Sequence

from mypy_boto3_logs.type_defs import FilteredLogEventTypeDef

def extract_messages(events: Sequence[FilteredLogEventTypeDef]) -> list[str]:
    return [event["message"] for event in events if "message" in event]
```

Example call using the normal boto3 runtime client:

```python
from boto3.session import Session
from mypy_boto3_logs.client import CloudWatchLogsClient

logs: CloudWatchLogsClient = Session(region_name="us-west-2").client("logs")

response = logs.filter_log_events(
    logGroupName="/aws/lambda/my-function",
    limit=50,
)

messages = [event["message"] for event in response.get("events", []) if "message" in event]
```

## Type Checking Notes

- Install the stubs in the same environment as `mypy`, `pyright`, or your editor language server.
- Import from `mypy_boto3_logs`, not `mypy-boto3-logs`. The package name uses hyphens; the Python module uses underscores.
- Standalone `mypy-boto3-logs` is best when you want one narrow dependency and explicit annotations.
- `boto3-stubs[logs]` is best when you want better inference from `Session.client("logs")` and broader boto3 typing support.

## Common Pitfalls

- Do not install only `mypy-boto3-logs` and expect AWS calls to work. You still need `boto3`.
- Do not pass the package name to boto3. The service id is `"logs"`, not `"mypy-boto3-logs"`.
- Do not assume the generated docs site and PyPI are on the same patch version. For this package, the docs page was ahead of the last published wheel during this session.
- Do not assume a resource-oriented API or waiter coverage just because other boto3 services have them. The maintainer docs for this package focus on the CloudWatch Logs client, paginators, literals, and typed shapes.
- Do not treat good type checking as proof that IAM permissions, log group names, retention settings, or throttling behavior are correct. Runtime failures still come from normal boto3 and botocore exceptions.
- Do not share one boto3 `Session` across threads or processes.

## Version-Sensitive Notes For `1.42.60`

- The version used here for this session was `1.42.60`, and PyPI still reported `1.42.60` on March 12, 2026.
- PyPI reports Python `>=3.9` for this release line.
- The package docs page currently says the generated stubs are compatible with `boto3==1.42.63`, so the docs site can move ahead of the last published `mypy-boto3-logs` wheel.
- If exact AWS model parity matters, pin `boto3`, `botocore`, and the stub package from the same validated release line instead of assuming "latest" is synchronized everywhere.

## Official Sources

- Maintainer docs root: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_logs/
- Maintainer versioning guide: https://youtype.github.io/boto3_stubs_docs/#versioning
- PyPI project page: https://pypi.org/project/mypy-boto3-logs/
- PyPI JSON API: https://pypi.org/pypi/mypy-boto3-logs/json
- boto3 credentials guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html
- boto3 session guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/session.html
- boto3 CloudWatch Logs `describe_log_groups`: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/logs/client/describe_log_groups.html
- boto3 CloudWatch Logs `filter_log_events`: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/logs/client/filter_log_events.html
