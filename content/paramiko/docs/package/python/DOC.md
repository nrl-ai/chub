---
name: package
description: "Paramiko SSHv2 library for Python clients, SFTP transfers, host-key verification, and low-level SSH automation"
metadata:
  languages: "python"
  versions: "4.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "paramiko,ssh,sftp,python,security,automation"
---

# Paramiko Python Package Guide

## Golden Rule

Use `paramiko` for low-level SSHv2 and SFTP automation in Python, verify host keys instead of blindly trusting them, and prefer key-based authentication over embedded passwords. If you just need high-level remote task execution, upstream points most users toward Fabric, which is built on Paramiko.

## Install

Pin the package version your project expects:

```bash
python -m pip install "paramiko==4.0.0"
```

Common alternatives:

```bash
uv add "paramiko==4.0.0"
poetry add "paramiko==4.0.0"
```

Optional Kerberos / GSS-API support:

```bash
python -m pip install "paramiko[gssapi]==4.0.0"
```

Version-specific install notes for `4.0.0`:

- `paramiko[invoke]`, `paramiko[all]`, and `paramiko[ed25519]` were removed in `4.0.0`.
- Old guides that tell you to install those extras are stale.
- `cryptography` remains a normal dependency, so key algorithms such as Ed25519 do not need a special extra.

## Core Client Setup

`SSHClient` is the usual entry point for application code:

```python
from pathlib import Path

import paramiko

with paramiko.SSHClient() as client:
    client.load_system_host_keys()
    client.connect(
        hostname="server.example.com",
        username="deploy",
        key_filename=str(Path.home() / ".ssh" / "id_ed25519"),
        look_for_keys=True,
        allow_agent=True,
        timeout=10,
        banner_timeout=10,
        auth_timeout=10,
    )

    stdin, stdout, stderr = client.exec_command("uname -a")
    print(stdout.read().decode("utf-8"))
```

Practical defaults:

- Call `load_system_host_keys()` so Paramiko can verify hosts against the user's `known_hosts`.
- Keep the default reject behavior for unknown host keys in production.
- Set `timeout`, `banner_timeout`, and `auth_timeout` explicitly when connecting over unreliable networks.

## Host Keys And Trust Policy

Host-key verification is the security boundary most agents get wrong.

Safe pattern:

```python
import paramiko

client = paramiko.SSHClient()
client.load_system_host_keys()
client.load_host_keys("/path/to/team-known_hosts")
client.connect(hostname="server.example.com", username="deploy")
```

Unsafe pattern except in disposable test environments:

```python
import paramiko

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
```

Use `AutoAddPolicy` only when the environment is controlled and host identities are managed out of band. Otherwise you are silently trusting the first key a server presents.

## Authentication

`SSHClient.connect()` tries authentication in a defined order. The important inputs are:

1. `pkey` or `key_filename`
2. Keys available from an SSH agent
3. Discoverable default private keys under `~/.ssh/`
4. Plain password authentication, if you pass `password=...`

### Load a private key explicitly

`PKey.from_path()` is convenient when you want Paramiko to infer the key type from a path:

```python
from pathlib import Path

import paramiko

key = paramiko.PKey.from_path(Path.home() / ".ssh" / "id_ed25519")

with paramiko.SSHClient() as client:
    client.load_system_host_keys()
    client.connect(
        hostname="server.example.com",
        username="deploy",
        pkey=key,
        passphrase="key-passphrase-if-needed",
    )
```

Notes:

- Prefer `pkey=` when your code already knows exactly which key object to use.
- Use `passphrase=` for encrypted private keys.
- `key_filename=` can also point at OpenSSH public certificates ending in `-cert.pub`; Paramiko will look for the matching private key automatically.

### Password authentication

```python
import os

import paramiko

with paramiko.SSHClient() as client:
    client.load_system_host_keys()
    client.connect(
        hostname="server.example.com",
        username="deploy",
        password=os.environ["SSH_PASSWORD"],
    )
```

