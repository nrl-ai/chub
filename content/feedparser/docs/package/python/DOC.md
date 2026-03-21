---
name: package
description: "feedparser Python package guide for parsing RSS and Atom feeds with practical HTTP, normalization, and version-drift notes"
metadata:
  languages: "python"
  versions: "6.0.12"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "feedparser,rss,atom,feeds,xml,http,syndication"
---

# feedparser Python Package Guide

## Golden Rule

Use `feedparser.parse(...)` for feed parsing and normalization, but take control of HTTP yourself when you need authentication, retries, timeouts, or stricter request behavior. For production code, treat parsed feeds as partially structured input: check `bozo`, prefer normalized fields like `entries`, `feed`, and `*_parsed`, and do not assume every feed exposes the same keys.

## Install

Pin the version your project expects:

```bash
python -m pip install "feedparser==6.0.12"
```

Common alternatives:

```bash
uv add "feedparser==6.0.12"
poetry add "feedparser==6.0.12"
```

`feedparser` is pure Python. You do not need an API key or service-specific auth just to parse a feed.

## Core Usage

### Parse a remote feed URL

```python
import feedparser

parsed = feedparser.parse("https://planetpython.org/rss20.xml")

if parsed.bozo:
    print(f"Feed was malformed: {parsed.bozo_exception!r}")

print(parsed.version)
print(parsed.feed.get("title"))

for entry in parsed.entries[:5]:
    print(entry.get("title"), entry.get("link"))
```

What to expect from the result:

- `parsed.feed`: feed-level metadata such as title, subtitle, and links
- `parsed.entries`: normalized list of feed entries
- `parsed.version`: detected feed type, such as RSS or Atom
- `parsed.headers`, `parsed.href`, `parsed.status`: HTTP metadata when parsing a URL
- `parsed.bozo`, `parsed.bozo_exception`: parse warning state for malformed feeds

The return value behaves like a dict with attribute access. In agent-written code, prefer `.get(...)` or presence checks because many feeds omit optional fields.

### Parse local bytes or XML you already fetched

If you already have the response body, parse that directly instead of making `feedparser` fetch the URL again:

```python
from io import BytesIO

import feedparser
import requests

url = "https://planetpython.org/rss20.xml"
response = requests.get(url, timeout=20)
response.raise_for_status()

headers = dict(response.headers)
headers.setdefault("content-location", response.url)

parsed = feedparser.parse(
    BytesIO(response.content),
    response_headers=headers,
)
```

Passing `content-location` matters when the feed contains relative links. It gives `feedparser` a base URI for link resolution.

### Parse in-memory XML safely

Do not pass untrusted raw strings directly to `feedparser.parse(...)`. The parser can interpret a bare string as a URL or filesystem path. Wrap raw XML in a file-like object:

```python
from io import BytesIO

import feedparser

xml_bytes = b"""<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Example</title>
    <item><title>Hello</title></item>
  </channel>
</rss>
"""

parsed = feedparser.parse(BytesIO(xml_bytes))
print(parsed.feed.get("title"))
```

## HTTP, Caching, And Auth

### Conditional requests with ETag and Last-Modified

Persist `etag` and `modified` between runs to avoid re-downloading unchanged feeds:

```python
import feedparser

feed_url = "https://planetpython.org/rss20.xml"
stored_etag = 'W/"abc123"'
stored_modified = "Wed, 05 Mar 2025 10:00:00 GMT"

parsed = feedparser.parse(
    feed_url,
    etag=stored_etag,
    modified=stored_modified,
)

if parsed.status == 304:
    print("Feed not modified")
else:
    print(parsed.get("etag"))
    print(parsed.get("modified"))
```

### Set a custom user agent or request headers

```python
import feedparser

parsed = feedparser.parse(
    "https://example.com/feed.xml",
    agent="MyFeedBot/1.0 (+https://example.com/bot)",
    request_headers={
        "Accept": "application/atom+xml, application/rss+xml, application/xml;q=0.9, */*;q=0.1",
    },
)
```

Use `request_headers`, not `extra_headers`. Some official doc pages still mention `extra_headers`, but the `v6.0.12` parser signature uses `request_headers`.

### Prefer explicit fetching for auth, proxies, retries, and timeouts

The official docs still show older `urllib2`-style auth examples. In modern Python, it is usually simpler to fetch the feed with your HTTP client of choice, then pass the body plus response headers to `feedparser`:

```python
from io import BytesIO

import feedparser
import requests

response = requests.get(
    "https://example.com/private-feed.xml",
    auth=("username", "password"),
    headers={"User-Agent": "MyFeedBot/1.0"},
    timeout=20,
)
response.raise_for_status()

headers = dict(response.headers)
headers.setdefault("content-location", response.url)

parsed = feedparser.parse(BytesIO(response.content), response_headers=headers)
```

This pattern gives you full control over auth, redirects, TLS, retries, proxy config, and observability without depending on the parser's built-in URL fetching path.

## Normalized Fields Agents Usually Need

Common entry fields:

- `entry.title`
- `entry.link`
- `entry.summary`
- `entry.content`
- `entry.author`
- `entry.tags`
- `entry.enclosures`
- `entry.published`, `entry.updated`
- `entry.published_parsed`, `entry.updated_parsed`

Prefer `*_parsed` over raw date strings:

```python
import calendar
from datetime import datetime, timezone

published = entry.get("published_parsed")
if published:
    published_at = datetime.fromtimestamp(calendar.timegm(published), tz=timezone.utc)
```

`feedparser` normalizes many date formats into UTC `time.struct_time` values. That is much more reliable than trying to parse publisher-specific date strings yourself.

## HTML And Link Handling

By default, `feedparser` resolves relative URIs in HTML content and sanitizes embedded markup in common text fields. You can disable either behavior per call:

```python
parsed = feedparser.parse(
    "https://example.com/feed.xml",
    resolve_relative_uris=False,
    sanitize_html=False,
)
```

Use this carefully:

- `sanitize_html=False` is only safe if you will escape or otherwise sanitize output before rendering it into HTML.
- `resolve_relative_uris=False` only affects relevant embedded HTML/markup handling. Do not assume every link-like field is rewritten the same way.

## Common Pitfalls

- `bozo == 1` is not always fatal. Many real feeds are slightly malformed but still yield usable `entries`.
- Do not assume keys exist. Use `.get(...)` for optional fields like `summary`, `author`, `tags`, and `published_parsed`.
- If you need deterministic HTTP behavior, fetch the feed yourself and pass `response_headers` into `feedparser.parse(...)`.
- Wrap untrusted XML strings in `BytesIO` or `StringIO`; a bare string can be treated as a URL or file path.
- Some official examples are still written against old Python 2 modules such as `urllib2`. Translate them to `urllib.request` or use `requests`.
- If relative links matter and you are parsing bytes rather than a URL, supply `content-location` in `response_headers`.
- Use `*_parsed` fields for dates. Raw `published` and `updated` strings vary widely across publishers.

## Version-Sensitive Notes For 6.0.12

- PyPI currently lists `feedparser 6.0.12`, released on September 10, 2025.
- The official changelog for `6.0.12` notes fixes for an `AssertionError` on Python 3.10+ and a `DeprecationWarning` from `re.sub`, plus Read the Docs configuration updates.
- The docs URL points to `https://feedparser.readthedocs.io/en/latest/`, but the docs site currently has version drift: `latest/` pages still identify themselves as `6.0.11`, while `en/releases/` pages and the changelog include `6.0.12`.
- For code generation, prefer `request_headers` and `response_headers` from the `v6.0.12` source tag over any stale doc page that still says `extra_headers`.
