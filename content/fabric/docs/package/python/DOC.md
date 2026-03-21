---
name: package
description: "Fabric Python package for SSH command execution, file transfer, and task-oriented deployment automation"
metadata:
  languages: "python"
  versions: "3.2.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "fabric,python,ssh,remote-execution,deployment,invoke,paramiko"
---

# Fabric Python Package Guide

## Golden Rule

Use modern Fabric 3.x APIs and docs, not legacy Fabric 1.x examples. For new code, import `Connection`, `task`, and the group helpers from `fabric`, and drive task collections through the `fab` CLI or normal Python modules.

## Install

Pin the version your project expects:

```bash
python -m pip install "fabric==3.2.2"
```

Common alternatives:

```bash
uv add "fabric==3.2.2"
poetry add "fabric==3.2.2"
```

Fabric installs the `fab` command and depends on Invoke and Paramiko. You do not need a separate Fabric CLI package.

## Setup And Authentication

Fabric is an SSH orchestration library. In practice, the important setup is:

1. SSH access to the target host
2. Working host aliases, usernames, keys, and jump-host rules in OpenSSH config when needed
3. Optional Fabric config files for defaults shared across tasks

Fabric reads SSH config from the usual OpenSSH locations, so aliases such as `Host web-prod` or `ProxyJump bastion` should live in `~/.ssh/config`.

Minimal SSH-config-based usage:

```sshconfig
Host web-prod
    HostName web1.example.com
    User deploy
    IdentityFile ~/.ssh/id_ed25519
```

```python
from fabric import Connection

c = Connection("web-prod")
print(c.run("hostname", hide=True).stdout.strip())
```

If you need to pass credentials directly, use `connect_kwargs`:

```python
from fabric import Connection

c = Connection(
    "web1.example.com",
    user="deploy",
    connect_kwargs={
        "key_filename": "~/.ssh/id_ed25519",
    },
)
```

Config files are useful for defaults such as `run.warn`, `sudo.password`, or SSH behavior. Fabric supports system-level config like `/etc/fabric.yml`, per-user config like `~/.fabric.yaml`, and project config files such as `fabric.yaml` loaded from the project root.

Example `fabric.yaml`:

```yaml
run:
  warn: true
connect_kwargs:
  key_filename: ~/.ssh/id_ed25519
forward_agent: true
```

## Core Usage

### Run remote commands with `Connection`

Use `Connection` as the main unit of work:

```python
from fabric import Connection

with Connection("web-prod") as c:
    result = c.run("uname -a", hide=True)
    print(result.stdout.strip())
```

Useful methods:

- `run(...)`: execute a remote shell command
- `sudo(...)`: run with privilege escalation
- `put(...)`: upload a local file
- `get(...)`: download a remote file

Example deployment-style workflow:

```python
from fabric import Connection

with Connection("web-prod") as c:
    c.put("dist/app.tar.gz", "/tmp/app.tar.gz")
    c.run("mkdir -p /srv/myapp")
    c.run("tar -xzf /tmp/app.tar.gz -C /srv/myapp")
    c.sudo("systemctl restart myapp")
```

### Author reusable tasks with `@task`

Fabric builds on Invoke-style task collections. The first argument is a context object, conventionally named `c`.

```python
from fabric import task

@task
def deploy(c):
    c.run("git -C /srv/myapp pull")
    c.sudo("systemctl restart myapp")

@task
def logs(c, service="myapp", lines=100):
    c.sudo(f"journalctl -u {service} -n {lines} --no-pager")
```

Run them with `fab`:

```bash
fab -H web1.example.com deploy
fab -H web1.example.com logs --service=myapp --lines=200
fab --list
```

For quick one-off commands, `fab` also supports running arbitrary shell commands after `--`:

```bash
fab -H web1.example.com,web2.example.com -- uname -a
```

### Work with multiple hosts

Use groups when you want the same command across multiple hosts from Python:

```python
from fabric import ThreadingGroup

group = ThreadingGroup("web1.example.com", "web2.example.com", user="deploy")
results = group.run("hostname", hide=True)

for connection, result in results.items():
    print(connection.host, result.stdout.strip())
```

Use `SerialGroup` when you want deterministic sequential execution. Use `ThreadingGroup` when host fan-out speed matters more than ordered execution.

## CLI Layout

Fabric looks for task collections in a `fabfile.py` module or package. A minimal project layout:

```text
my-project/
  fabfile.py
```

```python
# fabfile.py
from fabric import task

@task
def ping(c):
    c.run("hostname")
```

```bash
fab -H web1.example.com ping
```

If the codebase already exposes tasks in a package, keep using that structure instead of forcing everything into one file.

## Configuration Notes

- Prefer SSH config for host aliases, usernames, ports, identity files, and bastions. It is easier to reuse across Fabric, `ssh`, `scp`, and CI runners.
- Prefer project-level `fabric.yaml` only for defaults that belong to the repo, such as `run.warn`, `forward_agent`, or command timeouts.
- Keep secrets out of repo config files. Use SSH agents, environment variables, secret managers, or CI-injected files.
- `Connection` inherits Invoke/Fabric config, so command behavior like warning-on-failure can be centralized instead of repeated on every `run(...)`.

## Common Pitfalls

- Do not copy Fabric 1.x code using `from fabric.api import env, run, sudo, roles`. Modern Fabric is a rewrite with different imports and task patterns.
- `@task` functions still need the leading context argument even if you do not use it directly.
- `Connection.run(...)` raises on non-zero exit status by default. Use `warn=True` only when you plan to inspect failures explicitly.
- Remote commands run through a shell on the target host. Quote arguments carefully instead of interpolating untrusted strings directly into shell commands.
- `fab -H ...` controls target hosts for CLI task runs; plain `fab deploy` without hosts only works if your tasks create their own connections or host defaults are configured.
- SSH config and host keys still matter. If plain `ssh web-prod` fails, Fabric will usually fail too.
- Group execution is useful for fan-out, but partial failures are normal. Write deployment tasks so they surface which host failed and why.

## Version-Sensitive Notes

- The official docs explicitly split legacy `v1.x` documentation from the modern line. Treat blog posts or snippets using Fabric 1 APIs as incompatible unless you are maintaining an old codebase on purpose.
- Fabric `3.2.2` is a patch release in the current major line. The official changelog notes fixes around shell completion packaging and compatibility with newer Invoke behavior, so pinning to `3.2.2` is safer than copying guidance from early `3.2.0` posts.
- Fabric is tightly coupled to Invoke and Paramiko behavior. If task parsing or SSH authentication changes unexpectedly after an upgrade, verify the exact Fabric version first before debugging your task code.

## Official Sources Used For This Doc

- Main docs: `https://docs.fabfile.org/en/stable/`
- Getting started: `https://docs.fabfile.org/en/stable/getting-started.html`
- Configuration concepts: `https://docs.fabfile.org/en/stable/concepts/configuration.html`
- Connection API: `https://docs.fabfile.org/en/stable/api/connection.html`
- Group API: `https://docs.fabfile.org/en/stable/api/group.html`
- PyPI package page: `https://pypi.org/project/fabric/`
- Changelog: `https://www.fabfile.org/changelog.html`
