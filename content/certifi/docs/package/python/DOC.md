---
name: package
description: "certifi package guide for Python: Mozilla CA bundle access and trust-store usage"
metadata:
  languages: "python"
  versions: "2026.2.25"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "certifi,python,tls,ssl,certificates,ca-bundle"
---

# certifi Python Package Guide

## What It Is

`certifi` ships Mozilla's CA bundle for Python applications. Its job is narrow:

- expose the bundled `cacert.pem` file path
- expose the PEM contents as text

It does not create HTTP clients, manage TLS sessions, or let you edit the trust store in place.

## Install

```bash
pip install certifi
```

Pin the documented release when you need a specific root-certificate set:

```bash
pip install certifi==2026.2.25
```

With Poetry:

```bash
poetry add certifi
```

With `uv`:

```bash
uv add certifi
```

## Initialize / Setup

```python
import certifi

ca_bundle_path = certifi.where()
ca_bundle_pem = certifi.contents()
```

- `certifi.where()` returns the filesystem path to the packaged `cacert.pem`.
- `certifi.contents()` returns the CA bundle as ASCII PEM text.

Command-line helpers:

```bash
python -m certifi
python -m certifi --contents
```

Use `where()` when another library expects a CA file path. Use `contents()` only when you need the PEM text itself.

## Core Usage

### Pass the CA bundle path to an HTTP client

```python
import certifi
import requests

response = requests.get(
    "https://example.com",
    verify=certifi.where(),
    timeout=30,
)
response.raise_for_status()
```

### Build an `ssl` context from the certifi bundle

```python
import certifi
import ssl

ssl_context = ssl.create_default_context(cafile=certifi.where())
```

### Load the PEM contents directly

```python
import certifi

pem_text = certifi.contents()
assert "BEGIN CERTIFICATE" in pem_text
```

## Config / Auth

`certifi` has no authentication model and no package-specific environment variables.

Configuration happens in the consuming library:

- pass `certifi.where()` to a `verify`, `cafile`, or equivalent TLS option
- use a different CA file entirely if your application must trust private or corporate roots
- keep trust-store selection in app configuration, not by patching `certifi`

## Common Pitfalls

- Do not hardcode the installed path to `cacert.pem`; call `certifi.where()` at runtime.
- Do not treat `certifi.contents()` as a filename; it returns the PEM text.
- Do not edit the packaged CA bundle in `site-packages`; upstream explicitly does not support adding or removing certificates in place.
- Do not expect `certifi` updates to be automatic. If you need newer Mozilla roots, upgrade the package.
- Do not assume `certifi` itself changes hostname verification or TLS policy; it only supplies trust anchors.

## Version-Sensitive Notes

- `2026.2.25` was released on 2026-02-25.
- This release requires Python `>=3.7`.
- `certifi` uses date-like package versions, not semantic versions. Compare releases by their full published version string.
- The current implementation delays locating or extracting `cacert.pem` until `where()` is called. In zipimport or similar packaged environments, treat the returned path as a runtime detail rather than a stable install path.

## When To Reach For Something Else

- If you need the operating system trust store, configure your TLS client to use the system defaults instead of forcing `certifi`.
- If you need to trust an internal CA, supply your own CA bundle or client configuration rather than mutating `certifi`.
- If you need an HTTP API client, use `requests`, `urllib3`, `httpx`, or another network library; `certifi` is only the certificate bundle.

## Official Sources

- PyPI project page: https://pypi.org/project/certifi/
- Upstream repository: https://github.com/certifi/python-certifi
- Source for runtime helpers: https://raw.githubusercontent.com/certifi/python-certifi/master/certifi/core.py
- Source for CLI entrypoint: https://raw.githubusercontent.com/certifi/python-certifi/master/certifi/__main__.py
