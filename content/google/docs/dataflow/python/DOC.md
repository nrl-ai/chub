---
name: dataflow
description: "Google Cloud Dataflow Python guide for the legacy google-cloud-dataflow package row and the current Dataflow client split"
metadata:
  languages: "python"
  versions: "2.5.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,dataflow,gcp,apache-beam,pipelines,templates"
---

# Google Cloud Dataflow Python Guide

## Golden Rule

Treat `google-cloud-dataflow==2.5.0` as a legacy package row, not the modern Python 3 SDK for Google Cloud Dataflow.

As of March 12, 2026:

- PyPI still lists `google-cloud-dataflow 2.5.0`, released on June 27, 2018, with `Requires: Python >=2.7, <3.0`.
- The official Python reference at `https://cloud.google.com/python/docs/reference/dataflow/latest` is for the separate `google-cloud-dataflow-client` package, not for `google-cloud-dataflow`.
- Google Cloud's current pipeline quickstart for Python uses `apache-beam[gcp]` to submit Beam pipelines to the Dataflow service.

If you are writing new Python 3 code:

- Use `apache-beam[gcp]` to author and run Dataflow pipelines.
- Use `google-cloud-dataflow-client` only when you need the Dataflow control-plane API for jobs, templates, metrics, messages, or snapshots.
- Do not install `google-cloud-dataflow` unless you are intentionally maintaining a legacy Python 2 codebase pinned to that distribution.

## Package Split And Upstream Drift

- Version used here: `2.5.0`
- Registry URL: `https://pypi.org/project/google-cloud-dataflow/`
- Docs URL: `https://cloud.google.com/python/docs/reference/dataflow/latest`

Those URLs do not describe the same published distribution:

- `google-cloud-dataflow` is the old PyPI package row tracked here.
- `google-cloud-dataflow-client` is the current generated Python client library documented at the Google Cloud reference URL.
- `apache-beam[gcp]` is the modern package used to build and submit Python pipelines to Dataflow.

This entry is therefore a resolution guide first: it tells agents which package to use for which task before showing code.

## Install

### Legacy Compatibility Only

Only use this if the project is explicitly pinned to the old `google-cloud-dataflow` package and can still run under Python 2:

```bash
python2 -m pip install "google-cloud-dataflow==2.5.0"
```

This is usually the wrong choice for current code.

### Authoring And Running Dataflow Pipelines

Use Apache Beam with Google Cloud extras:

```bash
python -m pip install "apache-beam[gcp]"
```

### Calling The Dataflow Service API

Use the current generated client library:

```bash
python -m pip install "google-cloud-dataflow-client"
```

The current Google Cloud Python reference page identifies `google-cloud-dataflow-client 0.11.0` as the latest published client library.

## Auth And Project Setup

For modern Python 3 usage, follow normal Google Cloud authentication and project setup:

1. Enable billing and the Dataflow API on the target Google Cloud project.
2. Authenticate with Application Default Credentials (ADC).
3. Choose a region up front.
4. Create a GCS bucket for staging and temporary files.

Local development with ADC:

```bash
gcloud auth application-default login
gcloud config set project MY_PROJECT_ID
```

Service account based auth:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
export GOOGLE_CLOUD_PROJECT="my-project-id"
```

For Beam jobs, you will usually also need:

```bash
export DATAFLOW_REGION="us-central1"
export DATAFLOW_STAGING_LOCATION="gs://my-bucket/dataflow/staging"
export DATAFLOW_TEMP_LOCATION="gs://my-bucket/dataflow/temp"
```

## Which API Surface To Use

### Use `apache-beam[gcp]` when you need to run a pipeline

Typical tasks:

- ETL or batch transforms
- streaming pipelines
- writing to BigQuery, Pub/Sub, GCS, or other Beam connectors
- local testing before running on Dataflow

Minimal pipeline options for a Dataflow run:

```python
from apache_beam.options.pipeline_options import PipelineOptions

options = PipelineOptions(
    runner="DataflowRunner",
    project="my-project-id",
    region="us-central1",
    temp_location="gs://my-bucket/dataflow/temp",
    staging_location="gs://my-bucket/dataflow/staging",
    job_name="wordcount-20260312",
)
```

The Dataflow quickstart and pipeline options docs make `apache-beam[gcp]` the authoritative starting point for new Python pipeline code.

### Use `google-cloud-dataflow-client` when you need to manage jobs or templates

Typical tasks:

- list or inspect running jobs
- launch classic templates or flex templates
- fetch job messages or metrics
- create snapshots or drain/cancel jobs through the API

Import path:

```python
from google.cloud import dataflow_v1beta3
```

## Core Usage With `google-cloud-dataflow-client`

### Create A Jobs Client

The generated docs recommend using regional endpoints for service calls. Keep the request `location` aligned with the region where the job runs.

```python
from google.api_core.client_options import ClientOptions
from google.cloud import dataflow_v1beta3

REGION = "us-central1"
REGIONAL_ENDPOINT = "YOUR_REGIONAL_DATAFLOW_ENDPOINT"

client = dataflow_v1beta3.JobsV1Beta3Client(
    client_options=ClientOptions(api_endpoint=REGIONAL_ENDPOINT)
)
```

### List Jobs

`list_jobs` returns job summaries. Use `get_job` when you need the full job record.

```python
from google.cloud import dataflow_v1beta3

