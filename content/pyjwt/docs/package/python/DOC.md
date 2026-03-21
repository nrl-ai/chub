---
name: package
description: "PyJWT package guide for Python covering token signing, verification, claim validation, and JWKS key lookup"
metadata:
  languages: "python"
  versions: "2.11.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pyjwt,jwt,jws,jwk,jwks,authentication,security,python"
---

# PyJWT Python Package Guide

## What It Is

`PyJWT` is the upstream Python library for encoding and decoding JSON Web Tokens. The pip package is `PyJWT`, but the import name is `jwt`.

- Package name: `PyJWT`
- Import name: `jwt`
- Version covered: `2.11.0`
- Docs root used for this entry: `https://pyjwt.readthedocs.io/en/2.11.0/`

## Install

Use plain `PyJWT` when you only need HMAC signing and verification:

```bash
pip install PyJWT==2.11.0
```

Use the `crypto` extra for RSA, ECDSA, EdDSA, JWK, and JWKS workflows:

```bash
pip install "PyJWT[crypto]==2.11.0"
```

## Basic Setup

```python
import jwt

SECRET_KEY = "replace-me"
ALGORITHM = "HS256"
```

Common pitfall: `import pyjwt` is wrong for normal use. Import `jwt`.

## Encode And Decode

### HMAC example

```python
from datetime import UTC, datetime, timedelta

import jwt

secret = "replace-me"

payload = {
    "sub": "user-123",
    "iss": "https://auth.example.com",
    "aud": "api://my-service",
    "iat": datetime.now(UTC),
    "nbf": datetime.now(UTC),
    "exp": datetime.now(UTC) + timedelta(minutes=15),
}

token = jwt.encode(payload, secret, algorithm="HS256")

claims = jwt.decode(
    token,
    secret,
    algorithms=["HS256"],
    issuer="https://auth.example.com",
    audience="api://my-service",
)
```

`jwt.encode()` returns a string in PyJWT 2.x. Datetime values for claims like `exp`, `nbf`, and `iat` are accepted directly and encoded as JWT NumericDate values.

### RSA example

```python
import jwt

private_key_pem = open("private.pem", "rb").read()
public_key_pem = open("public.pem", "rb").read()

token = jwt.encode({"sub": "user-123"}, private_key_pem, algorithm="RS256")

claims = jwt.decode(token, public_key_pem, algorithms=["RS256"])
```

Asymmetric algorithms generally require `PyJWT[crypto]`.

## Registered Claims And Validation

Pass the expected claim values into `decode()` when you want PyJWT to verify them:

```python
claims = jwt.decode(
    token,
    secret,
    algorithms=["HS256"],
    issuer="https://auth.example.com",
    audience="api://my-service",
    subject="user-123",
    leeway=30,
    options={"require": ["exp", "iat", "nbf", "sub"]},
)
```

What matters in practice:

- `exp` rejects expired tokens.
- `nbf` rejects tokens before they become valid.
- `iat` is validated when enabled.
- `iss` is checked against the exact issuer string you pass.
- `aud` is checked against the audience you pass.
- `subject` lets you require one specific `sub` value.
- `leeway` handles small clock skew.

Important pitfall: `options={"require": [...]}` only checks that claims are present. It does not replace normal validation. For example, `aud` still needs an `audience=` value, and `sub` still needs `subject=` when you want to compare it to an expected user or client id.

If your service expects exactly one audience string and you do not want array-like audience handling, review the `strict_aud` decode option in the API reference.

## Reading Headers Or Claims Without Trusting The Token

Unverified reads are only for routing decisions such as choosing a key by `kid`.

```python
header = jwt.get_unverified_header(token)
claims = jwt.decode(
    token,
    options={"verify_signature": False},
    algorithms=["HS256"],
)
```

Do not treat unverified claims as authenticated data.

## JWKS And `PyJWKClient`

For OIDC and other JWKS-backed issuers, use `PyJWKClient` to fetch the signing key that matches the token header:

