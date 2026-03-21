---
name: mypy-boto3-glue
description: "mypy-boto3-glue package guide for Python typed boto3 Glue clients, paginators, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.43"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,glue,boto3,stubs,typing,mypy,pyright"
---

# mypy-boto3-glue Python Package Guide

## What It Is

`mypy-boto3-glue` is the maintainer-published type stub package for the AWS Glue part of `boto3`.

Use it when you want:

- static typing for `Session().client("glue")`
- IDE completion for Glue operations and response fields
- typed paginator classes
- generated `Literal` values for Glue enum-like strings
- generated `TypedDict` request and response shapes

Important boundary: this package does not make API calls by itself. Runtime behavior still comes from `boto3` and `botocore`.

## Install

Install `boto3` for runtime calls, then pick one typing package.

```bash
python -m pip install boto3
python -m pip install mypy-boto3-glue
```

Maintainer-supported alternatives:

```bash
python -m pip install "boto3-stubs[glue]"
python -m pip install "boto3-stubs-lite[glue]"
```

Practical rule:

- use `mypy-boto3-glue` when you only need Glue stubs
- use `boto3-stubs[glue]` when you want the broader boto3 typing package with Glue enabled
- use `boto3-stubs-lite[glue]` only if you need the lower-memory variant and can live without `session.client()` and `session.resource()` overloads
- do not install multiple stub variants unless you are deliberately managing which one wins in your type checker environment

## Recommended Setup

For agent-written code, explicit client annotations are the safest default.

```python
from __future__ import annotations

from boto3.session import Session
from botocore.config import Config
from mypy_boto3_glue import GlueClient

session = Session(profile_name="analytics-dev", region_name="us-east-1")

glue: GlueClient = session.client(
    "glue",
    config=Config(
        retries={
            "mode": "standard",
            "max_attempts": 10,
        }
    ),
)

response = glue.get_databases(MaxResults=25)

for database in response.get("DatabaseList", []):
    print(database["Name"])
```

Why this pattern works well:

- it keeps runtime construction in normal boto3 code
- it gives mypy and pyright a concrete `GlueClient`
- it keeps AWS profile, region, retry, and proxy settings in standard boto3/botocore configuration

If the stubs are installed only in development, keep imports behind `TYPE_CHECKING`:

```python
from __future__ import annotations

from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_glue import GlueClient

def make_glue_client() -> "GlueClient":
    return Session(region_name="us-east-1").client("glue")
```

## Core Typing Patterns

### Paginators

The package publishes typed paginator classes for Glue paginator operations.

```python
from boto3.session import Session
from mypy_boto3_glue import GlueClient
from mypy_boto3_glue.paginator import GetDatabasesPaginator

glue: GlueClient = Session(region_name="us-east-1").client("glue")
paginator: GetDatabasesPaginator = glue.get_paginator("get_databases")

for page in paginator.paginate():
    for database in page.get("DatabaseList", []):
        print(database["Name"])
```

### Literals

Use generated literal types when you want type checkers to catch misspelled AWS string constants.

```python
from mypy_boto3_glue.literals import AdditionalOptionKeysType

option_key: AdditionalOptionKeysType = "compositeRuleEvaluation.method"
```

### TypedDict Shapes

Use generated `type_defs` when you pass nested Glue request or response fragments between helpers.

```python
from mypy_boto3_glue.type_defs import NotificationPropertyTypeDef

notification: NotificationPropertyTypeDef = {
    "NotifyDelayAfter": 5,
}
```

These generated shapes are more precise than `dict[str, object]` and reduce avoidable casting.

## Auth And Runtime Config

`mypy-boto3-glue` does not have its own authentication layer. Credential and region resolution come from boto3.

Common inputs:

- `AWS_PROFILE` or `Session(profile_name="...")`
- `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`
- `AWS_DEFAULT_REGION`
- `~/.aws/config` and `~/.aws/credentials`
- IAM roles and other AWS runtime credential providers

Typical local setup:

```python
from boto3.session import Session

session = Session(profile_name="analytics-dev", region_name="us-west-2")
glue = session.client("glue")
```

Use `botocore.config.Config` when you need retries, timeouts, proxies, or custom endpoints. The stubs type the client surface; they do not change runtime behavior.

## Type Checker Notes

- `mypy` and `pyright` work best when the stub package is installed in the same environment used for analysis
- explicit annotations are the most reliable choice for standalone `mypy-boto3-glue`
- `boto3-stubs-lite[glue]` intentionally omits `session.client()` and `session.resource()` overloads, so explicit annotations are required there
- `from __future__ import annotations` avoids importing typing-only names too early in runtime paths

## Common Pitfalls

- Do not treat `mypy-boto3-glue` as a runtime SDK. Import and call AWS Glue through `boto3`.
- Keep the stub version close to the installed `boto3` version family. These packages are generated from boto3 service models and can drift.
- Do not assume every boto3 Glue convenience pattern is represented by one stable public type name. Generated symbols can change when AWS updates service models.
- If the stub package is a dev dependency only, avoid importing stub-only names unguarded in modules that production code imports.
- If autocomplete is missing after installation, check that your editor, virtual environment, and type checker are all looking at the same interpreter.

## Version-Sensitive Notes

- The version used here, the current PyPI release, and this doc all align on `1.42.43` as verified on March 12, 2026.
- The hosted docs site currently includes a local-generation example pinned to `boto3==1.42.65`, which is ahead of the standalone PyPI package version. Treat the docs site as the best API-shape reference, but pin installs from PyPI to the package version your project actually uses.
- When in doubt, pin `boto3` and `mypy-boto3-glue` together during upgrades and rerun your type checker after the bump.

## Official Sources

- Docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_glue/`
- PyPI: `https://pypi.org/project/mypy-boto3-glue/`
- Repository: `https://github.com/youtype/boto3-stubs`
- AWS boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- AWS boto3 configuration guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/configuration.html`
