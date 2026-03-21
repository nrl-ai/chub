---
name: package
description: "pip package guide for Python installs, environments, requirements, indexes, and configuration"
metadata:
  languages: "python"
  versions: "26.0.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "pip,python,packaging,pypi,requirements,virtualenv"
---

# pip Python Package Guide

## Golden Rules

- Use `python -m pip ...` instead of bare `pip ...` when you need to target a specific interpreter.
- Install into an isolated virtual environment unless you intentionally manage a system or base interpreter.
- Treat `pip` as a command-line tool, not a library. Do not `import pip` inside application code.
- Prefer a single trusted index for automated installs. Be careful with `--extra-index-url` because it can introduce dependency-confusion risk.

## What `pip` Is For

`pip` is the standard package installer for Python. It installs packages from package indexes, local archives, local project directories, VCS URLs, and requirement files.

For coding agents, the common workflow is:

1. Select the correct Python interpreter.
2. Create or activate a virtual environment.
3. Upgrade `pip` in that environment.
4. Install from `requirements.txt`, a constraints file, or explicit package specifiers.
5. Run `pip check` or equivalent validation before assuming the environment is usable.

## Install and Bootstrap

### Check the active `pip`

```bash
python -m pip --version
python3 -m pip --version
py -m pip --version
```

### Upgrade `pip`

```bash
python -m pip install --upgrade pip
```

### If `pip` is missing

Use `ensurepip` where it is available:

```bash
python -m ensurepip --upgrade
```

The official installation guide also provides `get-pip.py` for bootstrap scenarios where `pip` is not already present.

## Recommended Environment Setup

