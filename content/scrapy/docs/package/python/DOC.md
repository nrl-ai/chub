---
name: package
description: "Scrapy package guide for Python web crawling and scraping projects using the official Scrapy 2.14 docs"
metadata:
  languages: "python"
  versions: "2.14.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "scrapy,python,web-scraping,crawler,spider,twisted,asyncio"
---

# Scrapy Python Package Guide

## Golden Rule

Use `Scrapy` for structured crawlers and spiders, not ad hoc `requests` loops. Build a spider, yield `Request` and item objects, and let Scrapy manage scheduling, retries, throttling, cookies, exports, and concurrency.

As of March 12, 2026, the assigned package version for this entry is `2.14.1`, while the upstream `latest` docs root already identifies as `2.14.2`. The `2.14.2` release notes include security fixes, so prefer upgrading to `2.14.2` unless your project is explicitly pinned to `2.14.1`.

## Install

Pin the package version your project expects:

```bash
python -m pip install "Scrapy==2.14.1"
```

Common alternatives:

```bash
uv add "Scrapy==2.14.1"
poetry add "Scrapy==2.14.1"
```

Verify the installed CLI:

```bash
scrapy version -v
```

## Initialize A Project

Create a project scaffold when you want settings, middleware, pipelines, and multiple spiders:

```bash
scrapy startproject tutorial
cd tutorial
scrapy genspider quotes quotes.toscrape.com
```

Important generated files:

- `scrapy.cfg`: project entry point for `scrapy crawl ...`
- `tutorial/settings.py`: project-wide settings
- `tutorial/spiders/`: spider modules
- `tutorial/items.py`: item definitions when you want typed scraped output
- `tutorial/pipelines.py`: post-processing and persistence hooks

For a one-file spider without a full project:

```bash
scrapy runspider quotes_spider.py -O quotes.jsonl
```

## Core Usage

### Minimal spider

```python
import scrapy

class QuotesSpider(scrapy.Spider):
    name = "quotes"
    allowed_domains = ["quotes.toscrape.com"]
    start_urls = ["https://quotes.toscrape.com/"]

    def parse(self, response):
        for quote in response.css("div.quote"):
            yield {
                "text": quote.css("span.text::text").get(),
                "author": quote.css("small.author::text").get(),
                "tags": quote.css("div.tags a.tag::text").getall(),
            }

        next_page = response.css("li.next a::attr(href)").get()
        if next_page:
            yield response.follow(next_page, callback=self.parse)
```

Run it and export newline-delimited JSON:

```bash
scrapy crawl quotes -O quotes.jsonl
```

Rules that matter:

- `allowed_domains` should contain hostnames only, not schemes or paths.
- `start_urls` is for simple GET requests. If you need headers, cookies, auth, or POST bodies, override `start_requests()`.
- Use `response.follow(...)` for pagination and link traversal so Scrapy resolves relative URLs correctly.

### Requests, selectors, and item output

Scrapy callbacks receive `Response` objects. The common extraction methods are:

- `response.css("selector").get()` for one value
- `response.css("selector").getall()` for lists
- `response.xpath("//...").get()` when XPath is easier than CSS
- `response.follow(url_or_selector, callback=...)` to enqueue more work

Items can be plain dicts, `scrapy.Item`, or dataclasses you convert before yielding. Dicts are usually enough unless you need pipeline validation.

### Run from Python code

Use `CrawlerProcess` when you need to start Scrapy from a Python script instead of the CLI:

```python
from scrapy.crawler import CrawlerProcess
from scrapy.utils.project import get_project_settings

from tutorial.spiders.quotes import QuotesSpider

process = CrawlerProcess(get_project_settings())
process.crawl(QuotesSpider)
process.start()
```

This is the simplest route for embedding Scrapy in a script. If you are integrating with an already-running Twisted reactor or another event loop, read the asyncio/reactor notes before trying to start a second loop.

## Configuration And Auth

Most project configuration lives in `settings.py`. Common settings agents usually need first:

```python
BOT_NAME = "tutorial"

ROBOTSTXT_OBEY = True
USER_AGENT = "tutorial-bot/1.0 (+https://example.com/bot)"
DOWNLOAD_DELAY = 1.0
CONCURRENT_REQUESTS = 8

FEEDS = {
    "quotes.jsonl": {
        "format": "jsonlines",
        "overwrite": True,
    }
}
```

If a spider needs per-spider settings, prefer `update_settings()` over `custom_settings` when you need to merge or modify dict settings such as `FEEDS`:

```python
import scrapy

class QuotesSpider(scrapy.Spider):
    name = "quotes"

    @classmethod
    def update_settings(cls, settings):
        super().update_settings(settings)
        settings.set(
            "FEEDS",
            {
                "quotes.jsonl": {
                    "format": "jsonlines",
                    "overwrite": True,
                }
            },
            priority="spider",
        )
```

