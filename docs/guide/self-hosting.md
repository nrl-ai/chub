# Self-Hosting a Registry

Build and serve a private doc registry for your team's internal libraries, proprietary APIs, or any docs not in the public registry.

## When to self-host

- Internal or private libraries not in the public registry
- Docs behind a firewall or VPN
- Custom format or additional metadata
- Full control over versioning and update cadence

## Content format

Organize your docs in the standard content directory layout:

```
content/
  <author>/
    docs/<entry-name>/
      <lang>/DOC.md
      <lang>/<version>/DOC.md
    skills/<entry-name>/
      SKILL.md
```

Each `DOC.md` starts with YAML frontmatter followed by the markdown body:

```markdown
---
name: Auth Service API
description: "Internal authentication microservice — REST + token endpoints"
tags: auth, jwt, internal
version: "2.1"
---

# Auth Service API

...
```

## Building a registry

The `chub build` command compiles your content directory into a static registry:

```sh
# Build to ./dist/
chub build ./content -o ./dist

# Validate frontmatter and structure without writing output
chub build ./content --validate-only

# Set the base URL for CDN links in the output
chub build ./content -o ./dist --base-url https://docs.internal.company.com/chub
```

The build is incremental by default — a SHA-256 manifest skips unchanged files, so large registries rebuild quickly.

Output:

```
dist/
  registry.json          # doc index (all entries + search metadata)
  search-index.json      # inverted index for BM25 search
  <author>/
    docs/<entry-name>/
      <lang>/DOC.md
      <lang>/<version>/DOC.md
```

## Serving locally

For local development or testing, serve the output directory with any static file server:

```sh
# Using chub serve (built-in — takes a content dir, builds + serves)
chub serve ./content --port 4000

# Or serve the pre-built dist/ with any static server
npx serve ./dist --listen 4000
```

Your registry is then accessible at `http://localhost:4000`.

## Hosting on a static CDN

The `dist/` output is entirely static — plain JSON and markdown files. Host it on any static file server or CDN:

- **AWS S3 + CloudFront** — sync `dist/` to a bucket and enable static website hosting
- **Cloudflare R2** — zero egress cost, global distribution
- **GitHub Pages** — free for public repos; push `dist/` to a `gh-pages` branch
- **Any NGINX / Caddy server** — point the root at the `dist/` directory

No server-side logic is required. Chub clients fetch `registry.json` and individual docs directly over HTTPS.

## Connecting your team

Add the custom source to `.chub/config.yaml` (checked into your project repo):

```yaml
sources:
  - name: official
    url: https://cdn.aichub.org/v1
  - name: internal
    url: https://docs.internal.company.com/chub
```

All team members and agents who use the project's `.chub/` config will automatically have access to both the public registry and your internal registry. Search and `chub get` work transparently across all sources.

## Source disambiguation

When the same entry ID exists in multiple sources, use the `source:id` syntax to be explicit:

```sh
chub get internal:auth-service
chub get official:openai/chat
```

In MCP, agents can use the same syntax:

```json
{ "id": "internal:auth-service" }
```

## CI/CD integration

Automate registry builds on every content push. Example GitHub Actions workflow:

```yaml
name: Build and deploy registry

on:
  push:
    branches: [main]
    paths:
      - 'content/**'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install chub
        run: npm install -g @nrl-ai/chub

      - name: Build registry
        run: chub build ./content -o ./dist --base-url https://docs.internal.company.com/chub

      - name: Deploy to S3
        env:
          AWS_ACCESS_KEY_ID: ${{ secrets.AWS_ACCESS_KEY_ID }}
          AWS_SECRET_ACCESS_KEY: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
        run: aws s3 sync ./dist s3://my-chub-registry --delete

      # Or deploy to GitHub Pages:
      # - uses: peaceiris/actions-gh-pages@v4
      #   with:
      #     github_token: ${{ secrets.GITHUB_TOKEN }}
      #     publish_dir: ./dist
```

## Content doc frontmatter reference

Full list of supported frontmatter fields for `DOC.md` files:

| Field | Required | Description |
|---|---|---|
| `name` | Yes | Display name shown in search results and listings |
| `description` | Yes | Short description (1–2 sentences) for search and previews |
| `tags` | No | Comma-separated list of search tags |
| `version` | No | Doc version string (e.g. `"2.1"`, `"15.0"`) |
| `lang` | No | Language override (inferred from directory structure if omitted) |
| `source` | No | Source attribution for the content |

The `lang` and `version` are typically inferred from the directory structure (`<lang>/DOC.md` or `<lang>/<version>/DOC.md`) rather than set in frontmatter. Use frontmatter overrides only when the directory layout doesn't match.
