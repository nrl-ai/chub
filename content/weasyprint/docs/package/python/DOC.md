---
name: package
description: "WeasyPrint Python package for rendering HTML and CSS documents to PDF"
metadata:
  languages: "python"
  versions: "68.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "weasyprint,python,pdf,html,css,rendering"
---

# WeasyPrint Python Package Guide

## Golden Rule

Use `weasyprint` when you need standards-oriented HTML and CSS rendered to PDF from Python. Start with `HTML(...).write_pdf(...)` for the common case, switch to `HTML(...).render(...)` when you need page inspection or post-processing, always set `base_url` when your HTML contains relative asset paths, and regression-test visual output after upgrades because the maintainers explicitly warn that rendering can change across major releases.

## Install

Pin the package version your project expects:

```bash
python -m pip install "weasyprint==68.1"
```

Common alternatives:

```bash
uv add "weasyprint==68.1"
poetry add "weasyprint==68.1"
```

WeasyPrint is not just a pure-Python install. The official installation guide requires native rendering libraries such as Pango and related dependencies.

Typical system setup from the upstream install guide:

```bash
# Debian or Ubuntu
sudo apt install python3-pip libpango-1.0-0 libpangoft2-1.0-0 libharfbuzz-subset0
python -m pip install "weasyprint==68.1"
```

```bash
# macOS
brew install python pango libffi
python -m pip install "weasyprint==68.1"
```

Windows is supported, but the maintainers recommend using the official executable unless you specifically need the Python library in your environment.

## Initialize And Render A PDF

The smallest useful example renders an HTML string directly to a PDF file:

```python
from weasyprint import HTML

HTML(string="<h1>Hello PDF</h1>").write_pdf("report.pdf")
```

If your HTML references local files or relative URLs, pass `base_url` or those assets will not resolve:

```python
from pathlib import Path

from weasyprint import HTML

project_root = Path(__file__).resolve().parent
html = """
<html>
  <body>
    <h1>Quarterly Report</h1>
    <img src="assets/logo.png" alt="Logo">
  </body>
</html>
"""

HTML(string=html, base_url=project_root).write_pdf("report.pdf")
```

Render from an existing HTML file:

```python
from weasyprint import HTML

HTML(filename="templates/invoice.html").write_pdf("invoice.pdf")
```

Render from a URL:

```python
from weasyprint import HTML

HTML(url="https://example.com/invoice/123").write_pdf("invoice.pdf")
```

## Apply CSS And Fonts

Pass stylesheet objects through the `stylesheets` argument. Use a shared `FontConfiguration` when your CSS contains `@font-face`.

```python
from pathlib import Path

from weasyprint import CSS, HTML
from weasyprint.text.fonts import FontConfiguration

root = Path(__file__).resolve().parent
font_config = FontConfiguration()

HTML(
    string="""
    <h1 class="title">Invoice</h1>
    <p class="body">Rendered with WeasyPrint.</p>
    """,
    base_url=root,
).write_pdf(
    "invoice.pdf",
    stylesheets=[CSS(filename=root / "print.css", font_config=font_config)],
    font_config=font_config,
)
```

WeasyPrint targets print output, not browser screen rendering. If your CSS has both screen and print assumptions, test the printed result explicitly.

## Use `render()` When You Need Document Metadata

`write_pdf()` is the one-shot path. `render()` returns a `Document` that exposes pages and metadata before you write output.

```python
from weasyprint import HTML

document = HTML(filename="invoice.html").render()

print(len(document.pages))

first_page = document.copy(document.pages[:1])
first_page.write_pdf("invoice-cover.pdf")

document.write_pdf("invoice-full.pdf")
```

This is the right path when you need page counts, subsets, attachments, or further processing before the final PDF is written.

## Template-Driven Usage

WeasyPrint is often paired with Jinja2 or a web framework that renders HTML first and then hands the final HTML string to WeasyPrint.

