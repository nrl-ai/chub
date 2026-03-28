# Content Guide

How to author docs and skills for the Chub registry. Whether you're contributing to the public registry or building a private one for your team.

## Directory Structure

Content is organized by author, then by type (`docs` or `skills`), then by entry name:

```
my-content/
  acme/
    docs/
      widgets/
        DOC.md                    # single-language doc
        references/
          advanced.md             # additional reference file
      client/
        javascript/
          DOC.md                  # multi-language: JS variant
        python/
          DOC.md                  # multi-language: Python variant
      api/
        v1/
          DOC.md                  # multi-version: v1
        v2/
          DOC.md                  # multi-version: v2
    skills/
      deploy/
        SKILL.md                  # a skill
```

### Single-language docs

Place `DOC.md` directly in the entry directory:

```
author/docs/entry-name/DOC.md
```

### Multi-language docs

Create a subdirectory per language:

```
author/docs/entry-name/javascript/DOC.md
author/docs/entry-name/python/DOC.md
```

### Multi-version docs

When an API has breaking changes across major versions, create a subdirectory per version:

```
author/docs/entry-name/
  v1/
    DOC.md                  # versions: "1.0.0"
  v2/
    DOC.md                  # versions: "2.0.0"
```

Both `DOC.md` files must share the same `name` in frontmatter. The build groups them into a single registry entry with multiple versions. The highest version becomes the `recommendedVersion`.

You can combine multi-version with multi-language:

```
author/docs/entry-name/
  v1/
    javascript/DOC.md
    python/DOC.md
  v2/
    javascript/DOC.md
    python/DOC.md
```

### Skills

Place `SKILL.md` in the entry directory:

```
author/skills/entry-name/SKILL.md
```

### Reference files

Additional files (examples, advanced topics, error references) go alongside the entry file:

```
author/docs/widgets/
  DOC.md
  references/
    advanced.md
    errors.md
```

These are discoverable via `chub get` (shown in the footer) and fetchable with `--file` or `--full`.

## Frontmatter

Every `DOC.md` and `SKILL.md` starts with YAML frontmatter between `---` delimiters.

### DOC.md frontmatter

```yaml
---
name: widgets
description: "Acme widget API for creating and managing widgets"
metadata:
  languages: "javascript"
  versions: "2.0.0"
  revision: 1
  updated-on: "2026-01-01"
  source: maintainer
  tags: "acme,widgets,api"
---
```

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Entry name (used in the ID: `author/name`) |
| `description` | Yes | Short description for search results |
| `metadata.languages` | Yes | Language of this doc variant |
| `metadata.versions` | Yes | Package/SDK version this doc covers |
| `metadata.revision` | Yes | Content revision number (starts at 1, increment on update) |
| `metadata.updated-on` | Yes | Date this content was last revised (`YYYY-MM-DD`) |
| `metadata.source` | Yes | Trust level: `official`, `maintainer`, or `community` |
| `metadata.tags` | No | Comma-separated tags for filtering |

### SKILL.md frontmatter

```yaml
---
name: deploy
description: "Deployment automation skill for CI/CD pipelines"
metadata:
  revision: 1
  updated-on: "2026-01-01"
  source: community
  tags: "deploy,ci,automation"
---
```

Skills have the same fields as docs except `languages` and `versions` are not required (skills are language-agnostic).

## Versioning

### Package version vs API version

The `versions` field refers to the **package/SDK version** (the version on npm or pypi). This is what agents detect from `package.json` or `requirements.txt`.

If a library has a separate API versioning scheme (like Stripe's dated API versions), document that within the content body. The frontmatter `versions` stays as the package version.

### Updating content

When you improve content for the same package version (fix examples, add details, clarify wording):

1. Bump `revision` (e.g., 1 → 2)
2. Update `updated-on` to today's date
3. Keep `versions` the same

## Writing Guidelines

Content is markdown, written for LLM consumption:

- **Be direct.** Agents don't need introductions or marketing. Start with what the API does and how to use it.
- **Show code first.** A working example is worth more than a paragraph of explanation.
- **Cover the common case.** Don't exhaustively document every option. Cover what agents need 90% of the time.
- **Use reference files for depth.** Put advanced topics, error handling, and edge cases in separate reference files rather than making the main doc too long.

## Building

```sh
chub build my-content/                           # build to my-content/dist/
chub build my-content/ -o dist/                  # custom output directory
chub build my-content/ --validate-only           # validate without building
chub build my-content/ --base-url https://cdn.example.com/v1  # set CDN URL
```

The build process:
1. Discovers all `DOC.md` and `SKILL.md` files
2. Validates frontmatter (checks required fields)
3. Generates `registry.json` with entry metadata and `search-index.json` for BM25 search
4. Copies content files to the output directory

Builds are incremental by default — a SHA-256 manifest skips unchanged files.

### Validation

Run `--validate-only` to check your content without building:

```sh
chub build my-content/ --validate-only
```

## Using built content

Point your config at the build output to use it alongside the public registry:

```yaml
# ~/.chub/config.yaml
sources:
  - name: community
    url: https://cdn.aichub.org/v1
  - name: my-team
    path: /path/to/my-content/dist
```

Now `chub search` and `chub get` cover both public and your local content.

See [Self-Hosting](/guide/self-hosting) for serving your registry over HTTP or deploying to a CDN.
