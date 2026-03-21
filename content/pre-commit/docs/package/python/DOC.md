---
name: package
description: "pre-commit package guide for managing and running multi-language Git hooks in Python projects"
metadata:
  languages: "python"
  versions: "4.5.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pre-commit,git,hooks,linting,formatting,ci"
---

# pre-commit Python Package Guide

## Golden Rule

Use `pre-commit` as a Git-hook manager, keep your hook `rev` values pinned to immutable tags or SHAs, and run `pre-commit install` in every clone. Do not point hooks at mutable refs like `HEAD` or `main`; use `pre-commit autoupdate` when you want newer hook versions.

## Install

Install it into the environment that owns your repo tooling:

```bash
python -m pip install "pre-commit==4.5.1"
```

If you want the CLI without managing a virtualenv, the project also publishes a zipapp:

```bash
curl -L https://github.com/pre-commit/pre-commit/releases/download/v4.5.1/pre-commit-4.5.1.pyz -o pre-commit.pyz
python pre-commit.pyz --version
```

## Initial Setup

Create a starter config and install the Git hook scripts:

```bash
pre-commit sample-config > .pre-commit-config.yaml
pre-commit install
```

Useful variants:

```bash
pre-commit install --install-hooks
pre-commit install --hook-type pre-commit --hook-type pre-push
```

- `install` writes the hook script into `.git/hooks/`.
- `--install-hooks` eagerly creates hook environments instead of waiting for the first run.
- If you want `pre-push`, `commit-msg`, or other hook types, install them explicitly or set `default_install_hook_types` in config.

## Config Shape

The config file is YAML with a top-level `repos:` list. Each repo entry points at a Git repository that contains hook definitions.

Minimal example:

```yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: vX.Y.Z
    hooks:
      - id: check-yaml
      - id: end-of-file-fixer
      - id: trailing-whitespace
```

Common top-level settings:

- `minimum_pre_commit_version`: require a minimum local `pre-commit` version when you rely on newer features.
- `default_install_hook_types`: install additional hook types by default.
- `default_language_version`: set default runtimes such as `python: python3.12`.
- `default_stages`: restrict hooks to stages like `pre-commit` or `pre-push`.
- `files` and `exclude`: global include and exclude regexes.
- `fail_fast`: stop after the first failing hook.

Useful validation commands:

```bash
pre-commit validate-config
pre-commit validate-manifest
```

## Core Usage

Run hooks against all tracked files:

```bash
pre-commit run --all-files
```

Run one hook only:

```bash
pre-commit run black --all-files
```

Run hooks for a specific stage:

```bash
pre-commit run --hook-stage pre-push --all-files
```

Run against a diff range in CI:

```bash
pre-commit run --from-ref origin/HEAD --to-ref HEAD
```

Update pinned hook revisions in `.pre-commit-config.yaml`:

```bash
pre-commit autoupdate
```

Clean or rebuild cached environments when they drift:

```bash
pre-commit clean
pre-commit gc
pre-commit install-hooks
```

## Local And Meta Hooks

Use `repo: local` for project-specific commands that are not published as a reusable hook repo:

```yaml
repos:
  - repo: local
    hooks:
      - id: mypy
        name: mypy
        entry: mypy
        language: python
        types: [python]
        additional_dependencies:
          - mypy==1.18.2
```

Use `repo: meta` for hooks that validate your `pre-commit` setup itself:

```yaml
repos:
  - repo: meta
    hooks:
      - id: check-hooks-apply
      - id: check-useless-excludes
```

`pre-commit try-repo` is useful when testing a hook repo before publishing or before changing your main config:

```bash
pre-commit try-repo ../hook-repo my-hook-id --all-files
```

## CI, Cache, And Auth Notes

`pre-commit` itself has no API credentials, but it clones hook repositories with Git and caches hook environments on disk.

Important environment details:

- Cache directory defaults to `~/.cache/pre-commit`.
- Set `PRE_COMMIT_HOME` to move the cache.
- `XDG_CACHE_HOME` also affects the default cache location.
- Private hook repos need normal Git auth to work: SSH keys, Git credential helpers, or tokens embedded through your CI platform's Git configuration.
- If `pre-commit` runs inside wrappers such as `tox`, proxy and SSH variables may need to be passed through explicitly. The upstream docs call out `http_proxy`, `https_proxy`, `no_proxy`, and `SSH_AUTH_SOCK`.

A common CI pattern is to cache `PRE_COMMIT_HOME` between runs and execute:

```bash
pre-commit run --all-files --show-diff-on-failure
```

## Common Pitfalls

- Installing the package is not enough. You still need `pre-commit install` in each clone.
- New hooks should usually be tested with `pre-commit run --all-files`; otherwise only changed files run and broken old files stay hidden.
- Do not use mutable refs like `HEAD` in `rev`; the project intentionally rejects that pattern.
- If a hook needs a non-default Git stage, install that hook type and use explicit `stages`.
- `SKIP=hook_id pre-commit run --all-files` skips one hook temporarily; `git commit --no-verify` skips all hooks and is much easier to abuse.
- Existing Git hooks are installed in migration mode by default, so old hooks can still run unless you replace them intentionally.
- When a hook modifies files, `pre-commit` exits non-zero. Re-add the changed files and rerun.
- Hook environments are cached by config and runtime details. After interpreter upgrades or broken virtualenvs, `pre-commit clean` or `pre-commit gc` often resolves confusing failures.

## Version-Sensitive Notes For 4.5.1

- `4.5.1` fixes `repo: local` hooks that use `language: python` without `additional_dependencies`. If a local Python hook behaves differently on older installs, upgrade first.
- `4.5.0` adds `pre-commit migrate-config` and `pre-commit validate-config --allow-missing-config`, plus a `check-hooks-apply` improvement that may require `minimum_pre_commit_version: "4.5.0"` if you depend on the new behavior.
- `4.4.0` renamed the `system` and `script` language aliases to `unsupported` and `unsupported_script`. Older examples may still show the deprecated names.
- Since `3.2.0`, `stages` values should use actual Git hook names like `pre-commit`, `pre-push`, and `pre-merge-commit` instead of older aliases such as `commit` or `push`.
- `4.x` is stricter about stale patterns than many blog posts. If copied examples conflict with the current docs, prefer the current docs and release notes over third-party snippets.
