---
name: mgmt-authorization
description: "azure-mgmt-authorization Python package for Azure RBAC role assignments, role definitions, permissions, and deny-assignment management"
metadata:
  languages: "python"
  versions: "4.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,authorization,rbac,iam,role-assignments,role-definitions,python"
---

# azure-mgmt-authorization Python Package Guide

## What This Package Is For

`azure-mgmt-authorization` is the Azure Resource Manager SDK for Azure RBAC and related authorization resources from Python.

Use it when you need to:

- list or inspect role definitions
- create, read, list, or delete role assignments
- inspect the management-plane permissions the current caller has
- work with newer authorization namespaces such as deny assignments when required

Primary import surface:

```python
from azure.mgmt.authorization import AuthorizationManagementClient
```

This is an ARM management-plane client, not a Microsoft Graph client. Use it for subscription, resource-group, and resource scopes.

## Install

Install the package plus a credential provider:

```bash
python -m pip install "azure-mgmt-authorization==4.0.0" azure-identity
```

PyPI metadata for `4.0.0` requires Python `>=3.7`.

## Authentication And Setup

Use `azure-identity` credentials. `DefaultAzureCredential` is the default choice for both local development and hosted Azure environments.

Local development:

```bash
az login
az account set --subscription "<subscription-id-or-name>"
export AZURE_SUBSCRIPTION_ID="<subscription-id>"
```

Service principal environment variables:

```bash
export AZURE_CLIENT_ID="<app-client-id>"
export AZURE_TENANT_ID="<tenant-id>"
export AZURE_CLIENT_SECRET="<client-secret>"
export AZURE_SUBSCRIPTION_ID="<subscription-id>"
```

Client setup:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.authorization import AuthorizationManagementClient

subscription_id = os.environ["AZURE_SUBSCRIPTION_ID"]

client = AuthorizationManagementClient(
    credential=DefaultAzureCredential(),
    subscription_id=subscription_id,
)
```

For production, prefer managed identity or another non-interactive credential source. The constructor still needs a subscription ID even when the credential comes from Azure.

## Scope Handling

Many operations need a full ARM scope string:

```python
subscription_scope = f"/subscriptions/{subscription_id}"
resource_group_scope = (
    f"/subscriptions/{subscription_id}/resourceGroups/my-resource-group"
)
resource_scope = (
    f"{resource_group_scope}/providers/Microsoft.Storage/storageAccounts/myaccount"
)
```

If the scope is malformed, RBAC calls fail even when authentication is correct.

## Quick Start

List role assignments at subscription scope:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.authorization import AuthorizationManagementClient

subscription_id = os.environ["AZURE_SUBSCRIPTION_ID"]
scope = f"/subscriptions/{subscription_id}"

client = AuthorizationManagementClient(
    credential=DefaultAzureCredential(),
    subscription_id=subscription_id,
)

for assignment in client.role_assignments.list_for_scope(
    scope=scope,
    filter="atScope()",
):
    print(assignment.id)
    print(assignment.principal_id)
    print(assignment.role_definition_id)
```

Use `list_for_scope` for `4.0.0` code. Older examples that call `role_assignments.list()` are from an older package line.

## Core Usage

### List Role Definitions

Resolve the role definition you want before creating an assignment:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.authorization import AuthorizationManagementClient

subscription_id = os.environ["AZURE_SUBSCRIPTION_ID"]
scope = f"/subscriptions/{subscription_id}"

client = AuthorizationManagementClient(
    credential=DefaultAzureCredential(),
    subscription_id=subscription_id,
)

for role in client.role_definitions.list(scope=scope):
    print(role.id)
    print(role.role_name)
    print(role.type)
```

The service expects the full role definition ID. In practice, look it up once and reuse `role.id`.

### Create A Role Assignment

Role assignment names must be GUIDs. `principal_id` must be the Microsoft Entra object ID of the user, group, or service principal, not the application client ID.

```python
import os
import uuid

from azure.identity import DefaultAzureCredential
from azure.mgmt.authorization import AuthorizationManagementClient
from azure.mgmt.authorization.models import RoleAssignmentCreateParameters

subscription_id = os.environ["AZURE_SUBSCRIPTION_ID"]
principal_id = os.environ["AZURE_PRINCIPAL_OBJECT_ID"]
scope = f"/subscriptions/{subscription_id}"

client = AuthorizationManagementClient(
    credential=DefaultAzureCredential(),
    subscription_id=subscription_id,
)

reader_role = next(
    role
    for role in client.role_definitions.list(scope=scope)
    if getattr(role, "role_name", None) == "Reader"
)

assignment = client.role_assignments.create(
    scope=scope,
    role_assignment_name=str(uuid.uuid4()),
    parameters=RoleAssignmentCreateParameters(
        role_definition_id=reader_role.id,
        principal_id=principal_id,
        principal_type="ServicePrincipal",
    ),
)

