---
name: package
description: "nodeenv Python package guide for creating isolated Node.js environments from Python tooling"
metadata:
  languages: "python"
  versions: "1.10.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "nodeenv,nodejs,npm,python,virtualenv,cli"
---

# nodeenv Python Package Guide

## Golden Rule

Treat `nodeenv` as a CLI for creating an isolated Node.js toolchain under a directory or inside an existing Python virtual environment. Pin both the Python package version and the Node.js version you want, prefer the default prebuilt install path unless you specifically need a source build, and use the generated activation script before running `node`, `npm`, or globally installed npm CLIs.

## Install

Pin the package version in your Python environment:

```bash
python -m pip install "nodeenv==1.10.0"
```

Check that the CLI is available:

```bash
nodeenv --version
python -m nodeenv --help
```

Maintainer docs list these prerequisites:

- for `nodeenv` itself: Python, `make`, and `tail`
- for building Node.js with SSL support from source: `libssl-dev`

If you only use the default prebuilt downloads, you usually do not need a full source-build toolchain.

## Create A Standalone Node Environment

Create an environment in a project-local directory and pin the Node.js version explicitly:

```bash
python -m nodeenv --node=22.11.0 .nodeenv
source .nodeenv/bin/activate
node --version
npm --version
```

Useful version selectors from the maintainer docs:

- `--node=latest`: latest stable release
- `--node=lts`: latest LTS release
- `--node=system`: use the system-installed `node` instead of downloading one

The package defaults to `--prebuilt`, which installs Node.js from a prebuilt archive when available.

## Add Node To An Existing Python Virtual Environment

Use `-p` when you already have a Python virtual environment or Conda environment active and want Node.js tools to live inside that same prefix:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install "nodeenv==1.10.0"
nodeenv -p --node=22.11.0
node --version
npm --version
```

`nodeenv -p` resolves the target prefix from the active Python environment. In the source, it checks the current virtualenv or Conda prefix and errors if no Python virtual environment is active.

## Install And Reapply npm Packages From A Requirements File

`nodeenv` supports a plain-text requirements file with one npm package per line:

```text
eslint@8.57.1
typescript@5.6.3
prettier@3.6.2
```

Create a new environment and install those packages:

```bash
nodeenv --node=22.11.0 --requirements=requirements.txt .nodeenv
source .nodeenv/bin/activate
eslint --version
```

Update packages later without reinstalling Node.js:

```bash
source .nodeenv/bin/activate
nodeenv --requirements=requirements.txt --update .nodeenv
```

For modern npm versions, the implementation installs these packages with `npm install -g` inside the environment.

## Capture The Current Global Package Set

After activation, `nodeenv` defines a `freeze` shell function that writes installed npm packages in a format you can reuse later:

```bash
source .nodeenv/bin/activate
npm install -g eslint prettier
freeze requirements.txt
```

Use `freeze -l` to capture only packages local to the current environment:

```bash
freeze -l requirements-local.txt
```

## Activation And Environment Variables

The generated activation script is the normal entry point for shell usage:

```bash
source .nodeenv/bin/activate
echo "$NODE_VIRTUAL_ENV"
echo "$NPM_CONFIG_PREFIX"
```

When activated, `nodeenv`:

- sets `NODE_VIRTUAL_ENV` to the environment path
- prepends the environment `bin` directory and `lib/node_modules/.bin` to `PATH`
- sets `NODE_PATH` to the environment's module directory
- sets `NPM_CONFIG_PREFIX` and `npm_config_prefix` to the environment prefix
- defines `deactivate_node` and `freeze` shell functions

To leave the environment:

```bash
deactivate_node
```

If you are using `fish`, source `bin/activate.fish` instead of `bin/activate`.

## Run Node Without Activating The Shell

The maintainer docs include a `shim` script for invoking the environment's Node.js binary directly:

```bash
./.nodeenv/bin/shim --version
```

This is useful in scripts where you do not want to source an activation script first.

## Configuration Files

`nodeenv` can load defaults from config files. By default it looks at:

- `./tox.ini`
- `./setup.cfg`
- `~/.nodeenvrc`

The implementation loads those files in that precedence order, and if a `.node-version` file exists in the current directory, its value becomes the default Node.js version.

Example `~/.nodeenvrc`:

```ini
[nodeenv]
node = 22.11.0
npm = latest
prebuilt = True
jobs = 4
mirror = https://nodejs.org/download/release
```

The keys match the long command-line option names.

Use a specific config file:

```bash
nodeenv --config-file=/path/to/nodeenv.ini .nodeenv
```

Disable config-file loading entirely and use only built-in defaults plus CLI flags:

```bash
nodeenv --config-file= .nodeenv
```

## Automate From Python

`nodeenv` is documented as a CLI, not as a separate object-oriented Python API. For automation, invoke the module entry point with `subprocess`:

```python
import subprocess

subprocess.run(
    ["python", "-m", "nodeenv", "--node=22.11.0", ".nodeenv"],
    check=True,
)
```

If your automation depends on the current Python virtual environment, call `nodeenv -p` from an already activated environment instead of hard-coding paths.

## Common Pitfalls

- `bin/activate` must be sourced, not executed directly.
- `--update` only reinstalls npm packages from the requirements file; it does not reinstall or change the Node.js runtime.
- Existing destination directories require `--force` unless you are attaching to the current Python virtual environment with `-p`.
- `--source` is for source builds and is much slower than the default prebuilt flow.
- `--node=system` is not supported on Windows; the program exits with an error there.
- `--ignore_ssl_certs` disables certificate verification for downloads and is explicitly marked unsafe.
- Config surprises are common when `./tox.ini`, `./setup.cfg`, `~/.nodeenvrc`, or `.node-version` silently override the Node.js version you expected.
- On `x86_64` musl systems and on `riscv64`, the implementation switches prebuilt downloads to `unofficial-builds.nodejs.org` unless you explicitly set `--mirror`.

## Version-Sensitive Notes For 1.10.0

- The package version covered here is `1.10.0`.
- PyPI metadata for `1.10.0` declares `Requires-Python: >=2.7, !=3.0.*, !=3.1.*, !=3.2.*, !=3.3.*, !=3.4.*, !=3.5.*, !=3.6.*`.
- The maintainer README bundled with `1.10.0` documents `--prebuilt` as the default install mode and `--source` as the explicit alternative.
- The primary maintainer documentation for this release is the repository README mirrored on GitHub and PyPI rather than a separate docs site.

## Official Sources

- https://github.com/ekalinin/nodeenv
- https://pypi.org/project/nodeenv/