### Auth, headers, cookies, proxies

Scrapy itself does not have a package-level auth flow. Site auth is request-driven:

```python
import scrapy

class PrivateSpider(scrapy.Spider):
    name = "private"

    def start_requests(self):
        yield scrapy.Request(
            "https://example.com/api/items",
            headers={"Authorization": "Bearer TOKEN"},
            cookies={"sessionid": "cookie-value"},
            callback=self.parse_items,
        )

    def parse_items(self, response):
        yield {"status": response.status}
```

Useful built-in patterns:

- Basic HTTP auth: set `HTTPAUTH_USER` and `HTTPAUTH_PASS`
- Proxies: set `request.meta["proxy"] = "http://proxy:port"` or configure proxy middleware
- Browser-like headers: set `DEFAULT_REQUEST_HEADERS` or pass `headers=...` on each `Request`
- Robots compliance: set `ROBOTSTXT_OBEY = True` unless you intentionally have permission to bypass it

## Async And Concurrency Notes

Scrapy is built on Twisted. In modern Scrapy, callbacks can also be defined as coroutines:

```python
import scrapy

class AsyncSpider(scrapy.Spider):
    name = "async-spider"
    start_urls = ["https://quotes.toscrape.com/"]

    async def parse(self, response):
        yield {"title": response.css("title::text").get()}
```

If you need asyncio-compatible libraries inside Scrapy, use the asyncio reactor:

```python
TWISTED_REACTOR = "twisted.internet.asyncioreactor.AsyncioSelectorReactor"
```

Notes:

- Set the reactor before starting crawlers; changing it too late causes hard-to-debug runtime failures.
- Async callbacks do not remove Scrapy's concurrency controls; `CONCURRENT_REQUESTS`, delays, AutoThrottle, and downloader middleware still matter.
- Avoid blocking code inside callbacks. Blocking I/O defeats Scrapy's concurrency model.

## Common Commands

```bash
scrapy startproject tutorial
scrapy genspider quotes quotes.toscrape.com
scrapy crawl quotes
scrapy crawl quotes -O quotes.jsonl
scrapy runspider quotes_spider.py -O quotes.jsonl
scrapy shell https://quotes.toscrape.com/
scrapy list
scrapy check
```

`scrapy shell <url>` is the fastest way to debug selectors before you edit a spider.

## Common Pitfalls

- Do not put `https://` or `/path` in `allowed_domains`; keep it to hostnames like `example.com`.
- `start_urls` only issues GET requests. For login flows, POST requests, custom headers, or cookies, implement `start_requests()`.
- `scrapy crawl ... -O file.json` overwrites the target file. Use `-o` if you want append semantics where supported.
- If you override settings inside a spider, prefer `update_settings()` when you need to merge dictionaries like `FEEDS`; `custom_settings` is static and less flexible.
- Be explicit about `USER_AGENT`, delays, concurrency, and robots behavior. Default settings are rarely appropriate for production crawling.
- If you embed Scrapy in another Python process, do not try to start multiple reactors. One process gets one active Twisted reactor.
- When selectors look wrong, debug in `scrapy shell` first; many spider bugs are selector bugs, not downloader bugs.

## Version-Sensitive Notes For Scrapy 2.14.x

- An earlier version reference used `2.14.1`, but the upstream docs root on March 12, 2026 already reports `2.14.2`.
- The `2.14.2` release notes mention security fixes. If you are free to move within the same patch line, prefer `2.14.2`.
- The official docs recommend `update_settings()` for spider-level settings and support coroutine callbacks, which matters when adapting older pre-async examples from blog posts.
- When working from older tutorials, verify reactor and asyncio guidance against the current docs before mixing Scrapy with async libraries.

## Official Sources

- Docs root: https://docs.scrapy.org/en/latest/
- Tutorial and commands: https://docs.scrapy.org/en/latest/intro/tutorial.html and https://docs.scrapy.org/en/latest/intro/commands.html
- Request/response and spiders: https://docs.scrapy.org/en/latest/topics/request-response.html and https://docs.scrapy.org/en/latest/topics/spiders.html
- Settings, feed exports, practices, and asyncio: https://docs.scrapy.org/en/latest/topics/settings.html, https://docs.scrapy.org/en/latest/topics/feed-exports.html, https://docs.scrapy.org/en/latest/topics/practices.html, https://docs.scrapy.org/en/latest/topics/asyncio.html, and https://docs.scrapy.org/en/latest/topics/coroutines.html
- Release notes: https://docs.scrapy.org/en/latest/news.html
- PyPI: https://pypi.org/project/scrapy/
