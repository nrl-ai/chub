---
name: communication-email
description: "Azure Communication Services Email SDK for Python with setup, authentication, send polling, attachments, and version-sensitive notes"
metadata:
  languages: "python"
  versions: "1.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,communication-services,email,python,sdk,cloud"
---

# Azure Communication Services Email SDK For Python

## Golden Rule

Use `azure-communication-email` with `from azure.communication.email import EmailClient`, send mail with `begin_send(...)`, and poll the long-running operation until it reaches a terminal status. Valid credentials are not enough by themselves: the sender address must come from a linked, verified Azure Email Communication Services domain.

## Install

```bash
python -m pip install "azure-communication-email==1.1.0"
```

If you plan to authenticate with Microsoft Entra ID, install `azure-identity` too:

```bash
python -m pip install "azure-communication-email==1.1.0" azure-identity
```

Common alternatives:

```bash
uv add "azure-communication-email==1.1.0"
poetry add "azure-communication-email==1.1.0"
```

Version note:

- PyPI metadata for `1.1.0` requires Python `>=3.9`.
- The Azure quickstart still says Python `3.7+`.
- Treat PyPI as the package-version source of truth for runtime support.

## Azure Prerequisites

Before the SDK can send successfully, Azure must already be configured:

1. Create an Azure Communication Services resource.
2. Create an Email Communication Services resource with a provisioned domain.
3. Link that domain to the Communication Services resource.
4. Use a `senderAddress` from the linked domain's `MailFrom` value.

If the domain is not linked, or the sender address does not match the linked domain, `begin_send(...)` will fail even when authentication is correct.

## Authentication And Client Setup

The SDK supports three practical auth paths:

- connection string
- Microsoft Entra ID with `DefaultAzureCredential`
- endpoint plus `AzureKeyCredential`

### Connection string

This is the simplest path for scripts and local testing. The SDK does not require a fixed env var name; these examples use `AZURE_COMMUNICATION_CONNECTION_STRING`.

```python
import os
from azure.communication.email import EmailClient

connection_string = os.environ["AZURE_COMMUNICATION_CONNECTION_STRING"]
client = EmailClient.from_connection_string(connection_string)
```

### Microsoft Entra ID

Prefer this in production when managed identity, workload identity, or a service principal is available.

```python
import os
from azure.communication.email import EmailClient
from azure.identity import DefaultAzureCredential

endpoint = os.environ["AZURE_COMMUNICATION_SERVICE_ENDPOINT"]
client = EmailClient(endpoint, DefaultAzureCredential())
```

Typical environment variables for service-principal auth:

- `AZURE_TENANT_ID`
- `AZURE_CLIENT_ID`
- `AZURE_CLIENT_SECRET`
- `AZURE_COMMUNICATION_SERVICE_ENDPOINT`

### AzureKeyCredential

```python
import os
from azure.communication.email import EmailClient
from azure.core.credentials import AzureKeyCredential

endpoint = os.environ["AZURE_COMMUNICATION_SERVICE_ENDPOINT"]
access_key = os.environ["AZURE_COMMUNICATION_SERVICE_ACCESS_KEY"]

client = EmailClient(endpoint, AzureKeyCredential(access_key))
```

## Core Usage

### Minimal send

`begin_send(...)` takes a dict-shaped message payload and returns a poller.

```python
import os
from azure.communication.email import EmailClient

connection_string = os.environ["AZURE_COMMUNICATION_CONNECTION_STRING"]
sender = os.environ["AZURE_COMMUNICATION_SENDER_ADDRESS"]

client = EmailClient.from_connection_string(connection_string)

message = {
    "senderAddress": sender,
    "recipients": {
        "to": [
            {
                "address": "recipient@example.com",
                "displayName": "Recipient",
            }
        ]
    },
    "content": {
        "subject": "Hello from Azure Communication Services",
        "plainText": "This is a text body.",
        "html": "<html><body><p>This is an HTML body.</p></body></html>",
    },
}

poller = client.begin_send(message)
result = poller.result()

print(result["id"])
print(result["status"])
```

### Poll explicitly and fail on non-success

Email sending is a long-running operation. Poll the returned operation instead of assuming the request succeeded when `begin_send(...)` returned.

```python
from azure.communication.email import EmailClient
from azure.core.exceptions import HttpResponseError

POLL_INTERVAL_SECONDS = 10
POLL_TIMEOUT_SECONDS = 180

def wait_for_send(email_client: EmailClient, message: dict) -> dict:
    poller = email_client.begin_send(message)
    elapsed = 0

    while not poller.done():
        print(f"poller status: {poller.status()}")
        poller.wait(POLL_INTERVAL_SECONDS)
        elapsed += POLL_INTERVAL_SECONDS

        if elapsed > POLL_TIMEOUT_SECONDS:
            raise TimeoutError("Timed out waiting for Azure email send operation")

    result = poller.result()
    if result["status"] != "Succeeded":
        raise RuntimeError(str(result.get("error")))

    return result

try:
    send_result = wait_for_send(client, message)
    print(f"operation id: {send_result['id']}")
except HttpResponseError as exc:
    print(exc)
    raise
```

