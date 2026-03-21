---
name: package
description: "Cookiecutter Python package guide for generating projects from reusable templates"
metadata:
  languages: "python"
  versions: "2.7.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "cookiecutter,python,scaffolding,templates,cli,project-generation,jinja2"
---

# Cookiecutter Python Package Guide

## Golden Rule

Use `cookiecutter` as a scaffolding tool, not as a runtime library inside the generated app. Install it as a CLI tool when you mainly generate projects, and add it as a Python dependency only when you need `from cookiecutter.main import cookiecutter` in your own automation.

## Install

If you mainly want the CLI, prefer an isolated tool install:

```bash
uv tool install cookiecutter
# or
pipx install cookiecutter
```

If you want it in the current Python environment:

```bash
python -m pip install "cookiecutter==2.7.1"
```

If you are calling it from your own Python code, add it to the project environment:

```bash
uv add "cookiecutter==2.7.1"
# or
poetry add "cookiecutter==2.7.1"
```

Quick version check:

```bash
cookiecutter --version
```

## Core CLI Usage

Generate from a GitHub-hosted template:

```bash
uvx cookiecutter gh:audreyfeldroy/cookiecutter-pypackage
```

Generate from a local template directory:

```bash
cookiecutter ./cookiecutter-pypackage
```

Generate from a direct Git URL or a specific branch, tag, or commit:

```bash
cookiecutter https://github.com/audreyfeldroy/cookiecutter-pypackage.git
cookiecutter https://github.com/audreyfeldroy/cookiecutter-pypackage.git --checkout develop
```

Generate from a nested template inside a larger repository:

```bash
cookiecutter https://github.com/example/platform-templates.git --directory python-service
```

Write the result somewhere specific:

```bash
cookiecutter gh:audreyfeldroy/cookiecutter-pypackage --output-dir ./generated
```

Cookiecutter accepts:

- local template directories
- GitHub shorthand such as `gh:owner/repo`
- full `git+...` or `hg+...` repository URLs
- local or remote `.zip` archives
- `file://...` paths to remote file shares

## Non-Interactive Automation

Use `--no-input` when you want default values from `cookiecutter.json` and your user config without interactive prompts:

```bash
cookiecutter gh:audreyfeldroy/cookiecutter-pypackage --no-input
```

Override individual variables on the CLI by passing `key=value` pairs after the template:

```bash
cookiecutter gh:audreyfeldroy/cookiecutter-pypackage \
  --no-input \
  project_name="Acme Service" \
  project_slug="acme-service"
```

Important behavior:

- `--no-input` refreshes cached resources instead of reusing the existing clone.
- `--replay` reuses the previously saved answers for a template.
- `--replay` cannot be combined with `--no-input` or extra context.
- `--replay-file <path>` lets you point at a specific replay JSON file.

Useful overwrite controls:

```bash
cookiecutter ./template --overwrite-if-exists
cookiecutter ./template --skip-if-file-exists
cookiecutter ./template --keep-project-on-failure
```

## Programmatic Python Usage

The main library entry point is `cookiecutter.main.cookiecutter(...)`.

Basic usage:

```python
from cookiecutter.main import cookiecutter

cookiecutter("gh:audreyfeldroy/cookiecutter-pypackage")
```

Non-interactive generation with explicit values:

```python
from cookiecutter.main import cookiecutter

project_path = cookiecutter(
    "gh:audreyfeldroy/cookiecutter-pypackage",
    no_input=True,
    extra_context={
        "project_name": "Acme Service",
        "project_slug": "acme-service",
    },
    output_dir="generated",
    accept_hooks="yes",
)

print(project_path)
```

Replay an earlier generation:

```python
from cookiecutter.main import cookiecutter

cookiecutter(
    "gh:hackebrot/cookiedozer",
    replay=True,
)
```

Parameters agents commonly need:

- `checkout`: select a branch, tag, or commit after clone
- `no_input`: skip prompts and use defaults
- `extra_context`: override selected template variables
- `output_dir`: write generated files somewhere predictable
- `config_file` or `default_config`: control config loading
- `directory`: select a nested template within a repository
- `accept_hooks`: control whether hooks run
- `keep_project_on_failure`: preserve partial output for debugging

## Configuration And Environment

Cookiecutter looks for user config in `~/.cookiecutterrc` by default. This is YAML, not JSON.

Example:

```yaml
default_context:
  full_name: "Jane Developer"
  email: "jane@example.com"
  github_username: "janedev"
cookiecutters_dir: "~/.cookiecutters/"
replay_dir: "~/.cookiecutter_replay/"
abbreviations:
  gh: "https://github.com/{0}.git"
  gl: "https://gitlab.com/{0}.git"
```

Relevant config controls:

