---
name: package
description: "Pulumi Python package guide for project setup, stack config, outputs, secrets, and state backends"
metadata:
  languages: "python"
  versions: "3.225.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pulumi,python,iac,infrastructure,cloud,devops"
---

# Pulumi Python Package Guide

## Golden Rule

`pulumi` is the core Python SDK, not the whole toolchain. For real Pulumi programs you usually need all three pieces:

- the Pulumi CLI
- the `pulumi` Python package
- one or more provider packages such as `pulumi-aws`, `pulumi-kubernetes`, or `pulumi-random`

In code, keep deployment logic declarative. Treat `Output` values as asynchronous deployment-time values, use `Config` for stack settings, and avoid calling `Output.get()` inside a normal `pulumi up` program.

## Version-Sensitive Notes

- This entry is pinned to the version used here `3.225.1`.
- As of March 12, 2026, PyPI lists `3.225.1` as the latest stable `pulumi` release. Newer `3.226.0b...` entries are prereleases.
- The Pulumi install page currently advertises CLI `3.225.1`, so the CLI and Python SDK stable versions are aligned right now.
- The Python API reference uses a `latest` docs URL. Use PyPI for exact package pinning and the Pulumi docs for behavior, project structure, and CLI workflow.

## Install

Install the CLI first, then the Python packages for your project.

CLI examples:

```bash
brew install pulumi/tap/pulumi
```

```bash
curl -fsSL https://get.pulumi.com | sh
```

For Python, pin the core SDK and add the provider packages your stack uses:

```bash
python -m pip install "pulumi==3.225.1"
python -m pip install "pulumi-aws"
```

Common alternatives:

```bash
uv add "pulumi==3.225.1"
poetry add "pulumi==3.225.1"
```

If the project already has a lockfile, follow that instead of mixing package manager conventions.

## Initialize A Python Project

The fastest clean start is a Pulumi template for Python:

```bash
mkdir infra
cd infra
python -m venv .venv
source .venv/bin/activate
pulumi login
pulumi new aws-python
pulumi config set aws:region us-west-2
pulumi up
```

The generated project usually includes:

- `Pulumi.yaml`: project metadata and runtime
- `Pulumi.<stack>.yaml`: stack-scoped config values
- `__main__.py`: entry point for the Python program
- `requirements.txt` or `pyproject.toml`: Python dependencies, including `pulumi` and provider packages

If the stack already exists but the local environment does not, install dependencies first and then run:

```bash
pulumi stack select dev
pulumi preview
```

## Core Usage

### Read Config And Export Values

Use `pulumi.Config()` for stack config and `pulumi.export()` for stack outputs.

```python
import pulumi

config = pulumi.Config()

environment = config.get("environment") or pulumi.get_stack()
owner = config.require("owner")
db_password = config.require_secret("dbPassword")

pulumi.export("stackName", pulumi.get_stack())
pulumi.export("environment", environment)
pulumi.export("owner", owner)
pulumi.export(
    "serviceUrl",
    pulumi.Output.format("https://{}.{}.example.com", pulumi.get_project(), environment),
)
pulumi.export("dbPassword", db_password)
```

CLI commands that populate those values:

```bash
pulumi config set owner platform
pulumi config set environment prod
pulumi config set --secret dbPassword 'correct-horse-battery-staple'
```

### Work With `Output` Values Correctly

Pulumi values that come from resources, stack references, or secret config are often `Output[...]`, not plain Python values.

Use `Output.all(...)` and `apply(...)` to transform them:

```python
import pulumi

host = pulumi.Output.from_input("api.internal.example")
port = pulumi.Output.from_input(443)

origin = pulumi.Output.all(host=host, port=port).apply(
    lambda args: f"https://{args['host']}:{args['port']}"
)

pulumi.export("origin", origin)
```

Use `Output.format(...)` when string formatting is all you need:

```python
import pulumi

subdomain = pulumi.Output.from_input("app")
stack = pulumi.get_stack()

url = pulumi.Output.format("https://{}.{}.example.com", subdomain, stack)
pulumi.export("url", url)
```

Do not call `Output.get()` during deployment. The API reference explicitly limits `get()` to code that runs after deployment is complete.

### Build Reusable Components

For shared infrastructure building blocks, define a `ComponentResource` and call `register_outputs()`.

