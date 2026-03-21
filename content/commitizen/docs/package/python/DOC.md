---
name: package
description: "Commitizen Python CLI for conventional commits, semantic version bumps, changelog generation, and release automation in Git repositories"
metadata:
  languages: "python"
  versions: "4.13.9"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "commitizen,python,git,conventional-commits,semver,changelog,release"
---

# Commitizen Python Package Guide

## Golden Rule

Use `commitizen` as a repo-local release tool inside a Git repository, keep exactly one active Commitizen config file, and choose the version provider that matches how the project already stores its version. `cz bump` depends on commit history and tags, so treat it as release automation rather than a generic text-replacement command.

## Install

For a project-local setup, add it as a development dependency:

```bash
python -m pip install "commitizen==4.13.9"
```

Common alternatives:

```bash
uv add --dev "commitizen==4.13.9"
poetry add --group dev "commitizen==4.13.9"
pipx install "commitizen==4.13.9"
```

`pipx` is useful when you only need the CLI globally. Prefer a project dependency when CI, hooks, and local contributors should all use the same version.

## Configuration And Setup

Commitizen looks for configuration in this order:

1. `pyproject.toml`
2. `.cz.toml`
3. `cz.toml`
4. `.cz.json`
5. `cz.json`
6. `.cz.yaml`
7. `cz.yaml`

If it finds multiple config files, it warns and uses the first valid one. For Python projects, prefer `[tool.commitizen]` inside `pyproject.toml`.

### Minimal commit-message setup

Use this when you only want `cz commit` and `cz check`:

```toml
[tool.commitizen]
name = "cz_conventional_commits"
```

### Typical release-management setup

Use this when the project version lives in `project.version`:

```toml
[project]
name = "my-package"
version = "0.1.0"

[tool.commitizen]
name = "cz_conventional_commits"
version_provider = "pep621"
tag_format = "v$version"
update_changelog_on_bump = true
version_files = [
  "src/my_package/__init__.py:__version__",
]
```

Then initialize or inspect the config:

```bash
cz init
cz info
cz schema
```

`cz init` is interactive and writes one of the supported config formats. Keep only one config file afterward.

## Choose The Right Version Provider

The provider choice controls where `cz bump` reads and writes the project version:

- `commitizen`: default provider. Store the version in Commitizen config with `version = "..."`.
- `pep621`: use `[project] version` in `pyproject.toml`.
- `uv`: use when the project is managed by `uv`; it updates the PEP 621 version and `uv.lock`.
- `poetry`: use Poetry-managed version fields.
- `scm`: derive the version from Git tags. This is read-only and does not write version files.

For Python packaging work, `pep621` or `uv` is usually the safest choice. Use `scm` only when the repository already treats Git tags as the single source of truth.

Check the resolved project version with:

```bash
cz version -p
```

## Core Workflow

### Create a commit with the configured convention

```bash
git add .
cz commit
```

`cz commit` opens an interactive prompt based on the configured adapter, usually `cz_conventional_commits`.

### Validate commit messages

Validate a literal message:

```bash
cz check --message "feat(api): add release endpoint"
```

Validate a commit range in CI:

```bash
cz check --rev-range origin/main..HEAD
```

Validate the file passed by a Git `commit-msg` hook:

```bash
cz check --commit-msg-file .git/COMMIT_EDITMSG
```

### Bump the version and create a changelog

Typical release command:

```bash
cz bump --changelog --check-consistency
```

Useful variants:

```bash
cz bump --dry-run
cz bump --files-only
cz bump --increment PATCH
cz bump --prerelease alpha
cz bump --increment-mode=exact
```

Operational notes:

- `--check-consistency` verifies configured `version_files`, but the docs warn it can still update some files before failing.
- `--files-only` updates configured files without creating a commit or tag.
- `--increment-mode=linear` and `--increment-mode=exact` behave differently for prereleases; use `exact` only when you explicitly need to preserve the requested increment.
- Commitizen uses lightweight tags by default. Enable annotated or signed tags in config if your release process requires them.

### Generate or extend the changelog separately

```bash
cz changelog
cz changelog --incremental
```

Commitizen currently generates changelogs in Markdown only.

## Hook Integration

Use hooks when you want commit validation before history lands in the repository.

### pre-commit

```yaml
repos:
  - repo: https://github.com/commitizen-tools/commitizen
    rev: v4.13.9
    hooks:
      - id: commitizen
      - id: commitizen-branch
        stages: [pre-push]
```

Then install the hooks:

```bash
pre-commit install --hook-type commit-msg --hook-type pre-push
```

The upstream auto-check tutorial still shows an older placeholder revision in one example. Pin the hook revision you actually installed instead of copying that value blindly.

### Plain Git hook

```bash
printf '%s\n' '#!/bin/sh' 'cz check --allow-abort --commit-msg-file "$1"' > .git/hooks/commit-msg
chmod +x .git/hooks/commit-msg
```

## No External Auth

Commitizen does not require API keys or service credentials. The only runtime prerequisites are:

- a Git repository
- the configured version files, if any
- Git tags and commit history when using `cz bump`, `cz changelog`, or `scm`

## Common Pitfalls

- Do not keep both `pyproject.toml` and `.cz.toml` active unless you want the first one in the search order to win.
- The default `commitizen` version provider needs an explicit `version = "..."`. Without it, version and bump commands will not behave as expected.
- `scm` reads from tags; it does not write version files. Teams often choose it and then expect `cz bump` to update `pyproject.toml`.
- `cz bump` is only as good as the commit history since the previous matching tag. Bad tags or non-conforming commit messages produce bad version increments.
- If you mirror the version into multiple files, add `--check-consistency` before tagging a release and be ready to restore modified files if the check fails mid-run.
- `cz changelog` assumes Commitizen-style commit metadata. Existing repositories with inconsistent historical messages usually need a cleanup boundary or a first release tag.
- `cz commit --signoff` is deprecated and scheduled for removal in `v5`; pass raw git arguments after `--`, for example `cz commit -- -s`.

## Version-Sensitive Notes For 4.13.9

- PyPI lists `commitizen 4.13.9` as the current maintained release on March 12, 2026.
- The PyPI metadata for `4.13.9` requires Python `>=3.10`, so older Python runtimes need an older Commitizen release line.
- The maintained docs cover multiple version providers including `pep621`, `uv`, and `scm`; older blog posts often assume only the default provider and miss `uv.lock` or PEP 621 behavior.
- The `--signoff` shortcut on `cz commit` is already documented as deprecated ahead of `v5`, so do not build new automation around it.

## Official Sources

- Documentation root: https://commitizen-tools.github.io/commitizen/
- Configuration: https://commitizen-tools.github.io/commitizen/config/
- Version providers: https://commitizen-tools.github.io/commitizen/config/#version-providers
- Commit command: https://commitizen-tools.github.io/commitizen/commands/commit/
- Check command: https://commitizen-tools.github.io/commitizen/commands/check/
- Bump command: https://commitizen-tools.github.io/commitizen/commands/bump/
- Changelog command: https://commitizen-tools.github.io/commitizen/commands/changelog/
- Auto-check hooks tutorial: https://commitizen-tools.github.io/commitizen/tutorials/auto_check/
- PyPI package: https://pypi.org/project/commitizen/
