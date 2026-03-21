---
name: package
description: "lxml Python package guide for XML and HTML parsing, XPath, XSLT, and schema validation"
metadata:
  languages: "python"
  versions: "6.0.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "lxml,python,xml,html,xpath,xslt,elementtree"
---

# lxml Python Package Guide

## Golden Rule

Use `lxml` when a Python project needs fast XML or HTML processing with XPath, XSLT, schema validation, or ElementTree-compatible APIs. Import the specific module you need, most commonly `from lxml import etree` for XML and `from lxml import html` for HTML.

## Install

Pin the version your project expects:

```bash
python -m pip install "lxml==6.0.2"
```

Common alternatives:

```bash
uv add "lxml==6.0.2"
poetry add "lxml==6.0.2"
```

Optional extras published on PyPI:

```bash
python -m pip install "lxml[cssselect]==6.0.2"
python -m pip install "lxml[html5]==6.0.2"
python -m pip install "lxml[htmlsoup]==6.0.2"
python -m pip install "lxml[source]==6.0.2"
```

Source-build notes:

- Prebuilt wheels are usually the simplest path.
- If pip falls back to a source build on Linux, install `libxml2`, `libxslt`, and Python development headers first.
- If you want a static build of the bundled C libraries, the install docs use `STATIC_DEPS=true pip install lxml`.
- The install docs note that if you also use the official `libxml2` Python bindings in the same process, a static build is the safest way to avoid interference between the two extension modules.

## Core Modules

Use the smallest surface that fits the task:

- `lxml.etree`: XML parsing, tree construction, XPath, XSLT, Relax NG, XML Schema, and serialization.
- `lxml.html`: HTML parsing helpers, document cleanup, form handling, and link rewriting.
- `lxml.objectify`: object-like access to XML documents when you want attribute-style traversal.

## Initialize And Parse XML

For most XML work, start with an explicit parser so the behavior is obvious in agent-authored code:

```python
from lxml import etree

xml_bytes = b"""
<feed xmlns="urn:example">
  <entry id="1"><title>Hello</title></entry>
  <entry id="2"><title>World</title></entry>
</feed>
"""

parser = etree.XMLParser(
    ns_clean=True,
    no_network=True,
    recover=False,
    remove_blank_text=True,
    resolve_entities=False,
)

root = etree.fromstring(xml_bytes, parser=parser)
print(root.tag)
```

Useful parser options from the API reference:

- `no_network=True` is the default and blocks fetching related files over the network.
- `huge_tree=False` is the default; turning it on disables parser security restrictions for very deep or very large inputs.
- `decompress=False` is the current default.
- `resolve_entities` is configurable; be explicit when handling untrusted XML.

Parse from a file or file-like object:

```python
from lxml import etree

tree = etree.parse("document.xml")
root = tree.getroot()
```

## XPath And Namespaces

XPath is one of the main reasons to choose `lxml` over the stdlib XML stack.

```python
from lxml import etree

root = etree.fromstring(
    b"""
    <feed xmlns="urn:example">
      <entry id="1"><title>Hello</title></entry>
      <entry id="2"><title>World</title></entry>
    </feed>
    """
)

ns = {"x": "urn:example"}
titles = root.xpath("//x:entry/x:title/text()", namespaces=ns)
print(titles)
```

For repeated queries, compile the XPath once:

```python
from lxml import etree

find_titles = etree.XPath("//x:entry/x:title/text()", namespaces={"x": "urn:example"})
titles = find_titles(root)
```

Default namespaces are a common pitfall. Map them to an explicit prefix in `namespaces=...`; XPath cannot use an empty prefix for a default namespace.

## Build And Serialize XML

```python
from lxml import etree

root = etree.Element("items")
item = etree.SubElement(root, "item", id="123")
item.text = "example"

xml_bytes = etree.tostring(
    root,
    pretty_print=True,
    xml_declaration=True,
    encoding="UTF-8",
)

print(xml_bytes.decode("utf-8"))
```

