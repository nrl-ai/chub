---
name: keyvault-certificates
description: "Azure Key Vault certificates SDK for Python with CertificateClient setup, auth, create/import flows, async usage, and deletion/recovery patterns"
metadata:
  languages: "python"
  versions: "4.10.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,keyvault,certificates,python,security,cloud"
---

# azure-keyvault-certificates Python Package Guide

## What This Package Does

`azure-keyvault-certificates` is the official Azure SDK package for managing certificates in Azure Key Vault from Python. Use `CertificateClient` to create certificates, fetch the latest or a specific version, update mutable properties, import externally created certificates, list certificate metadata, and delete or recover certificates.

This package also exposes issuer, contact, backup, restore, and certificate policy operations, but most agent tasks start with `CertificateClient`.

Use adjacent packages when your workflow crosses Key Vault surfaces:

- `azure-identity` for authentication
- `azure-keyvault-secrets` for secret workflows
- `azure-keyvault-keys` for key workflows

## Install

```bash
pip install "azure-keyvault-certificates==4.10.0" azure-identity
```

For async code, install an async transport as well:

```bash
pip install aiohttp
```

## Version-Sensitive Notes

- This guide covers package version `4.10.0`, which PyPI lists as released on `2025-06-16`.
- `4.10.0` requires Python `3.9+`. Microsoft Learn's package overview, the PyPI metadata, and the upstream changelog agree on that. The quickstart page still says Python `3.7+`, which appears stale for this package version.
- `4.10.0` adds service API version `7.6`, makes `7.6` the default service API version, and uses the keyword argument `preserve_order` for certificate chain ordering.
- If you copied beta examples for `4.10.0b1`, rename `preserve_certificate_order` to `preserve_order`.

## Auth And Setup

You need:

- an Azure subscription
- an Azure Key Vault
- permission to manage certificates in that vault

For local development, the official docs use `DefaultAzureCredential` plus `az login`. For production, Microsoft recommends a managed identity.

```bash
az login
export VAULT_URL="https://<your-key-vault>.vault.azure.net"
```

If you are using Azure RBAC, the current quickstart assigns the `Key Vault Certificates Officer` role to manage certificates.

Minimum permissions depend on the operations you call:

- create: `certificates/create`
- get: `certificates/get`
- list: `certificates/list`
- import: `certificates/import`
- delete: `certificates/delete`
- recover: `certificates/recover`
- purge: `certificates/purge`

Waiting on some pollers also requires `certificates/get`.

## Initialize A Client

```python
import os

from azure.identity import DefaultAzureCredential
from azure.keyvault.certificates import CertificateClient

vault_url = os.environ["VAULT_URL"]
credential = DefaultAzureCredential()
client = CertificateClient(vault_url=vault_url, credential=credential)
```

If your environment only exposes the vault name, build the URL as `https://<name>.vault.azure.net`.

Advanced constructor options that matter in real deployments:

- `api_version="7.5"` or another supported version when you must pin the Key Vault service API instead of using the latest default
- `verify_challenge_resource=False` only for non-standard Key Vault domain scenarios where Azure explicitly tells you to disable challenge-resource validation

## Common Sync Workflow

This covers the core operations most agents need first: create, fetch, update, list, inspect versions, and delete.

```python
from azure.identity import DefaultAzureCredential
from azure.keyvault.certificates import CertificateClient, CertificatePolicy

credential = DefaultAzureCredential()

with CertificateClient(
    vault_url="https://my-key-vault.vault.azure.net/",
    credential=credential,
) as client:
    create_poller = client.begin_create_certificate(
        certificate_name="app-cert",
        policy=CertificatePolicy.get_default(),
    )
    created = create_poller.result()
    print(created.name)
    print(created.properties.version)

    current = client.get_certificate("app-cert")
    print(current.policy.issuer_name)

    updated = client.update_certificate_properties(
        certificate_name="app-cert",
        enabled=False,
        tags={"env": "dev"},
    )
    print(updated.properties.enabled)

    for props in client.list_properties_of_certificates():
        print(props.name)

    for version in client.list_properties_of_certificate_versions("app-cert"):
        print(version.version)

    delete_poller = client.begin_delete_certificate("app-cert")
    deleted = delete_poller.result()
    print(deleted.name)
```

When you need a specific version instead of the latest one:

```python
cert = client.get_certificate_version(
    certificate_name="app-cert",
    version="certificate-version-id",
)
print(cert.properties.version)
```

Use `get_certificate` when you need the latest certificate with its management policy. Use `get_certificate_version` when you need one exact version and do not need the policy object.

