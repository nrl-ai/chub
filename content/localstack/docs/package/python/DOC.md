---
name: package
description: "LocalStack Python package guide for running the LocalStack CLI and wiring Python AWS SDK clients to local emulated AWS services"
metadata:
  languages: "python"
  versions: "4.14.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "localstack,aws,python,boto3,testing,docker,cloud-emulator"
---

# LocalStack Python Package Guide

## Golden Rule

Use the `localstack` PyPI package to install the LocalStack CLI, but treat LocalStack as local infrastructure, not as an application library you import into your service code. Your Python app usually talks to LocalStack through normal AWS clients such as `boto3`, pointed at the LocalStack endpoint.

## Install

Pin the package version your project expects:

```bash
python -m pip install "localstack==4.14.0"
localstack --version
```

LocalStack requires Docker for the normal runtime path. Confirm Docker is available before starting services:

```bash
docker info >/dev/null
```

If you use the AWS CLI often, install `awslocal` as well:

```bash
python -m pip install awscli-local
```

## Start And Stop LocalStack

Detached local development flow:

```bash
localstack start -d
localstack wait
localstack status services
```

Useful lifecycle commands:

```bash
localstack logs
localstack stop
```

If you need a custom profile:

```bash
CONFIG_PROFILE=dev localstack start -d
```

Or:

```bash
localstack --profile=dev start -d
```

`CONFIG_PROFILE` and `--profile` apply to the CLI. The LocalStack docs explicitly note that profile-based configuration is not supported in Docker Compose.

## Minimal Docker Compose Setup

Pin the image tag instead of relying on `latest`-style behavior:

```yaml
services:
  localstack:
    image: localstack/localstack:4.14.0
    ports:
      - "4566:4566"
    environment:
      - SERVICES=s3,sqs
      - PERSISTENCE=1
      - DEBUG=1
    volumes:
      - "./volume:/var/lib/localstack"
      - "/var/run/docker.sock:/var/run/docker.sock"
```

Notes:

- `4566` is the edge port for all emulated AWS APIs.
- `SERVICES` limits startup to the services you actually need.
- `PERSISTENCE=1` enables disk-backed state.
- Mounting the Docker socket is required for features that need nested containers.

## Credentials And Auth

For most local development, use test credentials:

```bash
export AWS_ACCESS_KEY_ID=test
export AWS_SECRET_ACCESS_KEY=test
export AWS_DEFAULT_REGION=us-east-1
```

Important LocalStack-specific rules from the AWS credentials docs:

- LocalStack accepts any credentials by default, but the access key can influence the simulated account namespace.
- The secret access key is ignored by LocalStack itself, but LocalStack recommends keeping it set to `test`, especially for S3 presigned URL workflows.
- LocalStack rejects `AKIA...` and `ASIA...` access key patterns by default to reduce accidental use of real AWS credentials. If you pass a real-looking key anyway, many requests fall back to account `000000000000`.
- If you need multiple emulated accounts, use a valid 12-digit account ID as the access key.

For LocalStack Pro authentication, set a token before starting the container:

```bash
localstack auth set-token <your-auth-token>
```

Or pass it as an environment variable:

```bash
export LOCALSTACK_AUTH_TOKEN=<your-auth-token>
```

## Core Python Usage With boto3

Point your AWS SDK clients at LocalStack's edge endpoint. `localhost.localstack.cloud` is the official hostname LocalStack documents for localhost access:

```python
import boto3

s3 = boto3.client(
    "s3",
    endpoint_url="http://localhost.localstack.cloud:4566",
    aws_access_key_id="test",
    aws_secret_access_key="test",
    region_name="us-east-1",
)

s3.create_bucket(Bucket="demo-bucket")
print(s3.list_buckets())
```

SQS example:

```python
import boto3

sqs = boto3.client(
    "sqs",
    endpoint_url="http://localhost.localstack.cloud:4566",
    aws_access_key_id="test",
    aws_secret_access_key="test",
    region_name="us-east-1",
)

queue_url = sqs.create_queue(QueueName="jobs")["QueueUrl"]
sqs.send_message(QueueUrl=queue_url, MessageBody="hello")
print(sqs.receive_message(QueueUrl=queue_url))
```

If your app runs inside another container on the same Docker network, use the container hostname instead of `localhost`. The LocalStack networking guide documents `localstack-main` as the default main container name and also supports user-defined network aliases such as `localstack`.

## AWS CLI And `awslocal`

Prefer `awslocal` over `aws --endpoint-url ...` for routine shell automation:

```bash
awslocal s3 mb s3://demo-bucket
awslocal s3 ls
awslocal sqs create-queue --queue-name jobs
```

