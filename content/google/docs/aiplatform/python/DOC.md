---
name: aiplatform
description: "Google Cloud Vertex AI Python SDK and client library for training, pipelines, prediction, and Vertex AI resource management"
metadata:
  languages: "python"
  versions: "1.141.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,vertex-ai,ml,genai,pipelines,prediction"
---

# Google Cloud Vertex AI Python Package Guide

## Golden Rule

Use `google-cloud-aiplatform` for Vertex AI work in Python, authenticate with Application Default Credentials (ADC), and call `aiplatform.init(...)` once before creating jobs or resources.

Prefer the high-level SDK first:

- `from google.cloud import aiplatform` for most Vertex AI workflows
- `from google.cloud import aiplatform_v1` or `from google.cloud.aiplatform import gapic` when you need lower-level API coverage
- `import vertexai.preview` only for preview-only surfaces

## Install

Pin the package version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "google-cloud-aiplatform==1.141.0"
```

Common alternatives:

```bash
uv add "google-cloud-aiplatform==1.141.0"
poetry add "google-cloud-aiplatform==1.141.0"
```

PyPI also publishes extras for common workflows. The most useful ones for agents are:

```bash
python -m pip install "google-cloud-aiplatform[pipelines]==1.141.0"
python -m pip install "google-cloud-aiplatform[prediction]==1.141.0"
python -m pip install "google-cloud-aiplatform[full]==1.141.0"
```

If you are not sure which extra you need, start with the base package and only add extras when the upstream docs or import errors require them.

## Project And Auth Setup

Vertex AI expects a Google Cloud project, the Vertex AI API enabled, and ADC available to the process.

Enable the API:

```bash
gcloud services enable aiplatform.googleapis.com
```

For local development, create ADC with your user account:

```bash
gcloud auth application-default login
```

For production on Google Cloud, prefer an attached service account over a downloaded key file.

If you must use a service account key locally, point ADC at it:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/absolute/path/service-account.json"
```

Useful environment variables:

```bash
export GOOGLE_CLOUD_PROJECT="your-project-id"
export GOOGLE_CLOUD_LOCATION="us-central1"
export GOOGLE_CLOUD_STAGING_BUCKET="gs://your-staging-bucket"
```

## Initialize The SDK

Initialize once near process startup and reuse that configuration:

```python
import os

from google.cloud import aiplatform

PROJECT_ID = os.environ["GOOGLE_CLOUD_PROJECT"]
LOCATION = os.getenv("GOOGLE_CLOUD_LOCATION", "us-central1")
STAGING_BUCKET = os.getenv("GOOGLE_CLOUD_STAGING_BUCKET")

aiplatform.init(
    project=PROJECT_ID,
    location=LOCATION,
    staging_bucket=STAGING_BUCKET,
)
```

Why this matters:

- `project` and `location` determine where resources are created and looked up
- `staging_bucket` is reused by pipelines, training jobs, and other artifact-producing workflows
- any default service account, network, or CMEK settings should be set before creating resources

## Core Usage Patterns

### Reference an existing endpoint and send predictions

Use the resource-name constructor when the endpoint already exists:

```python
from google.cloud import aiplatform

endpoint = aiplatform.Endpoint(
    "projects/PROJECT_ID/locations/us-central1/endpoints/ENDPOINT_ID"
)

prediction = endpoint.predict(
    instances=[
        [6.4, 3.1, 5.5, 1.8],
        [5.1, 3.5, 1.4, 0.2],
    ]
)

print(prediction.predictions)
```

The instance shape must match the schema of the deployed model. Agents should not invent request bodies here.

### Submit a pipeline job

```python
from google.cloud import aiplatform

job = aiplatform.PipelineJob(
    display_name="daily-train",
    template_path="gs://my-bucket/pipelines/train.yaml",
    pipeline_root="gs://my-bucket/pipeline-root",
    parameter_values={
        "project": "your-project-id",
        "location": "us-central1",
    },
    enable_caching=True,
)

job.run(sync=True)
```

Use `sync=False` if your app should submit and return immediately, then poll job state later.

