---
name: package
description: "Rich Python package guide for terminal formatting, tables, progress bars, logging, markdown, and tracebacks"
metadata:
  languages: "python"
  versions: "14.3.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "rich,python,terminal,cli,logging,progress,markdown"
---

# Rich Python Package Guide

## Golden Rule

Use `rich` as a terminal rendering layer centered on `Console`, not as a grab bag of unrelated helpers. As of March 12, 2026, PyPI is at `14.3.3`, but the stable docs site still renders as `Rich 14.1.0 documentation`, so combine the stable guides with the upstream changelog for `14.3.x` behavior changes.

## Install

Pin the version your project expects:

```bash
python -m pip install "rich==14.3.3"
```

Common alternatives:

```bash
uv add "rich==14.3.3"
poetry add "rich==14.3.3"
```

Optional Jupyter extra:

```bash
python -m pip install "rich[jupyter]==14.3.3"
```

Sanity-check terminal support:

```bash
python -m rich
```

## Initialize And Basic Setup

For one-off formatted output, `rich.print` is the fastest entry point:

```python
from rich import print

print("[bold green]Build passed[/bold green]")
print({"status": "ok", "items": [1, 2, 3]})
```

For anything non-trivial, create a shared `Console` instance and route output through it:

```python
from rich.console import Console

console = Console()
console.print("Hello, [bold magenta]Rich[/bold magenta]!")
```

Use one shared console per output stream when possible. It keeps width detection, themes, recording, and progress rendering consistent.

## Core Usage

### Styled output, tables, and markdown

```python
from rich.console import Console
from rich.markdown import Markdown
from rich.table import Table

console = Console()

table = Table(title="Deployments", show_header=True, header_style="bold cyan")
table.add_column("Service")
table.add_column("Status")
table.add_column("Latency", justify="right")
table.add_row("api", "[green]healthy[/green]", "43 ms")
table.add_row("worker", "[yellow]degraded[/yellow]", "210 ms")

console.print(table)
console.print(Markdown("## Notes\nUse `Console` for multi-line output."))
```

Rich markup is enabled by default on `Console.print()`. If your text contains literal square brackets from user input or logs, either escape it or disable markup for that call.

### Logging and better tracebacks

Use Rich's logging handler instead of manually colorizing log strings:

```python
import logging

from rich.logging import RichHandler
from rich.traceback import install

install(show_locals=True)

logging.basicConfig(
    level="INFO",
    format="%(message)s",
    handlers=[RichHandler(rich_tracebacks=True)],
)

log = logging.getLogger("app")
log.info("server started")
```

`RichHandler` integrates with the standard `logging` module. Keep `markup=False` unless you control the log message content, otherwise messages containing `[` and `]` can be interpreted as markup.

### Progress bars and live status

For a single iterable, `track()` is the simplest choice:

```python
from rich.progress import track

for item in track(range(100), description="Processing"):
    process(item)
```

For multiple tasks or custom columns, use `Progress`:

```python
from time import sleep

from rich.progress import BarColumn, Progress, TextColumn, TimeRemainingColumn

with Progress(
    TextColumn("[progress.description]{task.description}"),
    BarColumn(),
    TimeRemainingColumn(),
    transient=True,
) as progress:
    task_id = progress.add_task("Uploading", total=5)
    for _ in range(5):
        sleep(0.2)
        progress.advance(task_id)
```

Rich redirects `stdout` and `stderr` while a progress display is active so plain `print()` calls do not usually destroy the progress layout. Even so, prefer writing through the same `Console` or the `Progress` instance for predictable formatting.

### Prompts and input

Use the prompt helpers instead of hand-rolling validation:

```python
from rich.prompt import Confirm, IntPrompt, Prompt

name = Prompt.ask("Project name", default="demo")
retries = IntPrompt.ask("Retry count", default=3)
deploy = Confirm.ask("Deploy now?", default=False)
```

### Pretty-printing and debugging

Rich can improve the Python REPL and ad hoc object inspection:

```python
from rich import inspect, pretty

pretty.install()

data = {"service": "api", "ports": [8000, 8001]}
inspect(data, methods=False)
```