Create a virtual environment and upgrade `pip` inside it:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
```

Windows PowerShell:

```powershell
py -m venv .venv
.venv\Scripts\Activate.ps1
py -m pip install --upgrade pip
```

If `pip` is installed for one interpreter but you need to manage another, use the global `--python` option:

```bash
python -m pip --python /path/to/python install requests
```

That is the safest way to avoid installing into the wrong environment when multiple Python versions are present.

## Core Commands

### Install packages

```bash
python -m pip install requests
python -m pip install "urllib3>=2,<3"
python -m pip install ./local-project
python -m pip install "git+https://github.com/pypa/pip.git"
```

### Upgrade or reinstall

```bash
python -m pip install --upgrade requests
python -m pip install --force-reinstall requests
```

### Uninstall

```bash
python -m pip uninstall requests
```

### Inspect what is installed

```bash
python -m pip list
python -m pip show requests
python -m pip freeze
python -m pip check
```

Use `pip freeze` for a snapshot of installed distributions. Use `pip check` to detect broken or incompatible installed requirements after changes.

## Requirements, Constraints, and Repeatable Installs

### Install from a requirements file

```bash
python -m pip install -r requirements.txt
```

Generate a simple pinned snapshot:

```bash
python -m pip freeze > requirements.txt
```

### Use constraints to limit resolver choices

```bash
python -m pip install -r requirements.txt -c constraints.txt
```

Constraints files do not trigger installation by themselves. They only limit versions that the resolver may choose for requirements requested elsewhere.

### Build constraints

For modern `pip`, build constraints can restrict versions used only inside isolated build environments:

```bash
python -m pip install --build-constraint build-constraints.txt .
```

This is useful when builds need older or pinned build dependencies without constraining the final runtime environment.

### Hash-checking mode for stronger repeatability

```bash
python -m pip install --require-hashes -r requirements.txt
```

Use this when you need repeatable installs with explicit artifact hashes.

## Indexes, Private Repositories, and Authentication

### Install from a specific index

```bash
python -m pip install --index-url https://pypi.org/simple requests
```

### Add credentials in the URL

```bash
python -m pip install --index-url https://username:password@example.com/simple internal-package
```

For HTTP basic auth, credentials can be embedded in the index URL. If a username or password contains reserved characters, percent-encode them.

### Prefer a single trusted index in automation

`--extra-index-url` is convenient, but the official docs warn it is unsafe because a public package can be chosen instead of an internal one. If you must use a private repository, prefer a single authoritative index or a controlled mirror.

### Keyring behavior

`pip` can use keyring integrations for credential lookup. In non-interactive automation, `--no-input` disables prompting, which can also affect keyring usage.

## Configuration

`pip` reads settings from three places, in this order of precedence:

1. Command-line options
2. Environment variables
3. Configuration files

### Useful environment variables

```bash
export PIP_INDEX_URL=https://mirror.example.com/simple
export PIP_EXTRA_INDEX_URL=https://packages.example.com/simple
export PIP_TIMEOUT=60
```

Long command-line options map to environment variables as `PIP_<OPTION_NAME>`. For example, `--index-url` becomes `PIP_INDEX_URL`.

### Configure with `pip config`

```bash
python -m pip config set global.index-url https://mirror.example.com/simple
python -m pip config set global.timeout 60
python -m pip config list
```

### Common config file locations

- Unix global: `/etc/xdg/pip/pip.conf`, then `/etc/pip.conf`
- macOS global: `/Library/Application Support/pip/pip.conf`
- User config: `~/.config/pip/pip.conf`
- Legacy user config: `~/.pip/pip.conf`
- Virtualenv site config: `$VIRTUAL_ENV/pip.conf`
- Windows user config: `%APPDATA%\\pip\\pip.ini`

Use the site-specific config inside a virtual environment when a project needs settings that should not leak into the user or system interpreter.

## Common Pitfalls

### Installing into the wrong interpreter

This is the most common `pip` mistake on systems with multiple Python versions. Prefer:

```bash
python -m pip install ...
```

not:

```bash
pip install ...
```

### Using `pip` outside a virtual environment by default

Project dependencies installed into a global interpreter are harder to reproduce and easier to break. Use `venv` unless you have an explicit reason not to.

### Importing `pip` as a library

The official user guide says you must not use `pip`'s internal APIs from your code. If you need package-management automation, shell out to `python -m pip` with `subprocess.run(...)`.

### Trusting `pip freeze` as a full lock workflow

`pip freeze` captures what is installed now. It does not replace reviewing resolver behavior, index policy, platform differences, or build dependencies.

### Mixing public and private indexes casually

If the same package name can exist on both public and private indexes, the wrong distribution may be selected. This matters most when using `--extra-index-url`.

## Version-Sensitive Notes for `26.0.1`

- The `pip lock` command now exists in the stable command set, but lockfile workflows are still newer than classic `requirements.txt` plus constraints patterns. Check the current command page before standardizing on it across teams.
- Build constraints are documented as available starting in `25.3`. Older `pip` versions do not support `--build-constraint`.
- `pip` only supports Python versions that are not end-of-life. On older Python interpreters, you may need an older `pip` release instead of `26.0.1`.

## Minimal Automation Pattern

```python
import subprocess
import sys

def pip(*args: str) -> None:
    subprocess.run([sys.executable, "-m", "pip", *args], check=True)

pip("install", "--upgrade", "pip")
pip("install", "-r", "requirements.txt")
pip("check")
```

This keeps `pip` bound to the current interpreter and avoids relying on unsupported internal imports.

## Official URLs

- Documentation: https://pip.pypa.io/en/stable/
- Installation: https://pip.pypa.io/en/stable/installation/
- User guide: https://pip.pypa.io/en/stable/user_guide/
- Configuration: https://pip.pypa.io/en/stable/topics/configuration/
- Authentication: https://pip.pypa.io/en/stable/topics/authentication/
- Install command reference: https://pip.pypa.io/en/stable/cli/pip_install/
- Lock command reference: https://pip.pypa.io/en/stable/cli/pip_lock/
- PyPI package: https://pypi.org/project/pip/
