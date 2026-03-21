---
name: package
description: "Docker SDK for Python package for connecting to Docker Engine, running containers, building images, and working with registries"
metadata:
  languages: "python"
  versions: "7.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "docker,containers,python,engine,registry,images"
---

# Docker SDK for Python Package Guide

## Golden Rule

Use the `docker` PyPI package as a client for an already running Docker Engine or Docker Desktop daemon. This package does not install Docker itself. In most projects, start with `docker.from_env()`, verify the daemon with `ping()`, and prefer `version="auto"` over relying on constructor defaults.

## Install

Pin the package version your project expects:

```bash
python -m pip install "docker==7.1.0"
```

Common alternatives:

```bash
uv add "docker==7.1.0"
poetry add "docker==7.1.0"
```

Optional websocket support for `attach_socket(..., ws=True)`:

```bash
python -m pip install "docker[websockets]==7.1.0"
```

Notes:

- `docker[tls]` is a no-op in current releases because TLS support ships in the main package.
- Installing this package is not enough by itself; Docker Engine or Docker Desktop must also be installed and running somewhere reachable.

## Connect To The Daemon

`docker.from_env()` is the normal entry point. It reads the same environment variables as the Docker CLI, including `DOCKER_HOST`, `DOCKER_TLS_VERIFY`, and `DOCKER_CERT_PATH`.

```python
import docker

client = docker.from_env()

try:
    client.ping()
    print("Docker daemon is reachable")
    print(client.version()["Version"])
finally:
    client.close()
```

If you need explicit connection settings, build the client yourself and pass `version="auto"`:

```python
import docker

client = docker.DockerClient(
    base_url="unix:///var/run/docker.sock",
    version="auto",
    timeout=60,
)
```

Remote daemon examples:

```python
import docker

client = docker.DockerClient(
    base_url="tcp://docker.example.internal:2376",
    version="auto",
    tls=True,
)
```

```python
import docker

client = docker.from_env(use_ssh_client=True)
```

Use the high-level `DockerClient` unless you need an endpoint that is only exposed through the low-level API. The low-level client is available as `client.api` or directly as `docker.APIClient(...)`.

## Authentication And Configuration

### Daemon connection config

Common environment variables:

- `DOCKER_HOST`: daemon URL such as `unix:///var/run/docker.sock`, `npipe:////./pipe/docker_engine`, `tcp://host:2376`, or `ssh://user@host`
- `DOCKER_TLS_VERIFY=1`: enable TLS verification for TCP connections
- `DOCKER_CERT_PATH`: directory containing client certs for TLS auth

Example with an explicit TLS config:

```python
import docker
from docker.tls import TLSConfig

tls_config = TLSConfig(
    client_cert=("/path/client-cert.pem", "/path/client-key.pem"),
    ca_cert="/path/ca.pem",
    verify=True,
)

client = docker.DockerClient(
    base_url="tcp://docker.example.internal:2376",
    version="auto",
    tls=tls_config,
)
```

### Registry auth

Log in before pulls or pushes that require credentials:

```python
import docker

client = docker.from_env()

client.api.login(
    username="registry-user",
    password="registry-password",
    registry="https://index.docker.io/v1/",
)
```

For one-off operations, you can also pass `auth_config={"username": "...", "password": "..."}` to image pull or push calls instead of mutating shared client state.

## Core Usage

### Run a one-off container

When `detach=False` (the default), `containers.run()` returns the container logs as bytes.

```python
import docker

client = docker.from_env()

output = client.containers.run(
    "alpine:3.20",
    ["echo", "hello from docker"],
    auto_remove=True,
)

print(output.decode("utf-8").strip())
client.close()
```

### Run a long-lived container

When `detach=True`, `containers.run()` returns a `Container` object instead.

```python
import docker

client = docker.from_env()

container = client.containers.run(
    "nginx:1.27-alpine",
    detach=True,
    name="example-nginx",
    ports={"80/tcp": 8080},
    environment={"NGINX_ENTRYPOINT_QUIET_LOGS": "1"},
)

try:
    print(container.status)
    container.reload()
    print(container.logs(tail=20).decode("utf-8"))
finally:
    container.remove(force=True)
    client.close()
```