## Configuration And Environment

Rich has no authentication model. Configuration is mostly terminal behavior and rendering defaults.

Useful `Console` options:

- `stderr=True`: send output to stderr instead of stdout
- `force_terminal=True`: emit terminal control codes even when auto-detection says the output is not a TTY
- `force_interactive=False`: disable interactive behaviors such as live animations
- `record=True`: retain output so you can call `export_text()`, `export_svg()`, or `export_html()`
- `soft_wrap=True`: disable cropping and let long text wrap naturally
- `safe_box=True`: prefer legacy-safe box characters when output must render on old Windows terminals

Example:

```python
from rich.console import Console

console = Console(
    stderr=True,
    force_terminal=True,
    force_interactive=False,
    record=True,
)

console.print("[bold]Build report[/bold]")
html = console.export_html()
```

Important environment variables from the console docs:

- `NO_COLOR`: disables colors; it takes precedence over `FORCE_COLOR`
- `FORCE_COLOR=1`: forces color output when Rich would otherwise disable it
- `TTY_COMPATIBLE=1`: tells Rich the target can display terminal escape sequences
- `TTY_INTERACTIVE=0`: disables interactive rendering such as animated progress bars
- `COLUMNS` and `LINES`: override detected terminal size
- `JUPYTER_COLUMNS` and `JUPYTER_LINES`: Jupyter-specific width and height defaults

For CI or GitHub Actions, the documented combination is:

```bash
export TTY_COMPATIBLE=1
export TTY_INTERACTIVE=0
```

## Common Pitfalls

- Do not mix `rich.print` and a separately configured `Console` indiscriminately. If your app depends on one theme, one output stream, or one recording buffer, use a shared `Console`.
- Do not enable markup on untrusted text. Escape it first or call `console.print(text, markup=False)`.
- `RichHandler` does not magically make file logs colorful. It is for terminal handlers; use plain structured logging for machine-readable log sinks.
- If colors disappear in CI or when piping, set `force_terminal=True` or the `TTY_COMPATIBLE` and `TTY_INTERACTIVE` environment variables explicitly.
- `record=True` is required before calling `export_text()`, `export_svg()`, or `export_html()`.
- `safe_box=False` may render poorly on old Windows terminals that cannot display Unicode box-drawing characters correctly.
- Progress and live displays are great for human-facing CLIs but usually wrong for non-interactive logs. Disable interactivity in CI.
- Rich word-wraps output by default. If you are rendering preformatted text where spacing must remain exact, test with `soft_wrap`, `overflow`, or raw file output settings.

## Version-Sensitive Notes For 14.3.x

- PyPI currently publishes `14.3.3` from February 19, 2026. The stable docs site is still branded `14.1.0`, so patch-level behavior after `14.1.0` is better verified from the changelog.
- `14.3.0` added better support for multi-codepoint glyphs, added the `UNICODE_VERSION` environment variable, exposed `locals_max_depth` and `locals_overflow` in `traceback.install()`, and changed Markdown header, table, and rule styling.
- `14.3.1` through `14.3.3` fix Unicode-width and grapheme-splitting issues. If your CLI renders emoji, ZWJ sequences, or other complex Unicode, stay on `14.3.3` rather than copying behavior from older blog posts.
- Since `14.1.0`, Live objects including `Progress` may be nested. If you see older guidance claiming nested live rendering is unsupported, that guidance is stale.

## Canonical Sources

- Stable docs: `https://rich.readthedocs.io/en/stable/`
- Console docs: `https://rich.readthedocs.io/en/stable/console.html`
- Progress docs: `https://rich.readthedocs.io/en/stable/progress.html`
- Prompt docs: `https://rich.readthedocs.io/en/stable/prompt.html`
- Logging docs: `https://rich.readthedocs.io/en/stable/logging.html`
- Pretty docs: `https://rich.readthedocs.io/en/stable/pretty.html`
- Traceback docs: `https://rich.readthedocs.io/en/stable/traceback.html`
- PyPI package page: `https://pypi.org/project/rich/`
- Upstream changelog: `https://github.com/Textualize/rich/blob/master/CHANGELOG.md`
