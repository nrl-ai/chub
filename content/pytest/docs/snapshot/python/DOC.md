---
name: snapshot
description: "pytest-snapshot package guide for snapshot testing with pytest fixtures and file-based snapshots"
metadata:
  languages: "python"
  versions: "0.9.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest-snapshot,pytest,python,testing,snapshots"
---

# pytest-snapshot Python Package Guide

## What It Is

`pytest-snapshot` is a small pytest plugin for file-based snapshot testing. Once installed, tests get a `snapshot` fixture that can compare strings or bytes against snapshot files on disk and update those files with explicit CLI flags.

The package is still published at `0.9.0`, and PyPI shows that `0.9.0` is the latest release as of March 12, 2026. The upstream source of truth is the maintainer GitHub repo README plus the plugin implementation itself.

## Install

Install it into the same environment that runs pytest:

```bash
python -m pip install pytest-snapshot==0.9.0
```

With uv:

```bash
uv add --dev pytest-snapshot==0.9.0
```

With Poetry:

```bash
poetry add --group dev pytest-snapshot==0.9.0
```

`pytest-snapshot` is a test-only dependency. It belongs in the test environment, not your application runtime dependencies.

## Initialize And First Run

After installation, write tests that accept the `snapshot` fixture. You do not import `snapshot` directly; pytest provides it.

Minimal example:

```python
def render_user(user_id: int) -> str:
    return f"user:{user_id}\nactive:true\n"

def test_render_user(snapshot):
    snapshot.snapshot_dir = "snapshots"
    snapshot.assert_match(render_user(123), "render_user.txt")
```

First create or refresh snapshots with:

```bash
pytest --snapshot-update
```

Then rerun normal verification without the update flag:

```bash
pytest
```

Important behavior: when `--snapshot-update` creates, changes, or deletes snapshot files, the run still fails on purpose so you review the diff and rerun tests normally.

## Core Usage

### Compare one string or bytes payload

Use `snapshot.assert_match(value, snapshot_name)` for a single file snapshot.

```python
import json

def make_payload():
    return {"name": "Ada", "roles": ["admin", "author"]}

def test_payload(snapshot):
    snapshot.snapshot_dir = "snapshots"
    payload = json.dumps(make_payload(), indent=2, sort_keys=True) + "\n"
    snapshot.assert_match(payload, "payload.json")
```

The value must be `str` or `bytes`. For dicts, lists, or custom objects, serialize first.

### Compare a directory tree

Use `snapshot.assert_match_dir(mapping, snapshot_dir_name)` when output is naturally multiple files. The README documents nested dict support, which becomes nested directories under the snapshot root.

```python
def test_generated_site(snapshot):
    snapshot.snapshot_dir = "snapshots"
    snapshot.assert_match_dir(
        {
            "index.html": "<h1>Home</h1>\n",
            "posts": {
                "hello.txt": "hello\n",
                "about.txt": "about\n",
            },
        },
        "site",
    )
```

This is useful for generators, codegen, config trees, and multi-file exports where one golden file is too coarse.

### Default snapshot location

If you do not set `snapshot.snapshot_dir`, the plugin derives a directory automatically under:

```text
snapshots/<test_module_stem>/<test_name>
```

For parametrized tests, the parameter id is appended as another path segment. Use an explicit `snapshot.snapshot_dir` when you want stable paths shared across helpers or multiple tests.

### Snapshot names are paths

`snapshot_name` can include subdirectories:

```python
def test_reports(snapshot):
    snapshot.snapshot_dir = "snapshots"
    snapshot.assert_match("ok\n", "reports/daily/status.txt")
```

Paths are resolved relative to `snapshot.snapshot_dir`. Absolute paths are only valid if they still live inside that snapshot root.

## Update And Deletion Workflow

Refresh existing snapshots:

```bash
pytest --snapshot-update
```

Allow deletion of stale files during directory snapshot updates:

```bash
pytest --snapshot-update --allow-snapshot-deletion
```

Use the deletion flag carefully. Without it, `assert_match_dir(...)` will report extra snapshot files instead of silently removing them.

Recommended team workflow:

1. Run `pytest --snapshot-update` after intentional output changes.
2. Review the changed snapshot files in git.
3. Rerun plain `pytest` to verify the updated snapshots now pass.

## Config And Auth

There is no service authentication, API key setup, or network configuration.

Project-level configuration is mostly:

- whether to rely on the default snapshot directory layout or set `snapshot.snapshot_dir` explicitly
- whether CI ever allows snapshot updates, which is usually `no`
- whether local update flows also allow deletions with `--allow-snapshot-deletion`
- how structured data is normalized before snapshotting

For stable diffs, normalize before snapshotting:

- JSON: `json.dumps(..., indent=2, sort_keys=True) + "\n"`
- timestamps, UUIDs, temp paths: replace or freeze them first
- platform-specific paths or line endings: normalize them before assertion

## Common Pitfalls

- `--snapshot-update` is not a green build mode. The plugin intentionally fails after modifying snapshot files so you notice and review the changes.
- Only `str` and `bytes` are accepted snapshot payloads. Serialize Python objects yourself instead of passing raw dicts or lists.
- Strings containing carriage returns (`\r`) are rejected by the plugin. If your content really needs CRLF semantics, snapshot bytes instead.
- If you do not set `snapshot.snapshot_dir`, the generated default path depends on the test file name, test name, and parametrization id. That can surprise refactors.
- `assert_match_dir(...)` compares the directory contents exactly. Renamed or removed files will fail until you rerun with both `--snapshot-update` and, if needed, `--allow-snapshot-deletion`.
- Snapshot names must stay inside the snapshot root. Do not try to escape the directory with absolute paths or `..` segments.
- This package has not shipped a new release since `0.9.0` in April 2022. If your project is on much newer pytest or Python releases, verify behavior in your environment instead of assuming recent ecosystem changes are reflected upstream.

## Version-Sensitive Notes

- PyPI lists `0.9.0` as the latest release, published on April 23, 2022.
- The published package metadata still advertises broad minimum compatibility: Python `>=3.5` and `pytest>=3.0.0`.
- The repository README and plugin source are the canonical docs. There is no separate hosted documentation site.
- Because the release is old, prefer direct upstream examples and source behavior over third-party blog posts, especially for recent pytest versions.

## Official Sources

- PyPI project page: `https://pypi.org/project/pytest-snapshot/`
- Maintainer repository: `https://github.com/joseph-roitman/pytest-snapshot`
- Upstream README: `https://raw.githubusercontent.com/joseph-roitman/pytest-snapshot/master/README.rst`
- Plugin implementation: `https://raw.githubusercontent.com/joseph-roitman/pytest-snapshot/master/pytest_snapshot/plugin.py`
