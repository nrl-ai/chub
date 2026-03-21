---
name: package
description: "python-dotenv package guide for loading, parsing, and managing .env files in Python projects"
metadata:
  languages: "python"
  versions: "1.2.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "python-dotenv,dotenv,environment,configuration,secrets,cli"
---

# python-dotenv Python Package Guide

## Golden Rule

Use `python-dotenv` to load development configuration into `os.environ`, but be explicit about when it should win over real environment variables. `load_dotenv()` does not override existing variables unless you pass `override=True`, and variable expansion only works with `${NAME}` syntax, not bare `$NAME`.

## Install

Pin the package version your project expects:

```bash
python -m pip install "python-dotenv==1.2.2"
```

Common alternatives:

```bash
uv add "python-dotenv==1.2.2"
poetry add "python-dotenv==1.2.2"
```

Install the CLI extra only if you need the `dotenv` command:

```bash
python -m pip install "python-dotenv[cli]==1.2.2"
```

## Initialize And Load Configuration

Basic usage:

```python
import os

from dotenv import load_dotenv

load_dotenv()

database_url = os.environ["DATABASE_URL"]
debug = os.getenv("DEBUG", "").lower() == "true"
```

Default `load_dotenv()` behavior:

- It searches for `.env` in the script directory and then walks upward.
- It loads keys into `os.environ`.
- It does not override existing environment variables unless you pass `override=True`.
- It returns `True` if it set at least one environment variable, otherwise `False`.

Use an explicit path when the working directory or process launcher is not trustworthy:

```python
from pathlib import Path

from dotenv import load_dotenv

BASE_DIR = Path(__file__).resolve().parent
load_dotenv(BASE_DIR / ".env")
```

Use `override=True` only when the `.env` file should win over already-exported variables:

```python
from dotenv import load_dotenv

load_dotenv(override=True)
```

Disable loading globally with an environment variable when a dependency calls `load_dotenv()` and you need to suppress it:

```bash
export PYTHON_DOTENV_DISABLED=1
```

## Parse Without Mutating The Environment

Use `dotenv_values()` when you want a parsed config dictionary instead of side effects in `os.environ`:

```python
import os

from dotenv import dotenv_values

config = {
    **dotenv_values(".env.shared"),
    **dotenv_values(".env.secret"),
    **os.environ,
}

print(config["DATABASE_URL"])
```

Important behavior:

- `dotenv_values()` returns `dict[str, str | None]`.
- A line like `EMPTY_VALUE=` becomes an empty string.
- A line with just `MISSING_VALUE` becomes `None`.
- `dotenv_values()` prefers values from the `.env` file during interpolation; `load_dotenv(override=False)` prefers existing environment variables first.

## Finding The Right `.env` File

If you need the resolved path first, call `find_dotenv()` explicitly:

```python
from dotenv import find_dotenv, load_dotenv

dotenv_path = find_dotenv(usecwd=True)
load_dotenv(dotenv_path)
```

`usecwd=True` is useful in shells, notebooks, debuggers, or task runners where search relative to the calling Python file is not what you want.

## Loading From Streams

`load_dotenv()` and `dotenv_values()` both accept text streams:

```python
from io import StringIO

from dotenv import load_dotenv

stream = StringIO("API_TOKEN=dev-token\nFEATURE_FLAG=true\n")
load_dotenv(stream=stream, override=True)
```

Use this when config comes from a secret manager, a generated string, or a test fixture instead of a file on disk.

## CLI Usage

The `dotenv` CLI is part of the optional `cli` extra.

Create or update keys:

```bash
dotenv set DATABASE_URL postgresql://localhost/app
dotenv set DEBUG true
```

Inspect current values:

```bash
dotenv list
dotenv list --format=json
dotenv get DATABASE_URL
```

Run a command with variables loaded from `.env`:

```bash
dotenv run -- python app.py
```

In `1.2.2`, `dotenv run` forwards flags directly to the target command, which matters if the child command uses options that look like CLI flags.

## Editing `.env` Files Programmatically

Use the helper functions when you need to read or rewrite `.env` files directly:

```python
from dotenv import get_key, set_key, unset_key

env_path = ".env"

set_key(env_path, "DEBUG", "true", quote_mode="auto")
print(get_key(env_path, "DEBUG"))
unset_key(env_path, "DEBUG")
```

Notes:

- `set_key()` creates the file if it does not exist.
- `quote_mode` must be one of `"always"`, `"auto"`, or `"never"`.
- `export=True` writes entries as `export KEY=value`.
- In `1.2.2`, `set_key()` and `unset_key()` do not follow symlinks by default. Pass `follow_symlinks=True` only if you intentionally want the old behavior.

## `.env` File Format

Supported patterns:

```dotenv
# comments are allowed
DOMAIN=example.org
ADMIN_EMAIL=admin@${DOMAIN}
ROOT_URL=${DOMAIN}/app
EMPTY_VALUE=
MULTILINE="first line
second line"
export FEATURE_FLAG=true
```

Rules that matter in practice:

- Keys may be unquoted or single-quoted.
- Values may be unquoted, single-quoted, or double-quoted.
- Spaces around keys, `=`, and values are ignored.
- Inline comments are allowed.
- `export` is accepted but does not change parsing semantics.
- Variable expansion requires `${NAME}` syntax.
- Bare `$NAME` is not expanded.

## IPython And Notebook Usage

The package ships an IPython extension:

```ipython
%load_ext dotenv
%dotenv
%dotenv -o path/to/.env
```

Useful flags:

- `-o` overrides existing variables.
- `-v` increases verbosity.

## Common Pitfalls

- Do not commit `.env` files with secrets. Add them to `.gitignore`.
- Do not assume `load_dotenv()` loads from the current shell directory. By default it uses the calling file path unless the runtime looks interactive or you use `find_dotenv(usecwd=True)`.
- Do not expect `.env` values to override already-exported CI or production variables unless you pass `override=True`.
- Do not use bare `$VAR` references in values. Use `${VAR}`.
- Do not import `python_dotenv`; the import package is `dotenv`.
- Do not assume the `dotenv` CLI exists after `pip install python-dotenv`; install `python-dotenv[cli]` if your tooling depends on it.
- Be careful with symlinked `.env` files after upgrading to `1.2.2`; `set_key()` and `unset_key()` no longer follow symlinks unless you opt in.

## Version-Sensitive Notes For 1.2.2

- `1.2.2` supports Python 3.10 through 3.14 and drops Python 3.9 support.
- `dotenv run` changed in `1.2.2` to forward flags directly to the target command.
- `set_key()` and `unset_key()` changed in `1.2.2` to avoid following symlinks by default and to preserve file mode more consistently.
- FIFO support for reading `.env` files landed in `1.2.1`; if older docs say named pipes are unsupported, they are outdated for the `1.2.x` line.
