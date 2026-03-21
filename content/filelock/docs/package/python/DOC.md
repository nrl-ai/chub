---
name: package
description: "filelock package guide for Python: cross-platform file locks, async locks, and SQLite-backed read/write locks"
metadata:
  languages: "python"
  versions: "3.25.2"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "filelock,locking,concurrency,filesystem,asyncio,sqlite"
---

# filelock Python Package Guide

## What It Does

`filelock` provides cross-platform process coordination with lock files. Use it when multiple Python processes, workers, or CLIs must avoid mutating the same on-disk resource at the same time.

The package now covers three main locking models:

- `FileLock`: the default choice for an exclusive lock. It uses platform-specific hard locks and falls back to a soft lock when the platform reports that hard locking is unavailable.
- `SoftFileLock`: only coordinates by creating a lock file. Use it only when you knowingly accept weaker guarantees.
- `ReadWriteLock`: a SQLite-backed read/write lock for read-heavy workloads where readers may proceed concurrently but writers must be exclusive.

Async variants exist for both exclusive and read/write locking: `AsyncFileLock` and `AsyncReadWriteLock`.

## Version Scope

- Package: `filelock`
- Language: `python`
- Version covered in frontmatter: `3.25.2`
- Current upstream release on PyPI as of March 11, 2026: `3.25.2`
- Python requirement on current PyPI metadata: `>=3.10`

If your project is pinned below `3.25.0`, read/write async support will not exist. If it is pinned below `3.24.0`, several lock-control features in this doc are also missing.

## Install

```bash
pip install filelock==3.25.2
```

```bash
uv add filelock==3.25.2
```

```bash
poetry add filelock==3.25.2
```

## Imports

```python
from filelock import (
    AsyncFileLock,
    AsyncReadWriteLock,
    FileLock,
    ReadWriteLock,
    SoftFileLock,
    Timeout,
)
```

## Choose The Right Lock

### `FileLock`

Use this first unless you have a specific reason not to.

- Best general-purpose option for local filesystems.
- Reentrant for the same lock object in the same process.
- Supports `timeout`, `blocking`, `poll_interval`, `thread_local`, `is_singleton`, `mode`, `lifetime`, and `cancel_check`.

### `SoftFileLock`

Use only when the environment cannot support hard locks reliably and you can tolerate weaker coordination.

- The presence of the lock file is the lock.
- Stale lock cleanup is harder and crash recovery is weaker.
- It still depends on all cooperating processes using the same convention.

### `ReadWriteLock`

Use this when reads are frequent and can safely happen concurrently.

- Backed by a SQLite database file.
- Requires the lock path to end in `.db`.
- Use `read_lock()` and `write_lock()` instead of `acquire()`.
- Call `close()` when you are done with the lock object.

If you are coordinating over NFS, SMB, Docker bind mounts, or other network filesystems, upstream recommends caution. The docs explicitly call out reliability limits for mounted volumes and network filesystems; if correctness matters, prefer a centralized lock service.

## Basic Exclusive Lock Usage

Lock a separate `.lock` file, not the data file you are protecting.

```python
from pathlib import Path
from filelock import FileLock, Timeout

data_path = Path("cache.json")
lock = FileLock("cache.json.lock", timeout=10)

try:
    with lock:
        current = data_path.read_text(encoding="utf-8") if data_path.exists() else "{}"
        data_path.write_text(current, encoding="utf-8")
except Timeout as exc:
    raise RuntimeError("Could not acquire cache lock within 10 seconds") from exc
```

Rules that matter in practice:

- Reuse the same lock object for the same path when possible.
- Keep the object in a variable; do not rely on a temporary object being garbage collected at the right time.
- Prefer the context manager form unless you need manual control.

## Explicit Acquire, Retry, And Cancellation

`acquire()` is useful when you need non-default retry or shutdown behavior.

```python
from threading import Event
from filelock import FileLock, Timeout

stop = Event()
lock = FileLock("jobs.lock", timeout=30, poll_interval=0.2)

try:
    lock.acquire(cancel_check=stop.is_set)
    try:
        run_job_queue()
    finally:
        lock.release()
except Timeout as exc:
    raise RuntimeError("jobs.lock stayed busy") from exc
```

Important controls:

- `timeout`: total wait time before raising `Timeout`
- `blocking=False`: fail immediately instead of waiting
- `poll_interval`: sleep interval between retries
- `cancel_check`: callable checked during waits so another thread can abort lock acquisition
- `lifetime`: treat a lock as stale after the given number of seconds
- `is_singleton=True`: reuse one lock instance per path inside the process
- `thread_local`: keep acquisition context per thread

`blocking=False` takes precedence over `timeout`. If you pass both, the non-blocking behavior wins and the call fails immediately.

## Async Usage

Use the async lock types directly in `asyncio` code.

```python
import asyncio
from filelock import AsyncFileLock, Timeout

lock = AsyncFileLock("worker.lock", timeout=5)

async def main() -> None:
    try:
        async with lock:
            await process_batch()
    except Timeout as exc:
        raise RuntimeError("worker.lock is busy") from exc

asyncio.run(main())
```

