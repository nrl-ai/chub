---
name: package
description: "Salt package guide for Python infrastructure automation, master/minion setup, states, and Python API usage"
metadata:
  languages: "python"
  versions: "3007.13"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "salt,python,configuration-management,orchestration,remote-execution,states"
---

# Salt Python Package Guide

## Golden Rule

Use the official Salt packages for production `salt-master` and `salt-minion` installs, and treat `pip install salt` as development or embedded usage with only reasonable-effort support. When you use Salt's Python APIs, run them on the master or minion host they target, and on onedir installs use Salt's bundled Python runtime instead of the system interpreter.

## Install

Preferred production path: use Salt's official packages from the install guide so you get the supported service layout, bundled runtime, and service units.

For a virtualenv, CI image, or local tooling experiment, install from PyPI:

```bash
python -m pip install "salt==3007.13"
```

If you want a clean isolated environment:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "salt==3007.13"
```

Important install notes:

- Salt's install guide says `pip` installs receive only reasonable-effort support.
- Salt's official onedir packages ship their own Python runtime. In the 3007 STS line, the packaged runtime is Python `3.10.x`.
- If you install extra Python dependencies into an onedir Salt install, use Salt's bundled tooling and runtime, not the system `pip`.

## Initialize A Master And Minion

Salt's standard deployment model is a master that publishes jobs and one or more minions that execute them.

Master configuration lives in:

- `/etc/salt/master`
- `/etc/salt/master.d/*.conf`

Minion configuration lives in:

- `/etc/salt/minion`
- `/etc/salt/minion.d/*.conf`

Minimal minion configuration:

```yaml
# /etc/salt/minion.d/master.conf
master: salt-master.example.internal
id: web-01
```

By default, the master listens on ports `4505` and `4506` on all interfaces, so restrict network access accordingly.

Start the services, then accept the minion key on the master:

```bash
sudo systemctl enable --now salt-master
sudo systemctl enable --now salt-minion

sudo salt-key -L
sudo salt-key -a web-01
```

Smoke test the connection:

```bash
sudo salt "*" test.ping
```

## Masterless Setup

For local-only state application, use `salt-call --local` and set the minion to local file mode:

```yaml
# /etc/salt/minion.d/local.conf
file_client: local
file_roots:
  base:
    - /srv/salt
pillar_roots:
  base:
    - /srv/pillar
```

Then apply states directly on the host:

```bash
sudo salt-call --local state.apply
```

This is the simplest way to use Salt as a local configuration engine without standing up a master.

## Core Usage

### Remote execution

```bash
sudo salt "*" test.ping
sudo salt "web-*" cmd.run "whoami"
sudo salt "db-*" pkg.version postgresql
```

Targeting is a major part of Salt. Start with simple glob targets, then move to grains, pillars, or compound targeting only when the simpler matchers are insufficient.

### States

Salt state files live under the state tree, typically `/srv/salt`. The top file maps targets to states:

```yaml
# /srv/salt/top.sls
base:
  "web-*":
    - nginx
```

```yaml
# /srv/salt/nginx.sls
nginx:
  pkg.installed: []
  service.running:
    - enable: true
```

Apply all matching states:

```bash
sudo salt "*" state.apply
```

Apply a specific state:

```bash
sudo salt "web-01" state.apply nginx
```

Do not put pillar data under `file_roots`; keep it under `pillar_roots` so pillar targeting rules still apply.

### Python API from the master

The most useful Python entry point for automation on the master is `salt.client.LocalClient`:

```python
import salt.client

client = salt.client.LocalClient()

ping = client.cmd("web-*", "test.ping")
versions = client.cmd("web-*", "test.version")

print(ping)
print(versions)
```

A state run from Python:

```python
import salt.client

client = salt.client.LocalClient()

result = client.cmd("web-01", "state.apply", ["nginx"])
print(result)
```

`LocalClient` talks to the master's publish and return system, so it should run on the master host and under the same user context as the master process.

## Configuration And Authentication

Salt has two distinct auth layers that agents often blur together:

1. Master-to-minion trust uses key acceptance. A minion is not manageable until its key is accepted on the master.
2. Human or service access to Salt interfaces uses external authentication (`external_auth`) for CLI, API, runner, and wheel access.

Minimal `external_auth` example:

```yaml
# /etc/salt/master.d/eauth.conf
external_auth:
  pam:
    ci-bot:
      - "web-*":
        - test.*
        - state.apply
```

Example CLI call using external auth:

```bash
salt -a pam "web-*" test.version
```

If you expose a REST interface, configure auth and TLS explicitly. Salt's netapi docs distinguish lightweight WSGI integration from the more feature-complete CherryPy-based API app; do not expose either one with broad unauthenticated network access.

## Common Pitfalls

- `pip install salt` is not the primary supported deployment path for production daemons.
- On onedir installs, Python scripts that import `salt.*` must run with Salt's bundled Python runtime.
- `LocalClient` is a master-side API, not a general remote client you can run anywhere on the network.
- New minions will show up in `salt-key` but cannot execute jobs until their keys are accepted.
- The master listens on `4505` and `4506` on all interfaces by default. Do not leave those ports broadly exposed.
- Salt loads configuration from the main file and every `*.conf` file under the matching `.d/` directory. Duplicate settings can override each other unexpectedly.
- Keep pillar data in `pillar_roots`, not `file_roots`.
- The installation docs warn against running an open master on the public internet because it is a vulnerability.

## Version-Sensitive Notes For 3007.13

- As of `2026-03-12`, PyPI lists `salt 3007.13`, released on `2026-02-12`.
- The docs URL used `https://docs.saltproject.io/en/latest/ref/`, which is a moving stable-reference root. For this package doc, the canonical versioned docs root is `https://docs.saltproject.io/en/3007/`.
- Salt's 3007 STS install guide documents official packages with a bundled Python `3.10.x` runtime. Do not assume that packaged installs follow your system Python version just because the PyPI package supports `>=3.8`.
- Since `3006.0`, official Salt packages run the master as the `salt` user rather than `root`. Scripts, custom paths, and file permissions that assumed a root-owned master need to be checked on modern installs.
- `cluster_id` is new in the 3007 line for clustered masters. If you are building HA or clustered topologies, use the 3007 docs instead of older cluster guides.

## Official Sources

- Salt reference docs: `https://docs.saltproject.io/en/3007/`
- Salt install guide: `https://docs.saltproject.io/salt/install-guide/en/latest/`
- Salt Python client API: `https://docs.saltproject.io/en/3007/ref/clients/index.html`
- Salt master config reference: `https://docs.saltproject.io/en/3007/ref/configuration/master.html`
- Salt minion config reference: `https://docs.saltproject.io/en/3007/ref/configuration/minion.html`
- Salt states tutorial: `https://docs.saltproject.io/en/3007/topics/tutorials/starting_states.html`
- Salt top file reference: `https://docs.saltproject.io/en/3007/ref/states/top.html`
- Salt external auth tutorial: `https://docs.saltproject.io/en/latest/topics/eauth/index.html`
- Salt netapi docs: `https://docs.saltproject.io/en/latest/ref/netapi/all/index.html`
- PyPI package page: `https://pypi.org/project/salt/`
