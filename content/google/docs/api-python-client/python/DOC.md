---
name: api-python-client
description: "Google API Python client for discovery-based access to Google APIs with API key, OAuth 2.0, and service account authentication"
metadata:
  languages: "python"
  versions: "2.192.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-api,googleapiclient,discovery,oauth,service-account,rest"
---

# Google API Python Client

## Golden Rule

Use `google-api-python-client` when you need the generic discovery-based client for Google APIs such as Drive, Sheets, Calendar, Gmail, YouTube, or Books. Install `google-api-python-client`, import `googleapiclient`, and pair it with the modern `google-auth` stack rather than older `oauth2client` examples.

This library is complete and in maintenance mode. Expect incremental updates and new API discovery documents, not a radically changing client model. For many Google Cloud products, prefer a service-specific client library when Google publishes one.

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-api-python-client==2.192.0"
```

For current auth flows, install the companion auth packages used by the maintainers:

```bash
python -m pip install \
  "google-auth>=2" \
  "google-auth-httplib2>=0.2.0" \
  "google-auth-oauthlib>=1.2.0"
```

Common alternatives:

```bash
uv add google-api-python-client google-auth google-auth-httplib2 google-auth-oauthlib
poetry add google-api-python-client google-auth google-auth-httplib2 google-auth-oauthlib
```

## Authentication Strategy

Choose auth based on the API and the data you need:

- `developerKey=` for APIs that explicitly allow API-key access
- OAuth 2.0 user credentials for Drive, Gmail, Calendar, YouTube, and other user-scoped data
- Service account credentials for server-to-server access where the target API supports them

Do not assume every Google API supports every auth mode. Service-specific docs still decide what is allowed.

Environment variables commonly used with this package:

```bash
export GOOGLE_API_KEY="..."
export GOOGLE_APPLICATION_CREDENTIALS="/absolute/path/service-account.json"
```

## Initialize A Service

The main entry point is `googleapiclient.discovery.build()`:

```python
from googleapiclient.discovery import build

service = build(
    "drive",
    "v3",
    credentials=credentials,
)
```

Key points:

- The first argument is the discovery API name, such as `"drive"` or `"sheets"`.
- The second argument is the API version, such as `"v3"` or `"v4"`.
- Calls are synchronous and usually end with `.execute()`.
- If you use a private API or a custom discovery document URL, pass `static_discovery=False`.

## Core Usage

### API key example

Use an API key only when the target API supports it. The Books API is a simple example:

```python
import os

from googleapiclient.discovery import build
from googleapiclient.errors import HttpError

service = build("books", "v1", developerKey=os.environ["GOOGLE_API_KEY"])

try:
    response = service.volumes().list(q="python", maxResults=5).execute()
    for item in response.get("items", []):
        volume = item.get("volumeInfo", {})
        print(volume.get("title"))
except HttpError as err:
    print(err.status_code, err.error_details)
finally:
    service.close()
```

### OAuth installed-app example

Use OAuth for user data. The current maintainer docs recommend the `google-auth-oauthlib` flow:

```python
from google_auth_oauthlib.flow import InstalledAppFlow
from googleapiclient.discovery import build

SCOPES = ["https://www.googleapis.com/auth/drive.metadata.readonly"]

flow = InstalledAppFlow.from_client_secrets_file(
    "client_secret.json",
    SCOPES,
)
credentials = flow.run_local_server(port=0)

service = build("drive", "v3", credentials=credentials)

try:
    response = service.files().list(
        pageSize=10,
        fields="files(id,name,mimeType)",
    ).execute()
    for file in response.get("files", []):
        print(file["id"], file["name"], file["mimeType"])
finally:
    service.close()
```

### Service account example

Use service accounts for server-side access where the API supports them:

```python
from google.oauth2 import service_account
from googleapiclient.discovery import build

SCOPES = ["https://www.googleapis.com/auth/calendar.readonly"]

