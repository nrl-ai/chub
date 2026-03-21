---
name: package
description: "Official itsdangerous package guide for Python token signing, URL-safe payloads, and expiring signatures"
metadata:
  languages: "python"
  versions: "2.2.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "itsdangerous,pallets,python,signing,tokens,security"
---

# itsdangerous Python Package Guide

## What It Is

`itsdangerous` signs data so you can send it through untrusted places such as URLs, cookies, or form fields and later verify that it was not modified.

Use it for integrity and optional expiration. Do not use it for confidentiality: the payload is readable by the client unless you encrypt it separately.

## Install

```bash
pip install itsdangerous==2.2.0
```

- Package: `itsdangerous`
- Version covered: `2.2.0`
- Python requirement: `>=3.8`
- Docs family: `2.2.x`

## Choose The Right API

| Need | API |
| --- | --- |
| Sign raw strings or bytes | `Signer` |
| Sign structured Python data | `Serializer` |
| Put signed data in URLs or cookies | `URLSafeSerializer` |
| Add expiration to structured data | `URLSafeTimedSerializer` |
| Add expiration to raw strings or bytes | `TimestampSigner` |

In most application code, start with `URLSafeSerializer` or `URLSafeTimedSerializer`.

## Setup

Keep the secret key outside source control and use a unique salt per token purpose.

```python
import os

from itsdangerous.url_safe import URLSafeTimedSerializer

SECRET_KEY = os.environ["ITSDANGEROUS_SECRET_KEY"]

email_tokens = URLSafeTimedSerializer(
    secret_key=SECRET_KEY,
    salt="email-confirm-v1",
)
```

Generate a secret once and store it in your environment or secret manager:

```bash
python3 -c 'import os; print(os.urandom(32).hex())'
```

## Core Usage

### Sign And Verify Structured Data

```python
from itsdangerous.exc import BadSignature
from itsdangerous.url_safe import URLSafeSerializer

serializer = URLSafeSerializer(
    secret_key="replace-me-from-env",
    salt="invite-link-v1",
)

token = serializer.dumps(
    {
        "user_id": 123,
        "role": "viewer",
    }
)

print(token)

try:
    payload = serializer.loads(token)
    assert payload["user_id"] == 123
except BadSignature:
    payload = None
```

Notes:

- `dumps()` returns a signed string.
- `loads()` verifies the signature before returning the payload.
- The default serializer is JSON, so prefer JSON-compatible payloads.

### URL-Safe Tokens With Expiration

Use timed serializers for password reset links, email verification links, and one-click actions.

```python
from typing import Optional

from itsdangerous.exc import BadSignature, SignatureExpired
from itsdangerous.url_safe import URLSafeTimedSerializer

serializer = URLSafeTimedSerializer(
    secret_key="replace-me-from-env",
    salt="password-reset-v1",
)

token = serializer.dumps({"user_id": 123})

def load_reset_token(token: str) -> Optional[int]:
    try:
        data = serializer.loads(token, max_age=3600)
    except SignatureExpired:
        return None
    except BadSignature:
        return None
    else:
        return int(data["user_id"])
```

Notes:

- `max_age` is in seconds.
- Expired tokens raise `SignatureExpired`.
- Other tampering or decode problems raise `BadSignature` or a subclass.

### Raw Signing Without Serialization

Use `Signer` when you already have the exact bytes or string you want to protect.

```python
from itsdangerous import Signer
from itsdangerous.exc import BadSignature

signer = Signer("replace-me-from-env", salt="download-token-v1")
signed_value = signer.sign("user:123")

try:
    original = signer.unsign(signed_value).decode()
except BadSignature:
    original = None
```

Use `TimestampSigner` instead when the raw value should expire.

## Config And Security Patterns

### Secret Key

- Use a long random secret.
- Load it from environment variables or a secret manager.
- Changing the secret invalidates existing tokens.
- If the secret leaks, rotate it immediately and invalidate old tokens.

### Salt Separation

Use a different salt for each token purpose. Do not reuse one salt for unrelated actions.

Good examples:

- `email-confirm-v1`
- `password-reset-v1`
- `unsubscribe-link-v1`

If you sign the same payload for different actions with the same salt, a token issued for one action may be replayed in another context.

### Key Rotation

`itsdangerous` supports key rotation by accepting a list of keys, oldest to newest. The newest key signs new tokens, and all keys are tried for verification.

