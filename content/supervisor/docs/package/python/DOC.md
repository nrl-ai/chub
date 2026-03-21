---
name: package
description: "Supervisor Python package guide for managing long-running UNIX processes with supervisord and supervisorctl"
metadata:
  languages: "python"
  versions: "4.3.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "supervisor,process-manager,unix,daemon,xml-rpc,supervisord,supervisorctl"
---

# Supervisor Python Package Guide

## Golden Rule

Use `supervisor` when you need a process-control system for long-running UNIX processes, and make sure the managed command stays in the foreground. Supervisor expects to own process lifecycle itself; if the child daemonizes, forks away, or double-backgrounds, Supervisor will treat it as a failed start.

## Install

Pin the version you expect:

```bash
python -m pip install "supervisor==4.3.0"
```

Common alternatives:

```bash
uv add "supervisor==4.3.0"
poetry add "supervisor==4.3.0"
```

Useful verification:

```bash
supervisord --version
echo_supervisord_conf | head
```

Version-sensitive packaging note:

- The `4.3.0` release notes say Python `<3.8` may still require `setuptools` to be installed, while Python `>=3.8` no longer needs it at runtime.

## Initialize And Start

Generate a starting config and edit it:

```bash
mkdir -p .supervisor
echo_supervisord_conf > .supervisor/supervisord.conf
```

Minimal practical config:

```ini
[unix_http_server]
file=/tmp/supervisor.sock
chmod=0700

[supervisord]
logfile=/tmp/supervisord.log
pidfile=/tmp/supervisord.pid
childlogdir=/tmp

[rpcinterface:supervisor]
supervisor.rpcinterface_factory = supervisor.rpcinterface:make_main_rpcinterface

[supervisorctl]
serverurl=unix:///tmp/supervisor.sock

[program:web]
command=/Users/me/app/.venv/bin/gunicorn myapp.wsgi:application --bind 127.0.0.1:8000
directory=/Users/me/app
autostart=true
autorestart=unexpected
startsecs=5
stopasgroup=true
killasgroup=true
stdout_logfile=/Users/me/app/logs/web.stdout.log
stderr_logfile=/Users/me/app/logs/web.stderr.log
environment=APP_ENV="production",PYTHONUNBUFFERED="1"
```

Start Supervisor with an explicit config path:

```bash
supervisord -c "$PWD/.supervisor/supervisord.conf"
```

Check status:

```bash
supervisorctl -c "$PWD/.supervisor/supervisord.conf" status
```

Why the explicit `-c` matters:

- Supervisor searches several default config locations if `-c` is omitted.
- The docs warn that using a config in the current directory without `-c` is a security risk when running as root.
- Agents should use an absolute path and keep the `supervisord` and `supervisorctl` config path aligned.

## Core Usage

### Manage one or more programs

Supervisor controls processes declared in `[program:x]` sections:

```ini
[program:worker]
command=/Users/me/app/.venv/bin/python worker.py
directory=/Users/me/app
user=appuser
autostart=true
autorestart=unexpected
startretries=3
redirect_stderr=true
stdout_logfile=/Users/me/app/logs/worker.log
```

Key behaviors:

- `command` should start exactly one long-running foreground process.
- `directory` sets the child working directory.
- `user` drops privileges for the child process if `supervisord` has permission to do so.
- `autorestart=unexpected` is usually safer than unconditional `true` for services that deliberately exit on certain conditions.
- `stopasgroup=true` and `killasgroup=true` help when the child spawns subprocesses and you want signals to reach the whole process group.

### Apply config changes correctly

After editing config:

```bash
supervisorctl -c "$PWD/.supervisor/supervisord.conf" reread
supervisorctl -c "$PWD/.supervisor/supervisord.conf" update
```

Common control commands:

```bash
supervisorctl -c "$PWD/.supervisor/supervisord.conf" status
supervisorctl -c "$PWD/.supervisor/supervisord.conf" start web
supervisorctl -c "$PWD/.supervisor/supervisord.conf" restart worker
supervisorctl -c "$PWD/.supervisor/supervisord.conf" tail -f web
supervisorctl -c "$PWD/.supervisor/supervisord.conf" shutdown
```

