---
name: package
description: "Jupyter Notebook package guide for Python with install, launch, configuration, security, extensions, and Notebook 7 migration notes"
metadata:
  languages: "python"
  versions: "7.5.5"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "notebook,jupyter,python,server,ipynb,interactive,development"
---

# notebook Python Package Guide

## Golden Rule

Treat `notebook` as a Jupyter application package, not as a normal Python library API. Start it with the `jupyter notebook` CLI, keep server configuration under Jupyter Server settings, and assume older Notebook 6 examples may be wrong for Notebook 7 unless they were updated for Jupyter Server and JupyterLab-based extensions.

## Install

Use a virtual environment unless you intentionally want a global Notebook install:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "notebook==7.5.5"
```

The Jupyter install guide also covers distribution-based installs such as Anaconda if your environment is managed that way.

Useful sanity checks:

```bash
jupyter --version
jupyter notebook --help
python -m pip show notebook
```

Notebook installs the IPython kernel for Python. If you need additional languages, install their kernels separately so they appear in the Notebook UI.

## Launch And Open Notebooks

Start the server in the current directory:

```bash
jupyter notebook
```

Open a specific notebook file:

```bash
jupyter notebook analysis.ipynb
```

Run on a fixed port without opening a browser:

```bash
jupyter notebook --no-browser --port 9999
```

The dashboard reflects the directory where you launched the command, so start the server in the project root or the notebook workspace you actually want exposed.

## Core Usage

### Create or edit notebooks interactively

Notebook is primarily a browser UI for `.ipynb` documents. Typical workflow:

1. Launch `jupyter notebook`
2. Open the dashboard in a browser
3. Create or open a notebook
4. Select the right kernel
5. Run cells incrementally and save the `.ipynb`

If code should run headlessly from the terminal, prefer the Jupyter CLI instead of trying to drive the browser manually:

```bash
jupyter execute analysis.ipynb
```

That is the simplest official path for non-interactive notebook execution.

### Use the right kernel

Notebook itself is not the kernel. The kernel controls what runtime executes code cells.

- Python notebooks normally use the bundled IPython kernel
- Other languages need separately installed kernels
- If the wrong environment is selected, imports and package resolution will fail even though the notebook server itself is running correctly

### Prefer Jupyter Server APIs for server-side code

Notebook 7 is built on Jupyter Server. If you are writing server extensions, auth helpers, or server-side integrations, use Jupyter Server imports instead of older Notebook 6 import paths.

Examples from the migration guidance:

```python
from jupyter_server.auth import passwd
from jupyter_server import serverapp
```

Do not assume older imports like `from notebook.auth import passwd` or `from notebook import notebookapp` are the current interfaces for Notebook 7 code.

### Extension model changed in Notebook 7

Notebook 7 uses the same extension system as JupyterLab for front-end customization and Jupyter Server for server extensions.

Practical implication:

- Old classic Notebook front-end extensions often do not work unchanged
- New extension work should target Notebook 7 / JupyterLab compatibility
- If a project depends on classic behavior, confirm whether it actually needs `nbclassic` instead of Notebook 7

## Configuration And Authentication

### Generate config

Notebook 7 relies on Jupyter Server configuration. Generate a config file with:

```bash
jupyter server --generate-config
```

Important server settings usually live in `jupyter_server_config.py` or `jupyter_server_config.json`.

Useful settings for coding agents and deployment scripts:

- `c.ServerApp.ip`
- `c.ServerApp.port`
- `c.ServerApp.open_browser`
- `c.ServerApp.root_dir`
- `c.ServerApp.base_url`
- `c.ServerApp.token`
- `c.ServerApp.password`
- `c.ServerApp.certfile`
- `c.ServerApp.keyfile`

Minimal local-dev example:

```python
c.ServerApp.ip = "127.0.0.1"
c.ServerApp.port = 8888
c.ServerApp.open_browser = False
c.ServerApp.root_dir = "/path/to/project"
```

### Token and password behavior

Jupyter Server uses token authentication by default. For local development, the launch output includes a URL with the token.

To set a password interactively:

```bash
jupyter server password
```

That command writes password configuration to `jupyter_server_config.json`. In Jupyter Server, values from the JSON config file take precedence over the Python config file, so check both if a setting appears to be ignored.

Or generate a hashed password in Python:

```python
from jupyter_server.auth import passwd

