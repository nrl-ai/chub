---
name: package
description: "docutils package guide for Python: CLI tools, publisher API, config files, and version-sensitive notes"
metadata:
  languages: "python"
  versions: "0.22.4"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "docutils,restructuredtext,rst,html,xml,documentation"
---

# docutils Python Package Guide

## What It Is

`docutils` is the reference Python toolkit for parsing reStructuredText and rendering it to formats such as HTML5, XML, LaTeX, man pages, ODT, and Docutils' internal doctree structures. For code, the main programmatic entry points are in `docutils.core`. For shell usage, the main entry points are `docutils` and the specialized `rst2*` tools.

Use it when you need to:

- convert `.rst` files into HTML or other output formats
- parse reStructuredText into a doctree for custom processing
- embed reStructuredText rendering into a Python application

## Install

`docutils` 0.22.4 requires Python 3.9 or later.

```bash
pip install docutils==0.22.4
```

With `uv`:

```bash
uv add docutils==0.22.4
```

If you want optional syntax highlighting for `.. code::` blocks, install Pygments too:

```bash
pip install docutils==0.22.4 pygments
```

If you want to parse Markdown through the generic front end, install a supported third-party parser such as `myst` first. A bare `docutils` install does not include Markdown parsing support.

## Initialize And Setup

There is no auth or remote service configuration. Setup is local:

- install the package into the Python environment used by your build or app
- use the CLI tools directly, or import `docutils.core`
- optionally add persistent runtime settings in `docutils.conf`

Important CLI entry points:

- `docutils`: generic front end; default conversion is reStructuredText to HTML5
- `rst2html5`, `rst2xml`, `rst2latex`, `rst2man`, `rst2odt`, `rst2pseudoxml`, `rst2s5`, `rst2xetex`

Since Docutils 0.21, the installed `rst2*` tools are console scripts without the `.py` suffix.

Check the installed version and available options:

```bash
docutils --version
docutils --help
rst2html5 --help
```

## Core CLI Usage

Convert a reStructuredText file to HTML5:

```bash
docutils input.rst output.html
```

The generic `docutils` front end can swap parser and writer components:

```bash
docutils --writer=xml input.rst output.xml
docutils --writer=latex2e input.rst output.tex
```

Use the specialized front ends when you already know the output format:

```bash
rst2html5 input.rst output.html
rst2xml input.rst output.xml
```

Log warnings to a file:

```bash
docutils --warnings=docutils-warnings.log input.rst output.html
```

Minimal `sample.rst`:

```rst
Title
=====

This is a paragraph with **strong text**.

- item one
- item two

.. code:: python

   print("hello")
```

## Programmatic Usage

The most useful high-level functions are:

- `publish_string()`: render from an input string
- `publish_file()`: render from file-like objects
- `publish_parts()`: return structured output parts for post-processing
- `publish_doctree()`: parse into a doctree instead of a rendered format

### Render reStructuredText to HTML

`publish_string()` returns bytes by default unless you request Unicode output.

```python
from docutils.core import publish_string

rst = """\
Title
=====

This is **Docutils** output.
"""

html = publish_string(
    source=rst,
    writer_name="html5",
    settings_overrides={
        "output_encoding": "unicode",
        "file_insertion_enabled": False,
        "raw_enabled": False,
    },
)

print(type(html))  # str
print(html)
```

### Get Structured Parts Instead Of Just One Blob

Use `publish_parts()` if your app needs the full HTML document plus metadata or specific writer-generated sections.

```python
from docutils.core import publish_parts

parts = publish_parts(
    source="Heading\n=======\n\nBody text.\n",
    writer_name="html5",
    settings_overrides={"output_encoding": "unicode"},
)

full_document = parts["whole"]
docutils_version = parts["version"]
```

### Parse To A Doctree

Use `publish_doctree()` when you need to inspect or transform the parsed document structure before rendering.

```python
from docutils.core import publish_doctree

doctree = publish_doctree(
    source="Heading\n=======\n\nParagraph.\n"
)

print(doctree.pformat())
```

### File-Like IO

`publish_file()` is useful when your app already has open streams:

```python
from io import StringIO
from docutils.core import publish_file

source = StringIO("Title\n=====\n\nFrom a file-like object.\n")
destination = StringIO()

publish_file(
    source=source,
    destination=destination,
    writer_name="html5",
    settings_overrides={"output_encoding": "unicode"},
)

html = destination.getvalue()
```

