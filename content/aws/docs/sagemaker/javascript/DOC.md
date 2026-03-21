---
name: sagemaker
description: "AWS SDK for JavaScript v3 client for managing Amazon SageMaker jobs, models, endpoints, and notebook resources"
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,sagemaker,machine-learning,training,endpoints"
---

# Amazon SageMaker SDK for JavaScript (v3)

Use `@aws-sdk/client-sagemaker` to manage Amazon SageMaker control-plane resources from JavaScript or TypeScript code: training jobs, models, endpoint configs, endpoints, notebook instances, and related metadata.

## Golden Rule

- Install `@aws-sdk/client-sagemaker`, not the legacy `aws-sdk` v2 package.
- This doc covers package version `3.1006.0`.
- Prefer `SageMakerClient` plus individual commands over the aggregated `SageMaker` client.
- `@aws-sdk/client-sagemaker` is the control-plane client. To invoke a deployed model endpoint, use `@aws-sdk/client-sagemaker-runtime`.
- Most create, update, stop, and delete operations are asynchronous. A successful response usually means SageMaker accepted the request, not that the resource is already ready.
- SageMaker job names, model names, endpoint config names, and endpoint names are separate identifiers. Keep them explicit and stable in your deployment flow.
- Many request fields depend on IAM roles, S3 URIs, container images, and regions lining up correctly.

## Install

```bash
npm install @aws-sdk/client-sagemaker
```

Common companion packages:

```bash
npm install @aws-sdk/client-sagemaker-runtime @aws-sdk/credential-providers
```

## Client Setup

### Minimal Node.js client

```javascript
import { SageMakerClient } from "@aws-sdk/client-sagemaker";

const sagemaker = new SageMakerClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

### Explicit credentials

```javascript
import { SageMakerClient } from "@aws-sdk/client-sagemaker";

const sagemaker = new SageMakerClient({
  region: "us-east-1",
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  },
});
```

If your environment already uses the standard AWS credential chain, keep the client config minimal and set only the region in code.

## Core Usage Pattern

The normal v3 flow is `client.send(new Command(input))`.

```javascript
import {
  DescribeTrainingJobCommand,
  SageMakerClient,
} from "@aws-sdk/client-sagemaker";

const sagemaker = new SageMakerClient({ region: "us-east-1" });

const response = await sagemaker.send(
  new DescribeTrainingJobCommand({
    TrainingJobName: "xgboost-demo-20260311",
  }),
);

console.log({
  status: response.TrainingJobStatus,
  secondaryStatus: response.SecondaryStatus,
  modelArtifacts: response.ModelArtifacts?.S3ModelArtifacts,
});
```

For most SageMaker automation, the workflow is create a resource, then poll the matching `Describe*` API until the status reaches the state you need.

## Common Operations

### Start a training job

```javascript
import {
  CreateTrainingJobCommand,
  SageMakerClient,
} from "@aws-sdk/client-sagemaker";

const sagemaker = new SageMakerClient({ region: "us-east-1" });

await sagemaker.send(
  new CreateTrainingJobCommand({
    TrainingJobName: "xgboost-demo-20260311",
    RoleArn: "arn:aws:iam::123456789012:role/service-role/AmazonSageMaker-ExecutionRole",
    AlgorithmSpecification: {
      TrainingImage: "123456789012.dkr.ecr.us-east-1.amazonaws.com/my-training-image:latest",
      TrainingInputMode: "File",
    },
    InputDataConfig: [
      {
        ChannelName: "train",
        DataSource: {
          S3DataSource: {
            S3DataType: "S3Prefix",
            S3Uri: "s3://my-bucket/sagemaker/train/",
            S3DataDistributionType: "FullyReplicated",
          },
        },
      },
    ],
    OutputDataConfig: {
      S3OutputPath: "s3://my-bucket/sagemaker/output/",
    },
    ResourceConfig: {
      InstanceCount: 1,
      InstanceType: "ml.m5.large",
      VolumeSizeInGB: 30,
    },
    StoppingCondition: {
      MaxRuntimeInSeconds: 3600,
    },
    HyperParameters: {
      epochs: "3",
      learning_rate: "0.2",
    },
  }),
);
```

`CreateTrainingJob` only queues the work. Use `DescribeTrainingJobCommand` to check `TrainingJobStatus`, inspect `FailureReason`, and read `ModelArtifacts` after the job completes.

### List recent training jobs

```javascript
import {
  ListTrainingJobsCommand,
  SageMakerClient,
} from "@aws-sdk/client-sagemaker";

const sagemaker = new SageMakerClient({ region: "us-east-1" });

const response = await sagemaker.send(
  new ListTrainingJobsCommand({
    StatusEquals: "Completed",
    SortBy: "CreationTime",
    SortOrder: "Descending",
    MaxResults: 20,
  }),
);

