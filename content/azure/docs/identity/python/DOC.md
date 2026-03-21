---
name: identity
description: "Azure Identity for Python: DefaultAzureCredential, managed identity, service principals, workload identity, async credentials, and auth configuration"
metadata:
  languages: "python"
  versions: "1.25.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,identity,azure-identity,python,authentication,entra-id,managed-identity,workload-identity"
---

# Azure Identity Python Package Guide

## Golden Rule

Use `azure-identity` for Microsoft Entra ID authentication in Python Azure SDK code. Start with `DefaultAzureCredential()` to get local development working quickly, but prefer a narrower credential such as `ManagedIdentityCredential`, `WorkloadIdentityCredential`, or `ClientSecretCredential` once the runtime environment is known.

Authentication success is not authorization success. The principal also needs the correct Azure RBAC or service-specific data-plane roles on the target resource.

## Install

Pin the package version when you want behavior aligned with this entry:

```bash
python -m pip install "azure-identity==1.25.2"
```

If you will use async credentials, install an async transport as well:

```bash
python -m pip install "azure-identity==1.25.2" aiohttp
```

If you need brokered developer authentication on supported Windows or WSL setups:

```bash
python -m pip install "azure-identity-broker"
```

## Choose The Credential

Use this selection rule:

- `DefaultAzureCredential`: best default when the same code must run both locally and in Azure.
- `ManagedIdentityCredential`: best for Azure-hosted production workloads with managed identity enabled.
- `WorkloadIdentityCredential`: best for AKS or other OIDC federation setups that use workload identity.
- `ClientSecretCredential`: use for CI or non-Azure hosting when a service principal secret is acceptable.
- `CertificateCredential`: use when the app registration authenticates with a certificate instead of a secret.
- `AzureCliCredential`, `AzureDeveloperCliCredential`, `AzurePowerShellCredential`: useful for explicit local-development-only flows.
- `DeviceCodeCredential` or `InteractiveBrowserCredential`: useful for scripts and developer tools that need user sign-in.

If the app always runs in one environment, prefer the specific credential for that environment over `DefaultAzureCredential`.

## DefaultAzureCredential

In `1.25.2`, `DefaultAzureCredential` tries these identities in order:

1. Environment credential from service principal environment variables
2. Workload identity
3. Managed identity
4. Shared token cache on Windows
5. Visual Studio Code credential when supported and broker support is installed
6. Azure CLI
7. Azure PowerShell
8. Azure Developer CLI
9. Interactive browser, only if explicitly enabled
10. Brokered authentication when `azure-identity-broker` is installed

Basic Azure SDK client setup:

```python
from azure.identity import DefaultAzureCredential
from azure.storage.blob import BlobServiceClient

account_url = "https://<storage-account>.blob.core.windows.net"

credential = DefaultAzureCredential()
client = BlobServiceClient(account_url=account_url, credential=credential)

for container in client.list_containers():
    print(container["name"])
```

Good defaults:

- Reuse one credential instance instead of constructing a new one for every request.
- Let Azure SDK clients handle scopes automatically unless you are calling `get_token()` yourself.
- Use `DefaultAzureCredential` for early development, then tighten the credential choice for production.

## Local Development

The easiest path is to authenticate one of the developer tools that `DefaultAzureCredential` can reuse:

```bash
az login
```

Or:

```bash
azd auth login
```

Then use the same application code:

```python
from azure.identity import DefaultAzureCredential
from azure.keyvault.secrets import SecretClient

vault_url = "https://<vault-name>.vault.azure.net"

credential = DefaultAzureCredential()
client = SecretClient(vault_url=vault_url, credential=credential)

secret = client.get_secret("example-secret")
print(secret.value)
```

Important behavior:

- Since `1.14.0`, `DefaultAzureCredential` continues through developer credentials until one succeeds.
- Deployed-service credentials still stop the chain if they can attempt token retrieval but fail.
- Local environment variables such as `AZURE_CLIENT_ID`, `AZURE_TENANT_ID`, and `AZURE_CLIENT_SECRET` can shadow your Azure CLI or `azd` login because `EnvironmentCredential` is tried first.