credentials = service_account.Credentials.from_service_account_file(
    "service-account.json",
    scopes=SCOPES,
)

service = build("calendar", "v3", credentials=credentials)

try:
    response = service.calendarList().list().execute()
    for item in response.get("items", []):
        print(item["id"], item.get("summary"))
finally:
    service.close()
```

If you need Google Workspace domain-wide delegation, call `credentials.with_subject("user@example.com")` after loading the service account.

### Read data from an API

Most requests follow the same pattern:

```python
service = build("sheets", "v4", credentials=credentials)

try:
    result = service.spreadsheets().values().get(
        spreadsheetId=spreadsheet_id,
        range="Sheet1!A1:C10",
    ).execute()
    rows = result.get("values", [])
finally:
    service.close()
```

### Pagination

Many list operations are paginated. Use the resource helper's `list_next()` method:

```python
files = []
request = service.files().list(
    pageSize=100,
    fields="nextPageToken, files(id,name)",
)

while request is not None:
    response = request.execute()
    files.extend(response.get("files", []))
    request = service.files().list_next(request, response)
```

### Resumable uploads

Use `MediaFileUpload` with `resumable=True` for larger uploads:

```python
from googleapiclient.http import MediaFileUpload

media = MediaFileUpload(
    "report.pdf",
    mimetype="application/pdf",
    resumable=True,
)

request = service.files().create(
    body={"name": "report.pdf"},
    media_body=media,
    fields="id,name",
)

response = None
while response is None:
    status, response = request.next_chunk()
    if status is not None:
        print(f"Uploaded {int(status.progress() * 100)}%")

print(response["id"], response["name"])
```

### Error handling

Google API request failures raise `HttpError`:

```python
from googleapiclient.errors import HttpError

try:
    response = service.files().list(pageSize=10).execute()
except HttpError as err:
    print(err.status_code)
    print(err.error_details)
```

## Configuration Notes

- `developerKey=` is for API key auth. `credentials=` is for OAuth or service-account auth.
- Scope mismatches often show up as `403` errors. Verify the exact scopes required by the target API.
- Discovery API name and version are unrelated to the package version. You still need values like `"drive"` and `"v3"`.
- Static discovery artifacts are bundled with current `2.x` releases. When you build against a custom discovery URL or private API, disable static discovery explicitly.
- `cache_discovery` is a legacy discovery-cache setting. Do not rely on older examples that expect file-cache behavior from the `oauth2client` era.

## Common Pitfalls

- The package name and import name differ: install `google-api-python-client`, import `googleapiclient`.
- `oauth2client` is deprecated. Start new code with `google-auth`, `google-auth-httplib2`, and `google-auth-oauthlib`.
- `httplib2.Http()` instances are not thread-safe. Give each thread its own HTTP instance if you customize transport.
- Not every API supports API keys, and not every API supports service accounts. Check the product-specific auth rules before choosing a credential type.
- Service accounts do not automatically see a user's Drive, Gmail, or Calendar data. You usually need explicit sharing or domain-wide delegation.
- For many Google Cloud services, the generic discovery client is not the preferred library. If a service-specific client exists, use that unless you specifically need the discovery-based REST surface.

## Version-Sensitive Notes For 2.192.0

- PyPI lists `2.192.0` as the latest release, published on March 5, 2026.
- The maintainer README states that this library is complete and in maintenance mode, so expect stability more than major new client abstractions.
- Current maintainer guidance uses the `google-auth` family for credentials. Older `oauth2client`-era snippets still exist in historical blog posts and some generated docs, but they should not be your starting point for new code.

## Official Sources

- Docs root: https://googleapis.github.io/google-api-python-client/docs/
- Getting started: https://googleapis.github.io/google-api-python-client/docs/start.html
- OAuth guide: https://googleapis.github.io/google-api-python-client/docs/oauth.html
- PyPI package: https://pypi.org/project/google-api-python-client/
- Maintainer README: https://github.com/googleapis/google-api-python-client