for (const job of response.TrainingJobSummaries ?? []) {
  console.log(job.TrainingJobName, job.TrainingJobStatus);
}
```

List APIs return summaries, not full details. Call the related `Describe*` command when you need complete configuration, artifact, or failure metadata.

### Register a model and create an endpoint

Creating an online inference endpoint is typically a three-step flow: create the model, create the endpoint config, then create the endpoint.

```javascript
import {
  CreateEndpointCommand,
  CreateEndpointConfigCommand,
  CreateModelCommand,
  SageMakerClient,
} from "@aws-sdk/client-sagemaker";

const sagemaker = new SageMakerClient({ region: "us-east-1" });

const modelName = "xgboost-demo-model";
const endpointConfigName = "xgboost-demo-endpoint-config";
const endpointName = "xgboost-demo-endpoint";

await sagemaker.send(
  new CreateModelCommand({
    ModelName: modelName,
    ExecutionRoleArn: "arn:aws:iam::123456789012:role/service-role/AmazonSageMaker-ExecutionRole",
    PrimaryContainer: {
      Image: "123456789012.dkr.ecr.us-east-1.amazonaws.com/my-inference-image:latest",
      ModelDataUrl: "s3://my-bucket/sagemaker/output/model.tar.gz",
    },
  }),
);

await sagemaker.send(
  new CreateEndpointConfigCommand({
    EndpointConfigName: endpointConfigName,
    ProductionVariants: [
      {
        VariantName: "AllTraffic",
        ModelName: modelName,
        InitialInstanceCount: 1,
        InstanceType: "ml.m5.large",
        InitialVariantWeight: 1,
      },
    ],
  }),
);

await sagemaker.send(
  new CreateEndpointCommand({
    EndpointName: endpointName,
    EndpointConfigName: endpointConfigName,
  }),
);
```

The endpoint is not ready immediately after `CreateEndpoint`. Poll `DescribeEndpointCommand` until `EndpointStatus` reaches `InService` before sending live traffic.

### Check endpoint rollout status

```javascript
import {
  DescribeEndpointCommand,
  SageMakerClient,
} from "@aws-sdk/client-sagemaker";

const sagemaker = new SageMakerClient({ region: "us-east-1" });

const response = await sagemaker.send(
  new DescribeEndpointCommand({
    EndpointName: "xgboost-demo-endpoint",
  }),
);

console.log({
  status: response.EndpointStatus,
  failureReason: response.FailureReason,
});
```

Treat endpoint creation and update like long-running infrastructure work. A 200-level response only confirms the rollout started.

### List notebook instances

```javascript
import {
  ListNotebookInstancesCommand,
  SageMakerClient,
} from "@aws-sdk/client-sagemaker";

const sagemaker = new SageMakerClient({ region: "us-east-1" });

const response = await sagemaker.send(
  new ListNotebookInstancesCommand({
    StatusEquals: "InService",
    MaxResults: 20,
  }),
);

for (const instance of response.NotebookInstances ?? []) {
  console.log(instance.NotebookInstanceName, instance.NotebookInstanceStatus);
}
```

This is useful for admin tooling and fleet audits. Notebook instances are managed resources; they are separate from Studio domains, user profiles, and runtime inference endpoints.

## Auth and Configuration Notes

- SageMaker control-plane calls need normal AWS SigV4 credentials plus the correct `region`.
- Training jobs, processing jobs, and model creation often also depend on an execution role that SageMaker can assume separately from the caller identity making the API call.
- S3 inputs and outputs must be readable and writable by the execution role you pass in requests such as `CreateTrainingJob` and `CreateModel`.
- Container image URIs must point to images your account and region can actually pull, commonly from Amazon ECR.

## SageMaker-Specific Gotchas

- Use `@aws-sdk/client-sagemaker-runtime` for `InvokeEndpoint` and other data-plane inference calls. They are not exposed by `@aws-sdk/client-sagemaker`.
- Many create and stop APIs are asynchronous. Poll the related `Describe*` API and inspect status fields and failure reasons.
- `HyperParameters` values are strings, even when they represent numbers or booleans for your training container.
- Endpoint deployment is a multi-resource workflow: model, endpoint config, and endpoint are distinct resources with distinct names.
- SageMaker often fails because IAM, ECR, S3, KMS, VPC, or region settings do not line up. Validate those dependencies before debugging SDK wiring.
- List operations usually return summaries and pagination tokens rather than everything in one call.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/client-sagemaker-runtime`: invoke deployed endpoints and other inference data-plane operations.
- `@aws-sdk/credential-providers`: shared config, IAM Identity Center, Cognito, and assume-role credential helpers.
- `@aws-sdk/client-sts`: caller identity checks and explicit role-assumption workflows around SageMaker automation.

## Version Notes

- This content is written for `@aws-sdk/client-sagemaker` version `3.1006.0`.
- Examples use the modular AWS SDK for JavaScript v3 import pattern and `client.send(new Command(...))` calls.
- If you are migrating from v2 `aws-sdk`, do not translate examples into `new AWS.SageMaker()` service-method style calls.