```python
from jinja2 import Environment, FileSystemLoader
from weasyprint import HTML

env = Environment(loader=FileSystemLoader("templates"))
template = env.get_template("invoice.html")

html = template.render(
    invoice_number="INV-2026-001",
    customer_name="Ada Lovelace",
    total="$125.00",
)

HTML(string=html, base_url="templates").write_pdf("invoice.pdf")
```

Make sure the `base_url` points at the directory that contains any relative CSS, images, or font files referenced by the rendered template.

## Resource Fetching, Auth, And Custom Inputs

WeasyPrint can fetch external resources while resolving images, stylesheets, fonts, and linked attachments. There is no built-in auth layer for your application, so authenticated resources usually require a custom fetcher.

In the 68.x line, the maintainers introduced the `weasyprint.URLFetcher` class and deprecated the older `default_url_fetcher` function. Use the class-based approach for new code.

Example with a bearer token for protected assets:

```python
from weasyprint import HTML
from weasyprint.urls import URLFetcher

class AuthFetcher(URLFetcher):
    def __init__(self, token: str) -> None:
        super().__init__(timeout=20)
        self.token = token

    def fetch(self, url, headers=None):
        request_headers = dict(headers or {})
        request_headers["Authorization"] = f"Bearer {self.token}"
        return super().fetch(url, headers=request_headers)

fetcher = AuthFetcher(token="secret-token")

HTML(
    string='<img src="https://internal.example.com/logo.png">',
    url_fetcher=fetcher,
).write_pdf("secured-assets.pdf")
```

Typical reasons to provide a custom fetcher:

- add HTTP headers, cookies, or bearer tokens for protected assets
- block outbound network access and only allow local or approved URLs
- rewrite application-specific URLs before WeasyPrint fetches them
- inject a shared cache or custom timeout policy

If you accept untrusted HTML or CSS, do not allow arbitrary network or filesystem access through resource URLs. The security guide calls out risks around untrusted input, infinite requests, local file access, and SVG handling.

## CLI Usage

The command-line tool is useful for debugging the same rendering stack outside your app:

```bash
weasyprint input.html output.pdf
```

This is useful when you want to distinguish "template generation is wrong" from "WeasyPrint rendering is wrong".

## Common Pitfalls

- Relative assets break silently unless you pass `base_url`.
- Missing native libraries cause import or rendering failures even when `pip install` succeeds.
- WeasyPrint logs unsupported CSS and recoverable issues; check logs instead of assuming the PDF is faithful.
- Browser layout expectations do not always match paged-media output. Test page breaks, counters, headers, and print CSS directly.
- Remote resources can make rendering slow or non-deterministic. Prefer local assets or a controlled fetcher in production.
- If you use `@font-face`, pass the same `FontConfiguration` to both `CSS(...)` and `write_pdf(...)`.
- Rendering untrusted HTML, CSS, or SVG without sandboxing is risky. Restrict network and filesystem access and cap input size and render time.

## Version-Sensitive Notes For 68.1

- PyPI currently lists `68.1`, and the stable docs are aligned with that release.
- The stable changelog notes a security fix in `68.0` for a vulnerability involving SVG `use` tags and another fix related to attachments. Treat pre-68.0 deployments as higher risk if they process untrusted content.
- `68.0` deprecated `default_url_fetcher` in favor of `weasyprint.URLFetcher`. New integrations should adopt the class-based fetcher rather than building new code on the deprecated function.
- The maintainers explicitly state that each major version can change PDF rendering, even when the public API stays mostly stable. Always regression-test generated PDFs before rolling out a major upgrade.

## Official Source URLs

- Stable docs: `https://doc.courtbouillon.org/weasyprint/stable/`
- First steps: `https://doc.courtbouillon.org/weasyprint/stable/first_steps.html`
- Installation: `https://doc.courtbouillon.org/weasyprint/stable/first_steps.html#installation`
- API reference: `https://doc.courtbouillon.org/weasyprint/stable/api_reference.html`
- Changelog: `https://doc.courtbouillon.org/weasyprint/stable/changelog.html`
- Security guide: `https://doc.courtbouillon.org/weasyprint/stable/first_steps.html#security`
- PyPI: `https://pypi.org/project/weasyprint/`
