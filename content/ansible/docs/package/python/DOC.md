---
name: package
description: "Ansible community package guide for Python-based automation with inventories, playbooks, collections, and vault"
metadata:
  languages: "python"
  versions: "13.4.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "ansible,automation,devops,playbooks,inventory,ssh,vault"
---

# Ansible Python Package Guide

## Golden Rule

Use the `ansible` PyPI package when you want the full Ansible community distribution, but remember that it is primarily a CLI/tooling package, not a Python application library. Pin the package version you expect, keep playbooks in a project directory with a local `ansible.cfg`, and prefer fully qualified collection names such as `ansible.builtin.ping` in tasks and ad hoc commands.

## Install

For project-local automation, use a virtual environment and pin the package:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "ansible==13.4.0"
```

Common alternatives:

```bash
pipx install "ansible==13.4.0"
uv tool install "ansible==13.4.0"
```

Check what you installed:

```bash
ansible-community --version
ansible --version
```

`ansible-community --version` reports the community package version. `ansible --version` reports the underlying `ansible-core` version and config search path, which is useful but easy to misread during debugging.

## Initialize A Project

Keep Ansible files together so the local `ansible.cfg` wins over user-global defaults:

```text
automation/
  ansible.cfg
  inventory.yml
  site.yml
  group_vars/
    web.yml
  collections/
    requirements.yml
```

Minimal `ansible.cfg`:

```ini
[defaults]
inventory = ./inventory.yml
host_key_checking = True
interpreter_python = auto_silent
stdout_callback = yaml

[ssh_connection]
pipelining = True
```

Minimal YAML inventory:

```yaml
all:
  children:
    web:
      hosts:
        web-1.example.com:
          ansible_user: deploy
          ansible_ssh_private_key_file: ~/.ssh/deploy_ed25519
    local:
      hosts:
        localhost:
          ansible_connection: local
```

Sanity-check the inventory and connection before writing a large playbook:

```bash
ansible-inventory --graph
ansible web -m ansible.builtin.ping
```

## Core Usage

### Run an ad hoc command

Ad hoc commands are useful for verifying connectivity or gathering a quick fact:

```bash
ansible web -m ansible.builtin.command -a "uname -a"
ansible web -m ansible.builtin.setup -a "filter=ansible_distribution*"
```

Use modules instead of raw shell commands when a built-in module exists. That keeps runs more predictable and idempotent.

### Run a playbook

```yaml
# site.yml
- name: Configure web hosts
  hosts: web
  become: true
  tasks:
    - name: Ensure nginx is installed
      ansible.builtin.package:
        name: nginx
        state: present

    - name: Ensure nginx is enabled and started
      ansible.builtin.service:
        name: nginx
        state: started
        enabled: true
```

```bash
ansible-playbook site.yml
ansible-playbook site.yml --check --diff
```

Useful validation commands:

```bash
ansible-playbook site.yml --syntax-check
ansible-lint site.yml
```

### Use roles and collections explicitly

Ansible 13 docs consistently favor FQCNs. Install required collections from a checked-in requirements file:

```yaml
# collections/requirements.yml
collections:
  - name: community.general
  - name: ansible.posix
```

```bash
ansible-galaxy collection install -r collections/requirements.yml
```

If your playbook depends on content outside the `ansible` community package, install that collection up front instead of assuming it is already present on the control node.

## Configuration And Authentication

### Config precedence

Ansible looks for configuration in this order:

1. `ANSIBLE_CONFIG`
2. `ansible.cfg` in the current directory
3. `~/.ansible.cfg`
4. `/etc/ansible/ansible.cfg`

For reproducible automation, prefer a repo-local config and avoid relying on workstation-global settings.

### SSH and privilege escalation

The common POSIX path is SSH plus `become`:

```yaml
- name: Update apt cache
  hosts: web
  become: true
  tasks:
    - name: Refresh package metadata
      ansible.builtin.apt:
        update_cache: true
```

Useful host or group variables:

- `ansible_user`
- `ansible_host`
- `ansible_port`
- `ansible_ssh_private_key_file`
- `ansible_become`
- `ansible_become_user`
- `ansible_python_interpreter`

Set `ansible_python_interpreter` when the remote host does not expose Python at the default path Ansible discovers.

### Secrets with Vault

Do not hard-code secrets in playbooks or plain inventory vars. Encrypt them with Vault:

```bash
ansible-vault encrypt group_vars/web.yml
ansible-vault edit group_vars/web.yml
ansible-playbook site.yml --ask-vault-pass
```

For CI, prefer a vault password file or secret manager wiring over interactive prompts.

## Common Pitfalls

- `ansible` is not the same package as `ansible-core`. The community package includes collections and tracks its own version line.
- `ansible --version` shows the core engine version, not the community package version. Use `ansible-community --version` when checking the `13.4.0` package line.
- The public docs use `latest` paths and the older source URL `https://docs.ansible.com/ansible/latest/` redirects into `https://docs.ansible.com/projects/ansible/latest/`. That root can drift after future releases.
- Use FQCNs like `ansible.builtin.copy` and `community.general.ufw`. Short names are more likely to collide across collections.
- Many modules need Python on the managed host. Minimal Linux images often fail until Python is installed or `ansible_python_interpreter` is set correctly.
- `command` and `shell` are not idempotent. Prefer purpose-built modules such as `package`, `service`, `template`, `copy`, `user`, or cloud-provider modules.
- A local `ansible.cfg` in a world-writable directory is ignored for security reasons. If config changes do not seem to apply, inspect `ansible --version` output and the current working directory permissions.
- Inventory parsing issues often look like auth failures. Run `ansible-inventory --list` or `--graph` before assuming the SSH key is wrong.

## Version-Sensitive Notes For 13.4.0

- PyPI lists `ansible 13.4.0` as the current release, published on `2026-02-24`.
- The `ansible` package release and maintenance page shows `13.x` as the latest major line and notes that only the latest major release of `ansible` is maintained.
- The same release page maps `ansible 13` to `ansible-core 2.20`.
- Current installation docs require Python `3.12` or newer for the control node.
- Because the docs root is `latest`, always validate the package version against PyPI or the Ansible release and maintenance table before copying examples into pinned automation.

## Official Sources

- Docs root: `https://docs.ansible.com/projects/ansible/latest/`
- Installation guide: `https://docs.ansible.com/projects/ansible/latest/installation_guide/index.html`
- Getting started: `https://docs.ansible.com/projects/ansible/latest/getting_started/index.html`
- Config reference: `https://docs.ansible.com/projects/ansible/latest/reference_appendices/config.html`
- Release and maintenance: `https://docs.ansible.com/projects/ansible/latest/reference_appendices/release_and_maintenance.html`
- Collection install docs: `https://docs.ansible.com/projects/ansible/latest/collections_guide/collections_installing.html`
- PyPI release page: `https://pypi.org/project/ansible/13.4.0/`
