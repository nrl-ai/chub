---
name: communication-sms
description: "azure-communication-sms for Python: send SMS with Azure Communication Services using connection strings or Azure Identity"
metadata:
  languages: "python"
  versions: "1.1.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,azure communication services,sms,python,messaging"
---

# azure-communication-sms Python Package Guide

## What This Package Does

`azure-communication-sms` is the Azure Communication Services SMS SDK for Python. Use it to create an `SmsClient`, send SMS to one or many recipients, and inspect one `SmsSendResult` per recipient.

The current official Microsoft Learn package overview and the current PyPI release both point to `1.1.0`, so this entry is aligned to that version.

## Install

Install the package itself:

```bash
pip install azure-communication-sms==1.1.0
```

If you want Microsoft Entra ID authentication, install `azure-identity` too:

```bash
pip install azure-communication-sms==1.1.0 azure-identity
```

PyPI currently lists `Requires: Python >=3.8` for `1.1.0`.

## Prerequisites

Before writing code, make sure you have:

- An Azure Communication Services resource
- A sender configured for that resource
- At least one destination phone number for testing
- Either a connection string or an Azure credential path for the resource

Use E.164 formatting for phone numbers such as `+14255550123`. The Microsoft quickstart also notes that `from_` can be a short code or an alphanumeric sender ID when your setup supports it.

## Configuration

The SDK does not require any specific environment variable names. These examples use a simple convention:

```bash
export ACS_CONNECTION_STRING='endpoint=https://<resource>.communication.azure.com/;accesskey=<key>'
export ACS_ENDPOINT='https://<resource>.communication.azure.com'
export ACS_SMS_FROM='+14255550123'
export ACS_SMS_TO='+14255550124'
```

If you use `DefaultAzureCredential`, set the usual Azure Identity environment variables or run under managed identity:

```bash
export AZURE_TENANT_ID='<tenant-id>'
export AZURE_CLIENT_ID='<client-id>'
export AZURE_CLIENT_SECRET='<client-secret>'
```

## Initialize The Client

### Connection String

This is the shortest path and the one used by the official SMS quickstart.

```python
import os
from azure.communication.sms import SmsClient

sms_client = SmsClient.from_connection_string(
    os.environ["ACS_CONNECTION_STRING"]
)
```

### Azure Identity

Use the endpoint constructor when your project already uses service principals, workload identity, or managed identity.

```python
import os
from azure.communication.sms import SmsClient
from azure.identity import DefaultAzureCredential

sms_client = SmsClient(
    endpoint=os.environ["ACS_ENDPOINT"],
    credential=DefaultAzureCredential(),
)
```

## Core Usage

### Send A Single Message

`send()` returns a list of `SmsSendResult`, even when you send to one recipient.

```python
import os
from azure.communication.sms import SmsClient

sms_client = SmsClient.from_connection_string(
    os.environ["ACS_CONNECTION_STRING"]
)

results = sms_client.send(
    from_=os.environ["ACS_SMS_FROM"],
    to=[os.environ["ACS_SMS_TO"]],
    message="Hello from Azure Communication Services",
    enable_delivery_report=True,
    tag="chub-sample",
)

result = results[0]
print(result.to)
print(result.successful)
print(result.message_id)
print(result.http_status_code)
print(result.error_message)
```

### Send To Multiple Recipients

```python
results = sms_client.send(
    from_=os.environ["ACS_SMS_FROM"],
    to=[
        "+14255550124",
        "+14255550125",
    ],
    message="Deployment finished successfully.",
    enable_delivery_report=True,
    tag="deploy-status",
)

for item in results:
    print(item.to, item.successful, item.http_status_code, item.error_message)
```

### Async Client

The package also exposes an async client in `azure.communication.sms.aio`.

```python
import os
from azure.communication.sms.aio import SmsClient

async def send_sms() -> None:
    client = SmsClient.from_connection_string(
        os.environ["ACS_CONNECTION_STRING"]
    )
    try:
        results = await client.send(
            from_=os.environ["ACS_SMS_FROM"],
            to=[os.environ["ACS_SMS_TO"]],
            message="Async hello",
        )
        print(results[0].successful)
    finally:
        await client.close()
```

## Response Handling

Treat every `SmsSendResult` independently:

- `to` is the recipient that this result applies to
- `successful` tells you whether that recipient was processed successfully
- `message_id` is present only when the message was processed
- `http_status_code` is the per-recipient status code
- `error_message` is set for failed or repeatable-error cases

A minimal failure check:

```python
failed = [item for item in results if not item.successful]
if failed:
    raise RuntimeError(
        "SMS send failed for: " + ", ".join(item.to for item in failed)
    )
```

## Delivery Reports And Tags

The quickstart documents two optional `send()` parameters that are easy to miss:

- `enable_delivery_report=True` requests delivery reporting on Azure Resource EventGrid
- `tag="..."` attaches metadata that is sent back with the delivery report

If your workflow needs post-send delivery status, persist the `message_id` and keep the tag stable enough to correlate downstream events.

## Common Pitfalls

### `from_` Has An Underscore

The parameter name is `from_`, not `from`, because `from` is a Python keyword.

### Keep Numbers In E.164 Format

Bad formatting is a common cause of failures. Normalize inputs before calling `send()`.

### Do Not Mix Auth Flows

- `SmsClient.from_connection_string(...)` expects the full Communication Services connection string
- `SmsClient(endpoint, credential)` expects the resource endpoint plus a credential object

Do not pass a connection string into the endpoint constructor.

### `azure-identity` Is Separate

`DefaultAzureCredential` is not bundled into `azure-communication-sms`. Install `azure-identity` explicitly if you use Entra ID auth.

### Handle Results Per Recipient

`send()` returning successfully does not mean every recipient succeeded. Always inspect the returned list.

## Version-Sensitive Notes

### Current Upstream Version Alignment

- Version used here: `1.1.0`
- Microsoft Learn package overview currently shows version `1.1.0`
- PyPI currently lists latest release `1.1.0` released on October 3, 2024

This means the earlier version reference and the current upstream package line are aligned.

### Python Version Mismatch Across Official Sources

As of March 12, 2026, the official sources still disagree slightly:

- Microsoft Learn package overview says Python `3.7` or later
- PyPI metadata says `Requires: Python >=3.8`

For new work pinned to `1.1.0`, prefer the stricter PyPI requirement and plan on Python `3.8+`.

### Practical Source Choice

The docs URL points to the package API namespace root. That page is valid, but the Microsoft Learn overview README is the more useful starting point for install, setup, and example code. Use the class reference pages when you need exact constructor and return-type details.

## Official Sources

- Microsoft Learn package overview: https://learn.microsoft.com/en-us/python/api/overview/azure/communication-sms-readme?view=azure-python
- Microsoft Learn package namespace root: https://learn.microsoft.com/en-us/python/api/azure-communication-sms/
- Microsoft Learn `SmsClient` reference: https://learn.microsoft.com/en-us/python/api/azure-communication-sms/azure.communication.sms.smsclient?view=azure-python
- Microsoft Learn `SmsSendResult` reference: https://learn.microsoft.com/en-us/python/api/azure-communication-sms/azure.communication.sms.smssendresult?view=azure-python
- Microsoft Learn SMS quickstart: https://learn.microsoft.com/en-us/azure/communication-services/quickstarts/sms/send
- PyPI project page: https://pypi.org/project/azure-communication-sms/
