---
name: mgmt-monitor
description: "Azure Monitor management SDK for Python for activity logs, metric alerts, action groups, data collection rules, and scheduled query rules"
metadata:
  languages: "python"
  versions: "7.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,azure-monitor,monitor,management,arm,alerts,metrics,observability"
---

# Azure Monitor Management SDK for Python

## Golden Rule

Use `azure-mgmt-monitor` for Azure Monitor control-plane work: creating and managing alerts, action groups, data collection rules, diagnostic settings, and related Azure Monitor resources. Do not use it for querying logs or metrics data at runtime; for that, use `azure-monitor-query` or `azure-monitor-querymetrics`.

This entry is pinned to the current stable PyPI release `7.0.0`. PyPI also lists `8.0.0b1` and `8.0.0b2` prereleases, so avoid copying preview examples into a `7.x` codebase unless the project is intentionally pinned to a beta.

## Install

Install the management client together with `azure-identity`:

```bash
python -m pip install "azure-mgmt-monitor==7.0.0" "azure-identity"
```

Common alternatives:

```bash
uv add "azure-mgmt-monitor==7.0.0" "azure-identity"
poetry add "azure-mgmt-monitor==7.0.0" "azure-identity"
```

Notes:

- `azure-mgmt-monitor` is a management-plane SDK. It assumes you are provisioning or configuring Azure Monitor resources through Azure Resource Manager.
- Install `azure-monitor-query` or `azure-monitor-querymetrics` separately if the same project also queries telemetry data.

## Authentication And Setup

Use Azure Identity credentials. These are the practical defaults:

1. `AzureCliCredential()` for local scripts after `az login`
2. `DefaultAzureCredential()` for reusable code that runs locally, in CI, and on Azure
3. `ClientSecretCredential()` only when you know the runtime is service-principal based

Required environment:

```bash
export AZURE_SUBSCRIPTION_ID="00000000-0000-0000-0000-000000000000"
```

If you authenticate with a service principal directly, also set:

```bash
export AZURE_TENANT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_SECRET="your-client-secret"
```

Basic client setup:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.monitor import MonitorManagementClient

subscription_id = os.environ["AZURE_SUBSCRIPTION_ID"]
credential = DefaultAzureCredential()

client = MonitorManagementClient(
    credential=credential,
    subscription_id=subscription_id,
)
```

Local development with explicit Azure CLI credentials:

```python
import os

from azure.identity import AzureCliCredential
from azure.mgmt.monitor import MonitorManagementClient

client = MonitorManagementClient(
    credential=AzureCliCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)
```

## What This Client Covers

`MonitorManagementClient` in `7.0.0` exposes a broad Azure Monitor management surface. High-value operation groups include:

- `activity_logs`
- `action_groups`
- `activity_log_alerts`
- `metric_alerts`
- `scheduled_query_rules`
- `diagnostic_settings`
- `data_collection_rules`
- `data_collection_endpoints`
- `data_collection_rule_associations`
- `azure_monitor_workspaces`

Practical rule:

- Use this package when the resource you are working with has an ARM resource ID and you are creating, listing, updating, or deleting Azure Monitor configuration.
- Use the query packages when you need to fetch logs or metrics results for application logic.

## Core Usage

### Read recent activity log events

`activity_logs.list()` requires an OData filter string. Build the filter explicitly instead of guessing parameter names.

```python
from datetime import datetime, timedelta, timezone

end_time = datetime.now(timezone.utc)
start_time = end_time - timedelta(hours=1)

event_data = client.activity_logs.list(
    filter=(
        "eventTimestamp ge "
        f"'{start_time.strftime('%Y-%m-%dT%H:%M:%SZ')}' "
        "and eventTimestamp le "
        f"'{end_time.strftime('%Y-%m-%dT%H:%M:%SZ')}'"
    )
)

for item in event_data:
    print(item.event_name.localized_value, item.resource_group_name)
```

Common filters also include `resourceGroupName eq 'my-rg'`, `resourceProvider eq 'Microsoft.Compute'`, and `status eq 'Failed'`.

### List data collection rules in a resource group

Use data collection rules and data collection endpoints for Azure Monitor Agent and ingestion configuration. The `7.0.0` client includes first-class operation groups for them.

```python
rules = client.data_collection_rules.list_by_resource_group("example-rg")

for rule in rules:
    print(rule.name, rule.location, rule.id)
```

To inspect one rule directly:

```python
rule = client.data_collection_rules.get(
    resource_group_name="example-rg",
    data_collection_rule_name="example-dcr",
)

print(rule.immutable_id)
```

### Create or update a metric alert

Metric alerts are one of the most common control-plane tasks. Use model classes instead of building ad hoc dictionaries when the payload is non-trivial.

```python
from datetime import timedelta

from azure.mgmt.monitor.models import (
    MetricAlertAction,
    MetricAlertResource,
    MetricAlertSingleResourceMultipleMetricCriteria,
    MetricCriteria,
)

resource_group_name = "example-rg"
vm_id = (
    "/subscriptions/00000000-0000-0000-0000-000000000000/"
    "resourceGroups/example-rg/providers/Microsoft.Compute/virtualMachines/example-vm"
)
action_group_id = (
    "/subscriptions/00000000-0000-0000-0000-000000000000/"
    "resourceGroups/example-rg/providers/microsoft.insights/actionGroups/example-ag"
)

