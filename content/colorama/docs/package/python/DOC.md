---
name: package
description: "colorama package guide for Python 0.4.6 - cross-platform ANSI color and cursor control for terminals"
metadata:
  languages: "python"
  versions: "0.4.6"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "colorama,python,terminal,ansi,windows,console"
---

# colorama Python Package Guide

## What It Is

`colorama` makes ANSI escape sequences usable in Windows terminals and exposes simple constants for colors, styles, and cursor movement.

For `0.4.6`, prefer `just_fix_windows_console()` if your goal is "make ANSI colors work on Windows". Use `init()` only when you need its legacy behavior or keyword flags.

## Installation

```bash
pip install colorama==0.4.6
```

With `uv`:

```bash
uv add colorama==0.4.6
```

## Quick Start

```python
from colorama import Fore, Style, just_fix_windows_console

just_fix_windows_console()

print(Fore.GREEN + "success")
print(Fore.RED + "error")
print(Style.RESET_ALL + "normal text")
```

What this does:

- On modern Windows consoles, it enables native ANSI handling.
- On older Windows consoles, it wraps `stdout` and `stderr` to translate ANSI codes.
- On non-Windows platforms, it does nothing.

## Recommended Initialization

Use this in CLI entry points or at process startup:

```python
from colorama import just_fix_windows_console

def main() -> None:
    just_fix_windows_console()
    # rest of CLI startup
```

Why this is the default:

- Safe to call multiple times.
- Safe on non-Windows platforms.
- Safe when output is redirected to files.
- New in `0.4.6`; this is the upstream-recommended API for most users.

## Core Usage

### Foreground, Background, Style

```python
from colorama import Back, Fore, Style, just_fix_windows_console

just_fix_windows_console()

print(Fore.YELLOW + "warning")
print(Back.BLUE + Fore.WHITE + "status line")
print(Style.BRIGHT + "important")
print(Style.RESET_ALL + "back to default")
```

Common constants:

- `Fore`: `BLACK`, `RED`, `GREEN`, `YELLOW`, `BLUE`, `MAGENTA`, `CYAN`, `WHITE`, `RESET`
- `Back`: same color set plus `RESET`
- `Style`: `DIM`, `NORMAL`, `BRIGHT`, `RESET_ALL`

Extended light colors are also available, for example `Fore.LIGHTGREEN_EX`.

### Cursor Positioning

```python
from colorama import Cursor, just_fix_windows_console

just_fix_windows_console()

print("line 1")
print(Cursor.UP(1) + Cursor.FORWARD(8) + "updated")
```

Use cursor helpers when you need lightweight terminal updates without pulling in a full TUI library.

### Manual ANSI Sequences

`colorama` also works if your code or another library emits raw ANSI escape sequences:

```python
from colorama import just_fix_windows_console

just_fix_windows_console()
print("\033[31mred text\033[39m")
```

This is useful when combining `colorama` with libraries such as `termcolor` or `rich`.

## Legacy `init()` API

`init()` still works and is required only if you need its keyword arguments:

```python
from colorama import Fore, init

init(autoreset=True)
print(Fore.RED + "auto-reset after each write")
print("already back to default")
```

Supported keyword arguments in `0.4.6`:

- `autoreset=False`: automatically reset styles after each write
- `strip=None`: force stripping ANSI sequences on or off
- `convert=None`: force Windows conversion on or off
- `wrap=True`: replace `sys.stdout` and `sys.stderr` with wrapped streams

Important behavior:

- `init()` is not safe to call repeatedly. Repeated calls can stack wrappers and break output.
- `deinit()` restores the original streams.
- `reinit()` re-enables previously wrapped streams more cheaply than another `init()`.
- `wrap=False` cannot be combined with truthy `autoreset`, `strip`, or `convert`; `0.4.6` raises `ValueError`.

## Advanced Stream Wrapping

If `init(wrap=True)` interferes with your program's stream handling, use `AnsiToWin32` directly:

```python
import sys

from colorama import AnsiToWin32, Fore, init

init(wrap=False)
stream = AnsiToWin32(sys.stderr).stream

print(Fore.CYAN + "sent through wrapped stderr", file=stream)
```

Use this pattern when a framework expects the real `sys.stdout`/`sys.stderr` objects but you still need Windows ANSI conversion on a specific stream.

## Setup and Configuration Notes

`colorama` has no authentication, network setup, or environment-variable configuration.

The only practical configuration surface is initialization:

- Prefer `just_fix_windows_console()` for simple compatibility.
- Use `init()` only if you specifically need `autoreset`, forced `strip`, forced `convert`, or `wrap=False`.
- Initialize once near process startup, not inside hot loops or helper functions.

## Common Pitfalls

### Forgetting to reset styles

Without a reset, later output can inherit styles:

```python
from colorama import Fore, Style

print(Fore.RED + "error" + Style.RESET_ALL)
```

If you do this constantly, `init(autoreset=True)` can be simpler.

### Using `init()` where `just_fix_windows_console()` is enough

For `0.4.6+`, this is the main version-specific trap. `just_fix_windows_console()` has fewer side effects and is the upstream recommendation for most users.

### Expecting `Style.DIM` to render distinctly on Windows

Upstream notes that dim text is not supported well on Windows consoles; it can appear the same as normal text.

### Assuming wrapped streams behave exactly like original streams

`init()` may replace `sys.stdout` and `sys.stderr`. That can matter for tests, logging setup, subprocess integrations, or frameworks that inspect raw stream objects.

### Expecting Colorama to be a full styling framework

`colorama` is intentionally small. It is mainly for ANSI compatibility on Windows. For richer styling, use another ANSI-producing library and let `colorama` handle Windows compatibility.

## Version-Sensitive Notes for `0.4.6`

- `just_fix_windows_console()` is available in `0.4.6` and is the preferred entry point for new code.
- The package version on PyPI is still `0.4.6`, released on `2022-10-25`.
- PyPI metadata for `0.4.6` lists support for Python `>=2.7` except `3.0` through `3.6`, so for current projects treat it as effectively Python `3.7+`.
- The GitHub repository default branch may describe newer development state than the `0.4.6` release. For coding against this pinned package version, prefer the `0.4.6` tag and the PyPI release page.

## Practical Agent Guidance

- If you see `Fore`, `Back`, `Style`, or raw `\033[` sequences in a CLI app, initialize `colorama` once near startup.
- For modern code on `0.4.6`, start with `just_fix_windows_console()`.
- If tests snapshot terminal output, be explicit about whether ANSI codes should be present or stripped.
- If output is redirected or piped, remember that `init()` may strip ANSI sequences depending on its heuristic and stream type.

## Official Sources

- PyPI: https://pypi.org/project/colorama/
- Repository: https://github.com/tartley/colorama
- Versioned README: https://github.com/tartley/colorama/blob/0.4.6/README.rst
- Versioned source exports: https://raw.githubusercontent.com/tartley/colorama/0.4.6/colorama/__init__.py
- Versioned initialization logic: https://raw.githubusercontent.com/tartley/colorama/0.4.6/colorama/initialise.py
