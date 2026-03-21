---
name: mypy-boto3-timestream-write
description: "Type stubs for boto3 Amazon Timestream Write clients, literals, and TypedDict request and response shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,timestream,boto3,type-stubs,mypy,pyright,python"
---

# mypy-boto3-timestream-write Python Package Guide

## Golden Rule

Use `boto3` for real Amazon Timestream for LiveAnalytics write calls and use `mypy-boto3-timestream-write` only for typing. If you want `Session.client("timestream-write")` to infer automatically, install `boto3-stubs[timestream-write]`; if you install only the standalone or lite package, annotate `TimestreamWriteClient` explicitly.

## Install

Recommended when you want automatic `Session.client(...)` typing:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[timestream-write]==1.42.3"
```

Lower-memory option:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[timestream-write]==1.42.3"
```

Standalone Timestream Write stubs only:

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-timestream-write==1.42.3"
```

Generate stubs locally when you need exact parity with a pinned boto3 build:

```bash
uvx --with "boto3==1.42.3" mypy-boto3-builder
```

Practical install rule:

- `boto3-stubs[timestream-write]`: best inference for `Session.client("timestream-write")`
- `boto3-stubs-lite[timestream-write]`: smaller install, but you should annotate the client explicitly
- `mypy-boto3-timestream-write`: service-only stubs without the bundled overload helpers

## Setup And AWS Auth

This package does not add its own auth or config layer. Credentials, region, retry behavior, custom endpoints, and profiles still come from normal `boto3` and `botocore` configuration.

Typical local setup:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Typed client setup:

```python
from boto3.session import Session
from mypy_boto3_timestream_write.client import TimestreamWriteClient

session = Session(profile_name="dev", region_name="us-east-1")
timestream_write: TimestreamWriteClient = session.client("timestream-write")
```

If the code type-checks but requests fail, debug the normal boto3 credential chain, IAM permissions, VPC endpoint routing, or Timestream service configuration rather than the stubs package.

## Core Usage

### Typed client for `write_records`

Timestream `MeasureValue` values are strings even when the logical type is numeric. Keep the measure type in `MeasureValueType` and stringify the actual values.

```python
from boto3.session import Session
from mypy_boto3_timestream_write.client import TimestreamWriteClient

client: TimestreamWriteClient = Session(region_name="us-east-1").client(
    "timestream-write"
)

response = client.write_records(
    DatabaseName="metrics",
    TableName="cpu_utilization",
    CommonAttributes={
        "Dimensions": [{"Name": "host", "Value": "web-1"}],
        "MeasureName": "cpu",
        "MeasureValueType": "DOUBLE",
        "TimeUnit": "SECONDS",
    },
    Records=[
        {"Time": "1731456000", "MeasureValue": "18.25"},
        {"Time": "1731456060", "MeasureValue": "19.75"},
    ],
)

print(response["RecordsIngested"]["Total"])
```

The AWS `WriteRecords` docs note that `CommonAttributes` dimensions must not overlap with per-record dimensions, and updates to an existing record must use a higher `Version` or the service can reject the write.

### Endpoint discovery and custom endpoint handling

The AWS Timestream endpoint-discovery guide says SDKs can discover ingestion endpoints automatically. `describe_endpoints()` is mainly useful when you need visibility into the resolved address or are managing custom endpoint behavior.

```python
from boto3.session import Session
from mypy_boto3_timestream_write.client import TimestreamWriteClient

client: TimestreamWriteClient = Session(region_name="us-east-1").client(
    "timestream-write"
)

endpoint = client.describe_endpoints()["Endpoints"][0]
print(endpoint["Address"])
print(endpoint["CachePeriodInMinutes"])
```

If you manually cache endpoints or pass a custom `endpoint_url`, respect the returned cache period and keep the region aligned with the target Timestream resources.

### Start a batch load task from S3

Use batch load for larger backfills instead of pushing many individual `write_records` calls.

```python
from boto3.session import Session
from mypy_boto3_timestream_write.client import TimestreamWriteClient

client: TimestreamWriteClient = Session(region_name="us-east-1").client(
    "timestream-write"
)