Important distinction:

- `reread` detects added, changed, or removed program config.
- `update` applies those changes.
- Editing the file alone does not restart running processes.

### Group related processes

When you want several named processes of the same shape, use `numprocs` and templates:

```ini
[program:queue]
process_name=%(program_name)s_%(process_num)02d
numprocs=2
command=/Users/me/app/.venv/bin/python worker.py --queue=%(process_num)s
directory=/Users/me/app
autostart=true
autorestart=unexpected
stdout_logfile=/Users/me/app/logs/queue_%(process_num)s.log
redirect_stderr=true
```

## XML-RPC API

Supervisor exposes an XML-RPC interface through the control socket or HTTP server. The default RPC namespace is `supervisor`.

If you need Python-side automation from another process, an HTTP listener is the simplest transport:

```ini
[inet_http_server]
port=127.0.0.1:9001
username=admin
password=change-me

[supervisorctl]
serverurl=http://127.0.0.1:9001
username=admin
password=change-me
```

Then call the API at `/RPC2`:

```python
from xmlrpc.client import ServerProxy

server = ServerProxy("http://admin:change-me@127.0.0.1:9001/RPC2")

print(server.supervisor.getSupervisorVersion())
print(server.supervisor.getState())
print(server.supervisor.getAllProcessInfo())
server.supervisor.startProcess("web", False)
server.supervisor.stopProcess("web", False)
```

Common useful methods from the official API docs:

- `getSupervisorVersion()`
- `getState()`
- `getAllProcessInfo()`
- `startProcess(name, wait)`
- `stopProcess(name, wait)`
- `tailProcessStdoutLog(name, offset, length)`
- `tailProcessStderrLog(name, offset, length)`

## Config And Auth Notes

Prefer a UNIX domain socket for local control:

```ini
[unix_http_server]
file=/tmp/supervisor.sock
chmod=0700
```

Use `inet_http_server` only when you specifically need TCP access. If you enable it:

- Bind to `127.0.0.1`, not `0.0.0.0`, unless you have a separate trusted tunnel or network boundary.
- Set `username` and `password`.
- Keep `supervisorctl` pointed at the same server URL and credentials.
- Treat the listener as administrative access to process control, not as a public endpoint.

Environment handling details that matter in practice:

- Child processes inherit Supervisor's environment plus values from the `environment=` setting.
- Use absolute paths for `command`, log files, and `directory` to avoid surprises when running under init systems or containers.
- If the child buffers logs heavily, set `PYTHONUNBUFFERED=1` or equivalent in `environment=`.

## Common Pitfalls

- Do not point Supervisor at a daemonizing command unless you pass that program's foreground flag such as `--nodaemon`, `--foreground`, or the equivalent for that service.
- Do not assume `reread` alone reloads changed processes; use `update` after it.
- Do not expose `[inet_http_server]` on a public interface. The official docs describe it as intended only for trusted environments.
- Do not forget `[rpcinterface:supervisor]`; XML-RPC clients and `supervisorctl` rely on that interface existing.
- Do not use mismatched socket settings between `[unix_http_server]` and `[supervisorctl]`; `supervisorctl` will fail to connect even when `supervisord` is healthy.
- The XML-RPC endpoint path is `/RPC2`, not `/`.
- If a process exits immediately with `spawnerr`, inspect the command path, working directory, file permissions, and whether the target program is trying to daemonize itself.

## Version-Sensitive Notes For 4.3.0

- `4.3.0` fixes an issue where a spawned process that exited immediately could incorrectly produce `spawnerr: unknown error making dispatchers for 'process_name': EINVAL` on Python `3.13`.
- The same release notes clarify that Python `<3.8` may still need `setuptools` at runtime, while Python `>=3.8` does not.
- The docs URL points directly at the API page. For package docs, the more useful canonical docs set is the root docs plus the running, configuration, subprocess, and API pages:
  - `https://supervisord.org/`
  - `https://supervisord.org/running.html`
  - `https://supervisord.org/configuration.html`
  - `https://supervisord.org/subprocess.html`
  - `https://supervisord.org/api.html`