print(assignment.id)
```

In `4.0.0`, `RoleAssignmentCreateParameters` also exposes optional fields such as `description`, `condition`, `condition_version`, and `delegated_managed_identity_resource_id`.

### Inspect Effective Permissions

Use the permissions API when you need to know which management actions the current caller has.

At resource-group scope:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.authorization import AuthorizationManagementClient

subscription_id = os.environ["AZURE_SUBSCRIPTION_ID"]

client = AuthorizationManagementClient(
    credential=DefaultAzureCredential(),
    subscription_id=subscription_id,
)

for permission in client.permissions.list_for_resource_group(
    resource_group_name="my-resource-group"
):
    print("actions:", permission.actions)
    print("not_actions:", permission.not_actions)
    print("data_actions:", permission.data_actions)
    print("not_data_actions:", permission.not_data_actions)
```

At a specific resource, `parent_resource_path` is required by the method signature. Use `""` when the target is not nested under another parent resource:

```python
for permission in client.permissions.list_for_resource(
    resource_group_name="my-resource-group",
    resource_provider_namespace="Microsoft.Storage",
    parent_resource_path="",
    resource_type="storageAccounts",
    resource_name="myaccount",
):
    print(permission.actions)
```

### Remove A Role Assignment

Delete by scope plus assignment GUID:

```python
client.role_assignments.delete(
    scope=f"/subscriptions/{subscription_id}",
    role_assignment_name="00000000-0000-0000-0000-000000000000",
)
```

If you only have the full assignment ID, parse the final GUID and reuse the original scope.

### Use Versioned Namespaces Only When Needed

The package also exposes versioned namespaces, for example:

```python
from azure.mgmt.authorization.v2022_04_01 import AuthorizationManagementClient
```

Use a versioned namespace when you need a specific Authorization API version or an operation group that does not exist on older namespaces. For common RBAC automation in `4.0.0`, the top-level import is simpler.

## Configuration And Auth Notes

- `DefaultAzureCredential` is the default auth path for new code.
- Set `AZURE_SUBSCRIPTION_ID` yourself; credential discovery does not supply it.
- The identity you authenticate with must already have RBAC rights on the target scope before it can inspect or change assignments.
- For local debugging, verify the effective identity and scope outside your script with `az account show` and `az role assignment list`.

## Common Pitfalls

- Do not pass an application client ID where the API expects `principal_id`; use the principal object ID.
- Do not invent scopes. Use full ARM resource IDs such as `/subscriptions/<id>` or `/subscriptions/<id>/resourceGroups/<name>`.
- Do not use this package for Microsoft Graph directory-role automation or app registration work.
- `role_assignment_name` must be a GUID string, not a display name.
- New RBAC assignments can be eventually consistent. A read immediately after create can briefly return 404 or incomplete results.
- `permissions.list_for_resource_group()` takes only the resource group name, while role-assignment APIs usually take a full scope string.
- `permissions.list_for_resource()` requires `parent_resource_path`; use `""` for a top-level resource.

## Version-Sensitive Notes

- PyPI shows `4.0.0` as the current stable release for this package and `5.0.0b1` as a prerelease.
- The Learn reference spans multiple Authorization API namespaces, including older `v2015_07_01` and newer `v2022_04_01` clients. Those namespaces do not expose identical operation groups.
- In `2.0.0`, the old unscoped `RoleAssignmentsOperations.list` method was removed. Use `list_for_scope`, `list_for_subscription`, `list_for_resource_group`, or `list_for_resource` instead.
- Older pre-1.0 examples may still use legacy Azure SDK credential patterns. For supported package lines, use `azure-identity`.
- PyPI release notes for `4.0.0` also note added validation operations for role assignment and role eligibility schedule requests, and removal of `AlertOperationOperations.list_for_scope`.

## Official Sources

- Package overview: `https://learn.microsoft.com/en-us/python/api/overview/azure/mgmt-authorization-readme?view=azure-python`
- API reference root: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-authorization/`
- `AuthorizationManagementClient`: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-authorization/azure.mgmt.authorization.v2022_04_01.authorizationmanagementclient?view=azure-python`
- `RoleAssignmentsOperations`: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-authorization/azure.mgmt.authorization.operations.roleassignmentsoperations?view=azure-python-preview`
- `RoleDefinitionsOperations`: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-authorization/azure.mgmt.authorization.operations.roledefinitionsoperations?view=azure-python-preview`
- `PermissionsOperations`: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-authorization/azure.mgmt.authorization.operations.permissionsoperations?view=azure-python-preview`
- `RoleAssignmentCreateParameters`: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-authorization/azure.mgmt.authorization.models.roleassignmentcreateparameters?view=azure-python-preview`
- Azure authorization overview for Python: `https://learn.microsoft.com/en-us/azure/developer/python/sdk/authorization/overview`
- Azure management samples entry point: `https://learn.microsoft.com/en-us/samples/azure-samples/azure-samples-python-management/authorization/`
- PyPI package page and release history: `https://pypi.org/project/azure-mgmt-authorization/`
