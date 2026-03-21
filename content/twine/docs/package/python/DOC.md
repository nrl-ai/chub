---
name: package
description: "Twine package guide for building, checking, and uploading Python distributions to PyPI and TestPyPI"
metadata:
  languages: "python"
  versions: "6.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "twine,pypi,packaging,publishing,python"
---

# Twine Python Package Guide

## Golden Rule

Use `twine` only for checking and uploading already-built distribution artifacts. Do not use `python setup.py upload`; the Python Packaging User Guide treats `python setup.py` as a deprecated command-line interface and recommends `python -m build`, `python -m twine check`, and `python -m twine upload` instead.

## Install

Install Twine into the environment that will publish your package:

```bash
python -m pip install "twine==6.2.0"
```

In most real workflows you also want `build` in the same environment:

```bash
python -m pip install "build" "twine==6.2.0"
```

## What Twine Does

Twine does not build wheels or source distributions. It uploads files you already created, usually from `dist/`.

Typical artifact workflow:

```bash
python -m build
ls dist/
```

You should normally publish both:

- a wheel: `dist/*.whl`
- a source distribution: `dist/*.tar.gz`

## Recommended Release Flow

### 1. Build distributions

Run this from the directory containing `pyproject.toml`:

```bash
python -m build
```

### 2. Validate package metadata and README rendering

`twine check` catches broken long-description rendering before upload. Use `--strict` so warnings fail CI:

```bash
python -m twine check --strict dist/*
```

### 3. Upload to TestPyPI first

```bash
python -m twine upload --repository testpypi dist/*
```

TestPyPI is a separate index with separate accounts and separate API tokens.

### 4. Verify the uploaded package

Install from TestPyPI before publishing to real PyPI:

```bash
python -m pip install \
  --index-url https://test.pypi.org/simple/ \
  --extra-index-url https://pypi.org/simple/ \
  your-package-name
```

The extra index is often necessary so dependencies still resolve from PyPI.

### 5. Upload to PyPI

```bash
python -m twine upload dist/*
```

## Authentication And Configuration

Twine can take configuration from:

1. Command-line flags
2. Environment variables
3. `.pypirc`
4. Keyring

The Twine docs state that command-line flags take precedence over environment variables. Twine also ships with default repository entries for `pypi` and `testpypi`, so you usually do not need to define those URLs yourself unless you want custom credentials or additional indexes.

### Environment variables

Useful variables for CI and other non-interactive environments:

```bash
export TWINE_USERNAME=__token__
export TWINE_PASSWORD=pypi-xxxxxxxxxxxxxxxxxxxxxxxxxxxx
export TWINE_NON_INTERACTIVE=1
```

If you need a non-default index:

```bash
export TWINE_REPOSITORY=testpypi
# or
export TWINE_REPOSITORY_URL=https://example.internal/legacy/
```

Use `TWINE_CERT` when uploading to an index that requires a custom CA bundle.

### `.pypirc`

For local publishing, a minimal `~/.pypirc` using API tokens looks like:

```ini
[pypi]
username = __token__
password = pypi-...

[testpypi]
username = __token__
password = pypi-...
```

If you add private indexes, include the full `distutils` index list and a section per repository:

```ini
[distutils]
index-servers =
    pypi
    testpypi
    private-repository

[pypi]
username = __token__
password = pypi-...

[testpypi]
username = __token__
password = pypi-...

[private-repository]
repository = https://packages.example.com/legacy/
username = my-user
password = my-password
```

Security notes:

- `.pypirc` stores credentials in plain text.
- On macOS and Linux, lock it down with `chmod 600 ~/.pypirc`.
- Prefer environment variables, keyring, or Trusted Publishing when possible.

### Keyring

Twine supports `keyring` and the Twine docs say it is installed with Twine. This avoids storing tokens directly in `.pypirc`.

Store PyPI and TestPyPI tokens like this:

```bash
keyring set https://upload.pypi.org/legacy/ __token__
keyring set https://test.pypi.org/legacy/ __token__
```

If keyring causes unwanted prompts, disable it:

```bash
keyring --disable
```

