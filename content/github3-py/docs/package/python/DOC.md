---
name: package
description: "github3.py package guide for Python - GitHub API client patterns for tokens, repositories, issues, and pagination"
metadata:
  languages: "python"
  versions: "4.0.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "github,api,rest,automation,python"
---

# github3.py Python Package Guide

## Golden Rule

Install `github3.py`, import it as `github3`, and authenticate with a token for GitHub.com work.

```bash
pip install github3.py==4.0.1
```

```python
import os
import github3

gh = github3.login(token=os.environ["GITHUB_TOKEN"])
```

The package name is `github3.py`, but the import name is `github3`.

## Installation

### pip

```bash
pip install github3.py
```

### Pin the documented version

```bash
pip install github3.py==4.0.1
```

### Poetry

```bash
poetry add github3.py@4.0.1
```

### uv

```bash
uv add github3.py==4.0.1
```

## Authentication And Setup

For GitHub.com automation, put a personal access token or GitHub App installation token in an environment variable and pass it to `github3.login`.

```bash
export GITHUB_TOKEN="ghp_your_token"
```

```python
import os
import github3

token = os.environ["GITHUB_TOKEN"]
gh = github3.login(token=token)
viewer = gh.me()
print(viewer.login)
```

### Recommended token setup

- Use a fine-grained personal access token when you only need a user-scoped script.
- Use a GitHub App installation token for automation that should avoid personal credentials.
- Scope the token to the repositories and permissions the script actually needs.

### Enterprise note

`github3.py` also documents GitHub Enterprise support. If you are targeting GitHub Enterprise Server, verify the base URL and enterprise-specific login flow in the upstream docs before writing code.

## Core Usage

### Get a repository object

```python
import os
import github3

gh = github3.login(token=os.environ["GITHUB_TOKEN"])
repo = gh.repository("owner", "repo-name")

print(repo.full_name)
print(repo.clone_url)
```

### Create an issue

```python
import os
import github3

gh = github3.login(token=os.environ["GITHUB_TOKEN"])
repo = gh.repository("owner", "repo-name")

issue = repo.create_issue(
    title="Bug report from automation",
    body="Created by github3.py"
)

print(issue.html_url)
```

### Read an existing issue

```python
import os
import github3

gh = github3.login(token=os.environ["GITHUB_TOKEN"])
issue = gh.issue("owner", "repo-name", 1)

print(issue.title)
print(issue.state)
```

### Create a gist

```python
import os
import github3

gh = github3.login(token=os.environ["GITHUB_TOKEN"])
gist = gh.create_gist(
    description="Example gist",
    files={"hello.py": {"content": "print('hello')\n"}},
    public=False,
)

print(gist.html_url)
```

## Pagination And Iteration

Many collection-returning methods are lazy iterators over paginated GitHub API responses. Iterate them directly instead of forcing everything into memory.

```python
import os
import github3

gh = github3.login(token=os.environ["GITHUB_TOKEN"])

for repo in gh.repositories():
    print(repo.full_name)
```

Use iterator limits when you only need a small sample, and keep pagination behavior in mind for orgs or users with many repositories, issues, or pull requests.

## Configuration Patterns

### Minimal env-driven helper

```python
import os
import github3

def get_github() -> github3.GitHub:
    token = os.environ["GITHUB_TOKEN"]
    return github3.login(token=token)
```

### Check the authenticated user early

```python
gh = get_github()
me = gh.me()

if me is None:
    raise RuntimeError("Authentication failed")
```

This is a useful early sanity check before mutating repositories or issues.

## Common Pitfalls

### Package name vs import name

Install `github3.py`, but write `import github3`.

### Do not rely on username/password auth for GitHub.com

Some upstream docs and examples still show `github3.login(username=..., password=...)`. The library exposes that API, but GitHub removed password authentication for the REST API on August 13, 2021. For GitHub.com, use a token instead.

### Treat iterators as paginated network calls

Objects returned from list-style methods usually fetch pages on demand. Repeated iteration can trigger repeated API traffic and rate-limit consumption.

### Expect permission-dependent behavior

Missing scopes or repository permissions commonly show up as `403` responses or empty-looking result sets. Check token permissions before debugging method names.

### Verify object methods against the right object type

`github3.py` has separate methods on the top-level `GitHub` session object and on resource objects like `Repository`, `Issue`, and `Gist`. Fetch the right object first, then call the method on that object.

## Version-Sensitive Notes

- The package version covered here is `4.0.1`, matching the PyPI release page.
- The canonical documentation root is `https://github3.readthedocs.io/en/latest/`. The source URL `https://github3py.readthedocs.io/en/latest/` appears to be stale or non-canonical.
- PyPI currently declares Python `>=3.7` for `4.0.1`.
- Upstream docs are useful but not fully aligned with modern GitHub.com auth guidance. Prefer token-based examples in new code even if older docs show password parameters.

## Official Sources

- Docs root: https://github3.readthedocs.io/en/latest/
- Quickstart/auth examples: https://github3.readthedocs.io/en/latest/examples/authentication.html
- README usage examples: https://github.com/sigmavirus24/github3.py
- PyPI package page: https://pypi.org/project/github3.py/
- GitHub authentication change notice: https://github.blog/changelog/2021-08-12-git-password-authentication-is-shutting-down/