The LocalStack AWS CLI docs call out a common pitfall: `aws --no-sign-request` can route requests to the wrong service. Use `awslocal` when possible.

## Init Hooks And Seeding Infrastructure

LocalStack can run bootstrap scripts during container startup. This is the cleanest way to pre-create buckets, queues, or other test infrastructure for integration tests.

Supported hook directories:

- `/etc/localstack/init/boot.d`
- `/etc/localstack/init/start.d`
- `/etc/localstack/init/ready.d`
- `/etc/localstack/init/shutdown.d`

`ready.d` is the most common choice because services are available by then. Example:

```bash
#!/bin/bash
set -euo pipefail

awslocal s3 mb s3://demo-bucket || true
awslocal sqs create-queue --queue-name jobs >/dev/null
```

Mount it into the container:

```yaml
volumes:
  - "./init/01-bootstrap.sh:/etc/localstack/init/ready.d/01-bootstrap.sh"
```

Hook files run in alphanumeric order. LocalStack supports shell and Python scripts plus Terraform files through the extension-based auto-detection in the init-hook docs.

## Persistence And State

Enable persistence when your tests or local workflows need state to survive restarts:

```bash
PERSISTENCE=1 localstack start -d
```

In Docker-based setups, LocalStack stores state under:

```text
/var/lib/localstack/state
```

You can also export and import state snapshots through the state-management API:

```bash
curl -X POST http://localhost:4566/_localstack/state/export
curl -X POST http://localhost:4566/_localstack/state/import
```

State files are not guaranteed to be portable across LocalStack versions, so do not assume a snapshot from one version will restore cleanly on another.

## Configuration Notes

- `SERVICES=s3,sqs,...` reduces startup time and keeps service initialization explicit.
- `EAGER_SERVICE_LOADING=1` preloads configured services on startup instead of waiting for first use.
- `LOCALSTACK_VOLUME_DIR` changes the host directory used for mounted LocalStack data.
- `MAIN_DOCKER_NETWORK` controls which Docker network child containers join.
- `GATEWAY_LISTEN` can change the bind address or edge port if `4566` conflicts with another process.

## Common Pitfalls

- The `localstack` package is mainly the CLI. Do not design your app around importing `localstack`; configure your normal AWS SDK clients to hit the LocalStack endpoint instead.
- LocalStack depends on Docker for the standard runtime flow. A successful `pip install` is not enough if Docker is missing or unhealthy.
- Pin the Docker image tag in Compose or CI. The install docs warn that a bare `docker compose up` can otherwise pull the current mainline image.
- If your code runs in Docker, `localhost` inside that container is not the LocalStack host. Use a shared-network hostname such as `localstack` or `localstack-main`.
- Keep region values consistent. Many AWS SDK examples silently default to `us-east-1`; mismatched regions can make resources appear "missing."
- For S3 presigned URLs and similar flows, keep both `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` set, even though LocalStack ignores the secret internally.
- `CONFIG_PROFILE` is a CLI feature, not a Compose feature.
- Persistence snapshots can break across version upgrades. Recreate fixtures if a restored state behaves strangely after a version bump.

## Version-Sensitive Notes For 4.14.0

- PyPI lists `localstack 4.14.0` with Python `>=3.10`, released on February 26, 2026.
- The LocalStack install docs currently show `localstack --version` returning `4.14.0`.
- The LocalStack credentials docs say the single authenticated image rollout for LocalStack for AWS starts on March 23, 2026. If your existing setup assumes unauthenticated image pulls, verify whether you now need `localstack auth set-token` or `LOCALSTACK_AUTH_TOKEN`.
- The PyPI project page and docs both support `pip install localstack`, but parts of the long PyPI project description still mention `localstack-cli-standalone`. Prefer the package named in this entry unless your environment specifically needs the standalone CLI distribution.

## Official Sources Used

- LocalStack docs root: `https://docs.localstack.cloud/references/`
- Install LocalStack: `https://docs.localstack.cloud/getting-started/installation/`
- Configuration: `https://docs.localstack.cloud/references/configuration/`
- AWS credentials: `https://docs.localstack.cloud/aws/capabilities/config/credentials/`
- AWS CLI integration: `https://docs.localstack.cloud/aws/integrations/aws-native-tools/aws-cli/`
- Boto3 integration: `https://docs.localstack.cloud/aws/integrations/aws-sdks/python-boto3/`
- Initialization hooks: `https://docs.localstack.cloud/aws/capabilities/config/initialization-hooks/`
- State management: `https://docs.localstack.cloud/aws/capabilities/state-management/`
- Networking / endpoint guidance: `https://docs.localstack.cloud/aws/capabilities/networking/accessing-endpoint-url/`
- PyPI package page: `https://pypi.org/project/localstack/`