### Trusted Publishing for GitHub Actions

For GitHub-hosted releases, PyPA now recommends Trusted Publishing instead of long-lived `PYPI_API_TOKEN` secrets. The Packaging guide recommends a workflow that:

- builds distributions with `python -m build`
- uses GitHub environments such as `pypi` and `testpypi`
- grants `permissions: id-token: write`
- publishes with `pypa/gh-action-pypi-publish@release/v1`

Use Twine locally or in CI when you need direct uploads with explicit credentials, custom repository URLs, or non-GitHub automation. Prefer Trusted Publishing for GitHub Actions release jobs.

## Core Commands

Check a build:

```bash
python -m twine check --strict dist/*
```

Upload to PyPI:

```bash
python -m twine upload dist/*
```

Upload to TestPyPI:

```bash
python -m twine upload --repository testpypi dist/*
```

Upload to a custom repository URL:

```bash
python -m twine upload --repository-url https://packages.example.com/legacy/ dist/*
```

Sign uploads with GPG:

```bash
python -m twine upload --sign dist/*
```

Use pre-generated `.asc` signatures:

```bash
gpg --detach-sign -a dist/my_package-1.2.3.tar.gz
python -m twine upload dist/my_package-1.2.3.tar.gz dist/my_package-1.2.3.tar.gz.asc
```

Fail fast in CI when credentials are missing:

```bash
python -m twine upload --non-interactive dist/*
```

## Common Pitfalls

### Twine does not build artifacts

If `dist/` is empty, Twine will not help. Build first with `python -m build`.

### `register` is not part of normal PyPI publishing

`twine register` exists for repositories that still require pre-registration, but the Twine docs explicitly say pre-registration is not supported on PyPI. Do not add `register` to normal PyPI release automation.

### TestPyPI and PyPI credentials are separate

TestPyPI has a separate account database and separate API tokens. A working PyPI token will not upload to TestPyPI.

### Use `__token__` for PyPI and TestPyPI

As of Twine `6.0.0`, workflows that explicitly pass a PyPI or TestPyPI username other than `__token__` fail. Either omit the username or use `__token__`.

### `--repository-url` overrides `--repository`

If both are set, `--repository-url` wins. This is a common source of confusion when `.pypirc` and CI environment variables both exist.

### `--skip-existing` is not a general-purpose safety net

In Twine `6.2.0`, support hacks for `--skip-existing` on indexes other than PyPI and TestPyPI were removed. Do not assume private indexes behave like PyPI here.

### `.tar.bz2`, `egg`, and `wininst` are no longer supported

Twine `6.1.0` removed support for `egg`, `wininst`, and `.tar.bz2` uploads. Expect modern `wheel` and `.tar.gz` artifacts instead.

### Hidden password input is normal

Twine intentionally does not echo credentials while prompting. On Windows terminals, standard paste shortcuts may not work in the prompt.

## Version-Sensitive Notes For 6.2.0

- `6.2.0` refreshes short-lived PyPI Trusted Publishing tokens during long-running uploads.
- `6.2.0` enforces `keyring >= 21.2.0`.
- `6.2.0` removes old compatibility hacks for `--skip-existing` on non-PyPI indexes.
- `6.1.0` added preliminary built-in Trusted Publishing support.
- `6.1.0` removed support for `egg`, `wininst`, and `.tar.bz2`.
- `6.0.0` changed PyPI/TestPyPI username handling so non-`__token__` usernames now fail.

## Official Sources

- Twine docs: https://twine.readthedocs.io/en/stable/
- Twine changelog: https://twine.readthedocs.io/en/stable/changelog.html
- PyPI project page: https://pypi.org/project/twine/
- `.pypirc` spec: https://packaging.python.org/en/latest/specifications/pypirc/
- TestPyPI guide: https://packaging.python.org/en/latest/guides/using-testpypi/
- GitHub Actions Trusted Publishing guide: https://packaging.python.org/en/latest/guides/publishing-package-distribution-releases-using-github-actions-ci-cd-workflows/
- `setup.py` deprecation guidance: https://packaging.python.org/en/latest/discussions/setup-py-deprecated/
