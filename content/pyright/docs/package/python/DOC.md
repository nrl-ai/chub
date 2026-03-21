---
name: package
description: "Pyright static type checker for Python projects, installed from PyPI and configured with pyrightconfig.json or pyproject.toml"
metadata:
  languages: "python"
  versions: "1.1.408"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pyright,python,typing,type-checking,static-analysis,linting"
---

# Pyright Python Package Guide

## Golden Rule

Use `pyright` as a development tool, not a runtime dependency. Install the PyPI package that matches your project tooling, pin the version in CI, and keep the type-checker configuration in exactly one place that your team actually uses.

For Python packaging, the `pyright` package on PyPI is a wrapper that installs and runs Microsoft Pyright. The CLI and config behavior come from the upstream Pyright docs.

## Install

Recommended for most Python projects:

```bash
python -m pip install "pyright[nodejs]==1.1.408"
```

That extra makes the wrapper manage a Node.js runtime for you. If your environment already provides Node.js and you want to use it, install without the extra:

```bash
python -m pip install "pyright==1.1.408"
```

Common project-tool variants:

```bash
uv add --dev "pyright[nodejs]==1.1.408"
poetry add --group dev "pyright[nodejs]==1.1.408"
```

Verify the actual engine version used in CI:

```bash
pyright --version
```

## Initialize And Configure

Pyright reads `pyrightconfig.json` if present. It can also read `[tool.pyright]` from `pyproject.toml`. If both exist, `pyrightconfig.json` takes precedence.

Minimal `pyproject.toml` setup:

```toml
[tool.pyright]
include = ["src", "tests"]
pythonVersion = "3.12"
typeCheckingMode = "standard"
venvPath = "."
venv = ".venv"
```

Equivalent `pyrightconfig.json`:

```json
{
  "include": ["src", "tests"],
  "pythonVersion": "3.12",
  "typeCheckingMode": "standard",
  "venvPath": ".",
  "venv": ".venv"
}
```

Good defaults for a typed package:

- `typeCheckingMode = "standard"` for existing codebases, then tighten selectively.
- Use `strict = ["src/your_package"]` or file-level `# pyright: strict` for new modules instead of flipping a large legacy repo to full strict mode at once.
- Set `pythonVersion` to the lowest Python version you support, not whatever happens to be installed on one developer machine.
- Use `stubPath = "typings"` only when you maintain custom stubs.

Example with targeted strictness:

```toml
[tool.pyright]
include = ["src", "tests"]
pythonVersion = "3.12"
typeCheckingMode = "standard"
strict = ["src/my_package"]
reportMissingTypeStubs = false
```

## Core Usage

Run against the whole project:

```bash
pyright
```

Check specific paths:

```bash
pyright src tests
```

Watch mode for local development:

```bash
pyright --watch
```

Machine-readable output for CI tooling:

```bash
pyright --outputjson
```

If you need to point Pyright at a specific interpreter for one-off checks, prefer `--pythonpath` over hard-coding virtualenv paths into shared config:

```bash
pyright --pythonpath .venv/bin/python
```

Useful package-author commands:

```bash
pyright --verifytypes my_package
pyright --createstub some_untyped_dependency
```

- `--verifytypes` checks whether a published package exposes a strong typed surface, especially around `py.typed`.
- `--createstub` is a fallback for generating starter stubs for untyped dependencies.

## Import Resolution And Environment Setup

Pyright resolves imports from the configured execution environment, installed packages, and local source roots. In practice, most false-positive import errors come from environment mismatch rather than from Pyright itself.

Recommended patterns:

- In editors, select the correct interpreter or virtualenv first.
- In CI, install project dependencies before running Pyright.
- For `src/` layouts, add `extraPaths = ["src"]` if your imports are not resolved the way you expect.
- Use explicit execution environments when different folders target different Python versions or import roots.

Example `src/` layout:

```toml
[tool.pyright]
include = ["src", "tests"]
extraPaths = ["src"]
venvPath = "."
venv = ".venv"
```

Editable install caveat from the upstream docs: import hooks that rely on executable code in `.pth` files are not statically analyzable. If editable installs are required, prefer path-based `.pth` strategies marked as compatible or strict by the packaging backend.

## CI And Pre-commit

Typical CI command:

```bash
python -m pip install "pyright[nodejs]==1.1.408"
pyright --outputjson
```

Minimal `pre-commit` hook:

```yaml
repos:
  - repo: local
    hooks:
      - id: pyright
        name: pyright
        entry: pyright
        language: system
        types: [python]
        pass_filenames: false
```

Use `pass_filenames: false` if your config depends on project-wide analysis. Passing only changed files can hide issues in dependent modules.

## Wrapper-Specific Configuration

The PyPI wrapper has no auth or remote-service configuration. It only controls how the Pyright engine is installed and launched.

Useful wrapper environment variables from the package README:

- `PYRIGHT_PYTHON_FORCE_VERSION`: force a specific Pyright engine version or `latest`
- `PYRIGHT_PYTHON_GLOBAL_NODE`: use a globally installed `node`
- `PYRIGHT_PYTHON_NODE_VERSION`: choose the Node.js version managed by the wrapper
- `PYRIGHT_PYTHON_CACHE_DIR`: customize where the wrapper caches downloads
- `PYRIGHT_PYTHON_VERBOSE`: print wrapper-level debug output

Avoid `PYRIGHT_PYTHON_FORCE_VERSION` in normal CI unless you deliberately want the wrapper to run a different engine than the pinned package version.

## Common Pitfalls

- Do not treat `pyright` as an importable runtime library. It is a CLI tool for static analysis.
- Do not keep both `pyrightconfig.json` and `[tool.pyright]` unless you want the JSON file to win.
- `reportMissingImports` usually means the interpreter, virtualenv, or installed dependencies are wrong, not that Pyright is broken.
- `venvPath` and `venv` are convenient locally but brittle in shared configs if every developer names environments differently.
- Editable installs that depend on import hooks can work at runtime and still fail static resolution in Pyright.
- `useLibraryCodeForTypes` is not a substitute for proper typing metadata. Untyped libraries still benefit from stubs or `py.typed`.
- Pyright is strict about control flow and `None` handling once you enable stricter modes. Many old blog posts assume looser defaults.

## Version-Sensitive Notes For 1.1.408

- As of 2026-03-12, the version used here `1.1.408` matches the live PyPI package version and the upstream GitHub release tag.
- The Microsoft docs site tracks upstream Pyright behavior and can move ahead of whatever version your project has pinned, so verify with `pyright --version` before assuming a newly documented flag exists in your environment.
- The wrapper can intentionally drift from the pinned package if you set `PYRIGHT_PYTHON_FORCE_VERSION` or related launcher variables. Avoid that unless you are debugging a version mismatch on purpose.
- For VS Code, Microsoft recommends Pylance for the editor experience. Use the PyPI package when you need the CLI in Python tooling, CI, or pre-commit.
