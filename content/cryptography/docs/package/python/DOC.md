---
name: package
description: "cryptography package guide for Python with installation, Fernet, AEAD, key serialization, and X.509 workflows"
metadata:
  languages: "python"
  versions: "46.0.5"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "cryptography,python,encryption,fernet,aead,x509,openssl"
---

# cryptography Python Package Guide

`cryptography` provides two layers:

- the recipes layer for common tasks such as Fernet
- the hazardous materials layer for lower-level primitives such as AES-GCM, key serialization, and X.509

Prefer the recipes layer unless you specifically need lower-level control.

## Installation

Use a pinned install when you need behavior that matches this doc:

```bash
python -m pip install "cryptography==46.0.5"
```

For a normal install:

```bash
python -m pip install cryptography
```

Important install notes from upstream:

- `cryptography` 46.0.5 requires Python `!=3.9.0`, `!=3.9.1`, `>=3.8`.
- If installation fails, upgrade `pip` first. The official docs call that the most common cause of install problems.
- On supported platforms, PyPI wheels are the normal path and already include the needed OpenSSL build.
- Building from source requires a C toolchain, OpenSSL and `libffi` headers, and Rust. The current upstream minimum Rust version shown in the latest install docs is `1.83.0`.
- On macOS source builds, do not rely on the base system OpenSSL/LibreSSL from Apple; use a supported OpenSSL installation instead.

## Setup and Secret Management

`cryptography` is a local library, not a network client. There is no service authentication step. Your "configuration" is usually:

- key material
- certificate material
- passwords for encrypted private keys or PKCS#12 blobs
- platform build settings only when you cannot use wheels

Practical rules:

- Store long-lived keys outside source control.
- Use environment variables or a secret manager to inject key material at runtime.
- If you derive keys from passwords, persist the salt alongside the encrypted data.
- Do not generate a new key on every process start unless token invalidation is intentional.

Example one-time Fernet key provisioning:

```python
import os
from cryptography.fernet import Fernet

key = os.getenv("FERNET_KEY")
if key is None:
    key = Fernet.generate_key().decode("ascii")
    print(f"Provision this once and store it securely: {key}")
    raise SystemExit(1)

fernet = Fernet(key.encode("ascii"))
```

## Core Usage

### Fernet for simple authenticated encryption

Use Fernet when you want a straightforward opaque token API and your payload fits in memory.

```python
from cryptography.fernet import Fernet, InvalidToken

key = Fernet.generate_key()
f = Fernet(key)

token = f.encrypt(b"api-key-or-session-data")

try:
    plaintext = f.decrypt(token, ttl=300)
except InvalidToken:
    plaintext = None
```

What matters:

- Fernet keys are URL-safe base64-encoded 32-byte keys.
- `decrypt(..., ttl=...)` rejects expired tokens.
- Fernet tokens include their creation timestamp in plaintext.
- Fernet is meant for messages that fit in memory; it is not a streaming file-encryption API.

### Rotate Fernet keys with `MultiFernet`

```python
from cryptography.fernet import Fernet, MultiFernet

new_key = Fernet(Fernet.generate_key())
old_key = Fernet(Fernet.generate_key())
ring = MultiFernet([new_key, old_key])

token = ring.encrypt(b"rotatable-secret")
rotated_token = ring.rotate(token)
plaintext = ring.decrypt(rotated_token)
```

Put the newest key first. Keep old keys available until all tokens that depend on them have been rotated or expired.

### Derive a Fernet key from a password

Use a KDF first. Do not feed a raw password directly into Fernet.

```python
import base64
import os

from cryptography.fernet import Fernet
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC

password = os.environ["APP_PASSWORD"].encode("utf-8")
salt = os.environ["APP_SALT"].encode("utf-8")

kdf = PBKDF2HMAC(
    algorithm=hashes.SHA256(),
    length=32,
    salt=base64.urlsafe_b64decode(salt),
    iterations=1_200_000,
)
key = base64.urlsafe_b64encode(kdf.derive(password))
f = Fernet(key)
```

Notes:

- Persist the salt so you can derive the same key later.
- The upstream Fernet docs use `1_200_000` PBKDF2 iterations as a current good default and explicitly say to tune upward as much as your system can tolerate.

### AES-GCM for protocol-level encryption

Use AEAD primitives when you need to control the nonce, associated data, or message framing yourself.

```python
import os

from cryptography.hazmat.primitives.ciphers.aead import AESGCM

key = AESGCM.generate_key(bit_length=256)
aesgcm = AESGCM(key)

nonce = os.urandom(12)
aad = b"request-id:123"
plaintext = b"top secret"

ciphertext = aesgcm.encrypt(nonce, plaintext, aad)
roundtrip = aesgcm.decrypt(nonce, ciphertext, aad)
```

