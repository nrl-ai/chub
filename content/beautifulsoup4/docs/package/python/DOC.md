---
name: package
description: "Beautiful Soup 4 package guide for Python with parser selection, searching, CSS selectors, and common pitfalls"
metadata:
  languages: "python"
  versions: "4.14.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "beautifulsoup4,bs4,html,xml,parsing,scraping"
---

# Beautiful Soup 4 Python Package Guide

## What It Is

`beautifulsoup4` is the Python package for the `bs4` library. It parses HTML or XML into a navigable tree so you can search, filter, and rewrite markup with Python objects instead of string manipulation.

Golden rules:

- Install `beautifulsoup4`, but import it as `from bs4 import BeautifulSoup`.
- Always choose a parser explicitly so the parse tree is predictable across machines.
- Beautiful Soup does not fetch URLs, execute JavaScript, or handle auth. Use `requests`, `httpx`, Selenium, Playwright, or another fetch/render layer separately.

## Install

Pin the package version your project expects:

```bash
python -m pip install "beautifulsoup4==4.14.3"
```

Parser backends are separate packages. Install one when you want faster or more standards-complete parsing:

```bash
python -m pip install "lxml"
python -m pip install "html5lib"
```

Common combinations:

```bash
python -m pip install "beautifulsoup4==4.14.3" "lxml"
python -m pip install "beautifulsoup4==4.14.3" "html5lib"
```

## Initialization And Parser Choice

Beautiful Soup accepts a markup string, bytes, or a filehandle:

```python
from bs4 import BeautifulSoup

html = b"""
<html>
  <body>
    <a class="story" href="https://example.com/1">One</a>
    <a class="story" href="https://example.com/2">Two</a>
  </body>
</html>
"""

soup = BeautifulSoup(html, "html.parser")
print(soup.select_one("a")["href"])
```

Choose the parser deliberately:

- `"html.parser"`: built-in, no extra dependency, good default for basic HTML.
- `"lxml"`: usually the fastest HTML parser and a common production choice.
- `"html5lib"`: most browser-like HTML5 parsing, but slower.
- `"xml"`: XML parsing, requires `lxml`.

Parser choice matters because invalid markup can produce different trees with different parsers. If you omit the parser, Beautiful Soup picks the best installed one, which can change behavior between environments.

## Core Usage

### Find elements

`find()` returns the first match or `None`. `find_all()` returns a list-like `ResultSet`.

```python
from bs4 import BeautifulSoup

html = """
<ul id="items">
  <li class="item featured"><a href="/a">Alpha</a></li>
  <li class="item"><a href="/b">Beta</a></li>
</ul>
"""

soup = BeautifulSoup(html, "html.parser")

first_item = soup.find("li", class_="item")
all_links = soup.find_all("a")

print(first_item.get_text(strip=True))
print([link["href"] for link in all_links])
```

Useful patterns:

- Use `class_=` instead of `class=` because `class` is a Python keyword.
- Use `attrs={...}` for attribute dictionaries when the field name is not a valid Python identifier.
- Check for `None` before dereferencing the result of `find()`.

### Use CSS selectors

Beautiful Soup supports CSS selector queries:

```python
from bs4 import BeautifulSoup

html = """
<ul id="items">
  <li class="item featured"><a href="/a">Alpha</a></li>
  <li class="item"><a href="/b">Beta</a></li>
</ul>
"""

soup = BeautifulSoup(html, "html.parser")

featured = soup.select_one("li.featured > a")
all_story_links = soup.select("li.item a[href]")

print(featured["href"])
print([node.get_text(strip=True) for node in all_story_links])
```

The newer `.css` convenience property is also available in recent 4.x releases:

```python
links = soup.css.select("li.item a")
```

For widest compatibility across older `beautifulsoup4` pins, prefer `select()` and `select_one()`.

### Traverse the tree

```python
from bs4 import BeautifulSoup

soup = BeautifulSoup("<p><b>bold</b> and <i>italic</i></p>", "html.parser")
p = soup.p

print(p.contents)
print(p.b.parent.name)
print([child.name for child in p.children if getattr(child, "name", None)])
```

