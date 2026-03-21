---
name: package
description: "Copier Python package guide for rendering and updating project templates from local paths and Git repositories"
metadata:
  languages: "python"
  versions: "9.13.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "copier,python,scaffolding,templates,jinja,code-generation"
---

# Copier Python Package Guide

## Golden Rule

Use `copier` when you need Python-accessible project scaffolding with update support. Treat templates as executable code: only run trusted templates, keep the generated project's answers file under version control, and use Git tags if you expect `copier update` to work cleanly.

## Install

Use the install mode that matches how you will call Copier:

```bash
# library use inside a project or virtualenv
python -m pip install "copier==9.13.1"

# standalone CLI
pipx install copier

# standalone CLI with uv
uv tool install copier
```

Other official install paths:

```bash
conda install -c conda-forge copier
brew install copier
```

If a template depends on extra Jinja extensions, install those in the same environment as Copier. Examples from the docs:

```bash
pip install jinja2-time
pipx inject copier jinja2-time
uv tool install --with jinja2-time copier
```

## Core Concepts

Copier works with three pieces:

- Templates: the source tree with `copier.yml` and Jinja-rendered files
- Questions and answers: prompt data defined in `copier.yml` and stored in an answers file
- Projects: generated output that can later be updated from the same template

Templates can be local paths, Git URLs, or shortcuts like `gh:owner/repo.git` and `gl:owner/repo.git`.

By default, Copier copies from the latest Git tag in the template repository, sorted using PEP 440 version rules. Use `--vcs-ref` or `vcs_ref=` when you need a specific tag, branch, or commit.

## Generate A Project

CLI:

```bash
copier copy gh:your-org/your-template.git ./my-project
```

CLI with non-interactive answers:

```bash
copier copy --defaults \
  --data project_name=my-project \
  --data module_name=my_project \
  path/to/template ./my-project
```

Python API:

```python
from copier import run_copy

run_copy(
    "path/to/template-or-git-url",
    "my-project",
    data={
        "project_name": "my-project",
        "module_name": "my_project",
    },
    defaults=True,
)
```

Notes:

- If the destination path does not exist, Copier creates it.
- If the destination path already exists, it must be writable.
- `--data` values override prompt defaults. `--data-file` is CLI-only.
- `cleanup_on_error=True` is the default for `copy`; if Copier created the destination and rendering fails, it removes that directory.

## Create A Minimal Template

Minimal template layout:

```text
my_copier_template/
  copier.yml
  {{project_name}}/
    {{module_name}}.py.jinja
  {{_copier_conf.answers_file}}.jinja
```

Minimal `copier.yml`:

```yaml
project_name:
  type: str
  help: What is your project name?

module_name:
  type: str
  help: What is your Python module name?
```

Example rendered file:

```python
print("Hello from {{module_name}}!")
```

Answers file template:

```jinja
# Changes here will be overwritten by Copier
{{ _copier_answers|to_nice_yaml -}}
```

Keep the answers file in the generated project if you want updates to work. The default path is `.copier-answers.yml`, but templates can change it with `_answers_file`.

## Update Or Recopy A Generated Project

Best-case update requirements from the official docs:

1. The destination contains a valid `.copier-answers.yml` or equivalent answers file.
2. The template is versioned with Git tags.
3. The generated project is versioned with Git.

Recommended update flow:

```bash
cd my-project
git status
copier update
```

Useful update variants:

```bash
# reuse previous answers
copier update --defaults

# change one answer and keep the rest
copier update --defaults --data package_manager=uv

# re-answer questions without moving to a newer template ref
copier update --vcs-ref=:current:

# check whether a newer template version exists
copier check-update
```

Python API:

```python
from copier import run_recopy, run_update

# Smart update: keep project evolution when possible
run_update("my-project", defaults=True)

# Recopy: regenerate from the template and keep answers,
# but ignore previous project history
run_recopy("my-project", defaults=True)
```