hashed = passwd("replace-me")
print(hashed)
```

If a password is set, token auth is no longer the default requirement. For remote access, use HTTPS and a real password or a higher-level deployment solution.

### Remote access and public servers

For a remote single-user server, you usually need all of:

- `c.ServerApp.ip = "0.0.0.0"` or another externally reachable address
- a password or token strategy you control deliberately
- TLS via `certfile` and `keyfile` or a trusted reverse proxy
- a fixed `base_url` when serving behind a subpath proxy

Notebook's public-server guidance is for a single-user server. If multiple users need separate sessions, use JupyterHub instead of exposing one Notebook server to many users.

## Common Pitfalls

- Notebook 7 is not Notebook 6 with a small skin change. It is based on Jupyter Server and JupyterLab components, so old extension and import examples are often stale.
- Use `ServerApp` settings for new configuration work. Older `NotebookApp` references are common in blog posts and issue threads.
- Do not import server auth helpers from old `notebook.*` modules in new code. Use `jupyter_server.*` imports instead.
- The launched server root is the current working directory unless you override `root_dir`. Agents often expose the wrong folder by launching from the wrong directory.
- A running Notebook server does not guarantee the notebook kernel is correct. Kernel/environment mismatches are a common source of "package installed but import fails" confusion.
- Browser access to a remote Notebook server without TLS and deliberate auth is unsafe.
- Classic Notebook front-end extensions and `nbextensions` guidance may not apply to Notebook 7. Validate extension compatibility before copying old setup steps.
- If a workflow explicitly requires the classic interface, install and use `nbclassic` rather than assuming Notebook 7 will behave the same.

## Version-Sensitive Notes For 7.5.5

- PyPI `7.5.5` requires Python `>=3.9`; Python 3.8 is no longer supported in the 7.5 line.
- The `7.5` series updates Notebook to JupyterLab `4.5`; compatibility claims for front-end extensions should be checked against Notebook 7 / JupyterLab 4, not against Notebook 6-era tutorials.
- The `7.5.5` changelog entry is a maintenance release that updates Notebook to JupyterLab `4.5.6` and includes dependency and CI maintenance. It is not a major behavioral rewrite, but it is still the patch level to target when the project says `7.5.5`.
- Upstream docs currently have minor patch-version header drift. When patch versions disagree, use the changelog and PyPI page to resolve the real current version.

## Official Sources

- Notebook docs root: `https://jupyter-notebook.readthedocs.io/en/stable/`
- Notebook changelog: `https://jupyter-notebook.readthedocs.io/en/stable/changelog.html`
- Notebook configuration overview: `https://jupyter-notebook.readthedocs.io/en/stable/configuring/config_overview.html`
- Notebook extension configuration: `https://jupyter-notebook.readthedocs.io/en/stable/configuring/config_extensions.html`
- Notebook 7 server-extension migration: `https://jupyter-notebook.readthedocs.io/en/stable/migrating/server-extensions.html`
- Jupyter install docs: `https://docs.jupyter.org/en/latest/install/notebook-classic.html`
- Jupyter kernel install docs: `https://docs.jupyter.org/en/latest/install/kernel_install.html`
- Jupyter running docs: `https://docs.jupyter.org/en/latest/running.html`
- Jupyter Server security docs: `https://jupyter-server.readthedocs.io/en/stable/operators/security.html`
- Jupyter Server public-server docs: `https://jupyter-server.readthedocs.io/en/stable/operators/public-server.html`
- PyPI package page: `https://pypi.org/project/notebook/7.5.5/`