```python
from __future__ import annotations

import pulumi

class ServiceEndpoint(pulumi.ComponentResource):
    url: pulumi.Output[str]

    def __init__(
        self,
        name: str,
        domain: pulumi.Input[str],
        opts: pulumi.ResourceOptions | None = None,
    ) -> None:
        super().__init__("example:components:ServiceEndpoint", name, None, opts)

        self.url = pulumi.Output.format("https://{}", domain)

        self.register_outputs(
            {
                "url": self.url,
            }
        )

service = ServiceEndpoint("api", domain="api.example.com")
pulumi.export("serviceUrl", service.url)
```

### Read Values From Another Stack

Use `StackReference` when one stack needs outputs from another stack:

```python
import pulumi

network = pulumi.StackReference("acme/network/prod")
subnet_id = network.get_output("subnetId")
vpc_id = network.get_output("vpcId")

pulumi.export("subnetId", subnet_id)
pulumi.export("vpcId", vpc_id)
```

This is the normal way to compose environments such as `network`, `platform`, and `application` stacks without hardcoding cloud identifiers.

## Config, Secrets, And Authentication

Pulumi has two separate auth layers that agents often mix up:

1. Pulumi backend auth for state storage and stack operations
2. Cloud provider auth for actually creating resources

### Pulumi Backend Login

Default Pulumi Cloud login:

```bash
pulumi login
```

Non-interactive CI usually uses an access token:

```bash
export PULUMI_ACCESS_TOKEN="..."
pulumi login
```

Other supported backends include local state and object storage:

```bash
pulumi login --local
pulumi login s3://my-pulumi-state
pulumi login azblob://my-container
pulumi login gs://my-pulumi-state
```

### Stack Config And Secret Values

- Use `pulumi config set` for normal values.
- Use `pulumi config set --secret` for credentials and anything that should stay encrypted in state.
- In Python, read normal values with `Config.get()` or `Config.require()`.
- Read encrypted values with `Config.get_secret()` or `Config.require_secret()`.

Provider config is typically namespaced. For example, AWS region lives under `aws:region`, not a plain project key:

```bash
pulumi config set aws:region us-west-2
```

### Cloud Provider Credentials

The `pulumi` package does not replace provider auth. After `pulumi login`, you still need the normal credentials for the provider package you are using:

- AWS: environment variables, shared config, profiles, IAM roles
- Azure: Azure CLI, service principal, managed identity
- GCP: ADC or service account credentials
- Kubernetes: `KUBECONFIG` or explicit cluster config

If `pulumi preview` succeeds but provider operations fail, the usual problem is missing provider credentials rather than a Pulumi backend login issue.

## Common Pitfalls

- Installing only `pulumi` is not enough. Most projects also need the Pulumi CLI and one or more provider packages.
- Do not create resources inside `Output.apply(...)` unless you are deliberately accepting deferred, harder-to-reason-about resource graphs. Prefer passing `Output` values directly into resource arguments.
- Do not call `Output.get()` in a normal Pulumi program. Use `apply`, `Output.all`, or `Output.format` instead.
- Secret config returns secret outputs. Treat them as `Output` values and avoid logging or stringifying them casually.
- `Pulumi.<stack>.yaml` is stack-specific. If config seems to "disappear", check that the selected stack is the one you expect.
- `pulumi refresh` updates state from the provider, but it does not replace understanding your desired program changes. Do not use refresh as a substitute for reviewing diffs.
- The project and stack names affect config namespaces, stack references, and backend paths. Renaming them midstream can break assumptions in automation and CI.
- Keep provider package versions in sync with the project lockfile. Copying examples that assume a different provider major version is a common source of broken code.

## Practical Guidance For Agents

1. Start by confirming the stack backend, selected stack, and provider package set before editing code.
2. When you need runtime-like values from resources, model them as `Output` transformations instead of forcing eager Python values.
3. Put user-provided settings in stack config, not hardcoded constants in `__main__.py`.
4. Use `ComponentResource` for repeatable abstractions and `StackReference` for cross-stack wiring.
5. When debugging, separate backend login issues from provider credential issues before changing code.

## Official Sources

- Python API reference: `https://www.pulumi.com/docs/reference/pkg/python/pulumi/`
- Install Pulumi CLI: `https://www.pulumi.com/docs/iac/download-install/`
- Pulumi Python language guide: `https://www.pulumi.com/docs/iac/languages-sdks/python/`
- Configuration concepts: `https://www.pulumi.com/docs/iac/concepts/config/`
- Inputs and outputs: `https://www.pulumi.com/docs/iac/concepts/inputs-outputs/`
- Stack outputs and references: `https://www.pulumi.com/docs/iac/concepts/stacks/`
- State backends and login: `https://www.pulumi.com/docs/iac/concepts/state-and-backends/`
- PyPI package page: `https://pypi.org/project/pulumi/`
