---
name: package
description: "ipykernel package guide for registering Python environments as Jupyter kernels and embedding IPython kernels"
metadata:
  languages: "python"
  versions: "7.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "ipykernel,jupyter,ipython,kernel,notebook,python"
---

# ipykernel Python Package Guide

## Golden Rule

Use `ipykernel` to make a specific Python environment available to Jupyter frontends, or to embed an IPython kernel inside a running Python process. Most projects should not call low-level `ipykernel` APIs directly in application code. The common path is:

1. install `ipykernel` into the environment that should execute notebook code
2. register that environment as a kernelspec
3. let JupyterLab, Notebook, VS Code, or another frontend launch the kernel

If notebooks keep using the wrong interpreter, the problem is usually the kernelspec pointing at the wrong Python binary, not the notebook code itself.

## Install

Install `ipykernel` in the exact environment you want notebook cells to run in:

```bash
python -m pip install "ipykernel==7.2.0"
```

Common alternatives:

```bash
uv add "ipykernel==7.2.0"
poetry add "ipykernel==7.2.0"
```

Verify the package version from that interpreter:

```bash
python -c "import ipykernel; print(ipykernel.__version__)"
```

`ipykernel` is only the kernel package. It does not replace the frontend. If the machine does not already have a notebook frontend, install one separately, such as JupyterLab or Notebook, in whichever environment will host the UI.

## Register The Current Environment As A Jupyter Kernel

The usual setup command is:

```bash
python -m ipykernel install --user --name myenv --display-name "Python (myenv)"
```

What the flags mean:

- `--user`: write the kernelspec into the current user's Jupyter data directory
- `--name`: stable internal kernel name; this must be unique on the machine
- `--display-name`: label shown in Jupyter frontends

After installing, confirm Jupyter can see it:

```bash
jupyter kernelspec list
```

If you recreate the virtual environment later, rerun `python -m ipykernel install ...` so the kernelspec points at the new interpreter path.

## Install A Kernel For A Different Jupyter Environment

It is common to keep the notebook frontend in one environment and the runtime kernel in another. In that case, run the kernel environment's Python with `--prefix` pointing at the frontend environment:

```bash
/path/to/kernel-env/bin/python -m ipykernel install \
  --prefix /path/to/jupyter-env \
  --name project-env \
  --display-name "Python (project-env)"
```

This is the practical fix when:

- JupyterLab is installed in a shared tooling environment
- the project runtime lives in a separate `venv`, Conda env, or container image
- VS Code or Notebook sees Jupyter but not the project interpreter

## Edit Or Customize A Kernelspec

If you need to inspect or modify the generated kernelspec JSON before installing it permanently, the IPython install docs recommend this flow:

```bash
ipython kernel install --prefix /tmp
```

Then edit the generated `kernel.json` under `/tmp/...` and install it with `jupyter kernelspec install`.

Typical reasons to customize a kernelspec:

- add environment variables for the kernel process
- change the visible display name
- ship a kernel into a shared Jupyter deployment path
- pin a non-default interpreter location

Avoid editing random files under Jupyter data directories by hand unless you know which kernelspec is active.

## Programmatic Kernelspec Installation

If you are building tooling around environment bootstrap, `ipykernel.kernelspec.install(...)` exposes the same install path from Python.

```python
from ipykernel.kernelspec import install

install(
    user=True,
    kernel_name="project-env",
    display_name="Python (project-env)",
    env={"PYTHONUNBUFFERED": "1"},
)
```

Useful arguments in `7.2.0` include:

- `user`, `prefix`: choose install destination
- `kernel_name`, `display_name`: control stable ID vs UI label
- `env`: inject environment variables into the kernel process
- `frozen_modules`: control whether the generated kernelspec enables CPython frozen modules

Prefer the CLI unless you are already writing automation in Python.

## Embed A Kernel Inside A Running Process

`embed_kernel()` starts an IPython kernel inside the current process so an external frontend can connect to it.

```python
from ipykernel.embed import embed_kernel

def main() -> None:
    data = {"message": "debug me"}
    embed_kernel(local_ns={"data": data})

if __name__ == "__main__":
    main()
```

Use this for debugging or interactive inspection of long-running processes, not for normal request handling. The `embed_kernel()` docs note that keyword arguments only affect the first embed call in a given process.

## Connection Info And Runtime Introspection

Jupyter frontends launch kernels with a connection file that contains the transport, ports, and HMAC key. From inside a running kernel, you can inspect the active connection info:

```python
from ipykernel.connect import get_connection_info

info = get_connection_info(unpack=True)
print(info["ip"], info["shell_port"])
```

Use this for debugging integration issues. Do not log or commit raw connection files or keys.

## Configuration And Security Notes

`ipykernel` does not use cloud-style API authentication. The important configuration surface is the Jupyter kernelspec and the connection file handed to the kernel process.

Practical rules:

- treat the kernelspec as the source of truth for which interpreter launches
- use `--display-name` for human-readable labels and `--name` for stable automation-friendly IDs
- use `env=` in the kernelspec only for non-secret defaults; prefer your normal secret-management path for credentials
- treat the connection file as sensitive because it contains the HMAC signing key and open ports
- let the frontend launch `ipykernel_launcher` with its generated `-f <connection-file>` argument instead of trying to hand-craft launch commands

If you are debugging kernel startup, inspect the active kernelspec before changing notebook code.

## Common Pitfalls

- Installing `ipykernel` into one environment does not automatically make another environment's Jupyter frontend use it. Register a kernelspec explicitly.
- Recreating or moving a virtual environment leaves old kernelspecs behind. If notebooks suddenly start the wrong interpreter, delete or reinstall the stale kernelspec.
- `--name` collisions overwrite the existing kernelspec for that internal name. Use stable, distinct names per environment.
- `ipykernel` is not a full frontend stack. You may still need `jupyterlab`, `notebook`, or an editor integration.
- Do not import low-level `ipykernel` internals just to run notebook code. Frontends start the kernel for you.
- `embed_kernel()` can block the process waiting for a client connection, so do not drop it into normal production request paths.
- If you are building a custom Jupyter integration, distinguish `ipykernel` from `jupyter_client`: the former runs the Python kernel, the latter manages client-side protocol connections.

## Version-Sensitive Notes For 7.x

- `ipykernel 7.0.0` introduced experimental subshell support and the changelog explicitly warns that downstream consumers could hit compatibility issues. If a toolchain is not ready for `7.x`, pin `<7` intentionally instead of mixing assumptions from old `6.x` material.
- `ipykernel 7.2.0` updates the kernel protocol version advertised in the kernelspec to `5.5`.
- `7.2.0` also switched the IOPub publisher socket to XPUB and upgraded `jupyter_client` to `8.8.0`, which matters mainly if you maintain tooling at the protocol or socket layer.
- The `frozen_modules` kernelspec option exists in `7.x`; it can improve startup behavior on newer CPython builds, but it may interact badly with some debugging workflows for frozen stdlib modules.

When copying older notebook setup instructions, check whether they were written for `6.x` because kernelspec details and protocol behavior changed in the `7.x` line.

## Official Sources Used For This Guide

- Stable docs: `https://ipykernel.readthedocs.io/en/stable/`
- Changelog: `https://ipykernel.readthedocs.io/en/stable/changelog.html`
- API reference: `https://ipykernel.readthedocs.io/en/stable/api/ipykernel.html`
- IPython kernel install guide: `https://ipython.readthedocs.io/en/stable/install/kernel_install.html`
- PyPI package page: `https://pypi.org/project/ipykernel/`