## Import An Existing Certificate

Use `import_certificate` when you already have a certificate file with its private key.

```python
from pathlib import Path

from azure.identity import DefaultAzureCredential
from azure.keyvault.certificates import CertificateClient

credential = DefaultAzureCredential()
with CertificateClient(
    vault_url="https://my-key-vault.vault.azure.net/",
    credential=credential,
) as client:
    imported = client.import_certificate(
        certificate_name="imported-cert",
        certificate_bytes=Path("certificate.pfx").read_bytes(),
        password="pfx-password",
        preserve_order=True,
    )

print(imported.name)
```

Important import rules from the official API docs:

- the imported certificate must contain the private key
- supported formats are `PFX` and `PEM`
- if you import `PEM`, the PEM must contain both the key and the x509 certificates
- for `PEM`, pass a `policy` whose `content_type` is `pem`
- in `4.10.0`, use `preserve_order`, not the beta name `preserve_certificate_order`

## Recovery And Permanent Deletion

Key Vault soft-delete behavior matters for automation. Deleting a certificate and immediately recreating it often fails until the deleted resource is recovered or purged.

```python
from azure.identity import DefaultAzureCredential
from azure.keyvault.certificates import CertificateClient

credential = DefaultAzureCredential()
with CertificateClient(
    vault_url="https://my-key-vault.vault.azure.net/",
    credential=credential,
) as client:
    delete_poller = client.begin_delete_certificate("app-cert")
    deleted = delete_poller.result()

    recover_poller = client.begin_recover_deleted_certificate("app-cert")
    recovered = recover_poller.result()
    print(recovered.name)

    delete_poller = client.begin_delete_certificate("app-cert")
    delete_poller.wait()
    client.purge_deleted_certificate("app-cert")
```

`purge_deleted_certificate` is irreversible and only works when the vault's recovery level allows purging.

## Async Usage

The package has async APIs under `azure.keyvault.certificates.aio`. Install an async transport first and close both the client and credential when finished.

```python
from azure.identity.aio import DefaultAzureCredential
from azure.keyvault.certificates import CertificatePolicy
from azure.keyvault.certificates.aio import CertificateClient

async with DefaultAzureCredential() as credential:
    async with CertificateClient(
        vault_url="https://my-key-vault.vault.azure.net/",
        credential=credential,
    ) as client:
        result = await client.create_certificate(
            certificate_name="async-cert",
            policy=CertificatePolicy.get_default(),
        )
        print(result.name)

        async for props in client.list_properties_of_certificates():
            print(props.name)
```

## Common Pitfalls

- `begin_create_certificate`, `begin_delete_certificate`, and `begin_recover_deleted_certificate` are long-running operations. Use `.result()` or `.wait()` before the next dependent step.
- `get_certificate` returns the latest version with the management policy attached. `get_certificate_version` returns one version without the policy. Choose the one that matches your next operation.
- `list_properties_of_certificates()` and `list_properties_of_certificate_versions()` return identifiers and properties, not the full certificate payload.
- The quickstart uses `KEY_VAULT_NAME`, but `CertificateClient` takes the full `vault_url`.
- `DefaultAzureCredential` is the default choice, but it only works if one of its credential sources is configured. For local work, `az login` is usually the fastest path.
- Deleting a certificate in a soft-delete-enabled vault leaves it recoverable. Reusing the same name can fail with a conflict until you recover or purge it.
- For custom policies, `begin_create_certificate` requires a valid certificate policy. A bad policy raises `ValueError`; service failures raise `HttpResponseError`.
- Enabling SDK HTTP logging can expose URLs, headers, and other request details. Treat `logging_enable=True` as sensitive.

## Official Sources

- Microsoft Learn package overview: `https://learn.microsoft.com/en-us/python/api/overview/azure/keyvault-certificates-readme?view=azure-python`
- Microsoft Learn API reference root: `https://learn.microsoft.com/en-us/python/api/azure-keyvault-certificates/`
- Microsoft Learn `CertificateClient` reference: `https://learn.microsoft.com/en-us/python/api/azure-keyvault-certificates/azure.keyvault.certificates.certificateclient?view=azure-python`
- Microsoft Learn quickstart: `https://learn.microsoft.com/en-us/azure/key-vault/certificates/quick-create-python`
- PyPI package page: `https://pypi.org/project/azure-keyvault-certificates/4.10.0/`
- Azure SDK for Python changelog: `https://raw.githubusercontent.com/Azure/azure-sdk-for-python/main/sdk/keyvault/azure-keyvault-certificates/CHANGELOG.md`