If you want a Python string instead of bytes, use `encoding="unicode"`.

## Parse HTML

Use `lxml.html` for real-world HTML instead of strict XML parsing:

```python
from lxml import html

doc = html.fromstring(
    """
    <html>
      <body>
        <a href="/a">First</a>
        <a href="/b">Second</a>
      </body>
    </html>
    """
)

links = doc.xpath("//a/@href")
print(links)
```

If you want CSS selectors, install the `cssselect` extra and use:

```python
from lxml.cssselect import CSSSelector

select_links = CSSSelector("a[href]")
nodes = select_links(doc)
```

## Stream Large XML Files

Use `iterparse()` for large documents so you do not keep the whole tree in memory:

```python
from lxml import etree

for event, elem in etree.iterparse("large.xml", events=("end",), tag="record"):
    record_id = elem.get("id")
    process_text = elem.findtext("title")
    print(record_id, process_text)

    elem.clear()
```

In long-running parses, clear processed elements promptly. Otherwise memory use grows even when you are iterating incrementally.

## Validation And XSLT

Validate against an XML Schema:

```python
from lxml import etree

schema_doc = etree.parse("schema.xsd")
schema = etree.XMLSchema(schema_doc)

doc = etree.parse("document.xml")
schema.assertValid(doc)
```

Run an XSLT transform:

```python
from lxml import etree

xml_doc = etree.parse("document.xml")
xslt_doc = etree.parse("transform.xsl")
transform = etree.XSLT(xslt_doc)

result = transform(xml_doc)
print(str(result))
```

## Configuration Notes

`lxml` does not have an auth model. The important setup variables are parser configuration and native-library packaging:

- Prefer explicit parser construction when whitespace handling, entity resolution, recovery mode, or network behavior matters.
- Keep XML parsing and HTML parsing separate. `lxml.html` is intentionally forgiving; `lxml.etree.XML()` and `etree.fromstring()` are not.
- If your environment builds from source, pin the package version and make native build dependencies part of the image or CI setup instead of relying on ad hoc compiler availability.

## Common Pitfalls

- `etree.tostring()` returns bytes unless you request `encoding="unicode"`.
- Namespace-aware XPath fails silently if you forget the namespace map.
- `recover=True` can help with malformed content, but it may also hide upstream data-quality problems. Use it intentionally.
- `huge_tree=True` relaxes parser safety limits. Do not switch it on just to "make the parse work" without understanding the input size and trust boundary.
- `iterparse()` is the right default for large XML feeds; naive `parse()` on multi-gigabyte files will consume too much memory.
- HTML tag soup should go through `lxml.html` or an HTML parser, not the strict XML parser.
- The install docs warn about mixing `lxml` with the official `libxml2` Python bindings in the same process unless `lxml` is built statically.

## Version-Sensitive Notes For 6.0.x

- PyPI lists `6.0.2` as the current package version, but the maintainer docs site and API reference pages are still branded `6.0.0`. Use PyPI and release notes to confirm patch-level behavior.
- `6.0.x` requires Python `>=3.8`.
- The `6.0.0` release notes document a default change for automatic gzip decompression: for libxml2 2.15+ builds, HTTP, FTP, and zlib support are no longer compiled in by default, so `decompress=False` is now the parser default.
- The `6.0.2` release notes call out a fix for `decompress=True` parser behavior and a fix for compilation with libxml2 2.15.0. If your project depends on explicit decompression or source builds against system libraries, patch-level upgrades matter.

## Official Sources

- Main docs: `https://lxml.de/`
- Installation: `https://lxml.de/installation.html`
- Parsing guide: `https://lxml.de/parsing.html`
- Tutorial: `https://lxml.de/tutorial.html`
- API reference: `https://lxml.de/apidoc/`
- Changelog: `https://github.com/lxml/lxml/releases`
- PyPI: `https://pypi.org/project/lxml/`
