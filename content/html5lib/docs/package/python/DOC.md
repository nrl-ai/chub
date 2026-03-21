---
name: package
description: "html5lib package guide for Python HTML5 parsing, tree building, and serialization"
metadata:
  languages: "python"
  versions: "1.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "html5lib,html,parser,dom,serialization"
---

# html5lib Python Package Guide

## Golden Rule

Use `html5lib` when you need browser-style HTML5 parsing and error recovery, not strict XML parsing. Pass bytes when you want encoding sniffing to match real HTML input, and choose the tree builder explicitly if downstream code expects stdlib ElementTree, `xml.dom.minidom`, or `lxml`.

## Install

Pin the package version your project expects:

```bash
python -m pip install "html5lib==1.1"
```

Common alternatives:

```bash
uv add "html5lib==1.1"
poetry add "html5lib==1.1"
```

Optional parser tree support:

```bash
python -m pip install lxml
```

`html5lib` can build trees for:

- `etree` (default; stdlib ElementTree)
- `dom` (`xml.dom.minidom`)
- `lxml` (`lxml.etree`, if `lxml` is installed)

## Core Usage

### Parse a full HTML document

Use `html5lib.parse()` for whole documents. The return type depends on the tree builder.

```python
import html5lib

document = html5lib.parse(
    b"<html><head><title>demo</title></head><body><p>Hello",
    treebuilder="etree",
)

root = document.getroot()
print(root.tag)
```

The parser accepts:

- a string filename
- a file-like object
- a text string
- raw bytes

### Parse an HTML fragment

Use `parseFragment()` when the input is a snippet instead of a complete document. Set `container` to the element context the fragment belongs in.

```python
import html5lib

fragment = html5lib.parseFragment(
    "<tr><td>cell</td></tr>",
    container="table",
    treebuilder="dom",
)

print(fragment.childNodes[0].toxml())
```

### Choose the right tree builder

If your code already expects a particular tree API, set `treebuilder` up front instead of converting later.

```python
import html5lib

etree_doc = html5lib.parse("<p>stdlib tree</p>", treebuilder="etree")
dom_doc = html5lib.parse("<p>minidom tree</p>", treebuilder="dom")
lxml_doc = html5lib.parse("<p>lxml tree</p>", treebuilder="lxml")
```

The project README notes that the `lxml` tree builder is not supported on PyPy because of segfault risk.

### Disable XHTML namespaces when tag matching should stay simple

By default, HTML namespace handling is enabled. If your code compares plain tag names like `"div"` instead of fully qualified XHTML tags, turn it off.

```python
import html5lib

document = html5lib.parse(
    "<div><p>hello</p></div>",
    treebuilder="etree",
    namespaceHTMLElements=False,
)

root = document.getroot()
print(root.tag)  # html instead of {http://www.w3.org/1999/xhtml}html
```

### Serialize parsed content back to HTML

Use `serialize()` to emit normalized HTML from a tree walker.

```python
import html5lib
from html5lib.serializer import serialize
from html5lib.treewalkers import getTreeWalker

document = html5lib.parse("<p class=test>Hello</p>", treebuilder="etree")
walker = getTreeWalker("etree")

html = serialize(
    walker(document),
    omit_optional_tags=False,
    quote_attr_values="always",
)

print(html)
```

## Setup And Configuration Notes

`html5lib` does not use network authentication or API credentials. The important configuration choices are parser and serializer behavior:

- `treebuilder`: choose `etree`, `dom`, or `lxml` based on the object model your code consumes
- `namespaceHTMLElements`: leave enabled when you want XHTML-aware trees; disable it for simpler tag-name matching
- `container` in `parseFragment()`: set the surrounding HTML element so fragment parsing follows the right insertion rules
- serializer options such as `omit_optional_tags` and `quote_attr_values`: control how normalized output is emitted

If you are parsing content from HTTP responses, prefer `response.content` bytes over decoded text when you want `html5lib` to apply its own HTML encoding detection.

## Common Pitfalls

- The docs URL points at `https://html5lib.readthedocs.io/en/latest/`, which is currently `1.2-dev`. For released `1.1` behavior, use `https://html5lib.readthedocs.io/en/stable/`.
- The default `etree` output uses XHTML namespaces. Plain `element.tag == "div"` checks often fail unless you pass `namespaceHTMLElements=False` or handle namespace-qualified tags.
- `parseFragment()` is not interchangeable with `parse()`. Use fragments for snippets and set `container` when table/list/form insertion rules matter.
- `html5lib` repairs malformed markup. That is the point of the library, but it also means output structure and serialization may differ from the raw input.
- The optional `lxml` tree builder requires `lxml` to be installed separately.
- The deprecated `html5lib.filters.sanitizer` module should not be used for new work.

## Version-Sensitive Notes

- PyPI still lists `html5lib 1.1` as the latest released package.
- The official `latest` Read the Docs site is ahead of the release and identifies itself as `1.2-dev`, so examples there can drift from the published wheel.
- The stable changelog page still labels the `1.1` section as `UNRELEASED`, even though PyPI shows the `1.1` release was published in June 2020. Treat PyPI and the stable API docs as the authoritative release references for package-version guidance.
- PyPI metadata for `1.1` still declares an old broad `Requires-Python` range (`>=2.7`, excluding Python `3.0` through `3.4`). If you need a formally supported modern-Python compatibility statement, verify against your target interpreter in CI instead of assuming the old metadata reflects current maintainer support policy.

## Official Sources

- Stable docs: `https://html5lib.readthedocs.io/en/stable/`
- Latest docs branch: `https://html5lib.readthedocs.io/en/latest/`
- API module docs: `https://html5lib.readthedocs.io/en/latest/html5lib.html`
- Changelog: `https://html5lib.readthedocs.io/en/latest/changes.html`
- PyPI: `https://pypi.org/project/html5lib/`
- PyPI JSON metadata: `https://pypi.org/pypi/html5lib/json`
- Repository: `https://github.com/html5lib/html5lib-python`
