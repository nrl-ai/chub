---
name: package
description: "watchdog Python package guide for filesystem event monitoring with Observer, event handlers, and watchmedo"
metadata:
  languages: "python"
  versions: "6.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "watchdog,python,filesystem,events,observer,watchmedo"
---

# watchdog Python Package Guide

## Golden Rule

Use `watchdog` for cross-platform filesystem watching in Python, start from the `Observer` plus an event handler, and always stop and `join()` the observer cleanly. For `6.0.0`, trust PyPI and maintainer source for version-specific behavior: the public Read the Docs site still serves old `2.1.5` pages and should be treated as a general guide, not as the exact `6.0.0` API reference.

## Install

Pin the package version your project expects:

```bash
python -m pip install "watchdog==6.0.0"
```

Common alternatives:

```bash
uv add "watchdog==6.0.0"
poetry add "watchdog==6.0.0"
```

Install the optional `watchmedo` CLI extras when you need command-line monitoring helpers or trick-based automation:

```bash
python -m pip install "watchdog[watchmedo]==6.0.0"
```

## Initialization

The common pattern is:

1. Create an event handler.
2. Create an `Observer`.
3. Schedule a path to watch.
4. Start the observer.
5. Keep the process alive.
6. Stop and `join()` on shutdown.

Minimal example:

```python
import time

from watchdog.events import FileSystemEventHandler
from watchdog.observers import Observer

class PrintEvents(FileSystemEventHandler):
    def on_any_event(self, event):
        print(event.event_type, event.src_path, "directory=" + str(event.is_directory))

path = "."
event_handler = PrintEvents()
observer = Observer()
observer.schedule(event_handler, path, recursive=True)
observer.start()

try:
    while True:
        time.sleep(1)
except KeyboardInterrupt:
    observer.stop()
finally:
    observer.join()
```

Notes:

- `recursive=False` is the default. Set `recursive=True` explicitly when you need subtree monitoring.
- `observer.start()` runs worker threads. If your process exits immediately, you will miss events.
- Always call both `stop()` and `join()`. Stopping without `join()` can leave background threads unfinished.

## Core Usage

### Subclass `FileSystemEventHandler`

Override only the callbacks you need:

```python
from watchdog.events import FileSystemEventHandler

class BuildHandler(FileSystemEventHandler):
    def on_created(self, event):
        if not event.is_directory:
            print("created:", event.src_path)

    def on_modified(self, event):
        if not event.is_directory:
            print("modified:", event.src_path)

    def on_moved(self, event):
        print("moved:", event.src_path, "->", event.dest_path)

    def on_deleted(self, event):
        print("deleted:", event.src_path)
```

Useful callbacks in modern `watchdog` include:

- `on_created`
- `on_deleted`
- `on_modified`
- `on_moved`
- `on_opened`
- `on_closed`
- `on_closed_no_write`
- `on_any_event`

### Filter with `PatternMatchingEventHandler`

Use the built-in filtering handler instead of hand-rolling glob checks in every callback:

```python
import time

from watchdog.events import PatternMatchingEventHandler
from watchdog.observers import Observer

class PythonOnlyHandler(PatternMatchingEventHandler):
    def __init__(self):
        super().__init__(
            patterns=["**/*.py"],
            ignore_patterns=["**/.venv/**", "**/__pycache__/**"],
            ignore_directories=True,
            case_sensitive=False,
        )

    def on_modified(self, event):
        print("python file changed:", event.src_path)

observer = Observer()
observer.schedule(PythonOnlyHandler(), ".", recursive=True)
observer.start()

try:
    while True:
        time.sleep(1)
except KeyboardInterrupt:
    observer.stop()
finally:
    observer.join()
```

In `6.0.0`, handlers like `PatternMatchingEventHandler` use keyword-only constructor arguments in the maintainer source. Older blog posts that pass patterns positionally are a migration risk.

### Choose polling explicitly for network filesystems

Use the polling backend when native events are unreliable, especially on CIFS/SMB mounts, some virtualized filesystems, or unusual container bind mounts:

```python
import time

from watchdog.events import FileSystemEventHandler
from watchdog.observers.polling import PollingObserver

class Handler(FileSystemEventHandler):
    def on_any_event(self, event):
        print(event.event_type, event.src_path)

observer = PollingObserver(timeout=1.0)
observer.schedule(Handler(), "/mnt/shared", recursive=True)
observer.start()

try:
    while True:
        time.sleep(1)
except KeyboardInterrupt:
    observer.stop()
finally:
    observer.join()
```

Prefer the default `Observer` first, but switch to `PollingObserver` when native backend behavior is inconsistent.

### Use `watchmedo` for local automation

`watchmedo` is the bundled CLI for quick monitoring and shell-command workflows.

Examples:

```bash
watchmedo log .
watchmedo shell-command --patterns="*.py" --recursive --command='pytest -q' .
```

This is useful for local development tasks, but application code should usually use the Python API directly.

## Configuration Notes

`watchdog` does not use API credentials or service authentication. The important configuration is runtime and OS behavior:

- Watch root: schedule the narrowest directory tree that satisfies the task.
- Recursion: set `recursive=True` only when needed; it increases the watch surface.
- Filtering: prefer built-in pattern and regex handlers to reduce noisy callbacks.
- Backend choice: default `Observer` uses a native platform backend when available; `PollingObserver` trades efficiency for portability.
- Process lifetime: a watcher needs a long-running process or thread owner.

Backend expectations from maintainer docs and repository guidance:

- Linux: inotify
- macOS: FSEvents or kqueue depending on backend availability
- BSD: kqueue
- Windows: `ReadDirectoryChangesW`
- Fallback: polling

## Common Pitfalls

- Do not trust the version banner on `python-watchdog.readthedocs.io/en/stable/` for `6.0.0`; it currently exposes old `2.1.5` content.
- Do not forget `observer.join()` after `observer.stop()`.
- `recursive` is not implied. Missing subtree events usually means you forgot `recursive=True`.
- File-save behavior varies by editor. Many editors write via temp-file rename, so you may see `created` plus `moved` instead of a simple `modified`.
- Network shares and mounted volumes may miss native events. Use `PollingObserver` when reproducibility matters more than efficiency.
- On Linux, large watch trees can hit the inotify watch limit. If you see `OSError: inotify watch limit reached`, raise `fs.inotify.max_user_watches`.
- On kqueue-based systems, deeply recursive monitoring can consume many file descriptors. Raise `ulimit -n` when needed.
- Ignore noisy paths like `.git`, `.venv`, `node_modules`, build outputs, and caches unless the task explicitly depends on them.

## Version-Sensitive Notes For `6.0.0`

- PyPI `6.0.0` requires Python `>=3.9`. The older Read the Docs pages still mention older Python support; treat that as historical information.
- The `6.0.0` release notes drop Python 3.8 support and add support for Python 3.13.
- If you are migrating from pre-5.x examples, re-check constructor signatures and keyword usage against current maintainer source before copying code.
- Pattern-filter behavior is version-sensitive across major lines. Keep glob patterns explicit and test them instead of assuming older examples still match the same files.

## Official Sources

- PyPI package page: `https://pypi.org/project/watchdog/`
- Maintainer docs root: `https://python-watchdog.readthedocs.io/en/stable/`
- Maintainer repository: `https://github.com/gorakhargosh/watchdog`
- Maintainer changelog for `6.0.0`: `https://github.com/gorakhargosh/watchdog/blob/v6.0.0/changelog.rst`