Rules that matter:

- Never reuse a nonce with the same key.
- Use a 12-byte nonce unless you have a specific reason not to.
- `associated_data` must match exactly on decrypt.
- In current upstream docs, `encrypt_into` and `decrypt_into` are marked as added in `47.0.0`; they are not part of `46.0.5`.

## Key Serialization and Loading

### Write an encrypted PEM private key

```python
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import rsa

key = rsa.generate_private_key(public_exponent=65537, key_size=2048)

pem_bytes = key.private_bytes(
    encoding=serialization.Encoding.PEM,
    format=serialization.PrivateFormat.PKCS8,
    encryption_algorithm=serialization.BestAvailableEncryption(b"changeit"),
)
```

### Load a PEM private key back

```python
from cryptography.hazmat.primitives.serialization import load_pem_private_key

loaded_key = load_pem_private_key(pem_bytes, password=b"changeit")
```

Serialization pitfalls:

- A PEM block beginning with `-----BEGIN CERTIFICATE-----` is an X.509 certificate, not a private key. Load it with X.509 certificate APIs, not key-loading APIs.
- SSH private keys use a different format and should be loaded with `load_ssh_private_key()`.
- When loading a private key, verify that the returned object is the key type you expect before using it.
- Leave `unsafe_skip_rsa_key_validation` at its default unless you fully control the input and understand the tradeoff.

### PKCS#12

`cryptography` can load and serialize PKCS#12 bundles:

- `pkcs12.load_key_and_certificates(...)` loads a blob into key and certificate objects.
- `pkcs12.serialize_key_and_certificates(...)` writes a PKCS#12 blob.

Upstream warns that PKCS#12 encryption is typically weak and should not be treated as a strong security boundary. Use it for compatibility, then wrap it in a stronger storage or transport boundary if you need real protection.

## X.509 and CSR Workflow

Use the X.509 APIs for certificate requests, local development certificates, or certificate parsing.

```python
from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.x509.oid import NameOID

key = rsa.generate_private_key(public_exponent=65537, key_size=2048)

csr = (
    x509.CertificateSigningRequestBuilder()
    .subject_name(
        x509.Name(
            [
                x509.NameAttribute(NameOID.COUNTRY_NAME, "US"),
                x509.NameAttribute(NameOID.ORGANIZATION_NAME, "Example Inc"),
                x509.NameAttribute(NameOID.COMMON_NAME, "example.com"),
            ]
        )
    )
    .add_extension(
        x509.SubjectAlternativeName(
            [x509.DNSName("example.com"), x509.DNSName("www.example.com")]
        ),
        critical=False,
    )
    .sign(key, hashes.SHA256())
)

csr_pem = csr.public_bytes(serialization.Encoding.PEM)
```

For existing PEM certificates, load them with `x509.load_pem_x509_certificate(...)` and then inspect fields or extract the public key.

## Common Pitfalls

- Do not copy old blog examples that still use outdated APIs or omit secure defaults. Follow current upstream examples.
- Use the recipes layer when possible. The hazmat layer is intentionally lower-level and easier to misuse.
- Do not reuse an AEAD nonce with the same key.
- Do not treat Fernet as a streaming or very-large-file encryption tool.
- Do not forget to persist the salt when deriving a key from a password.
- Do not serialize private keys without encryption unless you have a deliberate reason and a stronger outer protection mechanism.
- Do not assume the docs at `/en/latest/` match `46.0.5` exactly; the latest docs currently render `47.0.0.dev1`.

## Version-Sensitive Notes for 46.0.5

- `46.0.5` includes a security fix for `CVE-2026-26007`, involving malicious public keys on uncommon binary elliptic curves.
- The same release deprecates `SECT*` binary elliptic curves and says they will be removed in the next release.
- `46.0.0` removed Python 3.7 support and deprecated OpenSSL `< 3.0`.
- The PyPI project page shows `46.0.5` as the latest released package on `2026-02-10`, but the docs URL points at `/en/latest/`, which currently tracks unreleased `47.0.0.dev1` docs. Check versioned pages before using newly documented APIs.

## Official Sources Used

- Docs root: `https://cryptography.io/en/latest/`
- Installation: `https://cryptography.io/en/latest/installation/`
- Fernet: `https://cryptography.io/en/latest/fernet/`
- AEAD primitives: `https://cryptography.io/en/latest/hazmat/primitives/aead/`
- Key serialization: `https://cryptography.io/en/latest/hazmat/primitives/asymmetric/serialization/`
- X.509 tutorial: `https://cryptography.io/en/latest/x509/tutorial/`
- Changelog: `https://cryptography.io/en/latest/changelog/`
- PyPI project page: `https://pypi.org/project/cryptography/`
