---
name: package
description: "Kubernetes Python client for cluster API access, watches, exec and attach, YAML apply helpers, and custom resources"
metadata:
  languages: "python"
  versions: "35.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "kubernetes,k8s,python,cluster,containers,watch,custom-resources"
---

# Kubernetes Python Client Package Guide

## Golden Rule

Use the official `kubernetes` package, load configuration before creating API objects, and keep the client version aligned with the cluster minor version when possible. For this package line, `35.y.z` maps to Kubernetes `1.35`. For exec, attach, or port-forward style calls, use `kubernetes.stream.stream(...)` instead of calling the generated method directly, and recreate the API client afterward because the stream helper changes the transport layer.

## Install

Pin the client version explicitly when you need predictable generated API behavior:

```bash
python -m pip install "kubernetes==35.0.0"
uv add "kubernetes==35.0.0"
poetry add "kubernetes==35.0.0"
```

Optional extras published on PyPI:

```bash
python -m pip install "kubernetes[google-auth]==35.0.0"
python -m pip install "kubernetes[adal]==35.0.0"
```

Use those extras only if your kubeconfig or environment relies on those auth providers.

## Authentication And Setup

### Load local kubeconfig

Use this for development machines, CI jobs with a kubeconfig file, or admin scripts running outside the cluster:

```python
from kubernetes import client, config

config.load_kube_config(context="my-context")

core = client.CoreV1Api()
for namespace in core.list_namespace().items:
    print(namespace.metadata.name)
```

Pass `context=...` when the current kubeconfig context is not the one you want to target.

### Load in-cluster credentials

Use this only from code running inside Kubernetes with a mounted service account token and CA bundle:

```python
from kubernetes import client, config

config.load_incluster_config()

core = client.CoreV1Api()
pod = core.read_namespaced_pod(name="my-pod", namespace="default")
print(pod.metadata.name)
```

### Configure the API client directly

Use explicit configuration when you are not using kubeconfig or in-cluster auth:

```python
from kubernetes import client

cfg = client.Configuration()
cfg.host = "https://my-api-server:6443"
cfg.api_key = {"authorization": "YOUR_TOKEN"}
cfg.api_key_prefix = {"authorization": "Bearer"}
cfg.ssl_ca_cert = "/path/to/cluster-ca.crt"

api_client = client.ApiClient(cfg)
core = client.CoreV1Api(api_client)
```

Prefer a real cluster CA certificate over disabling TLS verification.

## Core Usage

### List pods in a namespace

```python
from kubernetes import client, config

config.load_kube_config()
core = client.CoreV1Api()

pods = core.list_namespaced_pod(namespace="default")
for pod in pods.items:
    print(pod.metadata.name, pod.status.phase)
```

### Create a deployment with the typed API

Generated API groups are versioned. Use the matching class such as `AppsV1Api`, `BatchV1Api`, or `NetworkingV1Api` instead of looking for a generic API surface.

```python
from kubernetes import client, config

config.load_kube_config()
apps = client.AppsV1Api()

deployment = client.V1Deployment(
    metadata=client.V1ObjectMeta(name="demo"),
    spec=client.V1DeploymentSpec(
        replicas=1,
        selector=client.V1LabelSelector(match_labels={"app": "demo"}),
        template=client.V1PodTemplateSpec(
            metadata=client.V1ObjectMeta(labels={"app": "demo"}),
            spec=client.V1PodSpec(
                containers=[
                    client.V1Container(
                        name="demo",
                        image="nginx:1.27",
                        ports=[client.V1ContainerPort(container_port=80)],
                    )
                ]
            ),
        ),
    ),
)

apps.create_namespaced_deployment(namespace="default", body=deployment)
```

### Watch for changes

Use `watch.Watch().stream(...)` for event streams rather than polling list endpoints in a loop:

```python
from kubernetes import client, config, watch

config.load_kube_config()
core = client.CoreV1Api()
w = watch.Watch()

for event in w.stream(core.list_namespaced_pod, namespace="default", timeout_seconds=30):
    obj = event["object"]
    print(event["type"], obj.metadata.name, obj.status.phase)
```

Always set `timeout_seconds` or stop the watcher explicitly.

### Work with CRDs through `CustomObjectsApi`

Custom resources are usually handled as dictionaries, not generated model classes:

