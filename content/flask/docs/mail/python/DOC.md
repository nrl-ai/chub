---
name: mail
description: "Flask-Mail extension guide for SMTP email sending, attachments, testing, and bulk delivery in Flask apps"
metadata:
  languages: "python"
  versions: "0.10.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,mail,email,smtp,testing,attachments,python"
---

# Flask-Mail Python Guide

## Install

Pin the version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "Flask-Mail==0.10.0"
```

Common alternatives:

```bash
uv add "Flask-Mail==0.10.0"
poetry add "Flask-Mail==0.10.0"
```

Flask-Mail depends on SMTP access. It does not provision an email service for you. You still need credentials and server settings from your mail provider or relay.

## Initialize In A Flask App

Load config before calling `Mail(app)` or `mail.init_app(app)`.

```python
import os

from flask import Flask
from flask_mail import Mail

mail = Mail()

def create_app() -> Flask:
    app = Flask(__name__)
    app.config.from_mapping(
        MAIL_SERVER=os.environ["MAIL_SERVER"],
        MAIL_PORT=int(os.getenv("MAIL_PORT", "587")),
        MAIL_USE_TLS=os.getenv("MAIL_USE_TLS", "true").lower() == "true",
        MAIL_USE_SSL=os.getenv("MAIL_USE_SSL", "false").lower() == "true",
        MAIL_USERNAME=os.getenv("MAIL_USERNAME"),
        MAIL_PASSWORD=os.getenv("MAIL_PASSWORD"),
        MAIL_DEFAULT_SENDER=os.environ["MAIL_DEFAULT_SENDER"],
    )

    mail.init_app(app)
    return app
```

Important setup rules:

- Configure the app before `mail.init_app(app)`. Flask-Mail reads mail settings during initialization.
- With the extension-factory pattern (`mail = Mail()` then `init_app()`), mail sending uses Flask's application context. Push `app.app_context()` for CLI scripts, background jobs, or one-off tasks outside requests.
- `MAIL_DEFAULT_SENDER` avoids repeating `sender=` on every message. If you do not set it, pass `sender=` explicitly when constructing a message.

## Core Sending Pattern

Use `Message` for the envelope and body, then send it through the configured `Mail` instance.

```python
from flask import Blueprint, jsonify
from flask_mail import Message

bp = Blueprint("mail_demo", __name__)

@bp.post("/send-welcome")
def send_welcome():
    msg = Message(
        subject="Welcome",
        recipients=["user@example.com"],
        body="Thanks for signing up.",
        html="<p>Thanks for signing up.</p>",
        reply_to="support@example.com",
    )

    mail.send(msg)
    return jsonify(sent=True)
```

Common `Message(...)` arguments you will actually use:

- `subject`
- `recipients`
- `body`
- `html`
- `sender`
- `cc`
- `bcc`
- `reply_to`
- `extra_headers`

Useful details from the official docs:

- `recipients` is a list; you can also call `msg.add_recipient("other@example.com")`.
- If `sender` is a two-item tuple such as `("Support", "support@example.com")`, Flask-Mail formats it as a display name plus address.
- `mail.send_message(...)` is a convenience wrapper around `Message(...)` plus `mail.send(...)`.

## Attachments

Attach bytes from a file-like object or other in-memory content:

```python
from flask import current_app
from flask_mail import Message

def send_report() -> None:
    msg = Message(
        subject="Daily report",
        recipients=["ops@example.com"],
        body="Attached.",
    )

    with current_app.open_resource("reports/daily.csv") as fp:
        msg.attach("daily.csv", "text/csv", fp.read())

    mail.send(msg)
```

Notes:

- `msg.attach(filename, content_type, data)` is the direct API.
- In `0.10.0`, attachment content type is detected from `filename` and `data` if you do not provide one, and attachment data may not be `None`.
- Set `MAIL_ASCII_ATTACHMENTS=True` if your relay mishandles UTF-8 attachment filenames.

## Bulk Email

For many messages in one job, reuse a connection:

```python
from flask_mail import Message

def send_batch(users: list[dict[str, str]]) -> None:
    with mail.connect() as conn:
        for user in users:
            msg = Message(
                subject=f"Hello, {user['name']}",
                recipients=[user["email"]],
                body="Your account update is ready.",
            )
            conn.send(msg)