`container.attrs` is cached. Call `container.reload()` before reading status or other mutable metadata that may have changed on the daemon.

### Execute a command in an existing container

```python
import docker

client = docker.from_env()
container = client.containers.run("python:3.12-alpine", ["sleep", "300"], detach=True)

try:
    result = container.exec_run(["python", "-c", "print('ok')"])
    print(result.exit_code)
    print(result.output.decode("utf-8").strip())
finally:
    container.remove(force=True)
    client.close()
```

### Pull and inspect images

```python
import docker
from docker.errors import ImageNotFound

client = docker.from_env()

try:
    image = client.images.pull("redis", tag="7-alpine")
    print(image.tags)
    print(client.images.get("redis:7-alpine").id)
except ImageNotFound:
    print("Image was not found in the registry")
finally:
    client.close()
```

### Build an image from a local directory

`images.build()` returns a tuple of `(image, build_logs)`.

```python
import docker

client = docker.from_env()

image, build_logs = client.images.build(
    path=".",
    tag="my-app:dev",
    rm=True,
)

for chunk in build_logs:
    if "stream" in chunk:
        print(chunk["stream"], end="")

print(image.tags)
client.close()
```

### Push an image and stream progress

For structured push progress, use the low-level API with `stream=True` and `decode=True`:

```python
import docker

client = docker.from_env()

for event in client.api.push(
    repository="yourname/my-app",
    tag="latest",
    stream=True,
    decode=True,
):
    print(event)

client.close()
```

## Common Errors And Pitfalls

- `docker` the Python package is not the Docker CLI and not the Docker daemon. If the daemon is not running, the SDK cannot fix that.
- On Linux, `/var/run/docker.sock` permission errors usually mean the current user cannot access the Docker socket.
- `containers.run(..., detach=False)` and many log APIs return bytes, not text. Decode them explicitly.
- Detached containers are not auto-cleaned unless you request it. Use `auto_remove=True` for one-off runs or call `container.remove(...)`.
- `container.attrs` is cached until `reload()`.
- A successful import does not prove the daemon is reachable. Use `client.ping()` early.
- If you run this code inside another container, mount the Docker socket or point the SDK at a remote daemon explicitly.
- Close clients when you are done so HTTP resources do not linger.

## Version-Sensitive Notes For 7.1.0

- PyPI lists `docker 7.1.0` for this package, and the stable docs site is the correct maintainer docs root for that line.
- The 7.1.0 release notes say the SDK bumped the default Docker Engine API version to `1.44` and the minimum supported Engine API version to `1.24`. If you must talk to very old daemons, `7.1.0` may be the wrong SDK version.
- The stable client reference still shows older constructor defaults in some places. Because of that source drift, prefer `docker.from_env()` or pass `version="auto"` explicitly instead of relying on implicit defaults.
- Since 7.0.0, `ssl_version` and `assert_hostname` were removed from `kwargs_from_env()`, `from_env()`, `DockerClient()`, and `APIClient()`. Do not copy examples that still pass those arguments.
- Since 7.0.0, `docker[tls]` no longer adds anything beyond the base package.
- The `websockets` extra is only needed for websocket-based attach support such as `attach_socket(..., ws=True)`.

## Official Sources

- Docs root: `https://docker-py.readthedocs.io/en/stable/`
- Client reference: `https://docker-py.readthedocs.io/en/stable/client.html`
- Containers guide: `https://docker-py.readthedocs.io/en/stable/containers.html`
- Images guide: `https://docker-py.readthedocs.io/en/stable/images.html`
- TLS guide: `https://docker-py.readthedocs.io/en/stable/tls.html`
- PyPI package metadata: `https://pypi.org/project/docker/`
- Maintainer release notes for 7.1.0: `https://github.com/docker/docker-py/releases/tag/7.1.0`
