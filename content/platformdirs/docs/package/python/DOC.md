---
name: package
description: "platformdirs package guide for Python: platform-specific user and shared directories for app data, config, cache, logs, state, and runtime files"
metadata:
  languages: "python"
  versions: "4.9.4"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "platformdirs,paths,filesystem,xdg,windows,macos,linux,android"
---

# platformdirs Python Package Guide

## What It Does

`platformdirs` resolves the correct platform-specific directories for application data, config, cache, logs, state, runtime files, and several user folders. It hides the OS-specific rules for Linux and other Unix systems, macOS, Windows, and Android.

Use it when code needs a writable or discoverable app directory and you do not want to hardcode `~/.config`, `~/Library`, `AppData`, `/var`, or similar platform-specific paths yourself.

## Version Scope

- Package: `platformdirs`
- Language: `python`
- Version covered in frontmatter: `4.9.4`
- Latest release on PyPI at the time of writing: `4.9.4` on 2026-03-05
- Python requirement from PyPI metadata: `>=3.10`

The API reference and docs site cover the 4.9.x line well, but the published changelog currently goes through `4.9.2` while PyPI already has `4.9.4`. For coding work, treat the 4.9.x API surface as current and verify any very recent release detail on PyPI when version drift matters.

## Install

```bash
pip install platformdirs==4.9.4
```

```bash
uv add platformdirs==4.9.4
```

```bash
poetry add platformdirs==4.9.4
```

## Choose An API Style

### `PlatformDirs` instance

Use this when one app needs several related directories.

```python
from platformdirs import PlatformDirs

dirs = PlatformDirs(appname="acme-tool", appauthor="Acme")

print(dirs.user_data_path)
print(dirs.user_config_path)
print(dirs.user_cache_path)
print(dirs.user_log_path)
print(dirs.user_runtime_path)
```

### Convenience functions

Use these for one-off lookups.

```python
from platformdirs import user_config_path, user_cache_dir

config_path = user_config_path("acme-tool", "Acme")
cache_dir = user_cache_dir("acme-tool", "Acme")
```

### `*_dir` vs `*_path`

- `*_dir` returns `str`
- `*_path` returns `pathlib.Path`

Prefer `*_path` in new code. It avoids manual `Path(...)` wrapping and makes file IO and joins straightforward.

## Initialization And Core Parameters

```python
from platformdirs import PlatformDirs

dirs = PlatformDirs(
    appname="acme-tool",
    appauthor="Acme",
    version="2.0",
    roaming=False,
    multipath=False,
    opinion=True,
    ensure_exists=False,
    use_site_for_root=False,
)
```

Parameters that matter in practice:

- `appname`: app-specific suffix. Pass this unless you explicitly want the raw platform root.
- `appauthor`: Windows-only in practice. It adds a vendor directory such as `Acme\\acme-tool`. Pass `False` to suppress that extra level on Windows.
- `version`: appends a version component so incompatible on-disk layouts can coexist.
- `roaming`: Windows-only. Redirects user data/config to roaming AppData instead of local AppData.
- `multipath`: on Unix/macOS, `site_data_dir` and `site_config_dir` can return all configured roots joined by `os.pathsep`.
- `opinion`: keeps opinionated subdirectories such as `Cache`, `Logs`, or `/log`.
- `ensure_exists`: creates directories on access.
- `use_site_for_root`: Unix-only. When running as root, redirects `user_*` lookups to `site_*` directories instead of `/root/...`.

## Core Usage

### Create app-local config, cache, and logs

```python
from platformdirs import PlatformDirs

dirs = PlatformDirs("acme-tool", "Acme", ensure_exists=True)

config_file = dirs.user_config_path / "settings.toml"
cache_file = dirs.user_cache_path / "responses.json"
log_file = dirs.user_log_path / "app.log"

config_file.write_text("debug = false\n", encoding="utf-8")
cache_file.write_text("{}", encoding="utf-8")
log_file.write_text("started\n", encoding="utf-8")
```

If you do not want property access to create directories, leave `ensure_exists=False` and create parents yourself:

```python
from platformdirs import user_data_path

db_file = user_data_path("acme-tool") / "state.db"
db_file.parent.mkdir(parents=True, exist_ok=True)
db_file.write_bytes(b"sqlite-bytes")
```

### Keep incompatible app data separated by version

```python
from platformdirs import user_data_path

state_dir = user_data_path("acme-tool", version="2026.03")
db_path = state_dir / "state.db"
```

The upstream how-to guide shows this as the intended pattern for migration between incompatible on-disk versions.

### Merge defaults from shared config and override per user

```python
from platformdirs import PlatformDirs

dirs = PlatformDirs("acme-tool", "Acme")

for base in dirs.iter_config_paths():
    candidate = base / "settings.toml"
    if candidate.exists():
        print(f"found config at {candidate}")

user_override = dirs.user_config_path / "settings.toml"
```

Use the iterator methods when you want the user path plus all shared candidate paths, rather than just one resolved string.

### Use the newer helper directories directly

```python
from platformdirs import user_documents_path, user_downloads_path, user_bin_path

documents = user_documents_path()
downloads = user_downloads_path()
scripts_dir = user_bin_path()
```

These helpers are useful for launcher registration, user-installed script locations, and file export workflows.

### Compatibility alias for `appdirs` migrations

```python
from platformdirs import AppDirs

dirs = AppDirs("acme-tool", "Acme")
assert dirs.user_cache_dir
```

`AppDirs` remains an alias for `PlatformDirs`, which helps when porting code from the older `appdirs` package.

## Platform Behavior You Need To Remember

### Linux and other Unix platforms