### Track experiment params and metrics

```python
from google.cloud import aiplatform

aiplatform.init(
    project="your-project-id",
    location="us-central1",
    experiment="baseline-experiment",
)

with aiplatform.start_run("xgboost-baseline"):
    aiplatform.log_params({"learning_rate": 0.1, "max_depth": 6})
    aiplatform.log_metrics({"accuracy": 0.94, "f1": 0.91})
```

### Drop to lower-level clients when needed

Use the lower-level API clients when the high-level wrapper has not exposed a new field or resource yet:

```python
from google.cloud import aiplatform_v1

job_client = aiplatform_v1.JobServiceClient()
model_client = aiplatform_v1.ModelServiceClient()
```

This package also exposes GAPIC clients through `google.cloud.aiplatform.gapic`, but most application code should start with the higher-level `aiplatform` module first.

## Config Notes For Agents

- Vertex AI is regional. Keep `location`, storage buckets, endpoints, and resource names aligned.
- Initialize once per process instead of scattering different `aiplatform.init(...)` calls across modules.
- Many helpers accept explicit `credentials`, `project`, `location`, `network`, `service_account`, or `encryption_spec_key_name` overrides. Prefer explicit overrides only when one job needs behavior different from the process default.
- Long-running operations are common. Use `sync=True` when your code depends on the result immediately, or keep the returned resource/job object and poll state explicitly.
- Preview features may require `vertexai.preview` imports and can change faster than the stable `aiplatform` surface.

## Common Pitfalls

- `gcloud auth login` is not enough for local SDK code. Use `gcloud auth application-default login` so ADC works in Python.
- Do not hard-code service account key files into repos or container images. Prefer attached identities on GCP.
- The package contains multiple namespaces. `google.cloud.aiplatform` is the normal entry point; do not mix random snippets from `aiplatform_v1beta1`, `gapic`, and `vertexai.preview` unless you know why.
- Region mismatches cause many confusing failures. A pipeline in `us-central1` with a bucket or endpoint in another region is a common source of errors.
- Pipeline and training examples often assume a staging bucket already exists. If `staging_bucket` is missing, artifact-heavy workflows usually fail later than you expect.
- Endpoint prediction payloads are model-specific. The SDK will not infer your instance schema from the endpoint resource name.
- Many online examples still use older generative APIs in this package. Check the deprecation note below before copying them.

## Version-Sensitive Notes

- PyPI currently lists `google-cloud-aiplatform 1.141.0` with `Python >=3.9`.
- Some Google-hosted reference pages and search snippets lag the actual PyPI package version. Treat PyPI as the source of truth for package-version metadata and the Vertex AI docs/reference pages as the source of truth for behavior and examples.
- Google has deprecated several generative AI modules in this package, including `vertexai.generative_models`, `vertexai.language_models`, `vertexai.vision_models`, `vertexai.tuning`, and `vertexai.caching`. The official deprecation page says they were deprecated on June 24, 2025 and are scheduled for removal on June 24, 2026.
- For new Gemini and generative AI application code, prefer the `google-genai` SDK and avoid starting fresh work on those deprecated `vertexai.*` generative modules.

## Official Sources Used

- PyPI package page: `https://pypi.org/project/google-cloud-aiplatform/`
- PyPI JSON metadata: `https://pypi.org/pypi/google-cloud-aiplatform/json`
- Vertex AI Python reference root: `https://docs.cloud.google.com/python/docs/reference/aiplatform/latest`
- Vertex AI install guide: `https://cloud.google.com/vertex-ai/docs/start/install-sdk`
- Vertex AI authentication guide: `https://cloud.google.com/vertex-ai/docs/authentication`
- Vertex AI Python SDK guide: `https://cloud.google.com/vertex-ai/docs/python-sdk/use-vertex-ai-python-sdk`
- Generative SDK deprecation notice: `https://cloud.google.com/vertex-ai/generative-ai/docs/deprecations/genai-vertexai-sdk`
- Official repository: `https://github.com/googleapis/python-aiplatform`