alert = client.metric_alerts.create_or_update(
    resource_group_name=resource_group_name,
    rule_name="vm-cpu-high",
    parameters=MetricAlertResource(
        location="global",
        description="Alert when average CPU is above 80 percent",
        severity=2,
        enabled=True,
        scopes=[vm_id],
        evaluation_frequency=timedelta(minutes=1),
        window_size=timedelta(minutes=5),
        target_resource_type="Microsoft.Compute/virtualMachines",
        target_resource_region="westus2",
        criteria=MetricAlertSingleResourceMultipleMetricCriteria(
            all_of=[
                MetricCriteria(
                    name="HighCpuCriterion",
                    metric_name="Percentage CPU",
                    metric_namespace="Microsoft.Compute/virtualMachines",
                    time_aggregation="Average",
                    operator="GreaterThan",
                    threshold=80,
                )
            ]
        ),
        actions=[MetricAlertAction(action_group_id=action_group_id)],
        auto_mitigate=True,
    ),
)

print(alert.id)
```

Practical notes:

- Metric alert resources live in Azure Monitor and typically use `location="global"`.
- `scopes` must contain full ARM resource IDs.
- `metric_namespace`, `target_resource_type`, and `target_resource_region` should match the resource being monitored.

### List scheduled query rules

Scheduled query rules are the management resource behind many log-search alerts:

```python
rules = client.scheduled_query_rules.list_by_resource_group("example-rg")

for rule in rules:
    print(rule.name, rule.enabled, rule.scopes)
```

If you are starting from an application requirement like "run a KQL query and return rows", that is not a scheduled query rule problem. Use `azure-monitor-query` instead.

### Configure diagnostic settings for a resource

Diagnostic settings attach log and metric export configuration to an existing Azure resource ID:

```python
resource_uri = (
    "/subscriptions/00000000-0000-0000-0000-000000000000/"
    "resourceGroups/example-rg/providers/Microsoft.Storage/storageAccounts/examplestorage"
)

settings = client.diagnostic_settings.list(resource_uri)

for item in settings.value:
    print(item.name)
```

This surface is commonly used to route logs and metrics to Log Analytics workspaces, storage accounts, or Event Hubs.

## Configuration Notes

- `subscription_id` is required. The client does not infer it from the credential.
- Management operations typically need Azure RBAC on the subscription, resource group, or target resource. Successful authentication is not enough.
- Most write operations use ARM resource IDs in payloads. When a call asks for a `scope`, `resource_uri`, `target_resource_id`, or `action_group_id`, pass the full `/subscriptions/...` resource ID.
- If you target sovereign clouds, keep the Azure Identity authority and the management endpoint aligned. Do not mix public-cloud credentials with a government or China ARM endpoint.

## Version-Sensitive Notes

### `7.0.0`

PyPI release metadata for `7.0.0` documents a large management-surface expansion. New operation groups called out there include:

- `action_groups`
- `activity_log_alerts`
- `activity_logs`
- `alert_rule_incidents`
- `alert_rules`
- `autoscale_settings`
- `baseline`
- `data_collection_endpoints`
- `data_collection_rule_associations`
- `data_collection_rules`
- `diagnostic_settings`
- `metrics`
- `scheduled_query_rules`

Treat older `azure-mgmt-monitor` examples with care. Blog posts or archived samples from the `1.x` to `5.x` era often cover a much smaller API surface and may not reflect current model names or operation groups.

### Prerelease `8.0.0b1` / `8.0.0b2`

PyPI currently lists `8.0.0b1` and `8.0.0b2` as prereleases. Unless your project is intentionally testing those betas, keep production guidance pinned to `7.0.0`.

## Common Pitfalls

- Using `azure-mgmt-monitor` when the task is actually logs query or metrics query at runtime
- Installing the management client but forgetting `azure-identity`
- Omitting `AZURE_SUBSCRIPTION_ID`
- Passing short names where the SDK expects full ARM resource IDs
- Treating `activity_logs.list()` like a simple list call and forgetting the required OData filter
- Copying old Azure Monitor samples that predate the `7.0.0` operation-group expansion
- Assuming every monitor resource belongs in the same Azure region; some alert resources are global while their target resources are regional

## Official Sources Used

- https://pypi.org/project/azure-mgmt-monitor/
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-monitor/
- https://learn.microsoft.com/en-us/python/api/overview/azure/mgmt-monitor-readme?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-monitor/azure.mgmt.monitor.monitormanagementclient?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-monitor/azure.mgmt.monitor.operations.activitylogsoperations?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-monitor/azure.mgmt.monitor.operations.datacollectionrulesoperations?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-monitor/azure.mgmt.monitor.operations.metricalertsoperations?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-monitor/azure.mgmt.monitor.models.metricalertresource?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-monitor/azure.mgmt.monitor.models.metricalertsingleresourcemultiplemetriccriteria?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-monitor/azure.mgmt.monitor.models.metriccriteria?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-monitor/azure.mgmt.monitor.models.metricalertaction?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-identity/azure.identity.defaultazurecredential?view=azure-python
