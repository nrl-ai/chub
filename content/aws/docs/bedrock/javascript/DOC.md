---
name: bedrock
description: "AWS SDK for JavaScript v3 Bedrock control-plane client for model discovery and Bedrock resource management."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,bedrock,javascript,nodejs,genai,foundation-models,llm,control-plane"
---

# `@aws-sdk/client-bedrock`

Use this package for the Amazon Bedrock **control plane** in AWS SDK for JavaScript v3. It is the package for discovering Bedrock models and managing Bedrock-side resources. It is **not** the package that sends prompts to models.

If you need to invoke a model, stream a response, or use chat-style inference APIs, use `@aws-sdk/client-bedrock-runtime` instead.

Prefer `BedrockClient` plus explicit command imports. The package also exposes an aggregated client, but command-based imports are the safer default for smaller bundles and clearer call sites.

## Install

```bash
npm install @aws-sdk/client-bedrock
```

Common companion packages:

```bash
npm install @aws-sdk/client-bedrock-runtime @aws-sdk/credential-providers
```

## What This Client Covers

Use `@aws-sdk/client-bedrock` for Bedrock management tasks such as:

- discovering which foundation models are available in a region
- reading model metadata such as provider, modalities, lifecycle status, and streaming support
- administrative Bedrock workflows that belong to the service control plane rather than runtime inference

Keep the boundary clear:

- `@aws-sdk/client-bedrock`: discovery and management
- `@aws-sdk/client-bedrock-runtime`: inference and response streaming

## Initialize the client

```javascript
import { BedrockClient } from "@aws-sdk/client-bedrock";

const bedrock = new BedrockClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

## Credentials and Region

- Node.js: the default credential provider chain usually works if AWS access is already configured through environment variables, shared AWS config files, ECS, EC2, or IAM Identity Center.
- Browser runtimes: avoid using this control-plane client directly in the browser unless you have a tightly scoped browser-safe credential story. Bedrock management operations are usually server-side.
- Region matters a lot with Bedrock. Model availability, account access, and related resources vary by region.
- Set the region in code, with `AWS_REGION`, or through shared AWS config.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

The v3 SDK uses `client.send(new Command(input))`.

```javascript
import {
  BedrockClient,
  ListFoundationModelsCommand,
} from "@aws-sdk/client-bedrock";

const bedrock = new BedrockClient({ region: "us-east-1" });

const response = await bedrock.send(new ListFoundationModelsCommand({}));

for (const model of response.modelSummaries ?? []) {
  console.log({
    modelId: model.modelId,
    modelName: model.modelName,
    providerName: model.providerName,
    inputModalities: model.inputModalities,
    outputModalities: model.outputModalities,
    streaming: model.responseStreamingSupported,
    lifecycleStatus: model.modelLifecycle?.status,
  });
}
```

For most application code, the control-plane client is the place where you discover or validate a model identifier before passing that identifier into your runtime inference flow.

## Common Operations

### Discover active text-capable models in a region

`ListFoundationModels` is the safest starting point when your code needs to discover which Bedrock model IDs are usable in a given region.

```javascript
import {
  BedrockClient,
  ListFoundationModelsCommand,
} from "@aws-sdk/client-bedrock";

const bedrock = new BedrockClient({ region: "us-east-1" });

const response = await bedrock.send(new ListFoundationModelsCommand({}));

const textModels = (response.modelSummaries ?? []).filter((model) => {
  const lifecycleStatus = model.modelLifecycle?.status?.toUpperCase();
  const outputModalities = (model.outputModalities ?? []).map((value) => value.toLowerCase());

  return lifecycleStatus === "ACTIVE" && outputModalities.includes("text");
});

for (const model of textModels) {
  console.log(model.modelId, model.providerName, model.responseStreamingSupported);
}
```

This is a practical way to avoid hard-coding assumptions about which providers or model IDs are enabled in a specific account and region.

### Choose a model ID for runtime inference

After discovery, keep only the metadata you actually need in application code and hand the selected model ID to your runtime client.

```javascript
import {
  BedrockClient,
  ListFoundationModelsCommand,
} from "@aws-sdk/client-bedrock";

const bedrock = new BedrockClient({ region: "us-east-1" });

const response = await bedrock.send(new ListFoundationModelsCommand({}));

const selectedModel = (response.modelSummaries ?? []).find((model) => {
  const outputs = (model.outputModalities ?? []).map((value) => value.toLowerCase());

  return model.modelLifecycle?.status?.toUpperCase() === "ACTIVE" && outputs.includes("text");
});

if (!selectedModel?.modelId) {
  throw new Error("No matching Bedrock model is available in this region/account");
}

console.log("Use this model ID with @aws-sdk/client-bedrock-runtime:", selectedModel.modelId);
```

The exact model you choose should be driven by your own access policy, region, provider requirements, and runtime feature needs.

## Bedrock-Specific Gotchas

- This package does **not** invoke models. If you are looking for prompt execution, streaming responses, or chat-style runtime calls, you want `@aws-sdk/client-bedrock-runtime`.
- Bedrock model availability is region-specific and account-specific. A model ID that works in one region or account may not be available in another.
- Discovery results are metadata, not proof that your IAM policy and Bedrock access configuration are sufficient for every downstream inference action.
- Check modalities before choosing a model. Some models are text-focused, some are multimodal, and not every model supports streaming.
- Control-plane credentials are usually privileged. Keep Bedrock administration code on the server side.
- The Bedrock service surface evolves quickly. Prefer explicit command imports and verify the exact resource workflow you need against the current AWS Bedrock documentation for your feature area.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/client-bedrock-runtime`: model invocation, streaming responses, and chat-style runtime APIs.
- `@aws-sdk/credential-providers`: shared config helpers, IAM Identity Center, STS assume-role flows, and other credential providers.

## Common Bedrock operations

### Print a compact inventory of model summaries

```javascript
import {
  BedrockClient,
  ListFoundationModelsCommand,
} from "@aws-sdk/client-bedrock";

const bedrock = new BedrockClient({ region: "us-east-1" });

const response = await bedrock.send(new ListFoundationModelsCommand({}));

const inventory = (response.modelSummaries ?? []).map((model) => ({
  modelId: model.modelId,
  name: model.modelName,
  provider: model.providerName,
  status: model.modelLifecycle?.status,
  inputs: model.inputModalities ?? [],
  outputs: model.outputModalities ?? [],
  streaming: model.responseStreamingSupported ?? false,
}));

console.table(inventory);
```

### Filter for streaming-capable text models

```javascript
import {
  BedrockClient,
  ListFoundationModelsCommand,
} from "@aws-sdk/client-bedrock";

const bedrock = new BedrockClient({ region: "us-east-1" });

const response = await bedrock.send(new ListFoundationModelsCommand({}));

const streamingTextModels = (response.modelSummaries ?? []).filter((model) => {
  const outputs = (model.outputModalities ?? []).map((value) => value.toLowerCase());
  return model.responseStreamingSupported === true && outputs.includes("text");
});

for (const model of streamingTextModels) {
  console.log(`${model.providerName}: ${model.modelId}`);
}
```

## Notes

- Treat the discovered `modelId` as configuration data. Resolve it once, store it where appropriate for your app, and re-check when you change regions or providers.
- If your real goal is application inference, keep this package at the discovery and admin edge of your system and move prompt execution into the runtime client.