Use `run_recopy()` only when you intentionally want a reset-style regeneration. The docs explicitly say it is not the recommended normal update path.

## Configuration And Trust

User settings live at `<CONFIG_ROOT>/settings.yml`:

- Linux: `~/.config/copier/settings.yml` in most setups
- macOS: `~/Library/Application Support/copier/settings.yml`
- Windows: `%USERPROFILE%\\AppData\\Local\\copier\\settings.yml`

You can override the location with `COPIER_SETTINGS_PATH`.

Example `settings.yml`:

```yaml
defaults:
  user_name: Jane Doe
  user_email: jane@example.com
  github_user: janedoe

trust:
  - https://github.com/your-org/your-template.git
  - https://github.com/your-org/
  - ~/templates/
```

Important behavior:

- `defaults` replace question defaults with the same name.
- `trust` entries ending in `/` are prefix matches; entries without `/` are exact matches.
- Templates that use Jinja extensions, migrations, or tasks are considered unsafe and are blocked unless you explicitly trust them.
- `--skip-tasks` only skips tasks, not migration tasks, and it does not imply `--trust`.

If you need to allow unsafe features from Python:

```python
from copier import run_copy

run_copy(
    "gh:your-org/your-template.git",
    "my-project",
    unsafe=True,
)
```

Only do this for repositories you have audited.

## Common Pitfalls

- Never edit `.copier-answers.yml` manually. The official docs call this unsupported and warn that it breaks the smart update algorithm.
- Do not assume `copier update` works well without Git tags on the template and Git history in the generated project.
- Review conflicts before committing. `--conflict inline` writes merge markers into files; `--conflict rej` writes `.rej` files.
- If your template generates one-time secrets or machine-local files, use `_skip_if_exists` so later updates do not overwrite them.
- `copier.yml` settings use underscore-prefixed names such as `_answers_file`, `_subdirectory`, `_templates_suffix`, and `_secret_questions`.
- Directories must not end with the template suffix. Files that should render normally use the suffix, which defaults to `.jinja`.
- If you apply multiple templates to one project, give each template its own answers file.
- `subdirectory` is for separating metadata from template source, not for hosting many unrelated templates in one Git repository. The docs recommend one template per repository.

## Version-Sensitive Notes For 9.13.1

- `9.13.1` fixes Git version parsing for vendor-suffixed patch versions, which matters on some packaged Git builds.
- `9.13.0` adds the `copier check-update` CLI subcommand.
- `9.12.0` introduced a smaller public settings API and explicit public `run_copy`, `run_recopy`, and `run_update` signatures. Prefer those public top-level functions over internal modules.
- `9.11.0` dropped Python 3.9 support. If your environment is still on 3.9, current Copier is not a valid target.
- `9.8.0` added the `:current:` VCS ref sentinel used by `copier update --vcs-ref=:current:`.
- `9.6.0` changed the standard Windows settings directory to `%USERPROFILE%\\AppData\\Local\\copier`; the older nested path is deprecated.
- `9.5.0` introduced user `defaults` and `trust` settings. Older pre-9.5 examples will not document them correctly.
- Templates written for Copier 5 or older may still use `.tmpl`. Current Copier defaults to `.jinja`, so older templates should set `_templates_suffix: .tmpl` explicitly if they still depend on that behavior.

## Official Sources

- Stable docs: `https://copier.readthedocs.io/en/stable/`
- Generating projects: `https://copier.readthedocs.io/en/stable/generating/`
- Updating projects: `https://copier.readthedocs.io/en/stable/updating/`
- Template configuration: `https://copier.readthedocs.io/en/stable/configuring/`
- User settings: `https://copier.readthedocs.io/en/stable/settings/`
- API reference: `https://copier.readthedocs.io/en/stable/reference/api/`
- Changelog: `https://copier.readthedocs.io/en/stable/changelog/`
- PyPI package: `https://pypi.org/project/copier/`