## Configuration And Runtime Settings

Docutils supports persistent config files plus per-call overrides.

By default it looks for config files in this order:

1. `/etc/docutils.conf`
2. `./docutils.conf`
3. `~/.docutils`

You can override the implicit config search path with `DOCUTILSCONFIG`. You can also append one or more explicit config files with `--config`; those are processed after the implicit ones and take priority.

Example `docutils.conf`:

```ini
[general]
report_level: 2
halt_level: 4
warning_stream: docutils-warnings.log

[parsers]
file_insertion_enabled: no
raw_enabled: no

[html5 writer]
embed_stylesheet: no
stylesheet_path: minimal.css,responsive.css
```

Useful runtime settings for automation:

- `report_level`: controls which system messages are reported; default is `2` (`warning`)
- `halt_level`: converts system messages at or above the threshold into exceptions or process exit; default is `4` (`severe`)
- `warning_stream`: sends warnings to a file instead of stderr
- `file_insertion_enabled`: disable `include` and similar file-loading behavior for untrusted input
- `raw_enabled`: disable the `raw` directive for untrusted input

Programmatic calls can override the same settings via `settings_overrides`.

## Common Patterns

### Strict Build That Fails On Errors

```python
from docutils.core import publish_string

html = publish_string(
    source=source_text,
    writer_name="html5",
    settings_overrides={
        "output_encoding": "unicode",
        "report_level": 2,
        "halt_level": 3,
    },
)
```

This reports warnings and turns `error` or `severe` system messages into failures.

### Safer Rendering For Untrusted Content

```python
safe_html = publish_string(
    source=source_text,
    writer_name="html5",
    settings_overrides={
        "output_encoding": "unicode",
        "file_insertion_enabled": False,
        "raw_enabled": False,
    },
)
```

This avoids loading external files and disables raw passthrough directives.

### Generic Front End With A Markdown Parser

If you installed a third-party Markdown parser supported by Docutils:

```bash
docutils --parser=markdown --writer=pseudoxml notes.md notes.xml
```

Do not assume this works in a clean environment unless the parser dependency is installed.

## Common Pitfalls

- `publish_string()` does not guarantee a `str` result unless you set `output_encoding` to `"unicode"`.
- The generic `docutils` CLI defaults to HTML5 output. If you need XML, LaTeX, or another format, set `--writer` explicitly or use the matching `rst2*` tool.
- Since 0.21, the installed tool names are `rst2html5`, `rst2xml`, and similar, not `rst2html.py` in your `PATH`.
- `./docutils.conf` is resolved relative to the current working directory, not the source file's directory. That matters in build systems.
- Hidden config can change output. If builds behave inconsistently across environments, inspect `DOCUTILSCONFIG`, local config files, and `--config` usage first.
- `file_insertion_enabled` and `raw_enabled` default to `True`. Disable both when converting untrusted input.
- Warnings do not stop processing by default because `halt_level` defaults to `4` (`severe`). If your pipeline should fail earlier, set `halt_level` explicitly.
- Markdown parsing is optional. Install the parser package before using `--parser=markdown`.

## Version-Sensitive Notes

- The version used here `0.22.4` matches the current stable PyPI release as of 2026-03-11.
- Since 0.21, Docutils requires Python 3.9+.
- Since 0.21, `rst2*` front ends are installed as console scripts without `.py`.
- The 0.22 line starts adding type hints. Upstream notes that these hints use Python 3.10 syntax, but normal runtime use still works on supported Python versions because they are treated as annotations unless type checking is activated.
- If you are upgrading from much older automation that shell-executes `rst2html.py` or similar, update those commands.

## Upstream Sources

- PyPI package page: https://pypi.org/project/docutils/
- Docs overview: https://docutils.sourceforge.io/docs/index.html
- Front-end tools: https://docutils.sourceforge.io/docs/user/tools.html
- Configuration reference: https://docutils.sourceforge.io/docs/user/config.html
- Publisher API: https://docutils.sourceforge.io/docs/api/publisher.html
- reStructuredText primer: https://docutils.sourceforge.io/docs/user/rst/quickstart.html
- Release history: https://docutils.sourceforge.io/HISTORY.html
