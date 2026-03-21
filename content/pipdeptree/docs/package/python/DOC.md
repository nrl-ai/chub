---
name: package
description: "pipdeptree CLI for inspecting installed Python package dependency trees and dependency conflicts"
metadata:
  languages: "python"
  versions: "2.31.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "python,pip,packaging,dependencies,cli,virtualenv"
---

# pipdeptree Python Package Guide

## Golden Rule

`pipdeptree` reports the packages installed in one Python environment. Run it against the exact interpreter or virtualenv your project uses, or the output will describe the wrong dependency graph.

## Install

Install it into the environment you want to inspect:

```bash
python -m pip install "pipdeptree==2.31.0"
```

If you want Graphviz output formats, install the optional extra:

```bash
python -m pip install "pipdeptree[graphviz]==2.31.0"
```

Typical project-local setup:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install -r requirements.txt
python -m pip install "pipdeptree==2.31.0"
```

## Environment Targeting

`pipdeptree` has no authentication or service configuration. The important setup is choosing the correct Python environment.

If you are already inside the right virtualenv, just run:

```bash
pipdeptree
```

If you want to inspect a specific interpreter:

```bash
pipdeptree --python /path/to/project/.venv/bin/python
```

If your virtualenv is active or discoverable, `2.21.0+` supports automatic interpreter detection:

```bash
pipdeptree --python auto
```

Useful environment-scoping flags:

- `--path /some/site-packages`: restrict package discovery to one or more paths
- `--local-only`: hide globally installed packages when the virtualenv can see them
- `--user-only`: inspect only the user site directory

## Core Usage

### Print the dependency tree

```bash
pipdeptree
```

This is the default text output. Warnings about conflicting or cyclic dependencies are printed to `stderr`.

### Show only selected packages

```bash
pipdeptree --packages requests,urllib3
```

Wildcards are supported in package filters, for example `somepackage.*`.

### Find why a package is installed

Use reverse mode to show which installed packages depend on a package:

```bash
pipdeptree --reverse --packages urllib3
```

### Fail CI on dependency warnings

```bash
pipdeptree --warn fail
```

`--warn fail` keeps warnings visible and exits non-zero if conflicts or cycles are found. Use this for CI checks. Use `--warn silence` when you need machine-readable output without warnings on `stderr`.

### Limit tree depth

```bash
pipdeptree --depth 2
```

This is useful when you only need the top of the graph instead of the full transitive tree.

### Exclude noisy packages

```bash
pipdeptree --exclude pip,setuptools,wheel
```

Combine with `--exclude-dependencies` if you want excluded packages and their dependency subtrees omitted.

## Machine-Readable Output

`2.30.0+` prefers `-o/--output` instead of the older dedicated output flags.

JSON formats:

```bash
pipdeptree -o json > deps.json
pipdeptree -o json-tree > deps-tree.json
```

Freeze-style output:

```bash
pipdeptree -o freeze --warn silence > locked-requirements.txt
```

Mermaid flowchart output:

```bash
pipdeptree -o mermaid > deps.mmd
```

Graphviz output:

```bash
pipdeptree -o graphviz-dot > deps.dot
pipdeptree -o graphviz-svg > deps.svg
```

Install the `graphviz` extra before using Graphviz renderers.

## Common Pitfalls

- Wrong interpreter: the most common mistake is running `pipdeptree` outside the project virtualenv and inspecting the wrong site-packages.
- Warnings on `stderr`: parsers often fail because the tree is on `stdout` while conflict/cycle warnings are on `stderr`. Use `--warn silence` for clean machine output.
- Deprecated flags: `-f`, `--json`, `--json-tree`, `--mermaid`, and `--graph-output` still exist in `2.31.0` but are deprecated in favor of `-o/--output`.
- Not a resolver: `pipdeptree` only inspects already installed packages. It does not solve dependencies from a requirements file or lock file.
- Pip internals: the project documents that it relies on pip internals, so unpinned `pip` upgrades can occasionally break automation.
- Optional Graphviz support: `graphviz-*` outputs need the `graphviz` extra and the relevant Graphviz tooling available.

## Version-Sensitive Notes

- The version used here `2.31.0` matches the live PyPI release as of March 12, 2026.
- `2.30.0` introduced `-o/--output` and deprecated the older output-specific flags. Prefer the new interface in new scripts.
- `2.31.0` added `--depth` support for Graphviz output, so depth-limited graph exports depend on at least this release.
- `--python auto` requires `2.21.0` or later. Do not assume it exists in older project environments.

## Official Sources

- Repository and README: `https://github.com/tox-dev/pipdeptree`
- Changelog: `https://github.com/tox-dev/pipdeptree/blob/main/CHANGELOG.md`
- PyPI project page: `https://pypi.org/project/pipdeptree/`
- PyPI JSON metadata: `https://pypi.org/pypi/pipdeptree/json`
