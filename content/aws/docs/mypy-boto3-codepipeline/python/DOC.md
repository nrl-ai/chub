---
name: mypy-boto3-codepipeline
description: "mypy-boto3-codepipeline package guide for typed boto3 CodePipeline clients, paginators, literals, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,codepipeline,boto3,mypy,pyright,stubs,typing,python,cicd"
---

# mypy-boto3-codepipeline Python Package Guide

## Golden Rule

Use `boto3` for real AWS calls and use `mypy-boto3-codepipeline` only for typing. If you want `Session.client("codepipeline")` to infer automatically, install `boto3-stubs[codepipeline]`. If you install only `mypy-boto3-codepipeline` or `boto3-stubs-lite[codepipeline]`, plan to add explicit type annotations.

## Version-Sensitive Notes

- The version used here `1.42.3` matches the official PyPI release and the maintainer docs checked on `2026-03-12`.
- The maintainer project states that `mypy-boto3-codepipeline` uses the same version as the related `boto3` release. Keep the stub package close to your installed `boto3` line so signatures and response shapes do not drift.
- The maintainer docs page is a stable package URL, not a release-pinned docs snapshot. Pin the package version in your environment when exact patch parity matters.
- The PyPI package page shows this release was generated with `mypy-boto3-builder 8.12.0`. If you need exact parity with a locally pinned boto3 build, the maintainer recommends generating stubs locally with `mypy-boto3-builder`.
- AWS runtime docs for CodePipeline are on a rolling latest-docs line, so use PyPI as the source of truth for the exact stub package version and AWS docs for runtime behavior.

## Install

Choose one install mode based on whether you want automatic type inference or smaller typing dependencies.

### Best inference: full boto3 stubs

Use this when you want `Session.client("codepipeline")` to infer without extra annotations:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[codepipeline]==1.42.3"
```

### Lower-memory option

Use this when editor performance matters more than overload-based inference:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[codepipeline]==1.42.3"
```

The maintainer docs say the lite package does not provide `session.client/resource` overloads, so explicit annotations become the normal workflow.

### Standalone package

Use this when you want only the CodePipeline typing package:

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-codepipeline==1.42.3"
```

### Generate locally when exact parity matters

```bash
uvx --with "boto3==1.42.3" mypy-boto3-builder
```

Then select `boto3-stubs` and the `CodePipeline` service.

## Initialization And Setup

`mypy-boto3-codepipeline` does not add credentials, retries, or endpoint behavior. Runtime configuration still comes from `boto3` and `botocore`.

Useful AWS credential and config facts for typed CodePipeline clients:

- Boto3 checks credentials in the normal provider chain, including explicit client/session credentials, environment variables, assume-role providers, IAM Identity Center, shared credentials files, config files, container credentials, and EC2 instance metadata.
- Use `AWS_PROFILE` or `boto3.Session(profile_name=...)` for local development instead of hardcoding keys.
- Set region explicitly in code or via `AWS_DEFAULT_REGION` because CodePipeline is regional.
- Retry and transport behavior still come from boto3 config such as `AWS_RETRY_MODE`, `AWS_MAX_ATTEMPTS`, or `botocore.config.Config(...)`.

Typical local setup:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
export AWS_RETRY_MODE=standard
export AWS_MAX_ATTEMPTS=10
```

Then create the real boto3 client:

```python
from boto3.session import Session
from botocore.config import Config

session = Session(profile_name="dev", region_name="us-east-1")
config = Config(retries={"mode": "standard", "max_attempts": 10})
codepipeline = session.client("codepipeline", config=config)
```

## Core Usage

### Zero-annotation workflow with `boto3-stubs[codepipeline]`

With the full extra installed, ordinary boto3 code should type-check without explicit annotations:

```python
from boto3.session import Session

session = Session(profile_name="dev", region_name="us-east-1")
client = session.client("codepipeline")

state = client.get_pipeline_state(name="MyPipeline")

for stage in state.get("stageStates", []):
    latest = stage.get("latestExecution") or {}
    print(stage["stageName"], latest.get("status"))
```

### Explicit client annotations

Use explicit annotations when you installed the standalone package or the lite package:

