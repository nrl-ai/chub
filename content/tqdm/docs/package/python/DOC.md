---
name: package
description: "tqdm progress bar and CLI for Python loops, async tasks, pandas operations, notebooks, and shell pipelines"
metadata:
  languages: "python"
  versions: "4.67.3"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "tqdm,python,progress-bar,cli,asyncio,pandas,jupyter"
---

# `tqdm` Python Package Guide

Use `tqdm` to add low-overhead progress bars around iterables, manual counters, async workloads, pandas operations, and shell pipelines. For mixed terminal and notebook environments, prefer `tqdm.auto` as the default import.

## Install

```bash
pip install tqdm==4.67.3
```

Common alternatives:

```bash
uv add tqdm
poetry add tqdm
```

Notebook/widget support can require the optional notebook extra:

```bash
pip install "tqdm[notebook]"
```

Package metadata for `4.67.3` requires Python `>=3.7`.

## Choose the Right Import

Use these imports intentionally:

```python
from tqdm.auto import tqdm, trange
```

- `tqdm.auto`: best default for code that may run in terminals or notebooks.
- `tqdm`: standard console progress bar.
- `tqdm.notebook`: notebook widget UI when you know the runtime is Jupyter.
- `tqdm.asyncio`: async helpers such as `gather` and `as_completed`.

## Basic Iterable Usage

Wrap the iterable directly:

```python
from time import sleep
from tqdm.auto import tqdm

for item in tqdm(range(100), desc="Processing", unit="item"):
    sleep(0.02)
```

Use `trange()` when you just need `range()` with a bar:

```python
from tqdm.auto import trange

for _ in trange(10, desc="Epoch"):
    ...
```

If the iterable does not expose a length, provide `total=` yourself so ETA and completion percentages work:

```python
from tqdm.auto import tqdm

def stream_rows():
    for i in range(500):
        yield i

for row in tqdm(stream_rows(), total=500, desc="Rows"):
    ...
```

## Manual Progress Updates

For downloads, queues, callbacks, or APIs where work completes in chunks, manage the counter explicitly:

```python
from tqdm.auto import tqdm

with tqdm(total=1024, unit="B", unit_scale=True, desc="Download") as bar:
    for chunk in chunks:
        write_chunk(chunk)
        bar.update(len(chunk))
```

Useful runtime updates:

```python
bar.set_description("Phase 2")
bar.set_postfix(errors=3, cached=True)
bar.update(1)
```

Use `leave=False` for short-lived nested or repeated bars that should disappear after completion.

## Async and Concurrent Workloads

For `asyncio`, use the dedicated helpers instead of manually updating a bar around awaited tasks:

```python
import asyncio
from tqdm.asyncio import tqdm_asyncio

async def fetch(i: int) -> int:
    await asyncio.sleep(0.1)
    return i

results = asyncio.run(
    tqdm_asyncio.gather(*(fetch(i) for i in range(50)), desc="Fetching")
)
```

For thread or process pools, the contrib helpers are usually simpler than wiring your own executor progress tracking:

```python
from tqdm.contrib.concurrent import thread_map

results = thread_map(render_item, items, desc="Rendering", max_workers=8)
```

## Pandas Integration

Enable pandas hooks once, then call the progress-aware methods:

```python
import pandas as pd
from tqdm.auto import tqdm

tqdm.pandas(desc="Transform")

df = pd.DataFrame({"x": range(1000)})
df["y"] = df["x"].progress_apply(lambda value: value * 2)
```

This requires pandas separately; `tqdm` does not install pandas for you.

## Notebook Usage

If you know the code runs in Jupyter and want notebook widgets explicitly:

```python
from tqdm.notebook import tqdm
```

For shared code that runs in both notebooks and terminals, keep using:

```python
from tqdm.auto import tqdm
```

That avoids the `autonotebook` warning path while still choosing the right frontend.

## CLI Usage

`tqdm` also ships a command-line interface that reads from `stdin`, writes the original stream to `stdout`, and renders progress on `stderr`.

```bash
python -m tqdm --help
```

Example pipeline:

```bash
find . -type f | python -m tqdm --unit files > /tmp/files.txt
```

Use CLI flags such as `--total`, `--unit`, `--unit_scale`, and `--bytes` when the input stream size is known or byte-oriented.

## Configuration and Environment

`tqdm` does not need authentication or service credentials.

Configuration is usually done through constructor arguments:

```python
from tqdm.auto import tqdm

for item in tqdm(
    items,
    desc="Jobs",
    total=len(items),
    mininterval=0.5,
    smoothing=0.1,
    dynamic_ncols=True,
):
    ...
```

The project also supports environment-variable overrides for many constructor options using the `TQDM_` prefix. The official docs explicitly show `TQDM_MININTERVAL=5` as an example.

Practical options:

- `desc`: label shown before the bar.
- `total`: required for accurate percentage and ETA when length is unknown.
- `disable`: turn bars off in logs, tests, or non-interactive environments.
- `leave`: keep or clear completed bars.
- `position`: place nested bars on separate lines.
- `dynamic_ncols`: adapt width to the current terminal.
- `unit`, `unit_scale`, `unit_divisor`: useful for bytes, files, or records.

## Common Pitfalls

- Prefer `from tqdm.auto import tqdm` in reusable application code. Hard-coding `tqdm.notebook` or plain `tqdm` is more brittle across environments.
- Wrap the real iterable, not `enumerate()` or `zip()` unless you also pass `total=`. Otherwise `tqdm` may not know the length.
- Use `tqdm.write(...)` instead of `print(...)` while a bar is active to avoid corrupting the display.
- Close manual bars with `with tqdm(...) as bar:` or `bar.close()`, especially in long-running services or tests.
- For erratic workloads, set `miniters=1` or tune `mininterval` if the bar appears stale.
- In CI, some IDE consoles, or plain log collectors, carriage-return updates may render poorly. Use `disable=True`, `ascii=True`, or less frequent refreshes in those environments.
- Nested bars need distinct `position` values if you manage them manually.

## Version-Sensitive Notes

- This guide covers PyPI version `4.67.3`.
- The upstream docs site is package-wide rather than version-pinned, so examples generally reflect the current `4.67.x` behavior instead of a frozen `4.67.3` snapshot.
- PyPI metadata for `4.67.3` lists Python `>=3.7`; check the package page before assuming support for older interpreters.
- If you depend on notebook widgets, contrib helpers, or CLI flags in automation, verify against the current release history because those surfaces change more often than the basic `tqdm(range(...))` API.

## Official Sources

- Docs: https://tqdm.github.io/
- API docs index: https://tqdm.github.io/docs/
- PyPI package: https://pypi.org/project/tqdm/
- Repository: https://github.com/tqdm/tqdm
