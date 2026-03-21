---
name: package
description: "JupyterLab package guide for Python projects using the JupyterLab 4.5.x application and extension workflow"
metadata:
  languages: "python"
  versions: "4.5.6"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "jupyterlab,jupyter,notebook,python,ide,extensions"
---

# JupyterLab Python Package Guide

## Golden Rule

Treat `jupyterlab` as a Python-installed web application built on `jupyter-server`, not as a normal import-and-call library. Install it in the same environment as the kernels and server extensions you need, launch it from the directory you want to expose, keep Jupyter Server authentication enabled unless another trusted auth layer is in front of it, and use prebuilt PyPI extensions instead of source builds.

## Install

Pin the package version your environment expects:

```bash
python -m pip install "jupyterlab==4.5.6"
```

Common environment-first workflow:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "jupyterlab==4.5.6"
```

If you use conda or mamba, install `jupyterlab` into the environment that will also hold the kernels and notebook dependencies:

```bash
mamba install -c conda-forge jupyterlab
```

Install JupyterLab in the same environment as packages that must be importable from notebooks. A common failure mode is launching JupyterLab from one interpreter while the notebook kernel or extension package lives in another.

## Start And Reconnect

Launch JupyterLab from the project directory you want to browse:

```bash
jupyter lab
```

Useful variants:

```bash
jupyter lab --notebook-dir "$PWD"
jupyter lab --preferred-dir "$PWD"
jupyter server list
```

Operational notes:

- The initial terminal output includes a tokenized local URL when token auth is enabled.
- `jupyter server list` is the quickest way to rediscover running servers and reconnect URLs.
- JupyterLab workspaces are URL-based. Keep the workspace URL if you need to reopen the same layout later.

## Core Workflow

### Use the right environment and kernel

JupyterLab only provides the UI and server shell. Your code executes in a kernel environment. If imports fail inside notebooks even though `jupyter lab` starts cleanly, verify which Python executable the kernel uses and install the package there.

### Inspect where JupyterLab is reading and writing state

These commands help when settings, extensions, or workspaces do not behave as expected:

```bash
jupyter lab path
jupyter --paths
```

Important locations from the upstream docs:

- Application directory: where the built frontend assets and staging data live. Override with `--app-dir` or `JUPYTERLAB_DIR`.
- User settings directory: defaults under `~/.jupyter/lab/user-settings/`. Override with `JUPYTERLAB_SETTINGS_DIR`.
- Workspaces directory: defaults under `~/.jupyter/lab/workspaces/`. Override with `JUPYTERLAB_WORKSPACES_DIR`.
- `labconfig/page_config.json`: used to disable, defer, or otherwise tune frontend extensions.

### Export or restore a workspace

JupyterLab ships a workspace CLI so layout state can be captured or reapplied:

```bash
jupyter lab workspaces export
jupyter lab workspaces import workspace.json
```

Use this in devcontainer, classroom, or remote-environment setups where a known sidebar and tab layout saves time.

## Configuration And Authentication

JupyterLab uses Jupyter Server for auth and transport. By default, a server with no password set requires a generated token in the URL or in an `Authorization: token ...` header.

Set a password when you need a stable local login flow:

```bash
jupyter server password
```

Key auth behavior from the upstream security docs:

- If neither token nor password is set, the server is effectively open to whoever can reach it.
- If you set a password, token auth is disabled by default.
- Tokens should stay enabled for ad hoc local and remote-tunnel workflows unless you have another deliberate auth boundary.

Common server config lives in `jupyter_server_config.py` or `jupyter_server_config.json`. Typical settings agents need to review:

- `ServerApp.root_dir`: what files the server can browse
- `ServerApp.ip`: whether the server binds only to localhost or a broader interface
- `ServerApp.open_browser`: whether a browser window opens automatically
- TLS, reverse-proxy, and password settings when exposing a remote server

If you expose JupyterLab outside localhost, do it behind explicit network controls and authenticated access. Do not rely on obscurity or a copied token URL as your only protection.

## Extensions

For JupyterLab 4.x, prefer prebuilt extensions distributed as Python packages on PyPI. The Extension Manager uses PyPI and `pip` by default in current stable docs, and it only shows packages marked with the PyPI classifier for prebuilt JupyterLab extensions.

Practical workflow:

1. Prefer the extension author's install instructions if they publish a Python package.
2. Install the package into the same environment as JupyterLab.
3. Restart JupyterLab and verify the extension appears.
4. Use `jupyter labextension list` when debugging enablement or version mismatches.

Administrative controls worth knowing:

- The Extension Manager can be disabled or switched to `readonly`.
- Extension listings can be constrained with allowlist or blocklist URLs.
- Frontend plugins and whole extensions can be disabled through `page_config.json`.

Avoid older source-build guidance unless an extension explicitly requires it. Source extension installs pull in Node.js and a frontend rebuild path that most agent workflows do not want.

## Common Pitfalls

- Stable ReadTheDocs pages are currently labeled as JupyterLab `4.5.0` docs even though PyPI lists `4.5.6`. Treat the stable docs as the `4.5.x` reference and confirm patch-level drift on PyPI before assuming a bug is fixed or a behavior changed.
- `jupyterlab` starting successfully does not prove notebook imports will work. Kernel environments and server environments are often different.
- Editing the wrong directory is common. Check `jupyter lab path` before modifying settings, workspaces, or app assets.
- Installing an extension package into a kernel-only environment will not make it available to the JupyterLab UI.
- Extension names, Python package names, and frontend plugin IDs are not always the same string.
- Remote servers should not bind to a public interface without deliberate password or token handling, TLS, and network controls.
- If you suppress build checks or run with a shared app directory, make sure that is intentional. Otherwise you can hide broken extension state from yourself.

## Version-Sensitive Notes

- PyPI lists `jupyterlab 4.5.6` as the current package version covered by this doc.
- The upstream stable docs still render as the `4.5.0` documentation branch. That is a documentation-label mismatch, not a reason to downgrade.
- JupyterLab 4 changed extension discovery and management substantially versus older 3.x guidance; prefer 4.x docs and prebuilt-extension instructions.
- The JupyterLab changelog notes JupyterLab 3 reached end of maintenance on May 15, 2024. Do not follow 3.x extension-install advice unless you are pinned to an old deployment.
- If you are automating against UI behavior, check the 4.5 changelog first; terminal, debugger, and notebook interaction details changed during the 4.5 series.

## Official Sources

- JupyterLab docs: `https://jupyterlab.readthedocs.io/en/stable/`
- Installation: `https://jupyterlab.readthedocs.io/en/stable/getting_started/installation.html`
- Starting and workspace URLs: `https://jupyterlab.readthedocs.io/en/stable/getting_started/starting.html`
- Directories and config paths: `https://jupyterlab.readthedocs.io/en/stable/user/directories.html`
- Extension manager and listings: `https://jupyterlab.readthedocs.io/en/stable/user/extensions.html`
- Changelog: `https://jupyterlab.readthedocs.io/en/stable/getting_started/changelog.html`
- Jupyter Server security: `https://jupyter-server.readthedocs.io/en/stable/operators/security.html`
- PyPI package page: `https://pypi.org/project/jupyterlab/`
- PyPI JSON metadata: `https://pypi.org/pypi/jupyterlab/json`
