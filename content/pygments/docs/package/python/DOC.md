---
name: package
description: "Pygments package guide for Python syntax highlighting, lexers, formatters, and CLI usage"
metadata:
  languages: "python"
  versions: "2.19.2"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "pygments,python,syntax-highlighting,lexer,formatter,html,cli"
---

# Pygments Python Package Guide

## What It Is

`pygments` is a syntax-highlighting library for Python applications and CLIs. It splits the job into:

- **Lexer**: tokenizes source text
- **Formatter**: turns tokens into output such as HTML, terminal ANSI, SVG, RTF, or LaTeX
- **Style**: controls colors and token styling

For most agent work, use the high-level `highlight()` API with an explicit lexer and formatter. Fall back to auto-detection only when the input language is genuinely unknown.

## Install

```bash
pip install Pygments==2.19.2
```

Common variants:

```bash
pip install Pygments
pip install "Pygments[plugins]"
pip install "Pygments[windows-terminal]"
```

- `plugins` is useful when you rely on third-party lexers, formatters, styles, or filters registered through entry points.
- `windows-terminal` installs color support for Windows console output.
- PyPI lists Python `>=3.8` for `2.19.2`.

## Core API

The usual imports are:

```python
from pygments import highlight
from pygments.lexers import PythonLexer, get_lexer_by_name, get_lexer_for_filename, guess_lexer
from pygments.formatters import HtmlFormatter, TerminalFormatter
```

### Highlight code to HTML

```python
from pygments import highlight
from pygments.lexers import PythonLexer
from pygments.formatters import HtmlFormatter

code = """
def greet(name: str) -> str:
    return f"hello, {name}"
""".strip()

formatter = HtmlFormatter(full=True, style="default")
html = highlight(code, PythonLexer(), formatter)
print(html[:200])
```

Use this when you know the language. It is more reliable than guessing.

### Generate CSS for HTML output

```python
from pygments.formatters import HtmlFormatter

formatter = HtmlFormatter(cssclass="codehilite", style="friendly")
css = formatter.get_style_defs(".codehilite")
```

`HtmlFormatter` only emits token spans and CSS classes by default. If you render HTML in a site or app, include the generated CSS or pass `noclasses=True` to inline styles.

### Highlight for terminal output

```python
from pygments import highlight
from pygments.lexers import get_lexer_by_name
from pygments.formatters import TerminalFormatter

code = "SELECT * FROM users WHERE active = 1;\n"
lexer = get_lexer_by_name("sql")
colored = highlight(code, lexer, TerminalFormatter())
print(colored, end="")
```

Use terminal formatters for CLI tools, debug output, and TUI-style utilities.

## Choosing a Lexer

### Prefer explicit selection

```python
lexer = get_lexer_by_name("python")
```

This is the safest option when the calling code already knows the language.

### Select from a filename

```python
lexer = get_lexer_for_filename("report.jinja2", template_text)
```

Use this when file extension or template name carries useful signal.

### Guess from content

```python
try:
    lexer = guess_lexer(text)
except Exception:
    lexer = get_lexer_by_name("text")
```

`guess_lexer()` is convenient but not deterministic enough for security-sensitive or user-visible formatting. Keep a plain-text fallback.

### Plain text fallback

```python
from pygments.lexers.special import TextLexer

lexer = TextLexer()
```

Use this when you would rather preserve content than risk the wrong lexer.

## Working With Formatters

Common formatter choices:

- `HtmlFormatter` for docs sites, emails, web apps, and generated reports
- `TerminalFormatter` for ANSI terminal output
- `Terminal256Formatter` and `TerminalTrueColorFormatter` when your terminal supports richer color
- `LatexFormatter`, `SvgFormatter`, and `RtfFormatter` for document-generation pipelines

Most lexers and formatters accept options as keyword arguments:

```python
lexer = get_lexer_by_name("python", stripnl=False)
formatter = HtmlFormatter(linenos=True, cssclass="source", style="monokai")
```

The command-line interface exposes the same idea through `-O` and `-P` options.

## Styles

Inspect available built-in styles:

```python
from pygments.styles import get_all_styles

styles = sorted(get_all_styles())
print(styles[:10])
```

Typical style names include `default`, `friendly`, `colorful`, `monokai`, `native`, and `dracula`.

If an application lets users choose a theme, validate the style name first instead of assuming it exists.

## CLI Usage With `pygmentize`

`pygmentize` is the packaged CLI and is often the fastest way to highlight files or pipes in automation.

### Highlight a file to HTML

```bash
pygmentize -f html -O full,style=friendly -o snippet.html app.py
```

### Force a lexer and formatter

```bash
pygmentize -l python -f terminal256 script.txt
```

### Guess the lexer from input