```python
from kubernetes import client, config

config.load_kube_config()
custom = client.CustomObjectsApi()

widget = custom.get_namespaced_custom_object(
    group="example.com",
    version="v1",
    namespace="default",
    plural="widgets",
    name="widget-1",
)

print(widget["metadata"]["name"])
```

This is the normal path for CRDs unless you generate and maintain your own typed client.

### Apply YAML manifests

Use the helper in `kubernetes.utils` when you want the client to create resources from an existing YAML file:

```python
from kubernetes import client, config, utils

config.load_kube_config()
api_client = client.ApiClient()

utils.create_from_yaml(
    api_client,
    "manifests/demo.yaml",
    namespace="default",
)
```

If the manifest contains cluster-scoped resources, do not force a namespace argument onto all objects.

### Exec into a pod

Use `stream(...)` around the generated exec method:

```python
from kubernetes import client, config
from kubernetes.stream import stream

config.load_kube_config()
core = client.CoreV1Api()

output = stream(
    core.connect_get_namespaced_pod_exec,
    "example-pod",
    "default",
    command=["/bin/sh", "-c", "echo hello"],
    stderr=True,
    stdin=False,
    stdout=True,
    tty=False,
)

print(output)
```

After using `stream(...)`, recreate the API object before making normal API calls:

```python
core = client.CoreV1Api()
```

## Error Handling

The client raises `kubernetes.client.exceptions.ApiException` for API failures:

```python
from kubernetes import client, config
from kubernetes.client.exceptions import ApiException

config.load_kube_config()
core = client.CoreV1Api()

try:
    core.read_namespaced_pod("missing-pod", "default")
except ApiException as exc:
    if exc.status == 404:
        print("pod not found")
    else:
        raise
```

For debugging, inspect `exc.status`, `exc.reason`, and `exc.body`.

## Configuration And Auth Notes

- `load_kube_config()` is the normal out-of-cluster path; `load_incluster_config()` is the normal in-cluster path.
- Service account RBAC still applies in-cluster. A mounted token does not bypass namespace or resource permissions.
- Kubeconfig auth flows may depend on optional extras such as `google-auth` or `adal`, or on external exec credential plugins installed separately on the machine.
- TLS failures are usually a real config problem. Fix the host name, CA bundle, or dependency mismatch rather than turning off verification.
- If you need separate credentials or clusters in one process, create separate `client.Configuration()` and `client.ApiClient()` instances instead of assuming a single global default is safe for all call sites.

## Common Pitfalls

- Client and cluster minors are not interchangeable. `35.y.z` is the exact match for Kubernetes `1.35`; other nearby minors are only partial matches in the upstream compatibility table.
- For exec or attach, calling the generated method directly is wrong. Wrap it with `stream(...)`.
- After `stream(...)`, reuse of the same API client can break ordinary operations because the helper changes the protocol handling.
- Watches do not end on their own in long-running processes. Set timeouts and restart loops deliberately.
- CRDs do not magically show up as generated Python classes. Use `CustomObjectsApi` unless you maintain your own generated client for that API group.
- Upstream troubleshooting still calls out `ssl.CertificateError` host mismatches when older dependency combinations are in play, especially around `ipaddress` and `urllib3`.
- On macOS, older system Python and OpenSSL setups can cause SSL problems. Use a supported Python build if TLS behavior looks inconsistent.

## Version-Sensitive Notes For `35.0.0`

- PyPI lists `35.0.0` as a stable release published on `2026-01-16`.
- The upstream compatibility matrix marks `35.y.z` as the exact match for Kubernetes `1.35`, with `34.y.z` and `36.y.z` shown as partial matches around that line.
- The project README still documents the `stream(...)` transport mutation caveat, so treat it as current behavior for this version.
- The generated docs root for this release line is `https://kubernetes.readthedocs.io/en/stable/`.

## Official Sources

- Repository and README: `https://github.com/kubernetes-client/python`
- PyPI package metadata: `https://pypi.org/project/kubernetes/`
- Generated docs root: `https://kubernetes.readthedocs.io/en/stable/`
- Config docs: `https://kubernetes.readthedocs.io/en/stable/kubernetes.config.html`
- Utils docs: `https://kubernetes.readthedocs.io/en/stable/kubernetes.utils.html`
- Watch docs: `https://kubernetes.readthedocs.io/en/stable/kubernetes.watch.html`
- Exec example: `https://github.com/kubernetes-client/python/blob/master/examples/pod_exec.py`
