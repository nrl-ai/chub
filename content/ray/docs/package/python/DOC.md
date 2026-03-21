---
name: package
description: "Ray for Python: distributed tasks, actors, cluster connection, runtime environments, and jobs"
metadata:
  languages: "python"
  versions: "2.54.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "ray,python,distributed-computing,parallelism,actors,tasks,clusters"
---

# Ray Python Package Guide

## What Ray Is For

`ray` is a Python-first distributed execution framework. Use it when you need to:

- parallelize Python functions with Ray tasks
- keep state in distributed workers with Ray actors
- scale the same code from local development to a Ray cluster
- package code and Python dependencies for remote execution with `runtime_env`

The Python import is:

```python
import ray
```

## Install

Use the base package unless you specifically need the dashboard, cluster launcher, or higher-level libraries:

```bash
pip install -U ray
```

Useful extras from the official install guide:

```bash
pip install -U "ray[default]"
pip install -U "ray[data]"
pip install -U "ray[train]"
pip install -U "ray[tune]"
pip install -U "ray[serve]"
pip install -U "ray[all]"
```

Notes for `2.54.0`:

- PyPI lists `Requires: Python >=3.10`.
- The install docs list Python `3.10` through `3.13` as supported for prebuilt wheels.
- Native Windows support is still marked beta, and multi-node support on Windows is not well tested.

## Initialize Ray

### Local runtime

Use plain `ray.init()` during development or when you want a local in-process Ray runtime:

```python
import ray

ray.init()

@ray.remote
def square(x: int) -> int:
    return x * x

result = ray.get(square.remote(12))
print(result)

ray.shutdown()
```

### Attach to an existing cluster

If a cluster is already running, connect explicitly instead of accidentally starting a new local runtime:

```python
import ray

ray.init(address="auto")
```

For Ray Client connections, use the `ray://` scheme:

```python
import ray

ray.init("ray://ray-head.example.com:10001")
```

If you need a quick local cluster for testing, the official CLI path is:

```bash
ray start --head
```

Then connect with `ray.init(address="auto")`.

## Core Usage Patterns

### Stateless parallel work with tasks

Decorate a function with `@ray.remote`, call `.remote(...)`, and resolve the returned `ObjectRef` with `ray.get(...)`:

```python
import ray

ray.init()

@ray.remote
def transform(x: int) -> int:
    return x * 10

refs = [transform.remote(i) for i in range(5)]
results = ray.get(refs)
print(results)  # [0, 10, 20, 30, 40]
```

Task options are set with `.options(...)` before `.remote(...)`:

```python
fast_ref = transform.options(num_cpus=1).remote(7)
print(ray.get(fast_ref))
```

### Stateful workers with actors

Use actors when state must live on the worker:

```python
import ray

ray.init()

@ray.remote
class Counter:
    def __init__(self):
        self.value = 0

    def increment(self):
        self.value += 1
        return self.value

counter = Counter.remote()
print(ray.get(counter.increment.remote()))  # 1
print(ray.get(counter.increment.remote()))  # 2
```

### Share large objects through the object store

Avoid closing over large Python values in every task. Put them once, then pass the `ObjectRef`:

```python
import ray

ray.init()

large_lookup = {str(i): i for i in range(100_000)}
lookup_ref = ray.put(large_lookup)

@ray.remote
def read_key(table, key):
    return table[key]

print(ray.get(read_key.remote(lookup_ref, "42")))
```

## Dependencies and Runtime Environment

Remote workers need access to your code and Python dependencies. Ray’s `runtime_env` is the official way to package that execution environment.

Common pattern:

```python
import ray

ray.init(
    runtime_env={
        "working_dir": ".",
        "pip": ["requests==2.32.5", "pydantic>=2,<3"],
    }
)
```

Practical guidance:

- Use `working_dir` so workers can import your local project code.
- Use `pip` in `runtime_env` for packages that are not already installed on every node.
- Keep remote functions and actor classes importable from normal Python modules. Top-level definitions are safer than nested closures.
- If you are already using a cluster image or container with dependencies preinstalled, keep `runtime_env` minimal and use it only for per-job differences.

## Config and Auth

### Address and cluster selection

Be explicit about where code should run:

- `ray.init()` starts or attaches to a local runtime.
- `ray.init(address="auto")` attaches to an already running cluster discovered from the local environment.
- `ray.init("ray://host:10001")` uses Ray Client to connect to a remote cluster.

### Dashboard and jobs endpoint

The docs use `http://127.0.0.1:8265` as the default dashboard and jobs endpoint. That surface is commonly used for Ray Jobs submission and cluster inspection.

### Token auth for jobs and dashboard APIs

Local development usually has no auth layer. For shared or remote clusters, newer Ray releases support token authentication for job and dashboard APIs.

If token auth is enabled on the cluster, the official docs show these environment variables:

```bash
export RAY_JOB_HEADERS='{"Authorization": "Bearer <token>"}'
export RAY_AUTH_TOKEN="<token>"
```

Use those only when the cluster is configured for auth. They are not required for a default local setup.

## Common Pitfalls

- `ObjectRef` is not the result value. Call `ray.get(...)` before using the actual data.
- `ray.init()` without an explicit address can hide cluster-connection mistakes by starting a local runtime instead.
- Workers do not automatically inherit all local imports or files. Use `runtime_env`, package your code properly, or preinstall dependencies on every node.
- Large captured objects make tasks slower to serialize and schedule. Use `ray.put(...)`, files, or object storage instead of repeatedly closing over large data.
- Actor methods are invoked with `.remote()`, just like tasks. `counter.increment()` is a normal Python method call and will not run remotely.
- Re-running cells or scripts without cleanup can leave old Ray state around. `ray.shutdown()` is the safe reset when iterating locally.

## Version-Sensitive Notes for `2.54.0`

- The canonical docs root `https://docs.ray.io/en/latest/` currently serves the Ray `2.54.0` docs.
- PyPI for `ray 2.54.0` requires Python `>=3.10`.
- The install guide says support for Pydantic v1 will be dropped in Ray `2.56`, so prefer Pydantic v2-compatible code paths now.
- Dashboard and jobs token auth is documented as available starting in Ray `2.52.0`; if you are upgrading older cluster tooling, check those auth assumptions explicitly.

## Official Sources

- Docs root: https://docs.ray.io/en/latest/
- Installation: https://docs.ray.io/en/latest/ray-overview/installation.html
- Getting started / tasks / actors: https://docs.ray.io/en/latest/ray-overview/getting-started.html
- `ray.init()` API: https://docs.ray.io/en/latest/ray-core/api/doc/ray.init.html
- Runtime environments: https://docs.ray.io/en/latest/ray-core/handling-dependencies.html
- Ray Client: https://docs.ray.io/en/latest/cluster/running-applications/job-submission/ray-client.html
- Jobs and dashboard auth: https://docs.ray.io/en/latest/cluster/running-applications/job-submission/security.html
- PyPI package: https://pypi.org/project/ray/2.54.0/