- Uses XDG-style defaults such as `~/.local/share`, `~/.config`, `~/.cache`, `~/.local/state`, and `/run/user/<uid>`.
- Honors `XDG_DATA_HOME`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, `XDG_STATE_HOME`, `XDG_RUNTIME_DIR`, `XDG_DATA_DIRS`, and `XDG_CONFIG_DIRS`.
- Shared directory helpers can be multi-root when `multipath=True`.
- `use_site_for_root=True` changes `user_*` resolution when the process runs as root.

### macOS

- Data, config, and state default to `~/Library/Application Support/<AppName>`.
- Cache defaults to `~/Library/Caches/<AppName>`.
- Logs default to `~/Library/Logs/<AppName>`.
- The same XDG environment variables are also honored on macOS and take precedence when set.

### Windows

- User data/config are under AppData and may include both `appauthor` and `appname`.
- `roaming=True` switches user data/config to roaming AppData.
- `Cache` and `Logs` suffixes are added by default when `opinion=True`.
- The docs reference `WIN_PD_OVERRIDE_*` environment variables for overriding default Windows roots.

### Android

- Android-specific roots are supported for data, cache, runtime, and user folders.
- Treat Android paths as platform-detected behavior; do not hardcode them unless your runtime contract guarantees Android.

## Configuration And Environment

There is no auth model. Configuration is local and path-resolution driven.

Common controls:

- Set XDG environment variables in tests, containers, or CI when you need deterministic Unix/macOS paths.
- On Windows, use the documented `WIN_PD_OVERRIDE_*` overrides only when you intentionally need non-default roots.
- Use `version=` when data formats are incompatible across app versions.
- Use `ensure_exists=True` only when directory creation on read is acceptable.
- Use `use_site_for_root=True` for root-owned services that should use system directories instead of `/root`.

Example: deterministic config paths in tests.

```python
import os
from platformdirs import user_config_path

os.environ["XDG_CONFIG_HOME"] = "/tmp/test-config"

config_dir = user_config_path("acme-tool")
assert str(config_dir) == "/tmp/test-config/acme-tool"
```

Example: Windows vendor folder suppression.

```python
from platformdirs import user_data_dir

path_without_vendor = user_data_dir("acme-tool", False)
```

## Common Pitfalls

### Treating `site_*` paths as normal app-write locations

`site_*` directories are generally shared and often require elevated permissions. Normal applications should usually write to `user_*` paths.

### Forgetting that macOS data and config share the same base location

On macOS, `user_data_dir` and `user_config_dir` both resolve under `~/Library/Application Support/<AppName>`. If your app wants separate logical areas, create subdirectories yourself.

### Expecting `multipath=True` to return a list

For `site_data_dir` and `site_config_dir`, `multipath=True` returns a path string joined by `os.pathsep`. Use `iter_*_paths()` or `iter_*_dirs()` if you need discrete candidates.

### Causing unintended side effects with `ensure_exists=True`

When enabled, property access can create directories. Keep it `False` in discovery code, dry runs, or read-only checks.

### Mixing `str` and `Path`

Do not call `Path` methods on `*_dir` results. Prefer `*_path` if you are going to join paths or read and write files.

### Omitting `appname`

Without `appname`, many helpers return the raw platform root rather than an app-specific subdirectory. That is rarely what application code wants.

### Assuming every helper appends `appname`

Helpers such as `user_bin_dir`, `site_bin_dir`, `user_applications_dir`, and `site_applications_dir` return shared script or launcher directories and do not append `appname` or `version`.

## Version-Sensitive Notes

### 4.9.x

- PyPI currently lists `4.9.4` as the latest release, published on 2026-03-05.
- The Read the Docs site was last updated on 2026-03-02 and its changelog currently documents `4.9.2`, `4.9.1`, and `4.9.0`, not `4.9.4`.
- `4.9.0` added `site_bin_dir` and `site_applications_dir`, and documented `use_site_for_root`.
- `4.9.2` was documentation-focused, not an API-shape change.

### 4.8.x

- `4.8.0` added `user_applications_dir`, `user_bin_dir`, `site_state_dir`, `site_log_dir`, and `use_site_for_root`.
- `4.8.0` also added Windows override support via `WIN_PD_OVERRIDE_*` and `PLATFORMDIRS_*` environment variables.

### 4.7.x

- `4.7.0` changed Unix runtime behavior to fall back to a temp directory when the runtime dir is not writable.

If your environment is pinned below `4.8.0` or `4.9.0`, verify that the newer helpers you plan to call actually exist before writing code against them.

## Recommended Agent Workflow

1. Prefer `PlatformDirs` when you need several directories for the same app.
2. Prefer `*_path` variants in new Python code.
3. Write normal app state to `user_*` directories, not `site_*`.
4. Use iterator helpers when merging config or data from user and shared locations.
5. Control XDG environment variables in tests before asserting exact Unix/macOS paths.
6. Confirm version-sensitive helpers if the project is pinned below `4.8.0` or `4.9.0`.

## Official Sources

- Documentation root: https://platformdirs.readthedocs.io/en/latest/
- Tutorial: https://platformdirs.readthedocs.io/en/latest/tutorial.html
- How-to guides: https://platformdirs.readthedocs.io/en/latest/howto.html
- Parameter reference: https://platformdirs.readthedocs.io/en/latest/parameters.html
- API reference: https://platformdirs.readthedocs.io/en/latest/api.html
- Platform details: https://platformdirs.readthedocs.io/en/latest/platforms.html
- Changelog: https://platformdirs.readthedocs.io/en/latest/changelog.html
- PyPI registry page: https://pypi.org/project/platformdirs/