Practical notes:

- `AsyncFileLock` runs the blocking lock operations in an executor.
- Upstream defaults async locks to `thread_local=False`.
- You can pass a custom `executor` or disable `run_in_executor` when you need tighter control.

## Read/Write Locks

`ReadWriteLock` is the new 3.25.x option when many readers may proceed concurrently but writes must exclude both readers and writers.

The lock path must end in `.db`.

```python
from filelock import ReadWriteLock, Timeout

rw_lock = ReadWriteLock("catalog.lock.db")

try:
    with rw_lock.read_lock(timeout=5):
        snapshot = load_catalog()

    with rw_lock.write_lock(timeout=10):
        save_catalog(snapshot)
finally:
    rw_lock.close()
```

Notes:

- `read_lock()` and `write_lock()` support `timeout` and `blocking`.
- This API does not use `lifetime` or `cancel_check`.
- `AsyncReadWriteLock` provides the same model for `asyncio`.

## Configuration And Environment

`filelock` has no auth model, no service credentials, and no required environment variables. All configuration is local to the lock object you create.

Controls you will actually use:

- Lock path choice: every cooperating process must use the same path convention.
- `timeout`, `blocking`, and `poll_interval`: determine wait behavior.
- `lifetime`: stale-lock recovery for exclusive locks.
- `is_singleton`: avoid same-process lock duplication bugs.
- `thread_local`: choose whether thread context should be isolated.
- `mode`: filesystem mode for newly created lock files.
- `executor` and `run_in_executor`: async execution strategy for async lock types.

Example: a non-blocking probe.

```python
from filelock import FileLock, Timeout

lock = FileLock("report.lock", blocking=False)

try:
    with lock:
        generate_report()
except Timeout:
    print("another worker already owns report.lock")
```

## Common Pitfalls

### Locking the protected file instead of a dedicated lock file

Use `report.csv.lock` to protect `report.csv`. Upstream explicitly recommends locking a separate file.

### Assuming the lock file's presence is the source of truth

On Windows and on some mounted filesystems, a released lock can leave the lock file behind temporarily. The lock state matters more than file presence.

### Creating multiple lock objects for the same path in one thread

Since `3.24.0`, `filelock` defends against some self-deadlock cases by raising `RuntimeError`, but the practical fix is still to reuse one lock object or set `is_singleton=True`.

### Treating `SoftFileLock` as equivalent to a hard lock

It only coordinates by file creation and cooperation. It does not give the same guarantees as a hard OS-backed lock.

### Using read/write locks without the `.db` suffix

`ReadWriteLock` requires a SQLite database path. Do not pass `catalog-lock` or `catalog.lock`; pass something like `catalog.lock.db`.

### Expecting network-mounted filesystems to behave like local disks

The concepts guide documents caveats for NFS, SMB, Docker bind mounts, and other mounted filesystems. If losing mutual exclusion would be unacceptable, do not rely on file-based locks alone.

### Depending on garbage collection for release

The tutorial warns that an unreferenced acquired lock may be released during garbage collection. Keep a stable reference and release explicitly or with a context manager.

## Version-Sensitive Notes

### 3.25.2

- Released on March 11, 2026.
- Upstream fixed an `EIO` close issue on Unix when the lock lived on a Docker bind mount or similar mounted volume.

### 3.25.1

- Restored best-effort lock file cleanup during `WindowsFileLock.release()`.

### 3.25.0

- Added `AsyncReadWriteLock`.

### 3.24.x

- Added stale-lock support through `lifetime`.
- Added `cancel_check` so another thread can abort acquisition.
- Added `poll_interval` to constructors for sync and async exclusive locks.
- Added self-deadlock detection when the same thread creates multiple `FileLock` objects for the same path.
- Improved exclusive lock behavior on platforms without POSIX-style hard locking by falling back to a soft lock when appropriate.

If your codebase was written against pre-`3.24` examples, double-check constructor signatures and do not assume read/write or cancellation features exist.

## Recommended Agent Workflow

1. Start with `FileLock` and a dedicated `.lock` file.
2. Use the context manager form unless you need cancellation or special retry control.
3. Reuse one lock object per path in-process.
4. Move to `ReadWriteLock` only when you actually need concurrent readers.
5. Treat mounted and network filesystems as suspicious until the deployment environment is verified.

## Official Sources

- Documentation root: https://py-filelock.readthedocs.io/en/latest/
- Tutorial: https://py-filelock.readthedocs.io/en/latest/tutorial.html
- How-to guide: https://py-filelock.readthedocs.io/en/latest/how-to.html
- Concepts guide: https://py-filelock.readthedocs.io/en/latest/concepts.html
- API reference: https://py-filelock.readthedocs.io/en/latest/api.html
- Changelog: https://py-filelock.readthedocs.io/en/latest/changelog.html
- PyPI: https://pypi.org/project/filelock/
- Repository: https://github.com/tox-dev/filelock