task = client.create_batch_load_task(
    DataModelConfiguration={
        "DataModel": {
            "TimeColumn": "time",
            "TimeUnit": "MILLISECONDS",
            "DimensionMappings": [
                {"SourceColumn": "host", "DestinationColumn": "host"},
            ],
            "MultiMeasureMappings": {
                "TargetMultiMeasureName": "metrics",
                "MultiMeasureAttributeMappings": [
                    {
                        "SourceColumn": "cpu",
                        "TargetMultiMeasureAttributeName": "cpu",
                        "MeasureValueType": "DOUBLE",
                    }
                ],
            },
        }
    },
    DataSourceConfiguration={
        "DataSourceS3Configuration": {
            "BucketName": "my-import-bucket",
            "ObjectKeyPrefix": "timestream/cpu/",
        },
        "CsvConfiguration": {"ColumnSeparator": ","},
        "DataFormat": "CSV",
    },
    ReportConfiguration={
        "ReportS3Configuration": {
            "BucketName": "my-import-bucket",
            "ObjectKeyPrefix": "timestream/reports/",
        }
    },
    TargetDatabaseName="metrics",
    TargetTableName="cpu_utilization",
)

print(task["TaskId"])
```

The AWS batch-load API requires both an S3 data source and an S3 report location. Check `describe_batch_load_task(...)` or `list_batch_load_tasks(...)` to monitor progress and failures.

### `TYPE_CHECKING` pattern for dev-only stubs

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_timestream_write.client import TimestreamWriteClient
else:
    TimestreamWriteClient = object

def make_client() -> "TimestreamWriteClient":
    return boto3.client("timestream-write", region_name="us-east-1")
```

This keeps stub imports out of runtime-only environments while preserving type checking.

### Literals and `TypedDict` shapes

The published maintainer docs for this package expose `client`, `literals`, `type_defs`, and example sections. Use literal aliases for enum-like strings and `TypedDict` definitions for helpers that pass request fragments around.

```python
from mypy_boto3_timestream_write.literals import BatchLoadDataFormatType
from mypy_boto3_timestream_write.type_defs import BatchLoadProgressReportTypeDef

data_format: BatchLoadDataFormatType = "CSV"

def processed_records(report: BatchLoadProgressReportTypeDef) -> int:
    return report.get("RecordsProcessed", 0)
```

## Configuration Notes

- Keep `boto3`, `botocore`, and the stubs package on the same release line when possible. The maintainer docs say the service-stub packages use the same version as the related `boto3` release.
- `timestream-write` is the boto3 service name. Do not use the PyPI package name in `session.client(...)`.
- Timestream Write does not have a boto3 resource interface. Plan around the typed client and generated request and response shapes.
- SDK endpoint discovery is normal for this service. If your environment overrides endpoints manually, test ingestion carefully.
- Batch load uses S3 source objects plus S3 error-report output; it is not a drop-in replacement for `write_records`.

## Common Pitfalls

- Treating `mypy-boto3-timestream-write` as a runtime SDK. Real AWS calls still require `boto3`.
- Installing only `mypy-boto3-timestream-write` or `boto3-stubs-lite[timestream-write]` and expecting `Session.client("timestream-write")` overload inference to appear automatically.
- Forgetting that `MeasureValue` is serialized as a string even when `MeasureValueType` is numeric.
- Duplicating the same dimension name in both `CommonAttributes` and individual records. The AWS `WriteRecords` API rejects overlapping dimensions.
- Updating an existing record without increasing `Version`, which can trigger `RejectedRecordsException`.
- Assuming batch-load tasks are synchronous. They need task-status polling and S3 report inspection.

## Version-Sensitive Notes

- The version used here `1.42.3` matches the official PyPI release and the maintainer docs checked on March 12, 2026.
- PyPI lists `mypy-boto3-timestream-write 1.42.3` as the current release, published on December 4, 2025, with `Requires: Python >=3.9`.
- The PyPI package page says this package was generated by `mypy-boto3-builder 8.12.0`.
- The generated docs root is a stable package URL, not a release-pinned docs URL. Pin the package version in your environment when exact patch parity matters.

## Official Sources

- Maintainer docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_timestream_write/
- Maintainer versioning guide: https://youtype.github.io/boto3_stubs_docs/#versioning
- PyPI package page: https://pypi.org/project/mypy-boto3-timestream-write/
- PyPI release page: https://pypi.org/project/mypy-boto3-timestream-write/1.42.3/
- Boto3 Timestream Write service reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/timestream-write.html
- Boto3 `write_records` reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/timestream-write/client/write_records.html
- Boto3 `create_batch_load_task` reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/timestream-write/client/create_batch_load_task.html
- Boto3 credentials guide: https://docs.aws.amazon.com/boto3/latest/guide/credentials.html
- Boto3 configuration guide: https://docs.aws.amazon.com/boto3/latest/guide/configuration.html
- AWS endpoint-discovery guide: https://docs.aws.amazon.com/timestream/latest/developerguide/Using-API.endpoint-discovery.describe-endpoints.implementation.html
