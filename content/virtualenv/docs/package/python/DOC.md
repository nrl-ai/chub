---
name: package
description: "virtualenv package guide for creating isolated Python environments with the official virtualenv docs"
metadata:
  languages: "python"
  versions: "21.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "virtualenv,python,packaging,environments,venv,pip"
---

# virtualenv Python Package Guide

## Golden Rule

Use `virtualenv` when you need a fast, configurable environment creator beyond the stdlib `venv` defaults, and create environments against an explicit interpreter whenever the Python version matters.

For this entry, the version used here `21.2.0` matches the live upstream release on PyPI dated March 9, 2026.

## Install

Preferred install via `pipx` so the tool stays isolated from project environments:

```bash
pipx install virtualenv
virtualenv --help
```

Pin the exact package version when reproducibility matters:

```bash
python -m pip install --user "virtualenv==21.2.0"
python -m virtualenv --help
```

Use the published zipapp if you need a no-install path:

```bash
curl -LO https://bootstrap.pypa.io/virtualenv.pyz
python virtualenv.pyz --help
```

## Create Environments

Basic creation:

```bash
python -m virtualenv .venv
```

Use an explicit interpreter path for stable builds:

```bash
/opt/python/3.12/bin/python -m virtualenv .venv
```

Use interpreter discovery rules when you want `virtualenv` to search:

```bash
virtualenv --python=3.12 .venv
virtualenv --python=">=3.12" .venv
virtualenv --python=cpython3.12-64-arm64 .venv
```

If your machine has version-manager shims or multiple candidates, use `--try-first-with` as a hint before the normal search:

```bash
virtualenv --python=3.12 --try-first-with ~/.pyenv/versions/3.12.9/bin/python .venv
```

## Activate Or Use Directly

Activation is optional. You can always call the environment's binaries directly.

POSIX shells:

```bash
source .venv/bin/activate
python --version
python -m pip install -U pip
deactivate
```

Windows `cmd.exe`:

```bat
.\.venv\Scripts\activate
python --version
```

PowerShell:

```powershell
.\.venv\Scripts\Activate.ps1
```

If PowerShell blocks activation scripts, the upstream docs recommend:

```powershell
Set-ExecutionPolicy RemoteSigned
```

## Core Options You Will Actually Use

Create an env that can still see global packages:

```bash
virtualenv --system-site-packages .venv
```

Force copies instead of symlinks:

```bash
virtualenv --copies .venv
```

Blow away and recreate an existing destination:

```bash
virtualenv --clear .venv
```

Skip VCS ignore generation:

```bash
virtualenv --no-vcs-ignore .venv
```

Create a bare environment without seeded package installers:

```bash
virtualenv --no-seed .venv
```

Control which activation scripts are generated and the prompt prefix:

```bash
virtualenv --activators bash,powershell --prompt . .venv
```

The default activators cover `bash`, `batch`, `cshell`, `fish`, `nushell`, `powershell`, and `python`.

## Seeding, Caches, And Offline-ish Setups

`virtualenv` creates environments in two phases:

1. Discover the base interpreter.
2. Build the environment, seed packages, install activators, and create VCS ignore files.

For seeding, the default method is `app-data`, which caches install images and makes repeat environment creation much faster than invoking `pip` from scratch every time.

Useful flags:

```bash
virtualenv --download .venv
virtualenv --no-periodic-update .venv
virtualenv --extra-search-dir /srv/wheels --extra-search-dir /opt/wheels .venv
virtualenv --upgrade-embed-wheels
```

Important defaults in current docs:

- `--seeder app-data`
- `--no-download` is the default
- `--pip bundle`
- `--setuptools none`

That last point matters on modern Python: upstream docs say `setuptools` is disabled by default for Python 3.12+ environments, and `wheel` is only installed by default on Python 3.8.

## Config And Authentication

There is no service authentication layer in `virtualenv`. Configuration is local: CLI flags, environment variables, and `virtualenv.ini`.

The docs say `virtualenv` looks for a standard `virtualenv.ini` unless `VIRTUALENV_CONFIG_FILE` overrides the path. Run `virtualenv --help` to print the active config-file location.

Useful environment variables:

```bash
export VIRTUALENV_PYTHON=/opt/python-3.12/bin/python
export VIRTUALENV_EXTRA_SEARCH_DIR=/srv/wheels,/opt/wheels
export VIRTUALENV_OVERRIDE_APP_DATA=/var/cache/virtualenv
```

Notes:

- Environment variable names mirror long CLI options with a `VIRTUALENV_` prefix.
- Options that accept multiple values, such as `VIRTUALENV_PYTHON` and `VIRTUALENV_EXTRA_SEARCH_DIR`, can use comma-separated or newline-separated values.
- The app-data directory is the seed cache. Override it when you need a shared or deterministic cache location in CI or managed build images.

## Use From Python

The supported Python API is CLI-shaped. Use `cli_run()` to create an environment and `session_via_cli()` if you only need the resolved session data.

```python
from virtualenv import cli_run, session_via_cli

session = cli_run(["--python=3.12", ".venv"])
print(session.interpreter)

preview = session_via_cli(["--python=3.12", ".venv"])
print(preview.creator)
```

The returned session object is documented as experimental. Treat it as inspection data, not a stable long-term interface.

## Common Pitfalls

- Do not rely on a generic launcher like `python3 -m virtualenv .venv` if your system updates `/usr/bin/python3` under you. Use an exact interpreter path or an explicit specifier.
- Activation is not required. In automation, calling `.venv/bin/python` or `.venv/Scripts/python.exe` is usually cleaner and less shell-dependent.
- `--system-site-packages` breaks isolation by design. Only use it when your toolchain truly expects global packages.
- If builds need internet-free seeding, populate wheel locations up front and use `--extra-search-dir`; `--download` is off by default.
- PowerShell activation can fail because of execution policy, not because the environment is broken.
- If your tooling imports `virtualenv.discovery.*` internals directly, that is now a migration target, not a stable contract.

## Version-Sensitive Notes For 21.2.0

- `21.2.0` was released on March 9, 2026.
- `21.2.0` fixes `--no-vcs-ignore` being ignored on the subprocess `venv` path for Python 3.13+.
- `21.2.0` fixes bash activation relocation fallback by using `BASH_SOURCE[0]`, which matters if the activate script is sourced from a different working directory.
- `21.1.0` adds inline type annotations and ships `py.typed`, so static type checkers now recognize `virtualenv` as a typed package.
- `21.0.0` extracts Python discovery into the separate `python-discovery` dependency. If older code imports `virtualenv.discovery.py_info.PythonInfo` or similar internals, switch to `python_discovery` instead of depending on compatibility shims.
- `20.38.0` moved app-data storage to the OS cache directory and migrates existing cache data on first use. If your CI or container image pinned the old cache path, update that assumption.

## Official Sources

- Stable docs: https://virtualenv.pypa.io/en/stable/
- Installation: https://virtualenv.pypa.io/en/stable/installation.html
- User guide: https://virtualenv.pypa.io/en/stable/user_guide.html
- CLI reference: https://virtualenv.pypa.io/en/stable/cli_interface.html
- Release history: https://virtualenv.pypa.io/en/stable/changelog.html
- PyPI release page: https://pypi.org/project/virtualenv/21.2.0/