Useful navigation attributes and iterators:

- `.parent`
- `.contents`
- `.children`
- `.descendants`
- `.next_sibling` / `.previous_sibling`
- `.next_element` / `.previous_element`

### Extract text

```python
from bs4 import BeautifulSoup

soup = BeautifulSoup("<div>Hello <b>world</b><br/>again</div>", "html.parser")
text = soup.get_text(" ", strip=True)
print(text)
```

Use `get_text(separator, strip=True)` when you need plain text. This is usually safer than relying on `.text` in cleanup pipelines because you can control whitespace.

### Modify markup

```python
from bs4 import BeautifulSoup

soup = BeautifulSoup("<div><span>old</span></div>", "html.parser")

span = soup.find("span")
span.string = "new"

badge = soup.new_tag("strong")
badge.string = "!"
span.insert_after(badge)

print(soup)
```

Mutation helpers to know:

- `append()`
- `insert()`
- `insert_before()` / `insert_after()`
- `replace_with()`
- `extract()`
- `decompose()`
- `unwrap()`
- `clear()`

### Parse only part of a large document

Use `SoupStrainer` when you only need a subset of the document:

```python
from bs4 import BeautifulSoup, SoupStrainer

only_links = SoupStrainer("a")
soup = BeautifulSoup(html, "html.parser", parse_only=only_links)

print([a.get("href") for a in soup.find_all("a")])
```

This reduces work on large inputs, but the docs note that `parse_only` is not supported by the `html5lib` tree builder.

## Configuration Notes

### No auth or transport layer

Beautiful Soup is only the parser layer. It does not do:

- HTTP requests
- cookies or sessions
- retries or timeouts
- browser rendering
- authentication flows

Fetch content separately, then hand the response body to Beautiful Soup.

### Encoding handling

Beautiful Soup uses Unicode, Dammit to detect and convert incoming encodings to Unicode. You can inspect what it chose:

```python
from bs4 import BeautifulSoup

markup = "<h1>Sacr\xe9 bleu!</h1>".encode("latin-1")
soup = BeautifulSoup(markup, "html.parser")

print(soup.h1.string)
print(soup.original_encoding)
```

If you already know the correct encoding, pass `from_encoding=` to avoid guesswork.

### XML mode

For XML documents, use:

```python
from bs4 import BeautifulSoup

xml = "<root><item id='1'/></root>"
soup = BeautifulSoup(xml, "xml")
```

The official docs state that `lxml` is currently the only supported XML parser backend.

## Common Pitfalls

- Do not install the obsolete `BeautifulSoup` package from PyPI. Use `beautifulsoup4`.
- Do not rely on the default parser choice. Different parser availability leads to different trees for malformed markup.
- `find()` can return `None`. Guard it before using `["href"]`, `.text`, or `.get_text()`.
- Beautiful Soup parses static markup only. If the page depends on client-side JavaScript, fetch the rendered HTML with a browser automation or rendering step first.
- `class` is multi-valued in HTML. Use `class_=` or CSS selectors instead of naïve string equality assumptions.
- `prettify()` is useful for debugging, not for round-tripping production HTML.
- `html5lib` is the slowest parser. Use it when HTML5 correctness matters more than speed.
- `parse_only=SoupStrainer(...)` does not work with `html5lib`.

For malformed documents, the docs recommend `bs4.diagnose.diagnose()` as a troubleshooting tool.

## Version-Sensitive Notes

- The version used here `4.14.3` matched the live PyPI package version on March 12, 2026.
- PyPI metadata for `4.14.3` declares `Requires-Python >=3.7`.
- The official docs mention that the example code in the docs is written for Python 3.8, so syntax in the guide is not a strict lower-bound guarantee for your runtime.
- The `.css` property was added in Beautiful Soup `4.12.0`. If you are pinned below `4.12`, use `select()` and `select_one()` instead.
- Since Beautiful Soup `4.9.0`, `get_text()` skips the contents of `script`, `style`, and `template` tags for parsers that represent those contents as special string containers. Older behavior can differ if you are maintaining legacy scrapers.