Documented operation states include `Running`, `Succeeded`, and `Failed`.

`Succeeded` means Azure accepted the send and handed it off for delivery. It does not guarantee inbox delivery, opens, or bounce handling. For delivery telemetry, use Azure Monitor or Event Grid.

### Recipients, reply-to, and headers

At least one recipient must be present in `to`, `cc`, or `bcc`.

```python
message = {
    "senderAddress": "sender@example.com",
    "recipients": {
        "to": [
            {"address": "to1@example.com"},
            {"address": "to2@example.com"},
        ],
        "cc": [
            {"address": "cc1@example.com"},
        ],
        "bcc": [
            {"address": "bcc1@example.com"},
        ],
    },
    "content": {
        "subject": "Status update",
        "plainText": "Sent to multiple recipients.",
    },
    "replyTo": [
        {"address": "replies@example.com", "displayName": "Support"},
    ],
    "headers": {
        "x-priority": "1",
    },
}
```

### File attachments

Attachments must be base64-encoded and must declare the correct MIME type.

```python
import base64
from pathlib import Path

attachment_bytes = Path("report.pdf").read_bytes()
attachment_b64 = base64.b64encode(attachment_bytes).decode("ascii")

message = {
    "senderAddress": "sender@example.com",
    "recipients": {
        "to": [{"address": "recipient@example.com"}],
    },
    "content": {
        "subject": "Monthly report",
        "plainText": "Attached is the latest report.",
    },
    "attachments": [
        {
            "name": "report.pdf",
            "contentType": "application/pdf",
            "contentInBase64": attachment_b64,
        }
    ],
}
```

### Inline attachments

`1.1.0` adds `contentId`, which lets the HTML body reference an attachment via `cid:...`.

```python
import base64
from pathlib import Path

image_b64 = base64.b64encode(Path("logo.png").read_bytes()).decode("ascii")

message = {
    "senderAddress": "sender@example.com",
    "recipients": {
        "to": [{"address": "recipient@example.com"}],
    },
    "content": {
        "subject": "Inline image example",
        "html": '<html><body><img src="cid:company-logo" /></body></html>',
        "plainText": "See the inline image in the HTML version.",
    },
    "attachments": [
        {
            "name": "logo.png",
            "contentType": "image/png",
            "contentInBase64": image_b64,
            "contentId": "company-logo",
        }
    ],
}
```

### Caller-supplied operation IDs

`1.1.0` also adds the optional `operation_id=` keyword on `begin_send(...)`.

```python
poller = client.begin_send(message, operation_id="signup-welcome-email-0001")
result = poller.result()
```

Use this when you need a stable caller-side identifier for the long-running send request.

## Configuration Notes

- `AZURE_COMMUNICATION_CONNECTION_STRING` in these examples is a project convention, not an SDK requirement.
- `AZURE_COMMUNICATION_SERVICE_ENDPOINT` is required when using `DefaultAzureCredential` or `AzureKeyCredential`.
- Keep `senderAddress` in configuration; do not hardcode temporary example addresses into production code.
- The client exposes an `api_version` keyword argument, but the reference warns that overriding the default can lead to unsupported behavior.

## Common Pitfalls

- Use `begin_send(...)`, not `send(...)`. The PyPI troubleshooting text still shows `client.send(message)`, but the documented client API is `begin_send(...)`.
- Do not treat `poller.result()["status"] == "Succeeded"` as proof of final mailbox delivery.
- Make sure at least one of `to`, `cc`, or `bcc` is populated.
- Use the linked domain's `MailFrom` sender address, not an arbitrary email string.
- Base64-encode attachment bytes and set `contentType` correctly.
- If you use inline images, the HTML `cid:` value must match the attachment `contentId`.
- The request payload is a nested dict, not a generated model object.

## Version-Sensitive Notes For `1.1.0`

- `contentId` on attachments is available in `1.1.0` and later.
- `operation_id` on `EmailClient.begin_send(...)` is available in `1.1.0` and later.
- The quickstart's Python-version prerequisite is stale relative to the `1.1.0` PyPI metadata.

## Official Sources

- Microsoft Learn package overview: `https://learn.microsoft.com/en-us/python/api/overview/azure/communication-email-readme?view=azure-python`
- Microsoft Learn API index: `https://learn.microsoft.com/en-us/python/api/azure-communication-email/`
- Microsoft Learn `EmailClient` reference: `https://learn.microsoft.com/en-us/python/api/azure-communication-email/azure.communication.email.emailclient?view=azure-python`
- Microsoft Learn send-email quickstart: `https://learn.microsoft.com/en-us/azure/communication-services/quickstarts/email/send-email`
- PyPI package page: `https://pypi.org/project/azure-communication-email/`
