---
name: package
description: "Legacy intercom-python package for Intercom API access; treat as an unfinished 2015 package and prefer the maintained python-intercom SDK for new code"
metadata:
  languages: "python"
  versions: "1.0.6"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "intercom,crm,customer-support,api,legacy"
---

# intercom-python Python Package Guide

## Golden Rule

Treat `intercom-python==1.0.6` as a legacy package, not a current Intercom SDK. The live PyPI project still says it is "not official" and "Not yet done!!!!!", and the docs URL points at Intercom's maintained `python-intercom` repository, which is a different package entirely. For new code, use `python-intercom`; only touch `intercom-python` when an existing environment is already pinned to it.

## Install

If a project is already pinned to the legacy package, install that exact version and inspect it locally before writing code against it:

```bash
python -m pip install "intercom-python==1.0.6"
```

The maintained Intercom SDK is a different package:

```bash
python -m pip install "python-intercom==5.0.1"
```

## Reality Check Before You Code

There are no reliable maintainer-published docs for the runtime API of `intercom-python==1.0.6` beyond the PyPI stub, so do not invent imports or method names. Inspect the installed package first:

```bash
python - <<'PY'
import importlib

for name in ("intercom", "intercom_python"):
    try:
        module = importlib.import_module(name)
        public = [item for item in dir(module) if not item.startswith("_")]
        print(name, module.__file__)
        print(public[:50])
    except Exception as exc:
        print(name, "IMPORT_FAILED", exc)
PY
```

Then inspect the actual code surface:

```bash
python - <<'PY'
import inspect
import importlib

module = importlib.import_module("intercom")
print(inspect.getsource(module))
PY
```

If the import name is different in your environment, adjust the second snippet to match what the first one discovered.

## Safer Setup For New Code

If you are not forced to keep `intercom-python`, migrate to Intercom's maintained SDK instead. The current maintainer docs show this initialization pattern:

```python
import os
from intercom import Intercom

client = Intercom(token=os.environ["INTERCOM_ACCESS_TOKEN"])
```

Authentication notes from Intercom's current REST docs:

- Use a bearer token in the `Authorization` header
- The default API hostname is `https://api.intercom.io`
- Region-specific hosts also exist for EU and AU workspaces, so confirm the workspace region before hard-coding URLs

## Core Usage Guidance

For `intercom-python==1.0.6` itself, use this workflow instead of assuming a modern SDK shape:

1. Install the exact pinned version.
2. Discover the real import path and exported symbols locally.
3. Check whether the package wraps HTTP requests directly or expects raw `requests` usage around it.
4. Verify every endpoint against current Intercom REST docs before using it in production.

For maintained integrations, prefer `python-intercom` and use the official namespace-based client methods documented in the repo reference and README.

## Configuration And Auth

`intercom-python` is old enough that its auth behavior cannot be trusted without local inspection. Before using it, verify all of the following in the installed code:

- Whether it expects an access token, basic auth, or some older Intercom credential format
- Whether it hard-codes `https://api.intercom.io` or allows region-specific base URLs
- Whether it sets request timeouts, retries, or pagination helpers
- Whether it depends on deprecated Intercom API paths

For the maintained SDK, keep the token in `INTERCOM_ACCESS_TOKEN` and let the client handle the `Authorization: Bearer ...` header.

## Common Pitfalls

- `intercom-python` and `python-intercom` are different packages. Do not mix their install commands or examples.
- The docs URL currently points to the maintained `python-intercom` repo, not the legacy `intercom-python` package being curated here.
- PyPI shows `intercom-python` as an unfinished package last released in 2015. That is a strong signal to avoid it for greenfield work.
- Old Intercom examples on blogs or gists may target deprecated authentication or endpoint behavior. Check the current REST docs before sending live requests.
- Intercom has region-specific API hosts. If you drop to raw HTTP, use the correct workspace region instead of assuming the US hostname.
- Do not assume async support, retries, or pagination helpers in `intercom-python`; verify them in the installed source.

## Version-Sensitive Notes

- `intercom-python 1.0.6` appears to be the latest and only practically relevant release of this legacy package on PyPI.
- Intercom's maintained package is `python-intercom 5.0.1`, released on `2026-01-30`, with current docs in `intercom/python-intercom`.
- The maintained SDK docs say `python-intercom` requires Python `>=3.8`. `intercom-python` predates that by a decade and may reflect Python 2 / early Python 3 assumptions.

## Official Sources Used

- PyPI package page for `intercom-python`: `https://pypi.org/project/intercom-python/`
- PyPI package page for Intercom's maintained SDK: `https://pypi.org/project/python-intercom/`
- Maintained SDK repository: `https://github.com/intercom/python-intercom`
- Intercom REST API auth and base URL docs: `https://developers.intercom.com/docs/references/2.13/rest-api/api.intercom.io/admins`
