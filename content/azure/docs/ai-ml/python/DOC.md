---
name: ai-ml
description: "Azure Machine Learning Python SDK for workspace access, jobs, models, pipelines, and online endpoints"
metadata:
  languages: "python"
  versions: "1.31.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,azure-machine-learning,azure-ai-ml,mlops,jobs,pipelines,models,endpoints"
---

# Azure Machine Learning Python SDK

## Golden Rule

Use `azure-ai-ml` as the control-plane SDK for Azure Machine Learning v2, authenticate it with `azure-identity`, and center most code around `MLClient`. Treat your local Python environment as the orchestration environment only; training and inference runtimes must be declared explicitly in Azure ML jobs or deployments.

## Install

Install the ML SDK and the Azure Identity credential package together:

```bash
python -m pip install "azure-ai-ml==1.31.0" azure-identity
```

Common alternatives:

```bash
uv add "azure-ai-ml==1.31.0" azure-identity
poetry add "azure-ai-ml==1.31.0" azure-identity
```

You also need:

- an Azure subscription
- an Azure Machine Learning workspace
- permission to the subscription, resource group, and workspace

## Authentication And Workspace Setup

The default path is `DefaultAzureCredential()` plus `MLClient(...)`.

```python
from azure.ai.ml import MLClient
from azure.identity import DefaultAzureCredential

ml_client = MLClient(
    credential=DefaultAzureCredential(),
    subscription_id="<subscription-id>",
    resource_group_name="<resource-group>",
    workspace_name="<workspace-name>",
)
```

If your repo or notebook environment already has a workspace `config.json`, `from_config()` is the shortest path:

```python
from azure.ai.ml import MLClient
from azure.identity import DefaultAzureCredential

ml_client = MLClient.from_config(
    credential=DefaultAzureCredential(),
)
```

Microsoft documents this `config.json` shape:

```json
{
  "subscription_id": "<subscription-id>",
  "resource_group": "<resource-group>",
  "workspace_name": "<workspace-name>"
}
```

Notes:

- The pipeline guide uses `DefaultAzureCredential()` first and falls back to `InteractiveBrowserCredential()` if token acquisition fails in local development.
- For sovereign clouds, Microsoft documents passing both `cloud=` to `MLClient` and the matching `authority=` to `DefaultAzureCredential`.
- `MLClient` accepts `enable_telemetry=False`; the package docs say telemetry is only collected in Jupyter Notebook usage and is forced off outside Jupyter.

## Core Usage

### Submit a command job

Use `command(...)` for the common "run this script on Azure ML" flow.

```python
from azure.ai.ml import Input, command

job = command(
    code="./src",
    command="python train.py --input_data ${{inputs.training_data}}",
    environment="azureml://registries/azureml/environments/sklearn-1.5/labels/latest",
    compute="cpu-cluster",
    inputs={
        "training_data": Input(
            type="uri_file",
            path="https://azuremlexamples.blob.core.windows.net/datasets/credit_card/default_of_credit_card_clients.csv",
        )
    },
    display_name="credit-default-train",
)

created_job = ml_client.jobs.create_or_update(job)
ml_client.jobs.stream(created_job.name)
```

Use this for training, preprocessing, evaluation, or batch-style scripts that need Azure-managed compute and environments.

### Compose a pipeline from reusable components

Use `load_component(...)` and the pipeline DSL when you need reusable multi-step workflows.

```python
from azure.ai.ml import Input, load_component
from azure.ai.ml.dsl import pipeline

train_component = load_component(source="./components/train.yml")

@pipeline(name="train_pipeline")
def train_pipeline(training_data):
    train_step = train_component(training_data=training_data)
    return {"model_output": train_step.outputs.model_output}

pipeline_job = train_pipeline(
    training_data=Input(type="uri_folder", path="azureml:my-training-data:1")
)

submitted = ml_client.jobs.create_or_update(
    pipeline_job,
    experiment_name="pipeline_samples",
)
ml_client.jobs.stream(submitted.name)
```

### Register a model asset

Register models explicitly when you want stable references for deployment or reuse.

```python
from azure.ai.ml.constants import AssetTypes
from azure.ai.ml.entities import Model

model = Model(
    path="./model",
    name="fraud-model",
    type=AssetTypes.CUSTOM_MODEL,
    description="Fraud model packaged for deployment",
)

registered_model = ml_client.models.create_or_update(model)
print(registered_model.id)
```

The model-management docs also support registering from datastore paths, `runs:/...`, or `azureml://jobs/...` URIs when the model comes from job outputs.

### Create an online endpoint and deployment

Use managed online endpoints for real-time inference. Endpoint and deployment creation are long-running operations, so wait on the poller result.