```

Set `MAIL_MAX_EMAILS` if your SMTP provider limits how many messages can be sent on one connection before reconnecting.

## Testing And Suppressed Delivery

Flask-Mail is usable in tests without sending real mail.

```python
from flask import Flask
from flask_mail import Mail

mail = Mail()

def test_mail_capture():
    app = Flask(__name__)
    app.config.update(
        TESTING=True,
        MAIL_DEFAULT_SENDER="noreply@example.com",
    )
    mail.init_app(app)

    with app.app_context():
        with mail.record_messages() as outbox:
            mail.send_message(
                subject="Testing",
                recipients=["dev@example.com"],
                body="hello",
            )

        assert len(outbox) == 1
        assert outbox[0].subject == "Testing"
```

Key behavior:

- `TESTING=True` suppresses actual sending.
- `MAIL_SUPPRESS_SEND=True` does the same outside test mode.
- `mail.record_messages()` captures the `Message` objects that would have been dispatched.
- The `email_dispatched` signal still fires even when sending is suppressed.

## Configuration And Auth

The core config keys from the official docs are:

- `MAIL_SERVER`: SMTP hostname, default `localhost`
- `MAIL_PORT`: SMTP port, default `25`
- `MAIL_USE_TLS`: enable STARTTLS
- `MAIL_USE_SSL`: enable implicit TLS
- `MAIL_DEBUG`: mail transport debug logging, defaults to `app.debug`
- `MAIL_USERNAME`
- `MAIL_PASSWORD`
- `MAIL_DEFAULT_SENDER`
- `MAIL_MAX_EMAILS`
- `MAIL_SUPPRESS_SEND`: defaults to `app.testing`
- `MAIL_ASCII_ATTACHMENTS`

Practical guidance:

- Keep SMTP credentials in environment variables or your secret manager, not in code.
- Match `MAIL_PORT`, `MAIL_USE_TLS`, and `MAIL_USE_SSL` to your provider's documented SMTP settings.
- If you are using a provider account with app passwords, API-generated SMTP passwords, or a relay username separate from your mailbox address, store exactly the provider-issued value in `MAIL_PASSWORD`.
- If your background worker or CLI command sends email outside a request, create an app and push `with app.app_context():` before calling `mail.send(...)`.

## Signals And Observability

Flask-Mail exposes an `email_dispatched` signal for logging, tests, or metrics.

```python
from flask_mail import email_dispatched

def log_message(app, message):
    app.logger.info("mail subject=%s to=%s", message.subject, message.recipients)

email_dispatched.connect(log_message)
```

In `0.10.0`, the signal behavior changed so the current Flask app is the sender and the `Message` object is passed as the `message` argument.

## Common Pitfalls

- If you initialize `Mail` before loading config, the extension will not pick up the SMTP settings you expected.
- If `TESTING=True`, mail is suppressed. This often looks like a configuration failure when you are manually checking delivery.
- If you omit both `sender=` and `MAIL_DEFAULT_SENDER`, your message setup is incomplete.
- Header injection protection is strict. Newlines in subject, sender, or recipient fields raise `BadHeaderError`.
- Flask-Mail sends through SMTP. If your provider requires a REST API instead of SMTP, use that provider's API client instead of forcing Flask-Mail into the wrong transport model.
- When using `mail = Mail()` plus `init_app()`, background jobs and scripts need an application context.

## Version-Sensitive Notes For 0.10.0

- `0.10.0` is the latest PyPI release as of March 12, 2026.
- `0.10.0` drops support for Python earlier than 3.8.
- `flask_mail.__version__` is deprecated in `0.10.0`. Use `importlib.metadata.version("flask-mail")` if you need the installed package version.
- The deprecated `is_bad_headers` helper is flagged for removal in the next version, so do not build new code around it.
- `Attachment.data` may not be `None` in `0.10.0`.
- `Attachment.content_type` is now inferred when possible from the filename and data.
- The upstream repository moved from the historical `mattupstate/flask-mail` location to `pallets-eco/flask-mail`. Use the Pallets-Eco repository and the Read the Docs site as canonical sources.