```python
from itsdangerous.url_safe import URLSafeTimedSerializer

SECRET_KEYS = [
    "oldest-key",
    "current-key",
    "newest-key",
]

serializer = URLSafeTimedSerializer(
    secret_key=SECRET_KEYS,
    salt="email-confirm-v1",
)
```

This is the safest way to rotate secrets without invalidating every in-flight token immediately.

### Upgrading Digest Settings

If you need to change signer parameters such as the digest method, use `fallback_signers` so older tokens still validate during rollout.

```python
import hashlib

from itsdangerous.serializer import Serializer

serializer = Serializer(
    secret_key="replace-me-from-env",
    signer_kwargs={"digest_method": hashlib.sha512},
    fallback_signers=[{"digest_method": hashlib.sha1}],
)
```

This is for compatibility rollouts, not for secrecy.

## Error Handling

Common exceptions:

- `BadSignature`: signature does not validate.
- `BadTimeSignature`: time-based signature is invalid.
- `SignatureExpired`: token is validly signed but older than `max_age`.

Pattern:

```python
from itsdangerous.exc import BadSignature, SignatureExpired

try:
    data = serializer.loads(token, max_age=3600)
except SignatureExpired:
    # Token was issued by you, but it is too old.
    ...
except BadSignature:
    # Token was tampered with or is otherwise invalid.
    ...
```

Avoid `loads_unsafe()` in production request handling. It skips signature verification and is only appropriate for tightly controlled debugging flows.

## Common Pitfalls

### Treating Signed Data As Encrypted

`itsdangerous` signs data; it does not hide the data. Do not put secrets, API keys, or private user information in a token unless you encrypt first.

### Reusing One Salt Everywhere

Salt is meant to separate trust contexts. Reusing a generic salt such as `"default"` across many token types weakens that separation and makes cross-purpose replay easier.

### Embedding The Secret Key In Code

Do not hardcode the secret in application code, tests that ship to users, or committed config files.

### Depending On `__version__`

In `2.2.0`, the `__version__` attribute is deprecated. Use feature detection or:

```python
from importlib.metadata import version

package_version = version("itsdangerous")
```

### Using Removed JWS APIs

Older examples may reference `JSONWebSignatureSerializer` or `TimedJSONWebSignatureSerializer`. JWS support was deprecated in `2.0.0` and removed in `2.1.0`. Use a dedicated JWT or JWS library such as Authlib when you need standards-based tokens.

### Assuming You Must Replace SHA-1 Immediately

The Pallets docs note that the default SHA-1 is used inside HMAC, where the collision concerns do not apply in the same way. If your environment still requires a different digest, configure it explicitly and plan compatibility with `fallback_signers`.

## Version-Sensitive Notes For 2.2.0

- `2.2.0` was released on `2024-04-16`.
- Python `3.7` support was dropped in `2.2.0`; current requirement is Python `>=3.8`.
- `__version__` is deprecated in `2.2.0`.
- `Serializer` typing in `2.2.0` is more explicit for type checkers.
- The changelog notes that FIPS builds may not expose the default `hashlib.sha1` in the same way at import time; if you run in FIPS-constrained environments, test token creation and verification explicitly.
- Since `2.1.0`, JWS functionality is gone and `itsdangerous.json` should not be imported.

## Practical Recommendations For Agents

- Default to `URLSafeTimedSerializer` for emailed action links and other expiring user-facing tokens.
- Use a purpose-specific salt and a stable payload schema such as `{"user_id": ..., "action": ...}`.
- Treat token verification failures as normal control flow, not as crashes.
- Prefer key rotation lists over forced global invalidation when you need secret rotation.
- If you are integrating with Flask or another framework that already has a secret-key convention, reuse the framework secret-management pattern instead of inventing a second config path.

## Official Sources

- Documentation: `https://itsdangerous.palletsprojects.com/en/stable/`
- Concepts: `https://itsdangerous.palletsprojects.com/en/stable/concepts/`
- Serialization: `https://itsdangerous.palletsprojects.com/en/stable/serializer/`
- URL-safe serialization: `https://itsdangerous.palletsprojects.com/en/stable/url_safe/`
- Timed signing: `https://itsdangerous.palletsprojects.com/en/stable/timed/`
- Signing: `https://itsdangerous.palletsprojects.com/en/stable/signer/`
- Changelog: `https://itsdangerous.palletsprojects.com/en/stable/changes/`
- PyPI release page: `https://pypi.org/project/itsdangerous/2.2.0/`
