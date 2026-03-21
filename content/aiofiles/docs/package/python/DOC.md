---
name: package
description: "aiofiles package guide for asyncio-friendly local file I/O in Python"
metadata:
  languages: "python"
  versions: "25.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aiofiles,asyncio,python,files,io,tempfile,threadpool"
---

# aiofiles Python Package Guide

## What It Is

`aiofiles` gives `asyncio` code an async-looking interface for normal local file operations. It does not provide true kernel-level async disk I/O. Instead, it delegates blocking file and selected `os` calls to an executor so the event loop can keep running other tasks.

Use it when:

- your application is already async
- you need to read or write local files without blocking unrelated coroutines
- you want an `async with` / `await` interface close to Python's built-in file APIs

Do not use it as a network storage client. For S3, HTTP, databases, or other remote backends, use the service-specific async library instead.

## Installation

Install the exact version covered here:

```bash
pip install aiofiles==25.1.0
```

With modern package managers:

```bash
uv add aiofiles==25.1.0
poetry add aiofiles==25.1.0
```

`aiofiles 25.1.0` does not support Python 3.8. If a project is pinned to Python 3.8, stay on `aiofiles 24.1.0` instead.

## Initialize And Open Files

There is no service setup, authentication, or client object. Import the module and use `aiofiles.open()` in the places where synchronous code would call built-in `open()`.

```python
import asyncio
import aiofiles

async def load_text(path: str) -> str:
    async with aiofiles.open(path, "r", encoding="utf-8") as f:
        return await f.read()

print(asyncio.run(load_text("notes.txt")))
```

`aiofiles.open()` accepts the usual `open()` arguments plus optional `loop=` and `executor=` keyword arguments. In modern code, rely on the running loop and only pass a custom executor when file work should not share the default thread pool.

## Core Usage

### Read And Write Text Files

```python
import aiofiles

async def rewrite_file(src: str, dst: str) -> None:
    async with aiofiles.open(src, "r", encoding="utf-8") as infile:
        text = await infile.read()

    async with aiofiles.open(dst, "w", encoding="utf-8") as outfile:
        await outfile.write(text.upper())
        await outfile.flush()
```

### Stream Lines With `async for`

`aiofiles` file objects support async iteration, which is the cleanest way to process large text files incrementally.

```python
import aiofiles

async def first_nonempty_line(path: str) -> str | None:
    async with aiofiles.open(path, "r", encoding="utf-8") as f:
        async for line in f:
            if line.strip():
                return line.rstrip("\n")
    return None
```

### Copy Binary Data In Chunks

```python
import aiofiles

CHUNK_SIZE = 1024 * 1024

async def copy_file(src: str, dst: str) -> None:
    async with aiofiles.open(src, "rb") as infile:
        async with aiofiles.open(dst, "wb") as outfile:
            while chunk := await infile.read(CHUNK_SIZE):
                await outfile.write(chunk)
```

### Use Temporary Files And Directories

`aiofiles.tempfile` wraps the common `tempfile` helpers with async context managers and async file methods.

```python
import aiofiles.tempfile

async def build_temp_payload() -> str:
    async with aiofiles.tempfile.NamedTemporaryFile("w+", encoding="utf-8") as f:
        await f.write("payload\n")
        await f.seek(0)
        return await f.read()
```

### Use Async Wrappers Around Selected `os` Functions

`aiofiles.os` exposes async wrappers for a subset of `os` helpers such as `rename`, `replace`, `remove`, `mkdir`, `makedirs`, `stat`, `listdir`, `scandir`, `path.abspath`, and `path.getcwd`.

```python
import aiofiles.os

async def move_into_place(tmp_path: str, final_path: str) -> None:
    await aiofiles.os.replace(tmp_path, final_path)
```

### Handle Standard Streams

`aiofiles.stdin`, `aiofiles.stdout`, and related helpers expose wrapped binary and text stdio objects when async code needs to interact with process streams using the same file-style API.

```python
import aiofiles

async def write_status() -> None:
    await aiofiles.stdout.write("ready\n")
    await aiofiles.stdout.flush()
```

## Configuration And Auth

There is no auth model and no package-level configuration file.

The main choices are local I/O settings:

- file mode such as `"r"`, `"w"`, `"a"`, `"rb"`, or `"wb"`
- encoding and newline handling for text files
- buffer sizing and temporary-file behavior
- executor selection for thread-pool isolation

If file work should not compete with other threadpool tasks, pass a dedicated executor:

```python
import asyncio
from concurrent.futures import ThreadPoolExecutor

import aiofiles

file_pool = ThreadPoolExecutor(max_workers=4)

async def read_with_custom_pool(path: str) -> str:
    async with aiofiles.open(path, "r", encoding="utf-8", executor=file_pool) as f:
        return await f.read()

try:
    print(asyncio.run(read_with_custom_pool("example.txt")))
finally:
    file_pool.shutdown(wait=True)
```

## Common Pitfalls

- `aiofiles` still uses blocking file APIs under the hood. It avoids blocking the event loop by moving that work into an executor.
- Always `await` methods like `read()`, `write()`, `seek()`, `flush()`, and `close()`. The wrapped file object is not interchangeable with a normal synchronous file handle.
- Prefer `async with` so files close correctly on errors and cancellations.
- Be explicit about text vs binary mode. The same type and encoding mistakes from built-in `open()` still apply.
- High concurrency can saturate the default executor. If file throughput matters, consider a separate `ThreadPoolExecutor`.
- `aiofiles.os` covers only selected functions. Check the current README before assuming a specific `os` or `os.path` helper exists.
- When unit testing, patch `aiofiles.threadpool.sync_open` or use the library's wrapping helpers instead of mocking `aiofiles.open()` like a normal synchronous file factory.

## Version-Sensitive Notes

- The version used here and the current covered version are both `25.1.0`.
- PyPI lists `25.1.0` as released on `2025-10-09`.
- `25.1.0` requires Python `>=3.9` and adds support for Python 3.14.
- If a project still needs Python 3.8, use `aiofiles 24.1.0` instead of upgrading.
- For long-lived internal docs or ingestion work, prefer the tagged `v25.1.0` sources over `main` when you need version-stable examples.

## Official Sources

- Repository: https://github.com/Tinche/aiofiles
- Tagged README for `25.1.0`: https://github.com/Tinche/aiofiles/blob/v25.1.0/README.md
- PyPI project page: https://pypi.org/project/aiofiles/
- PyPI release page for `25.1.0`: https://pypi.org/project/aiofiles/25.1.0/
- PyPI version metadata JSON: https://pypi.org/pypi/aiofiles/25.1.0/json
- GitHub release for `v25.1.0`: https://github.com/Tinche/aiofiles/releases/tag/v25.1.0
