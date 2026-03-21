---
name: mypy-boto3-codebuild
description: "mypy-boto3-codebuild Python package guide for typed AWS CodeBuild boto3 clients"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "python,aws,codebuild,boto3,typing,mypy,stub"
---

# mypy-boto3-codebuild Python Package Guide

## What This Package Is

`mypy-boto3-codebuild` is the generated type-stub package for the AWS CodeBuild client in `boto3`. It improves static analysis and autocomplete for CodeBuild operations, paginators, literals, and TypedDict request or response shapes.

It is not a runtime SDK. You still install and use `boto3` to talk to AWS.

## Installation

Install the runtime SDK plus the split CodeBuild stubs:

```bash
python -m pip install "boto3>=1.42,<1.43" "mypy-boto3-codebuild==1.42.3"
```

If your project already uses the bundled stubs package, the equivalent extra is:

```bash
python -m pip install "boto3-stubs[codebuild]==1.42.3"
```

Use one stub strategy per project. Mixing the split package and the bundled extras usually adds noise without adding capability.

## Initialize A Typed CodeBuild Client

With the split service package, explicitly annotate the client variable:

```python
import boto3
from mypy_boto3_codebuild.client import CodeBuildClient

session = boto3.Session(profile_name="dev", region_name="us-west-2")
codebuild: CodeBuildClient = session.client("codebuild")

projects = codebuild.list_projects()
print(projects["projects"])
```

The annotation matters. `boto3.client("codebuild")` still returns a normal boto3 client at runtime, but the stub type gives editors and type checkers the generated CodeBuild method signatures.

If you keep stubs in a dev-only dependency group, guard the import:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_codebuild.client import CodeBuildClient

codebuild: "CodeBuildClient" = boto3.client("codebuild", region_name="us-east-1")
```

## Core Imports

```python
from mypy_boto3_codebuild.client import CodeBuildClient
from mypy_boto3_codebuild.paginator import ListBuildsForProjectPaginator
from mypy_boto3_codebuild.type_defs import StartBuildInputTypeDef
```

Use these modules for the common cases:

- `client` for the typed service client
- `paginator` for typed paginator classes
- `type_defs` for request and response TypedDicts

## Core Usage

### Call normal CodeBuild APIs with typed responses

```python
import boto3
from mypy_boto3_codebuild.client import CodeBuildClient

codebuild: CodeBuildClient = boto3.client("codebuild", region_name="us-east-1")

response = codebuild.batch_get_projects(names=["my-project"])
for project in response["projects"]:
    print(project["name"], project.get("serviceRole"))
```

### Build request payloads with TypedDicts

```python
import boto3
from mypy_boto3_codebuild.client import CodeBuildClient
from mypy_boto3_codebuild.type_defs import StartBuildInputTypeDef

codebuild: CodeBuildClient = boto3.client("codebuild", region_name="us-east-1")

request: StartBuildInputTypeDef = {
    "projectName": "my-project",
    "environmentVariablesOverride": [
        {"name": "APP_ENV", "value": "ci", "type": "PLAINTEXT"},
    ],
}

response = codebuild.start_build(**request)
print(response["build"]["id"])
```

### Use typed paginators for repeated listing

```python
import boto3
from mypy_boto3_codebuild.client import CodeBuildClient
from mypy_boto3_codebuild.paginator import ListBuildsForProjectPaginator

codebuild: CodeBuildClient = boto3.client("codebuild", region_name="us-east-1")
paginator: ListBuildsForProjectPaginator = codebuild.get_paginator("list_builds_for_project")

for page in paginator.paginate(projectName="my-project", sortOrder="DESCENDING"):
    for build_id in page["ids"]:
        print(build_id)
```

## Authentication And Configuration

`mypy-boto3-codebuild` adds no authentication behavior. Credential loading and region resolution are exactly the same as `boto3`.

Use the normal boto3 credential chain:

- `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and optional `AWS_SESSION_TOKEN`
- shared AWS config and credentials files
- named profiles such as `profile_name="dev"`
- IAM roles in EC2, ECS, Lambda, or other AWS runtimes

Example:

```python
import boto3
from mypy_boto3_codebuild.client import CodeBuildClient

session = boto3.Session(profile_name="dev", region_name="us-west-2")
codebuild: CodeBuildClient = session.client("codebuild")
```

For agents, the practical rule is simple: solve auth and region setup exactly as you would for any other boto3 client, then layer the CodeBuild typing imports on top.

## Version-Sensitive Notes

### Use the version used here for this entry

This repo entry is intentionally pinned to `mypy-boto3-codebuild==1.42.3`, because that is the version used here for this package session.

### The docs site currently shows newer generated examples

The maintainer docs root for `mypy_boto3_codebuild` currently shows install snippets for newer generated boto3-stubs builds than the target version. Treat the docs site as the canonical API-shape reference, but treat the pinned PyPI package version as authoritative for this entry.

If you copy snippets from the docs site, re-pin them to `1.42.3` for this package doc and verify the matching boto3 family you actually install.

### Keep boto3, botocore, and the stubs in the same release family

These stubs are generated from boto3 and botocore service models. If your installed `boto3` or `botocore` drifts too far from the stub version, type hints can advertise request fields or response members that do not match runtime behavior.

For `1.42.3`, prefer a `boto3` and `botocore` version from the same `1.42.x` family when possible.

## Common Pitfalls

### Treating the package as a runtime dependency only

`mypy-boto3-codebuild` does not create clients, sign requests, or make AWS API calls. Runtime behavior still comes from `boto3`.

### Expecting `boto3.client("codebuild")` to become fully typed by itself

With the split service package, you usually need an explicit `CodeBuildClient` annotation on the variable returned by `boto3.client(...)` or `Session.client(...)`.

### Forgetting to install the stubs where imports are evaluated

If code imports `mypy_boto3_codebuild` at runtime but the package is only installed in a dev environment, production imports will fail. Use a `TYPE_CHECKING` guard if the stubs are not part of the runtime environment.

### Copying mismatched versions from the docs site

The docs site is generated from the broader project and can show newer version pins than the PyPI package version covered here.

## Official Sources

- Maintainer docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_codebuild/
- PyPI project: https://pypi.org/project/mypy-boto3-codebuild/
- Upstream repository: https://github.com/youtype/mypy_boto3_builder
- Boto3 credentials guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html
