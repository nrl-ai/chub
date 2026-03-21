---
name: mypy-boto3-acm
description: "Type stubs for boto3 ACM in Python, including typed clients, paginators, waiters, literals, and TypedDict request/response shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,acm,boto3,python,typing,type-stubs,mypy,pyright"
---

# `mypy-boto3-acm` Python Package Guide

## Golden Rule

`mypy-boto3-acm` is a stubs-only package for static typing. Keep using `boto3` for real ACM API calls, then choose one typing mode:

- Use `boto3-stubs[acm]` when you want `Session().client("acm")` to infer types automatically in IDEs, `mypy`, and `pyright`.
- Use `mypy-boto3-acm` when you only want ACM-specific type stubs and are willing to annotate `ACMClient`, paginators, waiters, literals, and `type_defs` explicitly.
- Use `boto3-stubs-lite[acm]` if full stubs are too heavy for PyCharm or memory-constrained environments. The lite package is more RAM-friendly, but upstream notes that it does not provide `session.client/resource` overloads.

## Install

Recommended when you want automatic type discovery:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[acm]==1.42.3"
```

Lower-memory option:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[acm]==1.42.3"
```

Standalone ACM-only stubs:

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-acm==1.42.3"
```

Common alternatives:

```bash
uv add "boto3==1.42.3" "boto3-stubs[acm]==1.42.3"
poetry add "boto3==1.42.3" "boto3-stubs[acm]==1.42.3"
```

Notes:

- `boto3-stubs[acm]` is the easiest path if you want `Session().client("acm")` to be inferred without extra annotations.
- `boto3-stubs-lite[acm]` is the safer choice when PyCharm becomes slow on large `Literal` overloads.
- `mypy-boto3-acm` does not replace `boto3`. If you install only the stubs package, type checking can pass while runtime imports or AWS calls still fail.

## Authentication And Runtime Setup

The typing package does not change runtime auth, retries, endpoints, or permissions. Those still come from `boto3` and the normal AWS credential chain.

Typical local setup:

```bash
export AWS_PROFILE="dev"
export AWS_DEFAULT_REGION="us-east-1"
```

Or use shared config and credentials files:

```bash
aws configure
```

Typed client setup:

```python
from boto3.session import Session
from mypy_boto3_acm import ACMClient

session = Session(profile_name="dev", region_name="us-east-1")
acm: ACMClient = session.client("acm")
```

Useful environment variables:

- `AWS_PROFILE`
- `AWS_DEFAULT_REGION`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`

ACM in boto3 is client-oriented. AWS documents an ACM client surface with one paginator and one waiter; unlike services such as S3 or IAM, there is no boto3 ACM resource interface to type here.

## Core Usage

### Typed ACM client

```python
from boto3.session import Session
from mypy_boto3_acm import ACMClient

acm: ACMClient = Session(region_name="us-east-1").client("acm")

response = acm.list_certificates(CertificateStatuses=["ISSUED"])

for cert in response["CertificateSummaryList"]:
    print(cert["CertificateArn"], cert.get("DomainName"))
```

### Typed paginator

AWS documents only one ACM paginator: `list_certificates`.

```python
from boto3.session import Session
from mypy_boto3_acm import ACMClient
from mypy_boto3_acm.paginator import ListCertificatesPaginator

acm: ACMClient = Session(region_name="us-east-1").client("acm")
paginator: ListCertificatesPaginator = acm.get_paginator("list_certificates")

for page in paginator.paginate(CertificateStatuses=["ISSUED"]):
    for cert in page["CertificateSummaryList"]:
        print(cert["CertificateArn"])
```

### Typed waiter

AWS documents one ACM waiter: `certificate_validated`.

```python
from boto3.session import Session
from mypy_boto3_acm import ACMClient
from mypy_boto3_acm.waiter import CertificateValidatedWaiter

acm: ACMClient = Session(region_name="us-east-1").client("acm")
waiter: CertificateValidatedWaiter = acm.get_waiter("certificate_validated")

waiter.wait(
    CertificateArn="arn:aws:acm:us-east-1:123456789012:certificate/...",
    WaiterConfig={"Delay": 60, "MaxAttempts": 30},
)
```

### Typed request shapes with `type_defs`

Use `type_defs` when you want explicit `TypedDict` request or response structures in helper functions.

```python
from mypy_boto3_acm.type_defs import RequestCertificateRequestTypeDef, TagTypeDef

tags: list[TagTypeDef] = [
    {"Key": "service", "Value": "payments"},
]

request: RequestCertificateRequestTypeDef = {
    "DomainName": "api.example.com",
    "ValidationMethod": "DNS",
    "SubjectAlternativeNames": ["www.example.com"],
    "Tags": tags,
}
```

### Literals for constrained values

```python
from mypy_boto3_acm.literals import CertificateStatusType, ValidationMethodType

status: CertificateStatusType = "ISSUED"
validation_method: ValidationMethodType = "DNS"
```

### `TYPE_CHECKING` guard for dev-only stubs

If production images should not carry stub packages, keep the imports in a type-checking branch and fall back to `object` to avoid a known `pylint` complaint documented on PyPI.

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_acm import ACMClient
else:
    ACMClient = object

acm = Session(region_name="us-east-1").client("acm")
typed_acm: "ACMClient" = acm
```

## Tooling Notes

- Upstream documents that `boto3-stubs[acm]` works with VSCode, PyCharm, Emacs, Sublime Text, `mypy`, and `pyright`.
- PyCharm can be slow on `Literal` overloads. If that happens, upstream recommends `boto3-stubs-lite` or disabling the built-in PyCharm type checker and running `mypy` or `pyright` separately.
- The standalone `mypy-boto3-acm` package is useful when you want only ACM typings installed, but your factories and helper functions should return explicit `ACMClient`, paginator, waiter, or `type_defs` types instead of relying on overload inference.

## Common Pitfalls

- Do not treat `mypy-boto3-acm` as the runtime SDK. Real AWS calls still require `boto3`.
- Do not mismatch `boto3` and stubs versions casually. This package tracks the related boto3 version line, so pin them together when you want predictable signatures.
- Do not expect the lite package to infer `Session().client("acm")` automatically. Add explicit `ACMClient` annotations.
- Do not import the hyphenated package name in code. The import root uses underscores: `mypy_boto3_acm`.
- Do not expect a typed ACM resource interface. ACM is exposed through a client, one paginator, one waiter, literals, and `type_defs`.
- Do not assume type stubs validate AWS credentials, IAM permissions, regional availability, or certificate state. They only improve static typing.

## Version-Sensitive Notes

- PyPI lists `mypy-boto3-acm 1.42.3` as the latest release on March 12, 2026, released on December 4, 2025.
- The hosted docs site is ahead of the published package for this entry: on March 12, 2026 it showed generation commands for `boto3==1.42.61` while PyPI still published `1.42.3`. When those disagree, prefer the installed package version and PyPI metadata as the source of truth for exact compatibility.
- PyPI states this package was generated with `mypy-boto3-builder 8.12.0`.
- Based on the maintainer repository's migration guidance for `boto3-stubs`, newer `types-boto3` packages use `types_boto3_<service>` import roots instead of `mypy_boto3_<service>`. If your team migrates to that newer family, update imports accordingly.

## Official Sources

- PyPI package page: `https://pypi.org/project/mypy-boto3-acm/`
- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_acm/`
- AWS ACM boto3 reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/acm.html`
- Maintainer repository: `https://github.com/youtype/types-boto3`