Use passwords only when key-based auth is not available. Keep secrets in environment variables or a secret manager, not in source code.

## Run Commands

`exec_command()` returns file-like objects for stdin, stdout, and stderr:

```python
import paramiko

with paramiko.SSHClient() as client:
    client.load_system_host_keys()
    client.connect(hostname="server.example.com", username="deploy")

    stdin, stdout, stderr = client.exec_command("python3 --version")
    exit_code = stdout.channel.recv_exit_status()

    print("exit:", exit_code)
    print("stdout:", stdout.read().decode("utf-8"))
    print("stderr:", stderr.read().decode("utf-8"))
```

If you need an interactive shell, use `invoke_shell()` instead of trying to force shell semantics through `exec_command()`.

## SFTP

Open SFTP from an authenticated `SSHClient`:

```python
import paramiko

with paramiko.SSHClient() as client:
    client.load_system_host_keys()
    client.connect(hostname="server.example.com", username="deploy")

    with client.open_sftp() as sftp:
        sftp.put("build/output.txt", "/tmp/output.txt")
        sftp.get("/var/log/app.log", "app.log")
        print(sftp.listdir("/tmp"))
```

Useful SFTP operations:

- `put()` and `get()` for whole-file transfers
- `file()` for remote file-like access
- `listdir()` / `listdir_attr()` for directory reads
- `mkdir()`, `remove()`, `rename()`, and `posix_rename()` for remote filesystem changes
- `stat()` and `lstat()` for metadata checks before overwriting files

## SSH Config Integration

Paramiko can parse OpenSSH-style config files:

```python
from pathlib import Path

import paramiko

config = paramiko.SSHConfig.from_path(Path.home() / ".ssh" / "config")
host = config.lookup("prod-box")
identity_files = host.get("identityfile", [])

with paramiko.SSHClient() as client:
    client.load_system_host_keys()
    client.connect(
        hostname=host.get("hostname", "prod-box"),
        username=host.get("user"),
        port=int(host.get("port", 22)),
        key_filename=identity_files[0] if identity_files else None,
    )
```

This is useful when your deployment already centralizes `Host`, `User`, `Port`, and `IdentityFile` in `~/.ssh/config`.

## Configuration And Compatibility Notes

- Use `disabled_algorithms` only for deliberate compatibility workarounds with old servers, not as a blanket default.
- `sock=` on `connect()` lets you supply an existing socket or proxy-wrapped connection when a direct TCP connection is not appropriate.
- Paramiko is a low-level SSH library. It does not read shell startup files or emulate every OpenSSH CLI behavior for you.
- Close `SSHClient`, `Transport`, and `SFTPClient` objects promptly. Context managers are the safest pattern.

## Common Pitfalls

- Unknown host keys are rejected by default. Do not "fix" this with `AutoAddPolicy` unless you accept the security tradeoff.
- `exec_command()` is not interactive. Commands that require a TTY, prompt for input, or expect shell state often need `invoke_shell()` or explicit command wrapping.
- Remote paths in SFTP are server paths, not local paths. Validate them with `stat()` or `listdir()` when debugging transfer failures.
- Authentication failures are often key-selection failures. When debugging, pass `pkey=` or a single `key_filename=` instead of relying on agent discovery.
- Some timeout failures happen after the TCP connection succeeds. Set `banner_timeout` and `auth_timeout`, not just `timeout`.

## Version-Sensitive Notes For 4.0.0

- Paramiko `4.0.0` requires Python `>=3.9`.
- `4.0.0` removes DSA / DSS key support entirely. Old code using `DSSKey` or DSA host keys will break and should be migrated to Ed25519, ECDSA, or RSA.
- Old installation advice for `paramiko[invoke]`, `paramiko[all]`, or `paramiko[ed25519]` is obsolete in `4.0.0`.
- The versioned docs under `/en/4.0/` are a better reference for this package entry than the floating `/en/stable/` URL when you need version-accurate behavior.