```python
from boto3.session import Session
from mypy_boto3_codepipeline import CodePipelineClient

session = Session(profile_name="dev", region_name="us-east-1")
client: CodePipelineClient = session.client("codepipeline")

response = client.start_pipeline_execution(name="MyPipeline")
print(response["pipelineExecutionId"])
```

### Typed paginators

The maintainer docs expose typed paginators for operations such as `list_action_executions`, `list_pipeline_executions`, `list_rule_executions`, and `list_webhooks`.

```python
from boto3.session import Session
from mypy_boto3_codepipeline import CodePipelineClient
from mypy_boto3_codepipeline.paginator import ListPipelineExecutionsPaginator

session = Session(profile_name="dev", region_name="us-east-1")
client: CodePipelineClient = session.client("codepipeline")

paginator: ListPipelineExecutionsPaginator = client.get_paginator(
    "list_pipeline_executions"
)

for page in paginator.paginate(pipelineName="MyPipeline"):
    for execution in page.get("pipelineExecutionSummaries", []):
        print(execution["pipelineExecutionId"], execution["status"])
```

### Literals and typed request fragments

Use generated literals and `TypedDict` shapes in helpers that construct CodePipeline request data:

```python
from mypy_boto3_codepipeline.literals import ActionCategoryType
from mypy_boto3_codepipeline.type_defs import AWSSessionCredentialsTypeDef

category: ActionCategoryType = "Build"

creds: AWSSessionCredentialsTypeDef = {
    "accessKeyId": "AKIA...",
    "secretAccessKey": "secret",
    "sessionToken": "token",
}
```

Generated `type_defs` are useful when your code assembles request/response fragments before the actual boto3 call.

### Runtime-safe `TYPE_CHECKING` pattern

If production installs omit stub packages, keep the imports type-only:

```python
from __future__ import annotations

from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_codepipeline import CodePipelineClient
else:
    CodePipelineClient = object

session = Session(profile_name="dev", region_name="us-east-1")
client: "CodePipelineClient" = session.client("codepipeline")
```

This matches the maintainer guidance for avoiding runtime stub dependencies and also avoids the `pylint` undefined-name issue.

## CodePipeline-Specific Notes

- The boto3 service name is `codepipeline`, not `mypy-boto3-codepipeline`.
- CodePipeline is regional. A client pointed at the wrong region will not see the pipeline you expect.
- Typed helpers are most useful around client methods the AWS docs highlight for operational workflows: `get_pipeline_state`, `list_pipeline_executions`, `list_action_executions`, `list_rule_executions`, `start_pipeline_execution`, `retry_stage_execution`, and `stop_pipeline_execution`.
- The AWS service reference lists paginator support for `ListActionExecutions`, `ListActionTypes`, `ListDeployActionExecutionTargets`, `ListPipelineExecutions`, `ListPipelines`, `ListRuleExecutions`, `ListTagsForResource`, and `ListWebhooks`.
- These stubs improve editor help and static checking, but they do not validate that your pipeline name, IAM permissions, or deployment state are correct at runtime.

## Common Pitfalls

- `mypy-boto3-codepipeline` is stub-only. It does not replace `boto3` and does not create a runtime client by itself.
- The PyPI package name uses hyphens, but Python imports use underscores: `mypy_boto3_codepipeline`.
- `boto3-stubs[codepipeline]` gives the best inference experience. `boto3-stubs-lite[codepipeline]` and standalone installs usually require explicit `CodePipelineClient` annotations.
- Keep the stub version aligned with the installed `boto3` line. Type hints can silently drift even when runtime code still works.
- Do not hardcode AWS credentials in application code. Prefer profiles, IAM Identity Center, assume-role config, container credentials, or instance/task roles.
- PyCharm can be slow on large `Literal` overloads. The maintainer docs recommend the lite variant if editor performance becomes a problem.
- Stubs only describe shapes. They will not catch wrong region selection, missing IAM permissions, or pipeline names that do not exist.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_codepipeline/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-codepipeline/`
- Maintainer repository: `https://github.com/youtype/types-boto3`
- AWS boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- AWS boto3 configuration guide: `https://docs.aws.amazon.com/boto3/latest/guide/configuration.html`
- AWS CodePipeline boto3 reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/codepipeline.html`