```bash
cat query.sql | pygmentize -g
```

### List lexers, formatters, filters, and styles

```bash
pygmentize -L
```

### Print CSS for a style

```bash
pygmentize -S monokai -f html -a .highlight
```

Useful flags from the official CLI docs:

- `-l` picks a lexer explicitly
- `-f` picks a formatter explicitly
- `-O` passes a comma-separated formatter or lexer option list
- `-P` passes one option at a time
- `-F` adds a filter
- `-g` guesses the lexer
- `-L` lists available components
- `-S` prints CSS for HTML styling

## Filters and Plugins

Pygments can apply filters between lexing and formatting, and it can load third-party extensions through Python entry points.

Typical plugin groups:

- `pygments.lexers`
- `pygments.formatters`
- `pygments.styles`
- `pygments.filters`

For agent work, the practical rule is:

1. Install the plugin package.
2. Let it register entry points.
3. Refer to the new lexer, formatter, style, or filter by its advertised alias or class.

You usually do not need custom runtime bootstrap code beyond importing or selecting the component after installation.

## Configuration and Environment

Pygments does not need API keys, tokens, or service credentials.

Configuration is usually local to the call site:

- constructor kwargs on lexers and formatters
- chosen style name
- CSS generation settings for HTML output
- CLI flags for one-off commands

If your app needs stable rendering, pin:

- package version
- explicit lexer
- explicit formatter
- explicit style

That avoids output drift from auto-detection or defaults changing later.

## Common Pitfalls

### Package name vs import name

Install with `Pygments` on PyPI, import as `pygments`.

### Missing CSS in HTML output

If HTML output looks unstyled, you probably generated token spans without also shipping CSS. Use `get_style_defs()` or `pygmentize -S ... -f html`.

### Guessing the wrong language

`guess_lexer()` and `pygmentize -g` are helpful for ad hoc tools, but they can misclassify short snippets or mixed templates. Prefer `get_lexer_by_name()` whenever the caller already knows the language.

### Template and generated-file mismatches

Filename-based detection can choose a template lexer or a generic lexer depending on the extension. For Jinja, Django, and other templating inputs, test the exact filename patterns you expect in production.

### CLI option syntax

With `pygmentize`, `-O` takes a comma-separated option string, while `-P` passes options individually. Mixing them carelessly is a common source of invalid examples.

### Treating highlighted output as sanitized HTML

`HtmlFormatter` formats tokens into HTML, but that does not make arbitrary source content safe for every embedding context. Escape or sandbox user-controlled surrounding markup appropriately in your application.

## Version-Sensitive Notes For 2.19.2

- `2.19.2` is the version listed on the official PyPI release page and covered here.
- The official changelog for `2.19.2` calls out fixes around Lua lexer regressions introduced in `2.19.0`; if you saw Lua highlighting break on `2.19.0` or `2.19.1`, upgrade to `2.19.2`.
- The changelog page also shows `2.20.0` as unreleased and notes planned Python 3.8 deprecation there. Based on that official note, `2.19.2` is the safe documented target if you still support Python 3.8.

## Minimal Recipes

### HTML snippet for a web page

```python
from pygments import highlight
from pygments.lexers import get_lexer_by_name
from pygments.formatters import HtmlFormatter

code = "print('hello')\n"
lexer = get_lexer_by_name("python")
formatter = HtmlFormatter(cssclass="highlight")

html = highlight(code, lexer, formatter)
css = formatter.get_style_defs(".highlight")
```

### Terminal highlighting inside a CLI

```python
from pygments import highlight
from pygments.lexers import get_lexer_by_name
from pygments.formatters import Terminal256Formatter

def colorize(code: str, language: str) -> str:
    lexer = get_lexer_by_name(language)
    return highlight(code, lexer, Terminal256Formatter(style="native"))
```

### Safe fallback when language is unknown

```python
from pygments import highlight
from pygments.lexers import guess_lexer, get_lexer_by_name
from pygments.formatters import HtmlFormatter
from pygments.util import ClassNotFound

def to_html(code: str) -> str:
    try:
        lexer = guess_lexer(code)
    except ClassNotFound:
        lexer = get_lexer_by_name("text")
    return highlight(code, lexer, HtmlFormatter())
```

## Official Sources

- Docs root: https://pygments.org/docs/
- Quickstart: https://pygments.org/docs/quickstart/
- CLI docs: https://pygments.org/docs/cmdline/
- API docs: https://pygments.org/docs/api/
- Styles: https://pygments.org/docs/styles/
- Plugins: https://pygments.org/docs/plugins/
- Changelog: https://pygments.org/docs/changelog/
- Download and install: https://pygments.org/download/
- PyPI package: https://pypi.org/project/Pygments/
