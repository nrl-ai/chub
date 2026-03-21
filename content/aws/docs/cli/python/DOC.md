---
name: cli
description: "awscli package guide for Python environments: install AWS CLI v1, configure credentials and profiles, and run AWS commands safely in scripts"
metadata:
  languages: "python"
  versions: "1.44.56"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,awscli,cli,python,cloud,devops"
---

# awscli Python Package Guide

## What This Package Is

`awscli` is the PyPI package that installs the AWS CLI v1 `aws` executable.

Use it when you need to:

- install a pinned AWS CLI into a Python virtual environment
- run AWS commands from a shell, CI job, or deployment script
- call the CLI from Python with `subprocess`

Do not use `awscli` as your default Python SDK. If the task is "call AWS APIs from Python code", prefer `boto3` or `botocore`.

For this package entry, the version used here is `1.44.56`, which is part of the AWS CLI v1 line. Use AWS CLI v1 docs under `https://docs.aws.amazon.com/cli/v1/`, not the moving `latest/reference` URL.

## Install

Pin the package version explicitly in automation:

```bash
python -m pip install "awscli==1.44.56"
aws --version
```

Expected prefix:

```text
aws-cli/1.44.56
```

Use a virtual environment if the machine may also have AWS CLI v2 or an OS-packaged `aws` binary on `PATH`:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "awscli==1.44.56"
which aws
aws --version
```

If you need to confirm the installed Python package and executable match:

```bash
python -m pip show awscli
which aws
aws --version
```

## Python Compatibility

AWS CLI v1 installation docs group versions `1.39.0` and later into the Python `3.9+` support band. Treat `awscli==1.44.56` as a Python 3.9+ package unless your environment-specific testing proves otherwise.

For reproducible CI, pin both:

- the Python runtime
- the `awscli` version

## First-Time Setup

The basic setup flow is:

```bash
aws configure
```

That writes to the shared AWS CLI files:

- `~/.aws/credentials`
- `~/.aws/config`

Sanity-check the active identity before making changes:

```bash
aws sts get-caller-identity
```

Common smoke tests:

```bash
aws s3 ls
aws ec2 describe-regions --output json
```

## Credentials, Config Files, and Profiles

AWS CLI v1 uses two files with different profile-section conventions:

- `~/.aws/credentials` uses `[default]` or `[work]`
- `~/.aws/config` uses `[default]` or `[profile work]`

Example:

```ini
# ~/.aws/credentials
[default]
aws_access_key_id = AKIA...
aws_secret_access_key = ...

[work]
aws_access_key_id = AKIA...
aws_secret_access_key = ...
aws_session_token = ...
```

```ini
# ~/.aws/config
[default]
region = us-west-2
output = json

[profile work]
region = us-east-1
output = json
```

Pick a non-default profile either per command:

```bash
aws s3 ls --profile work
```

or for a shell session:

```bash
export AWS_PROFILE=work
```

Useful config commands:

```bash
aws configure list
aws configure get region --profile work
aws configure set region us-east-1 --profile work
aws configure set output json --profile work
```

Assume-role profile example:

```ini
[profile admin]
role_arn = arn:aws:iam::123456789012:role/Admin
source_profile = work
region = us-east-1
output = json
```

## Environment Variables

Common environment variables for automation:

```bash
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
export AWS_SESSION_TOKEN=...
export AWS_DEFAULT_REGION=us-west-2
export AWS_DEFAULT_OUTPUT=json
export AWS_PROFILE=work
```

Useful path overrides:

```bash
export AWS_SHARED_CREDENTIALS_FILE=/tmp/aws-credentials
export AWS_CONFIG_FILE=/tmp/aws-config
```

Prefer:

- named profiles for local development across multiple accounts
- explicit environment variables for short-lived CI jobs
- `AWS_SESSION_TOKEN` whenever credentials come from STS or assumed roles

## Core Usage

CLI command shape:

```bash
aws <service> <operation> [options]
```

Help is available at every level:

```bash
aws help
aws s3 help
aws s3 cp help
```

Good defaults for scripts:

- use `--output json`
- use `--query` to extract only the fields you need
- pass `--region` explicitly when the script must be portable
- pass `--profile` explicitly when the account must be unambiguous

Examples:

```bash
aws sts get-caller-identity --output json
```

```bash
aws ec2 describe-instances \
  --filters "Name=instance-state-name,Values=running" \
  --query 'Reservations[].Instances[].{Id:InstanceId,Type:InstanceType,AZ:Placement.AvailabilityZone}' \
  --output json