client = dataflow_v1beta3.JobsV1Beta3Client()

request = dataflow_v1beta3.ListJobsRequest(
    project_id="my-project-id",
    location="us-central1",
)

for job in client.list_jobs(request=request):
    print(job)
```

### Get A Single Job

```python
from google.cloud import dataflow_v1beta3

client = dataflow_v1beta3.JobsV1Beta3Client()

request = dataflow_v1beta3.GetJobRequest(
    project_id="my-project-id",
    location="us-central1",
    job_id="2026-03-12_10_00_00-1234567890123456789",
)

job = client.get_job(request=request)
print(job)
```

### Launch A Classic Template

The template launch request takes a regional `location`, a GCS template path, launch parameters, and optional runtime environment values such as `temp_location` and `service_account_email`.

```python
from google.cloud import dataflow_v1beta3

client = dataflow_v1beta3.TemplatesServiceClient()

request = dataflow_v1beta3.LaunchTemplateRequest(
    project_id="my-project-id",
    location="us-central1",
    gcs_path="gs://dataflow-templates/latest/Word_Count",
    launch_parameters=dataflow_v1beta3.LaunchTemplateParameters(
        job_name="wordcount-template-run",
        parameters={
            "inputFile": "gs://dataflow-samples/shakespeare/kinglear.txt",
            "output": "gs://my-bucket/output/wordcount",
        },
        environment=dataflow_v1beta3.RuntimeEnvironment(
            temp_location="gs://my-bucket/dataflow/temp",
            service_account_email="dataflow-runner@my-project-id.iam.gserviceaccount.com",
        ),
    ),
    validate_only=False,
)

response = client.launch_template(request=request)
print(response)
```

Set `validate_only=True` first if you want the API to validate the request without launching a job.

## Config Notes For Beam Pipelines

When the task is "run this pipeline on Dataflow", the configuration that usually matters most is on the Beam side:

- `runner="DataflowRunner"`
- `project`
- `region`
- `temp_location`
- `staging_location`
- `job_name`

Useful optional flags to remember for Beam-based runs:

- `save_main_session=True` when worker imports depend on local module state
- `setup_file` or `requirements_file` when workers need extra dependencies
- `service_account_email` when jobs must run under a non-default identity
- `subnetwork`, `network`, and worker machine options for private networking or sizing

## Common Pitfalls

- Installing the wrong package is the biggest failure mode here. `google-cloud-dataflow` is not the current Python 3 client library.
- The legacy `google-cloud-dataflow` package is Python 2 only. For Python 3 projects, it is the wrong dependency.
- The docs URL in the queue points to `google-cloud-dataflow-client`, whose import path is `google.cloud.dataflow_v1beta3`. Do not assume the import name matches the old PyPI package name.
- The generated client docs repeatedly recommend regional endpoints. Do not mix a regional `location` with the wrong service endpoint.
- Template launch requests expect GCS paths such as `gs://...`; local filesystem paths will not work for `gcs_path` or `temp_location`.
- `projects.templates.launch` without a regional `location` defaults to `us-central1`; Google recommends the regional `projects.locations.templates.launch` form instead.
- `list_jobs` returns summaries and its `view` selector is deprecated. Call `get_job` if you need full details.
- `update_job` only updates job state. Do not treat it as a general patch endpoint.
- The Dataflow API docs warn not to send confidential information in raw string fields unless the transport itself is protected.

## Version-Sensitive Notes

- The frontmatter version for this entry remains `2.5.0` because that is the legacy PyPI package this guide covers.
- As of March 12, 2026, `google-cloud-dataflow 2.5.0` is still the latest release on PyPI for that legacy package, and it still requires Python `<3.0`.
- As of March 12, 2026, the Google Cloud Python reference page reports `google-cloud-dataflow-client 0.11.0` as the latest generated client library.
- If you are curating or debugging a modern Dataflow Python project, treat the package split explicitly:
  - pipeline authoring: `apache-beam[gcp]`
  - service management API: `google-cloud-dataflow-client`
  - legacy compatibility only: `google-cloud-dataflow==2.5.0`

## Official Sources Used

- PyPI legacy package: `https://pypi.org/project/google-cloud-dataflow/`
- Google Cloud Python reference root: `https://cloud.google.com/python/docs/reference/dataflow/latest`
- Dataflow client package page: `https://pypi.org/project/google-cloud-dataflow-client/`
- Dataflow Python guide and quickstart: `https://cloud.google.com/dataflow/docs/guides/installing-beam-sdk`
- Dataflow pipeline options: `https://cloud.google.com/dataflow/docs/guides/setting-pipeline-options`
- Dataflow authentication: `https://cloud.google.com/docs/authentication/application-default-credentials`
- Jobs client reference: `https://cloud.google.com/python/docs/reference/dataflow/latest/google.cloud.dataflow_v1beta3.services.jobs_v1_beta3.JobsV1Beta3Client`
- Templates service reference: `https://cloud.google.com/python/docs/reference/dataflow/latest/google.cloud.dataflow_v1beta3.services.templates_service.TemplatesServiceClient`
- Launch template request reference: `https://cloud.google.com/python/docs/reference/dataflow/latest/google.cloud.dataflow_v1beta3.types.LaunchTemplateRequest`
- Dataflow template launch REST docs: `https://cloud.google.com/dataflow/docs/reference/rest/v1b3/projects.locations.templates/launch`