```python
import jwt

jwks_client = jwt.PyJWKClient(
    "https://issuer.example.com/.well-known/jwks.json",
    cache_keys=True,
)
signing_key = jwks_client.get_signing_key_from_jwt(token)

claims = jwt.decode(
    token,
    signing_key.key,
    algorithms=["RS256"],
    issuer="https://issuer.example.com/",
    audience="api://my-service",
)
```

This path normally needs `PyJWT[crypto]`. In practice, create one `PyJWKClient` per issuer and reuse it so its caching options can help instead of refetching on every request.

## Common Exceptions

Catch PyJWT exceptions around `decode()` and branch on the failure mode you care about:

```python
import jwt

try:
    claims = jwt.decode(token, secret, algorithms=["HS256"])
except jwt.ExpiredSignatureError:
    ...
except jwt.InvalidAudienceError:
    ...
except jwt.InvalidIssuerError:
    ...
except jwt.InvalidSubjectError:
    ...
except jwt.MissingRequiredClaimError:
    ...
except jwt.InvalidTokenError:
    ...
```

`InvalidTokenError` is the common base class when you do not need finer-grained handling.

## Auth And Configuration Patterns

Keep these values in app configuration instead of hard-coding them across handlers:

- signing or verification key material
- accepted algorithms
- expected issuer
- expected audience
- expected subject when applicable
- JWKS URL
- allowed clock skew (`leeway`)

Example:

```python
from dataclasses import dataclass

@dataclass(frozen=True)
class JwtConfig:
    algorithms: list[str]
    issuer: str | None = None
    audience: str | None = None
    subject: str | None = None
    jwks_url: str | None = None
    leeway_seconds: int = 30
```

Build the allowed algorithm list from trusted app config, never from token content.

## High-Value Pitfalls

### Always hard-code allowed algorithms

Pass `algorithms=[...]` to `decode()`. PyJWT's API docs explicitly warn not to compute accepted algorithms from attacker-controlled token headers.

### The package name and import name differ

Install `PyJWT`, but write `import jwt`.

### Install `PyJWT[crypto]` for asymmetric algorithms and JWKS

Plain `PyJWT` is enough for HMAC flows. RSA, ECDSA, EdDSA, JWK, and JWKS use cases depend on `cryptography`.

### `require` checks presence, not semantic validity

It only enforces that a claim exists. You still need `issuer=`, `audience=`, `subject=`, and normal expiration validation when those checks matter.

### Do not trust unverified payloads

`get_unverified_header()` and `verify_signature=False` are for key selection or debugging only.

### Do not accept unsigned tokens by accident

PyJWT 2.10.x made `"none"` algorithm handling more explicit. Treat unsigned tokens as invalid unless your protocol intentionally uses them.

## Version Notes For 2.11.0

- The target version version is `2.11.0`.
- The `stable` docs root currently renders 2.11.0, but this entry uses versioned URLs for deterministic package-version context.
- The 2.11.0 changelog is mostly a maintenance release for agents: Python 3.14 support, a new `algorithm_name` property on JWKs, and broader typing around `encode()` and `decode()`.
- The biggest behavior changes that still affect upgrades came in 2.10.x: stricter `sub` and `jti` validation in 2.10.0 and the issuer-matching fix in 2.10.1.

## Official Sources

- Stable docs root: https://pyjwt.readthedocs.io/en/stable/
- Versioned docs root: https://pyjwt.readthedocs.io/en/2.11.0/
- Installation: https://pyjwt.readthedocs.io/en/2.11.0/installation.html
- Usage: https://pyjwt.readthedocs.io/en/2.11.0/usage.html
- API reference: https://pyjwt.readthedocs.io/en/2.11.0/api.html
- Changelog: https://pyjwt.readthedocs.io/en/2.11.0/changelog.html
- PyPI project: https://pypi.org/project/PyJWT/
- PyPI version page: https://pypi.org/project/PyJWT/2.11.0/
