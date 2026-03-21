---
name: package
description: "psutil Python package guide for process and system monitoring"
metadata:
  languages: "python"
  versions: "7.2.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "psutil,python,process,system,monitoring,metrics"
---

# psutil Python Package Guide

## Golden Rule

Use `psutil` for local process and system inspection, not for remote host management. Prefer module-level APIs for machine-wide metrics, `Process(pid)` for per-process inspection, and `oneshot()` or `process_iter(attrs=...)` when gathering many fields so you do not pay repeated syscalls.

## Install

Pin the package version your project expects:

```bash
python -m pip install "psutil==7.2.2"
```

Common alternatives:

```bash
uv add "psutil==7.2.2"
poetry add "psutil==7.2.2"
```

Notes:

- Wheels are available for common Linux, macOS, and Windows targets. If pip falls back to building from source, you need a working C compiler toolchain.
- In containers, metrics reflect the runtime and host/kernel limits visible from that container, not necessarily the full physical machine.

## Setup And Privilege Model

`psutil` has no API keys, service endpoints, or authentication flow. The practical setup variables are:

- which process you want to inspect
- what privilege level the current user has
- which operating system features are available

Basic import and capability check:

```python
import psutil

print(psutil.__version__)
print(psutil.boot_time())
print(psutil.cpu_count(logical=True))
```

Privilege expectations:

- Reading your own processes is usually fine without elevation.
- Inspecting other users' processes, open connections, or some memory details may raise `psutil.AccessDenied`.
- Some APIs are OS-specific or partially implemented on certain platforms. Check `hasattr(psutil.Process(), "...")` or the docs before assuming parity across Linux, macOS, Windows, and BSD.

## Core Usage

### Read system-wide metrics

```python
import psutil

cpu_percent = psutil.cpu_percent(interval=1.0)
vm = psutil.virtual_memory()
disk = psutil.disk_usage("/")
net = psutil.net_io_counters()

print({
    "cpu_percent": cpu_percent,
    "mem_available": vm.available,
    "mem_percent": vm.percent,
    "disk_percent": disk.percent,
    "bytes_sent": net.bytes_sent,
    "bytes_recv": net.bytes_recv,
})
```

Use `interval=1.0` or another real interval when you need a meaningful CPU percentage sample. The first non-blocking call is commonly `0.0`.

### Iterate processes safely

`process_iter()` is the standard way to scan running processes. Request only the fields you need:

```python
import psutil

for proc in psutil.process_iter(
    attrs=["pid", "name", "username", "status", "memory_info"],
    ad_value=None,
):
    info = proc.info
    rss = info["memory_info"].rss if info["memory_info"] else None
    print(info["pid"], info["name"], info["status"], rss)
```

Why this pattern matters:

- it avoids repeated lookups for the same process
- it degrades more cleanly when a field is unavailable
- it reduces races compared with manual per-PID probing

### Inspect one process

```python
import psutil

proc = psutil.Process()

with proc.oneshot():
    print(proc.pid)
    print(proc.name())
    print(proc.status())
    print(proc.cpu_percent(interval=0.1))
    print(proc.memory_info().rss)
    print(proc.open_files())
```

Use `oneshot()` when reading several attributes from one `Process`. It batches internal lookups and is significantly cheaper than independent calls.

### Handle transient process errors

Processes can exit while you inspect them. Catch `NoSuchProcess` and `AccessDenied` explicitly:

```python
import psutil

try:
    proc = psutil.Process(12345)
    print(proc.exe())
except psutil.NoSuchProcess:
    print("process exited")
except psutil.AccessDenied:
    print("permission denied")
```

### Work with subprocesses you create

`psutil.Popen` gives you `subprocess.Popen` behavior plus psutil process methods:

```python
import psutil

proc = psutil.Popen(["python", "-c", "import time; time.sleep(5)"])
print(proc.pid)
print(proc.is_running())
print(proc.wait(timeout=10))
```

This is useful when you need both process lifecycle control and psutil inspection without manually wrapping the child PID.

## Configuration Notes

There is no global client object or config file. Most "configuration" in `psutil` is call-level:

- `interval` for CPU sampling
- `percpu=True` when you need per-core metrics
- `nowrap=True` on selected counters to avoid wraparound surprises
- `attrs=[...]` and `ad_value=...` for process iteration
- explicit `pid` selection for per-process reads

Useful examples:

```python
import psutil

print(psutil.cpu_percent(interval=0.5, percpu=True))
print(psutil.disk_io_counters(nowrap=True))
print(psutil.net_io_counters(nowrap=True))
```

If you need a stable snapshot over time, normalize these choices in your own wrapper instead of scattering raw psutil calls across the codebase.

## Common Pitfalls

- `cpu_percent()` and `Process.cpu_percent()` usually need a real interval or a warm-up call. A first immediate call often returns `0.0`.
- For memory pressure checks, prefer `virtual_memory().available` over `virtual_memory().free`. "Free" memory is often the wrong signal on modern OSes.
- Treat every per-process read as racy. PIDs can disappear between discovery and inspection.
- Catch `psutil.NoSuchProcess`, `psutil.AccessDenied`, and sometimes `psutil.ZombieProcess` in monitoring code.
- Use `psutil.net_connections()` or `Process.net_connections()`; older `connections()` names are deprecated in `7.x`.
- Do not assume all metrics exist on all platforms. Sensor, swap, open-files, terminal, and connection details vary by OS and privilege level.
- If you hold a `Process` object for a long time, verify it still refers to the same live process before acting on it. PID reuse is real.
- Scanning every process and calling many getters inside a loop is expensive. Prefer `process_iter(attrs=...)` and `oneshot()`.

## Version-Sensitive Notes

### `7.2.2` versus the live `latest` docs

The docs root `https://psutil.readthedocs.io/en/latest/` renders `psutil 8.0.0` as of `2026-03-12`, not `7.2.2`. That matters because newer docs include changes that are not valid for this package version.

### Relevant `6.x` and `7.x` behavior to remember

- Since `6.0.0`, `process_iter()` no longer checks whether a yielded PID was reused. If you keep `Process` objects around, call `is_running()` or refresh state before taking action.
- Since `7.0.0`, `connections()` is deprecated in favor of `net_connections()`.
- `7.2.0` added `Process.heap_info()` and `Process.heap_trim()` on supported platforms. These are not portable across every OS.
- `7.2.2` improves `Popen.wait()` by using better kernel facilities on newer Linux and BSD/macOS combinations, so long waits on managed child processes behave better than older examples may suggest.

### `8.0.0` examples you should not blindly backport

The current `latest` docs describe `8.0.0` changes including:

- enum values for `Process.status()`
- enum values for the `kind` argument in connection APIs
- changed field order in `cpu_times()` and `cpu_times_percent()`
- support policy shifted to Python `3.7+`

If your project is pinned to `7.2.2`, keep using the `7.x` behavior your tests observe and do not assume those `8.0.0` API details.

## Official Sources

- Docs root: `https://psutil.readthedocs.io/en/latest/`
- Install guide: `https://psutil.readthedocs.io/en/latest/#install`
- API reference: `https://psutil.readthedocs.io/en/latest/#psutil-documentation`
- PyPI release page: `https://pypi.org/project/psutil/`