- `default_context`: default answers injected into every template run
- `cookiecutters_dir`: where cloned templates are cached
- `replay_dir`: where replay JSON files are stored
- `abbreviations`: custom template shorthands

Override config discovery when needed:

```bash
cookiecutter --config-file ./cookiecutter.yaml gh:audreyfeldroy/cookiecutter-pypackage
```

Or set it via environment:

```bash
export COOKIECUTTER_CONFIG="$PWD/cookiecutter.yaml"
```

Use `--default-config` for isolated tests and reproducible automation when you do not want any developer-specific settings to leak in.

### Auth Notes

Cookiecutter does not have its own API auth model. Authentication depends on how you fetch the template:

- Git or Mercurial credentials are handled by the underlying VCS tooling.
- SSH URLs use your existing SSH agent or key setup.
- Password-protected ZIP templates can use `COOKIECUTTER_REPO_PASSWORD` in automated environments.

## Template Authoring Essentials

A minimal template repository usually has:

```text
cookiecutter-my-template/
├── cookiecutter.json
├── hooks/
│   ├── pre_gen_project.py
│   └── post_gen_project.py
└── {{ cookiecutter.project_slug }}/
    ├── pyproject.toml
    └── README.md
```

Core rules:

- `cookiecutter.json` defines prompt variables and defaults.
- The generated project directory should usually be templated, such as `{{ cookiecutter.project_slug }}`.
- Files and paths are rendered through Jinja.
- Private variables start with `_` or `__` and are not prompted the same way as normal user-facing fields.

Useful authoring features:

- Choice variables: lists in `cookiecutter.json` create menu-style prompts.
- Dictionary variables: use structured nested config values.
- `_copy_without_render`: copy file contents verbatim while still rendering paths.
- `_extensions`: load Jinja extensions you already have installed.
- local extensions: use `local_extensions.py` in the template root for custom filters or tags.
- `hooks/`: run setup or validation before and after generation.
- `templates/`: use Jinja `extends` and `include` for shared file fragments.
- `templates` in `cookiecutter.json`: create nested template catalogs in a single repo.
- `__prompts__`: provide human-readable labels for prompts and options.

## Hooks

Cookiecutter supports three hook stages:

- `pre_prompt`: runs before prompt rendering and is available in `2.4.0+`
- `pre_gen_project`: runs after answers are collected and before rendering
- `post_gen_project`: runs after the project is generated

Prefer Python hooks for cross-platform behavior. Shell hooks can work, but they are more fragile across developer machines and CI environments.

In automation, decide explicitly whether hooks should run:

```bash
cookiecutter ./template --accept-hooks yes
cookiecutter ./template --accept-hooks ask
cookiecutter ./template --accept-hooks no
```

## Common Pitfalls

- Do not add `cookiecutter` as an application runtime dependency unless your code actually generates projects at runtime.
- The output directory cannot be the same as the template input directory.
- Existing generated directories will fail unless you use `--overwrite-if-exists` or `--skip-if-file-exists`.
- `--no-input` and `--replay` are mutually exclusive.
- `--replay` also cannot be combined with extra context on the CLI.
- Cached templates live under `~/.cookiecutters/` unless your config changes that location.
- Custom Jinja extensions are not installed automatically; template users must install those dependencies first.
- Hooks execute code from the template. Treat untrusted templates the same way you would treat untrusted bootstrap scripts.
- When generating Jinja templates from Jinja templates, use escaping or `_copy_without_render` or you will render content too early.

## Version-Sensitive Notes

- `2.7.1` is the current PyPI release as of March 12, 2026, and supports Python `3.10` through `3.14`.
- The current maintainers now show `uv`, `uvx`, and `uv tool install` prominently in the PyPI package description, but the official docs still document `pip`, `pipx`, `conda`, and Homebrew flows. All remain valid depending on your environment.
- `pre_prompt` hooks require Cookiecutter `2.4.0+`.
- Template inheritance with a `templates/` directory requires Cookiecutter `2.2+`.
- Local extensions require Cookiecutter `2.1+`.
- Nested configuration files using the `templates` key require Cookiecutter `2.5.0+`.
- If a project is pinned to an older `1.x` or early `2.x` release, do not assume these newer template authoring features exist.

## Official Sources

- Docs: `https://cookiecutter.readthedocs.io/en/stable/`
- Installation: `https://cookiecutter.readthedocs.io/en/stable/installation.html`
- Usage: `https://cookiecutter.readthedocs.io/en/stable/usage.html`
- CLI options: `https://cookiecutter.readthedocs.io/en/stable/cli_options.html`
- User config: `https://cookiecutter.readthedocs.io/en/stable/advanced/user_config.html`
- API reference: `https://cookiecutter.readthedocs.io/en/stable/cookiecutter.html`
- PyPI: `https://pypi.org/project/cookiecutter/`