## Production Credentials

### Managed identity

Use managed identity for Azure-hosted apps when available:

```python
from azure.identity import ManagedIdentityCredential
from azure.storage.queue import QueueServiceClient

credential = ManagedIdentityCredential()
client = QueueServiceClient(
    account_url="https://<storage-account>.queue.core.windows.net",
    credential=credential,
)

for queue in client.list_queues():
    print(queue["name"])
```

For a user-assigned managed identity, pass the client ID:

```python
from azure.identity import ManagedIdentityCredential

credential = ManagedIdentityCredential(client_id="<managed-identity-client-id>")
```

### Service principal with a secret

Use this for CI or non-Azure hosting when secrets are acceptable:

```python
from azure.identity import ClientSecretCredential
from azure.mgmt.resource import SubscriptionClient

credential = ClientSecretCredential(
    tenant_id="<tenant-id>",
    client_id="<client-id>",
    client_secret="<client-secret>",
)

client = SubscriptionClient(credential)

for subscription in client.subscriptions.list():
    print(subscription.subscription_id)
```

Equivalent environment variables:

```bash
export AZURE_TENANT_ID="<tenant-id>"
export AZURE_CLIENT_ID="<client-id>"
export AZURE_CLIENT_SECRET="<client-secret>"
```

### Service principal with a certificate

`EnvironmentCredential` also supports certificate-based authentication:

```bash
export AZURE_TENANT_ID="<tenant-id>"
export AZURE_CLIENT_ID="<client-id>"
export AZURE_CLIENT_CERTIFICATE_PATH="/path/to/cert.pem"
export AZURE_CLIENT_CERTIFICATE_PASSWORD="<optional-password>"
```

Set this only when Subject Name/Issuer authentication requires it:

```bash
export AZURE_CLIENT_SEND_CERTIFICATE_CHAIN=true
```

### Workload identity

Use `WorkloadIdentityCredential` for Kubernetes or OIDC federation setups. `DefaultAzureCredential` can pick it up automatically when the workload identity environment is configured, but it is often better to instantiate `WorkloadIdentityCredential` directly when the deployment model is fixed.

## Non-SDK HTTP Calls

Use `get_token()` when the target library is not an Azure SDK client:

```python
import requests
from azure.identity import ClientSecretCredential

credential = ClientSecretCredential(
    tenant_id="<tenant-id>",
    client_id="<client-id>",
    client_secret="<client-secret>",
)

token = credential.get_token("https://management.azure.com/.default")

response = requests.get(
    "https://management.azure.com/subscriptions?api-version=2020-01-01",
    headers={"Authorization": f"Bearer {token.token}"},
    timeout=30,
)
response.raise_for_status()
print(response.json())
```

For Microsoft APIs protected by Entra ID, the scope is usually the resource URI plus `/.default`.

## Async Credentials

Async credentials live under `azure.identity.aio` and should be paired with async Azure SDK clients:

```python
from azure.identity.aio import DefaultAzureCredential
from azure.keyvault.secrets.aio import SecretClient

credential = DefaultAzureCredential()
client = SecretClient("https://<vault-name>.vault.azure.net", credential)

secret = await client.get_secret("example-secret")
print(secret.value)

await credential.close()
```

A safer pattern is using the credential as an async context manager:

```python
from azure.identity.aio import DefaultAzureCredential

async with DefaultAzureCredential() as credential:
    ...
```

## Config And Auth Details

### Environment variables

`EnvironmentCredential` and `DefaultAzureCredential` commonly depend on:

- `AZURE_TENANT_ID`
- `AZURE_CLIENT_ID`
- `AZURE_CLIENT_SECRET`
- `AZURE_CLIENT_CERTIFICATE_PATH`
- `AZURE_CLIENT_CERTIFICATE_PASSWORD`
- `AZURE_AUTHORITY_HOST`

