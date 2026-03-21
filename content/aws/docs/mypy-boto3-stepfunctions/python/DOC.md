---
name: mypy-boto3-stepfunctions
description: "mypy-boto3-stepfunctions type stubs for boto3 Step Functions clients, paginators, literals, and TypedDict request/response shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,stepfunctions,boto3,type-stubs,mypy,pyright,python"
---

# mypy-boto3-stepfunctions Python Package Guide

## Golden Rule

`mypy-boto3-stepfunctions` is a typing package for `boto3` Step Functions usage. It improves autocomplete and static checking, but it does not replace `boto3`, does not configure AWS credentials, and does not make runtime API calls by itself.

Use it in one of these modes:

- Install `boto3-stubs[stepfunctions]` when you want `Session().client("stepfunctions")` to infer the `SFNClient` type automatically.
- Install `mypy-boto3-stepfunctions` when you only want the standalone Step Functions stubs and are willing to add explicit type annotations.
- Keep runtime setup in normal `boto3` and AWS config.

## Install

### Recommended for most projects

```bash
python -m pip install boto3 'boto3-stubs[stepfunctions]'
```

This gives you the runtime SDK plus service-specific type discovery for Step Functions.

### Standalone Step Functions stubs

```bash
python -m pip install "mypy-boto3-stepfunctions==1.42.3" boto3
```

Use this when you only want the Step Functions stubs package. In this mode, explicit annotations are usually the clearest approach.

### Lite mode for lower memory / PyCharm

```bash
python -m pip install boto3 'boto3-stubs-lite[stepfunctions]'
```

The maintainer docs recommend the lite variant for PyCharm performance, but it does not provide `session.client()` overload inference. Plan to annotate your client variables explicitly.

## Runtime Setup And Auth

This package has no package-specific auth or config. Use normal `boto3` setup.

Boto3 searches for credentials in a defined order that includes explicit client parameters, explicit `Session(...)` parameters, environment variables, shared credentials/config files, container credentials, and EC2 instance metadata.

Common local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

For project-specific behavior, create an explicit session or pass a `Config` object:

```python
from boto3.session import Session
from botocore.config import Config

session = Session(profile_name="dev")

config = Config(
    region_name="us-east-1",
    retries={"max_attempts": 10, "mode": "standard"},
)
```

Use explicit sessions in tests, CLIs, and multi-account code so credentials and region do not come from hidden global state.

## Core Usage

### Typed Step Functions client

```python
from boto3.session import Session
from mypy_boto3_stepfunctions import SFNClient

session = Session(profile_name="dev", region_name="us-east-1")
client: SFNClient = session.client("stepfunctions")
```

AWS exposes Step Functions in boto3 as a low-level client. The service reference lists client methods such as `start_execution`, `start_sync_execution`, `describe_execution`, `list_executions`, `get_execution_history`, and `validate_state_machine_definition`.

### Start an execution with typed output

Step Functions expects `input` as JSON text, not a Python dict.

```python
import json

from boto3.session import Session
from mypy_boto3_stepfunctions import SFNClient
from mypy_boto3_stepfunctions.type_defs import StartExecutionOutputTypeDef

client: SFNClient = Session(region_name="us-east-1").client("stepfunctions")

response: StartExecutionOutputTypeDef = client.start_execution(
    stateMachineArn="arn:aws:states:us-east-1:123456789012:stateMachine:OrderWorkflow",
    name="order-123",
    input=json.dumps({"order_id": "123", "retry": False}),
)

print(response["executionArn"])
```

### Typed paginator

Step Functions has generated paginator types for `get_execution_history`, `list_activities`, `list_executions`, `list_map_runs`, and `list_state_machines`.

```python
from boto3.session import Session
from mypy_boto3_stepfunctions import SFNClient
from mypy_boto3_stepfunctions.paginator import ListExecutionsPaginator

client: SFNClient = Session(region_name="us-east-1").client("stepfunctions")
paginator: ListExecutionsPaginator = client.get_paginator("list_executions")

for page in paginator.paginate(
    stateMachineArn="arn:aws:states:us-east-1:123456789012:stateMachine:OrderWorkflow"
):
    for execution in page.get("executions", []):
        print(execution["name"], execution["status"])
```

### Literals and TypedDict request shapes

```python
from mypy_boto3_stepfunctions.literals import StateMachineTypeType
from mypy_boto3_stepfunctions.type_defs import StartExecutionInputTypeDef

machine_type: StateMachineTypeType = "STANDARD"

request: StartExecutionInputTypeDef = {
    "stateMachineArn": "arn:aws:states:us-east-1:123456789012:stateMachine:OrderWorkflow",
    "input": "{}",
}
```

The generated `type_defs` module is useful when you want precise request and response dictionaries in helper functions or tests.

### Keep stub imports out of runtime-only environments

If stubs are installed only in dev or CI, gate them behind `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING
from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_stepfunctions import SFNClient
else:
    SFNClient = object

def make_client() -> "SFNClient":
    return Session(region_name="us-east-1").client("stepfunctions")
```

The upstream PyPI page calls out this pattern specifically as the way to avoid a production dependency on the stubs package while keeping pylint satisfied.

## Tooling Notes

- `mypy` and `pyright` are both first-class upstream targets.
- `boto3-stubs[stepfunctions]` is the easiest option if you want IDE auto-discovery without explicit annotations.
- `boto3-stubs-lite[stepfunctions]` trades inference quality for lower IDE overhead.
- The standalone package is best when you want service-specific imports without the broader extras package.

## Common Pitfalls

- Installing only `mypy-boto3-stepfunctions` and expecting AWS calls to work. You still need `boto3`.
- Forgetting the package/import name split: install `mypy-boto3-stepfunctions`, import `mypy_boto3_stepfunctions`.
- Using `session.client("sfn")`. The boto3 service name is `"stepfunctions"`, even though the typed client class is `SFNClient`.
- Passing a raw dict to `start_execution(..., input=...)`. Step Functions expects a JSON string.
- Expecting `boto3-stubs-lite[stepfunctions]` to infer `session.client("stepfunctions")` automatically. Add explicit `SFNClient` annotations in lite mode.
- Assuming there is a Step Functions resource layer in boto3 like S3 has. The official Step Functions boto3 reference is client-based, and the generated stub docs for this package expose client, paginator, literal, and `type_defs` modules.
- Treating successful type checking as proof that credentials, region, IAM permissions, or state machine ARNs are correct. Typing and runtime access are separate concerns.

## Version-Sensitive Notes

- This entry is pinned to version used here `1.42.3`.
- PyPI currently serves `mypy-boto3-stepfunctions 1.42.3` as the latest release, published on `2025-12-04`.
- The maintainer docs landing page is not version-pinned. On `2026-03-12` it still showed a local generation example against `boto3==1.42.62`, so use PyPI as the authoritative source for package-version pinning.
- Upstream states that `mypy-boto3-stepfunctions` follows the related `boto3` version. If a symbol looks missing, first check whether your installed `boto3`, `botocore`, and stub package are aligned closely enough.

## Official Sources

- Maintainer docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_stepfunctions/
- PyPI package page: https://pypi.org/project/mypy-boto3-stepfunctions/
- Boto3 Step Functions reference: https://docs.aws.amazon.com/boto3/latest/reference/services/stepfunctions.html
- Boto3 credentials guide: https://docs.aws.amazon.com/boto3/latest/guide/credentials.html
- Boto3 configuration guide: https://docs.aws.amazon.com/boto3/latest/guide/configuration.html
