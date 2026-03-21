---
name: channels
description: "Legacy django-channels 0.7.0 guide for Django notification delivery; renamed to kawasemi and unrelated to the official Django Channels framework"
metadata:
  languages: "python"
  versions: "0.7.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django-channels,kawasemi,django,python,notifications,slack,hipchat,twitter,yo"
---

# django-channels Python Package Guide

## Golden Rule

`django-channels==0.7.0` is a legacy Django notification library. It is not the official Django Channels ASGI/WebSocket framework documented at `https://channels.readthedocs.io/en/latest/`.

Use this package only when you are maintaining an older codebase that already depends on its notification API. For new work, prefer the renamed successor `kawasemi` or another maintained notification library.

## Project Status And Upstream Drift

- Ecosystem: `pypi`
- Package: `django-channels`
- Version covered: `0.7.0`
- Release date from PyPI: `2016-10-04`
- Import surface shown on the official PyPI page: `import channels`
- Supported notification targets in the `django-channels` PyPI description: HipChat, Slack, Twitter, and Yo
- Supported runtimes listed on PyPI: Python `2.7+` and `3.3+`, Django `1.8`, `1.9`, and `1.10`

Important context:

- The docs URL, `https://channels.readthedocs.io/en/latest/`, currently points to the unrelated official Django Channels framework and on March 12, 2026 shows the `4.3.2` docs.
- The maintainers later renamed this notification library to `kawasemi` specifically to avoid that confusion.
- The maintained docs that still describe this project family now live under `https://kawasemi.readthedocs.io/`, but those docs are for the renamed package and a newer `2.0.0` documentation build.

## Install

If you truly need this legacy package, pin the exact version:

```bash
pip install "django-channels==0.7.0"
pip install -U setuptools pip
```

If you are modernizing the project instead of preserving the old dependency, install the renamed package:

```bash
pip install kawasemi
```

## Minimal Setup

The official successor docs show the same overall model used by this project family:

1. Define a notification backend configuration mapping.
2. Keep provider credentials in Django settings or environment variables.
3. Call a single send helper from application code.

Slack is the clearest backend example still documented by the maintainers:

```python
import os

CHANNELS = {
    "slack": {
        "_backend": "<legacy Slack backend class>",
        "url": os.environ["SLACK_WEBHOOK_URL"],
        "username": "deploy-bot",
        "icon_emoji": ":ghost:",
        "channel": "#ops",
    }
}
```

Notes:

- In the renamed `kawasemi` docs, the Slack backend class is `kawasemi.backends.slack.SlackChannel`.
- Because `django-channels 0.7.0` predates the rename, verify the exact legacy dotted backend path in the installed package before hardcoding it.
- Slack requires an Incoming Webhook URL. The later maintainer docs also show optional `username`, `icon_url` or `icon_emoji`, and a default target `channel`.

## Core Usage

The official `django-channels` PyPI page shows this minimal usage pattern:

```python
import channels

channels.send("Sample notification.")
```

That simple send call is the safest starting point for a legacy codebase.

If you migrate to the renamed package, the later docs show the Django helper form:

```python
from kawasemi.django import send

send("Sample notification.")
```

The successor docs also show backend-specific options for Slack, including attachments and `unfurl_links`. Treat those as migration guidance, not as a guarantee that every option name is unchanged in `0.7.0`.

## Config And Credentials

For coding agents, the practical pattern is:

1. Put provider secrets in environment variables.
2. Build the `CHANNELS` mapping in Django settings.
3. Send notifications from a narrow wrapper function so the legacy dependency is isolated.

Example wrapper:

```python
import channels

def notify_deploy_finished(environment: str, commit_sha: str) -> None:
    channels.send(f"[{environment}] deploy finished for {commit_sha}")
```

Prefer a wrapper like this instead of scattering raw `channels.send(...)` calls across the codebase. It makes later migration to `kawasemi` or another notifier much easier.

## Common Pitfalls

- Do not use `django-channels` when you actually need ASGI, WebSockets, channel layers, or consumers. That is the separate `channels` framework.
- Avoid installing legacy `django-channels` and the official `channels` framework in the same environment. They collide conceptually and can confuse imports, debugging, and dependency review because the legacy package also uses the `channels` import name.
- `0.7.0` is from 2016 and targets Django `1.8` to `1.10`. Expect breakage on modern Django and modern Python unless the project is heavily pinned.
- HipChat and Yo are effectively obsolete targets. Slack is the only backend from the old PyPI description that is still likely to be useful without replacing the provider integration yourself.
- The `kawasemi` docs are useful for successor context, but they are not a frozen `0.7.0` reference. Verify import paths and backend names against the installed package if you have to keep the legacy dependency.

## Version-Sensitive Notes

- `django-channels` `0.7.0` is the final release published under the old package name.
- The project was later renamed to `kawasemi`; the `kawasemi` PyPI page explicitly records that rename.
- The docs URL is stale for this package slot. It points to the official Django Channels framework, not this notification library.
- If you are starting greenfield work, do not choose `django-channels 0.7.0`. Use a maintained dependency and re-validate all provider integrations.

## Official URLs

- Legacy package: `https://pypi.org/project/django-channels/`
- Renamed package: `https://pypi.org/project/kawasemi/`
- Successor docs root: `https://kawasemi.readthedocs.io/`
- Successor Slack backend docs: `https://kawasemi.readthedocs.io/en/latest/backends/slack.html`
- Docs URL that is actually a different project: `https://channels.readthedocs.io/en/latest/`