If both secret and certificate settings are present, secret-based configuration wins.

### Sovereign or private clouds

Credentials default to Azure Public Cloud. For Azure Government or other clouds, set the authority explicitly:

```python
from azure.identity import AzureAuthorityHosts, DefaultAzureCredential

credential = DefaultAzureCredential(
    authority=AzureAuthorityHosts.AZURE_GOVERNMENT,
)
```

Or set it for the environment:

```bash
export AZURE_AUTHORITY_HOST="https://login.microsoftonline.us"
```

Developer-tool credentials such as `AzureCliCredential` use the tool's own cloud configuration instead of this setting.

### Narrowing the default chain

If you need to control `DefaultAzureCredential` without rewriting code:

```bash
export AZURE_TOKEN_CREDENTIALS=prod
```

Or choose one credential explicitly:

```bash
export AZURE_TOKEN_CREDENTIALS=ManagedIdentityCredential
```

Then require the environment variable to be set:

```python
from azure.identity import DefaultAzureCredential

credential = DefaultAzureCredential(require_envvar=True)
```

Version notes for this behavior:

- `AZURE_TOKEN_CREDENTIALS=prod` and `dev` are supported in `azure-identity` `1.23.0+`
- individual credential names are supported in `1.24.0+`

### Continuous Access Evaluation

CAE support is per-token-request:

```python
token = credential.get_token(
    "https://storage.azure.com/.default",
    enable_cae=True,
)
```

CAE is not supported for developer credentials or managed identity credentials.

### Token caching

`azure-identity` uses in-memory token caching by default and also supports persistent disk caching for supported credential types. Caching improves performance, but it can also hide configuration drift during debugging, so inspect the active credential path when behavior looks inconsistent.

## Common Pitfalls

- Authentication succeeds but service calls still fail because the principal lacks RBAC or data-plane permissions.
- Local service principal environment variables unexpectedly override Azure CLI or Azure Developer CLI credentials.
- `DefaultAzureCredential` adds avoidable latency in production because it probes multiple credentials before finding the right one.
- Async credentials are created but never closed.
- The wrong cloud authority is used for sovereign cloud deployments.
- CAE is enabled against a credential type that does not support it.
- `azure-identity-broker` is expected to work everywhere; brokered auth is specifically tied to supported Windows or WSL scenarios.
- Azure AD B2C is not supported by this library.

## Version-Sensitive Notes

- `1.25.2` is the current stable PyPI release as of March 12, 2026. PyPI also lists prerelease `1.26.0b2`, which should not be treated as the stable target.
- `1.25.2` fixed token-cache bypass issues when claims are passed to `get_token()` or `get_token_info()`, improved empty-response error handling, bumped the minimum `msal` dependency to `>=1.31.0`, and added more debug logging for cache hits.
- `1.24.0` added support for selecting an individual credential with `AZURE_TOKEN_CREDENTIALS`.
- `1.24.0` also re-enabled `VisualStudioCodeCredential`, but it requires `azure-identity-broker`.
- `1.24.0` dropped Python 3.8 support; current stable requires Python `>=3.9`.
- `1.14.0` changed `DefaultAzureCredential` continuation behavior so developer credential failures no longer stop the chain.
- `1.14.0` also added per-request CAE support.

## Official Sources

- Microsoft Learn package overview: `https://learn.microsoft.com/en-us/python/api/overview/azure/identity-readme?view=azure-python`
- Microsoft Learn credential chains guidance: `https://learn.microsoft.com/en-us/azure/developer/python/sdk/authentication/credential-chains`
- Microsoft Learn `DefaultAzureCredential` reference: `https://learn.microsoft.com/en-us/python/api/azure-identity/azure.identity.defaultazurecredential?view=azure-python`
- Microsoft Learn async package reference: `https://learn.microsoft.com/en-us/python/api/azure-identity/azure.identity.aio?view=azure-python`
- PyPI package page and release history: `https://pypi.org/project/azure-identity/`