```

```bash
aws s3 cp ./build.zip s3://my-bucket/build.zip --profile work --region us-east-1
```

## Calling The CLI From Python

Treat `awscli` as a subprocess dependency, not as a stable in-process library API.

```python
import json
import subprocess

def caller_identity(profile: str = "default", region: str = "us-west-2") -> dict:
    result = subprocess.run(
        [
            "aws",
            "sts",
            "get-caller-identity",
            "--profile",
            profile,
            "--region",
            region,
            "--output",
            "json",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)
```

Guidelines:

- keep `shell=False`
- request JSON output and parse it
- pin `--profile` and `--region`
- surface `stderr` on failures because AWS CLI errors are usually actionable

## Output, Queries, and Pagination

AWS CLI v1 supports `json`, `text`, and `table` output.

- `json` is the safest default for automation
- `table` is for humans
- `text` is easy to break if you later change the query shape

AWS's filtering guide warns that on paginated responses, `--query` behaves differently with `--output text` because the query is applied per page. For stable machine-readable results, prefer `--output json` and then parse the JSON.

Examples:

```bash
aws iam list-users --output json --query 'Users[].UserName'
```

```bash
aws cloudformation describe-stacks \
  --stack-name my-stack \
  --output json \
  --query 'Stacks[0].Outputs'
```

If a command is paginated and you need complete results in a script, read the command help for pagination flags and test with realistic account-sized datasets.

## Version-Sensitive Notes

- `awscli` on PyPI is the AWS CLI v1 line. Do not mix v1 guidance with AWS CLI v2 blog posts or examples.
- The docs URL, `https://docs.aws.amazon.com/cli/latest/reference/`, is not version-pinned. For this package, prefer `https://docs.aws.amazon.com/cli/v1/`.
- AWS's v1 documentation now carries an end-of-support notice. For new greenfield environments, prefer AWS CLI v2 unless the project is explicitly pinned to `awscli` v1 on PyPI.
- PyPI may show newer releases than the version covered here. This doc is intentionally pinned to `1.44.56`.

## Common Pitfalls

### Wrong `aws` on `PATH`

If the machine has multiple AWS CLI installs, the binary you invoke may not match the Python package you just installed.

Check:

```bash
which aws
aws --version
python -m pip show awscli
```

### Wrong docs

Do not use generic AWS CLI "latest" docs when the task is pinned to a PyPI `awscli` version. Start from AWS CLI v1 docs.

### Missing region

Many commands fail or behave inconsistently when no region is configured. Set one in the profile, export `AWS_DEFAULT_REGION`, or pass `--region`.

### Temporary credentials without session token

If the credentials come from STS or an assumed role, `AWS_SESSION_TOKEN` is required with the access key and secret key.

### Account mix-ups

Scripts that rely on the implicit default profile are easy to run against the wrong account. Prefer explicit `--profile` and `--region`.

### Fragile `text` parsing

`--output text` is convenient for shell experiments but brittle in automation, especially with pagination and `--query`.

## Official Sources

- AWS CLI v1 user guide: `https://docs.aws.amazon.com/cli/v1/userguide/cli-chap-welcome.html`
- AWS CLI v1 install guide: `https://docs.aws.amazon.com/cli/v1/userguide/cli-chap-install.html`
- AWS CLI v1 command reference: `https://docs.aws.amazon.com/cli/v1/reference/`
- AWS CLI v1 config files guide: `https://docs.aws.amazon.com/cli/v1/userguide/cli-configure-files.html`
- AWS CLI v1 environment variables guide: `https://docs.aws.amazon.com/cli/v1/userguide/cli-configure-envvars.html`
- AWS CLI v1 output format guide: `https://docs.aws.amazon.com/cli/v1/userguide/cli-usage-output-format.html`
- AWS CLI v1 filtering guide: `https://docs.aws.amazon.com/cli/v1/userguide/cli-usage-filter.html`
- PyPI package page: `https://pypi.org/project/awscli/`
