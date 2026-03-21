---
name: package
description: "DVC package guide for Python projects: data versioning, remotes, pipelines, and dvc.api access"
metadata:
  languages: "python"
  versions: "3.66.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "dvc,data-versioning,mlops,pipelines,python,git"
---

# DVC Python Package Guide

## Golden Rule

Use the `dvc` CLI for tracking data and pipelines, and use `dvc.api` only for read-style access from Python code. Do not build on DVC's internal Python modules unless the upstream API reference explicitly documents them.

## Install

DVC is published as a Python package and installs a `dvc` command-line tool.

```bash
pip install dvc
dvc version
```

For cloud or remote storage backends, install the matching extras up front instead of discovering missing protocol support later:

```bash
pip install "dvc[s3]"
pip install "dvc[gs]"
pip install "dvc[azure]"
pip install "dvc[ssh]"
```

Notes:

- PyPI metadata for `3.66.1` requires Python `>=3.9`.
- PyPI publishes backend extras such as `s3`, `gs`, `azure`, `ssh`, `oss`, `webdav`, and `webhdfs`.
- If `dvc push` or `dvc pull` fails with missing remote support, reinstall with the correct extra for that storage type.

## Initialize A Repository

DVC is normally used inside a Git repository:

```bash
git init
dvc init
git add .dvc .dvcignore
git commit -m "Initialize DVC"
```

This creates the `.dvc/` directory and local configuration. Keep the repository metadata in Git; do not commit the DVC cache.

## Track Data Files Or Directories

Use `dvc add` to put large data under DVC control instead of Git:

```bash
dvc add data/raw
git add data/raw.dvc .gitignore
git commit -m "Track raw dataset with DVC"
```

What this does:

- `dvc add` stores file metadata in a `.dvc` file.
- DVC updates `.gitignore` so the actual data stays out of Git.
- Teammates clone the Git repo, then fetch the tracked data with `dvc pull`.

Typical sync flow:

```bash
dvc push
dvc pull
dvc status
```

## Configure Remote Storage

Add a default remote so data can be pushed and pulled outside your local cache:

```bash
dvc remote add -d storage s3://my-bucket/my-prefix
```

Store credentials in local config or environment-backed provider credentials, not in tracked config:

```bash
dvc remote modify --local storage access_key_id "$AWS_ACCESS_KEY_ID"
dvc remote modify --local storage secret_access_key "$AWS_SECRET_ACCESS_KEY"
```

Practical rules:

- Use `--local` for secrets so they go into `.dvc/config.local`, which is not meant for Git.
- Prefer the cloud provider's normal credential chain when possible. For S3, DVC can use standard AWS credentials and boto3-compatible auth flows.
- Install the matching extra before configuring the remote backend.

Common remote examples:

```bash
dvc remote add -d storage gs://my-bucket/path
dvc remote add -d storage azure://mycontainer/path
dvc remote add -d storage ssh://user@example.com/path/to/cache
```

## Build Reproducible Pipelines

Use `dvc stage add` to declare commands, dependencies, and outputs in `dvc.yaml`:

```bash
dvc stage add \
  -n train \
  -d src/train.py \
  -d data/features.csv \
  -o models/model.pkl \
  python src/train.py
```

Then run and re-run the pipeline through DVC:

```bash
dvc repro
git add dvc.yaml dvc.lock
git commit -m "Add training pipeline"
```

Use this model when you want cacheable, dependency-aware execution. If a file is not declared with `-d` or `-o`, DVC cannot use it to determine whether a stage is stale.

## Use `dvc.api` From Python

The stable Python entry points are under `dvc.api`. Use them when application code needs to read versioned data from a DVC repository or remote.

Read a text file from the current repository:

```python
import dvc.api

params_text = dvc.api.read("params.yaml", repo=".")
print(params_text)
```

Open a tracked file from a specific repo and revision:

```python
import dvc.api

with dvc.api.open(
    "data/model.bin",
    repo="https://github.com/iterative/example-get-started",
    rev="main",
    mode="rb",
) as stream:
    model_bytes = stream.read()
```

When you need an `fsspec`-style filesystem interface, use `dvc.api.DVCFileSystem` instead of shelling out to `dvc` from Python.

## Authentication And Configuration Notes

- Local project config lives in `.dvc/config`.
- Secret or machine-specific config belongs in `.dvc/config.local`.
- Remote credentials can come from provider SDK defaults, environment variables, or `dvc remote modify --local`.
- Keep Git credentials and DVC remote credentials conceptually separate. Git access controls source checkout; DVC remote auth controls data transfer.

## Common Pitfalls

- Installing plain `dvc` but forgetting the remote extra for S3, GCS, Azure, SSH, or another backend.
- Committing data files or `.dvc/cache` to Git instead of only the `.dvc` metadata, `dvc.yaml`, and `dvc.lock`.
- Saving secrets in `.dvc/config` instead of `.dvc/config.local` or provider-managed credentials.
- Expecting `dvc repro` to react to undeclared inputs. Every dependency and output that affects a stage must be declared.
- Treating DVC as a general-purpose Python SDK. For application code, stay on the documented `dvc.api` surface.

## Version-Sensitive Notes For `3.66.1`

- `3.66.1` is in the DVC 3.x line. If you work with repositories created on DVC 2.x, expect upgrade friction around cache/object hashes.
- The DVC 3.0 upgrade guide notes a new hash method for tracked files. Older repos can require `dvc cache migrate --dvc-files` before pushes, pulls, or status checks behave as expected.
- If a team mixes older DVC clients with 3.x metadata, align client versions before debugging missing cache objects or checksum mismatches.
- Use PyPI package metadata, not third-party blog posts, for the authoritative runtime requirement: Python `>=3.9` for `3.66.1`.

## Official Sources

- Docs root: https://dvc.org/doc
- API reference: https://dvc.org/doc/api-reference
- Install docs: https://dvc.org/doc/install
- Getting started: https://dvc.org/doc/start
- Remote storage docs: https://dvc.org/doc/user-guide/data-management/remote-storage
- S3 remote auth/config: https://dvc.org/doc/user-guide/data-management/remote-storage/amazon-s3
- `dvc remote add`: https://dvc.org/doc/command-reference/remote/add
- `dvc remote modify`: https://dvc.org/doc/command-reference/remote/modify
- Upgrade guide: https://dvc.org/doc/user-guide/upgrade
- PyPI package: https://pypi.org/project/dvc/