```python
import uuid

from azure.ai.ml import MLClient
from azure.ai.ml.entities import (
    CodeConfiguration,
    ManagedOnlineDeployment,
    ManagedOnlineEndpoint,
)

endpoint_name = f"credit-endpoint-{str(uuid.uuid4())[:8]}"

endpoint = ManagedOnlineEndpoint(
    name=endpoint_name,
    description="Real-time fraud scoring endpoint",
    auth_mode="key",
)
endpoint = ml_client.online_endpoints.begin_create_or_update(endpoint).result()

deployment = ManagedOnlineDeployment(
    name="blue",
    endpoint_name=endpoint_name,
    model="azureml:fraud-model:1",
    environment="azureml:fraud-inference-env:1",
    code_configuration=CodeConfiguration(
        code="./deploy",
        scoring_script="score.py",
    ),
    instance_type="Standard_DS3_v2",
    instance_count=1,
)

deployment = ml_client.begin_create_or_update(deployment).result()
```

Invoke the deployment with a request JSON file:

```python
result = ml_client.online_endpoints.invoke(
    endpoint_name=endpoint_name,
    deployment_name="blue",
    request_file="./deploy/sample-request.json",
)

print(result)
```

## Configuration And Auth Checklist

- Install both `azure-ai-ml` and `azure-identity`; the ML SDK does not replace the credential package.
- Keep the workspace triple available: subscription ID, resource group, workspace name.
- Prefer `DefaultAzureCredential()` for local dev, CI, and Azure-hosted runtimes unless your environment requires a specific credential type.
- Use a checked-in or generated `config.json` only if the workspace identity is safe to keep in the repo; never commit secrets.
- Define runtime environments for jobs and deployments explicitly instead of assuming local packages are present remotely.
- Ensure compute exists before submitting jobs unless your workflow is intentionally serverless.

## Common Pitfalls

- The install name and import path differ: install `azure-ai-ml`, import from `azure.ai.ml`.
- `MLClient.from_config()` fails if `config.json` is missing or malformed.
- `create_or_update()` and `begin_create_or_update()` are not interchangeable. Jobs and many asset operations use `create_or_update()`, while endpoints, deployments, compute, schedules, registries, and workspaces commonly use the long-running `begin_create_or_update()` flow.
- A job spec without an explicit Azure ML environment is brittle. The remote runtime is not inferred from your local venv.
- For online deployments, you need a model, an inference environment, a scoring script, and usually traffic routing. Creating just the endpoint is not enough.
- Quota and SKU availability errors are common for online deployments. Microsoft explicitly notes that you may need to switch `instance_type` if the chosen VM is not available.
- Azure ML exceptions come from Azure Core in many cases, so catch `azure.core.exceptions.HttpResponseError` for API failures.

## Version-Sensitive Notes

- PyPI currently lists `1.31.0` released on `2025-12-30`.
- Upstream Python-version guidance is inconsistent. The PyPI project metadata still says "Python 3.7 or later" and "tested with Python 3.8 ... 3.14", but the `1.31.0` release history says support for Python `3.7` and `3.8` was dropped and Python `3.14` was added. For new `1.31.0` work, treat `3.9+` as the safe baseline.
- `1.31.0` adds `default_deployment_template` support on `Model` entities for online deployments.
- `1.30.0` removed the `msrest` and `six` dependencies. If older internal tooling relied on those transitive packages, re-test after upgrading.

## Official Sources

- PyPI project and release history: `https://pypi.org/project/azure-ai-ml/`
- PyPI JSON metadata: `https://pypi.org/pypi/azure-ai-ml/json`
- Azure AI ML Python API root: `https://learn.microsoft.com/en-us/python/api/azure-ai-ml/`
- `MLClient` reference: `https://learn.microsoft.com/en-us/python/api/azure-ai-ml/azure.ai.ml.mlclient?view=azure-python`
- Package reference for `command`, `Input`, and `load_component`: `https://learn.microsoft.com/en-us/python/api/azure-ai-ml/azure.ai.ml?view=azure-python`
- Pipeline how-to: `https://learn.microsoft.com/en-us/azure/machine-learning/how-to-create-component-pipeline-python?view=azureml-api-2`
- Model management guide: `https://learn.microsoft.com/en-us/azure/machine-learning/how-to-manage-models?view=azureml-api-2`
- Online endpoint quickstart: `https://learn.microsoft.com/en-us/azure/machine-learning/tutorial-azure-ml-in-a-day?view=azureml-api-2`
- Online endpoint deployment guide: `https://learn.microsoft.com/en-us/azure/machine-learning/how-to-deploy-online-endpoints?view=azureml-api-2`
