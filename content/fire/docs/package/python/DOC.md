---
name: package
description: "Python Fire guide for turning Python functions, classes, and objects into CLIs"
metadata:
  languages: "python"
  versions: "0.7.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "fire,python,cli,command-line,repl,google"
---

# Fire Python Package Guide

## Golden Rule

Use `fire.Fire(...)` with an explicit function, class, object, or command map inside `if __name__ == "__main__":`. Avoid bare `fire.Fire()` in production CLIs unless you intentionally want to expose the whole module for exploration or debugging. When you need Fire's own flags such as help, trace, completion, or interactive mode, pass them after a standalone `--`.

## Install

Pin the package version your project expects:

```bash
python -m pip install "fire==0.7.1"
```

Common alternatives:

```bash
uv add "fire==0.7.1"
poetry add "fire==0.7.1"
```

Optional tooling:

```bash
python -m pip install ipython
```

Notes:

- `fire` is a local CLI-generation library. It does not require API keys, service credentials, or network configuration.
- `ipython` is optional, but `--interactive` is more useful when IPython is installed.

## Initialize A CLI

Expose a single function:

```python
import fire

def hello(name: str = "world", excited: bool = False) -> str:
    greeting = f"Hello, {name}"
    return f"{greeting}!" if excited else greeting

if __name__ == "__main__":
    fire.Fire(hello)
```

Run it:

```bash
python hello.py
python hello.py --name=Ada
python hello.py --name=Grace --excited
python hello.py -- --help
```

Fire turns function parameters into CLI arguments automatically:

- positional args map left-to-right
- keyword args use `--name=value` or `--name value`
- boolean flags accept forms like `--debug` and `--nodebug`

## Core Usage

### Group commands with a class

Classes work well when you want shared state and multiple subcommands:

```python
import fire

class Calculator:
    def __init__(self, precision: int = 2):
        self.precision = precision

    def add(self, x: float, y: float) -> float:
        return round(x + y, self.precision)

    def mul(self, x: float, y: float) -> float:
        return round(x * y, self.precision)

if __name__ == "__main__":
    fire.Fire(Calculator)
```

Examples:

```bash
python calc.py add 2 3
python calc.py --precision=4 mul 2.5 3
```

Important detail: constructor arguments must be passed as named flags before the member you want to call.

### Group commands with a dict

If you want explicit command names instead of exposing every class member, pass a dict:

```python
import fire

def greet(name: str) -> str:
    return f"hello {name}"

def version() -> str:
    return "1.0.0"

if __name__ == "__main__":
    fire.Fire(
        {
            "greet": greet,
            "version": version,
        }
    )
```

This is often the safest pattern for small CLIs because it exposes only the commands you choose.

### Return values become output

Fire prints primitive return values directly:

```python
import fire

def status():
    return {"healthy": True, "workers": 4}

if __name__ == "__main__":
    fire.Fire(status)
```

For custom objects, Fire may show help/inspection output instead of a useful value unless the object has a meaningful string representation. If you need stable output for automation, return primitives, dicts, lists, or implement `__str__`.

### Async functions are supported

Fire will await async functions before printing the result:

```python
import asyncio
import fire

async def fetch_message(name: str) -> str:
    await asyncio.sleep(0.1)
    return f"done for {name}"

if __name__ == "__main__":
    fire.Fire(fetch_message)
```

## Configuration And Fire Flags

Fire has no separate config file format. The practical configuration surface is:

1. the Python object you expose with `fire.Fire(...)`
2. the command arguments you define on that object
3. Fire's own flags, passed after a standalone `--`

Useful built-in flags:

- `-- --help`: show generated help and docstrings
- `-- --interactive`: drop into a REPL with the final result in scope
- `-- --completion [bash|fish]`: print a shell completion script
- `-- --trace`: show Fire's resolution trace for debugging
- `-- --separator=X`: change the separator used for command chaining
- `-- --verbose`: include private members in help output

Examples:

```bash
python calc.py add 2 3 -- --trace
python calc.py -- --completion bash
python calc.py add 2 3 -- --interactive
```

If `-` is part of the actual value you need to pass or the default separator causes ambiguous parsing, set a different separator explicitly:

```bash
python tool.py deploy prod /srv/app -- --separator=/
```

## Help And Docstrings

Fire uses Python names, signatures, and docstrings to build help text. If you want the generated CLI to be usable:

- write docstrings on functions and classes
- keep parameter names stable and descriptive
- prefer explicit command maps or narrow classes over dumping a large module into `Fire`

Minimal example:

```python
def greet(name: str):
    """Return a friendly greeting."""
    return f"hello {name}"
```

## Common Pitfalls

- Bare `fire.Fire()` exposes the current module's globals and imported objects. That is useful for ad hoc exploration but too broad for a stable CLI.
- Fire's own flags must come after a standalone `--`. Without that separator, `--trace`, `--help`, or `--interactive` may be parsed as arguments for your function or method instead.
- Constructor args and method args are parsed differently. For `fire.Fire(MyClass)`, constructor values should be passed as flags before the method name.
- Fire treats hyphens and underscores as equivalent in argument and member names. That is convenient, but it can hide naming mistakes if your code relies on both forms.
- Fire parses command-line values into Python types. Strings that look like numbers, lists, dicts, or booleans can be coerced unexpectedly, so quote carefully when you need a literal string.
- Complex custom objects do not automatically serialize well. Return plain data structures for scripts that need stable machine-readable output.
- The default separator is `-`. If your commands or data also use `-` in ways Fire interprets as chaining, change it with `-- --separator=...`.
- `--interactive` works without IPython, but the experience is better if `ipython` is installed.

## Version-Sensitive Notes For `0.7.1`

- As of `2026-03-12`, PyPI still lists `0.7.1` as the current release for `fire`.
- The official docs site is a general guide rather than a versioned per-release reference. For this package, the guide and CLI reference still align with `0.7.1`.
- The GitHub `v0.7.1` release notes call out packaging and compatibility work such as `pyproject.toml`, wheel publishing, IPython inspector compatibility, and a fix for class attribute inspection. Those changes do not materially change the basic CLI authoring patterns in the guide.
- Older tutorials that mention Python 2 are stale. The `0.7.x` line is Python 3 only, and PyPI metadata for `0.7.1` requires Python `>=3.7`.

## Official Sources

- Guide: `https://google.github.io/python-fire/guide/`
- CLI flags reference: `https://google.github.io/python-fire/using-cli/`
- PyPI package metadata: `https://pypi.org/project/fire/`
- PyPI JSON metadata: `https://pypi.org/pypi/fire/0.7.1/json`
- GitHub release: `https://github.com/google/python-fire/releases/tag/v0.7.1`
