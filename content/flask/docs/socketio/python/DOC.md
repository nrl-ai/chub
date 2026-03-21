---
name: socketio
description: "Flask-SocketIO package guide for Flask realtime apps, event handlers, rooms, auth, testing, and deployment"
metadata:
  languages: "python"
  versions: "5.6.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask-socketio,flask,socketio,websocket,realtime,rooms,auth,gunicorn"
---

# Flask-SocketIO Python Package Guide

## What It Is

`Flask-SocketIO` adds a Socket.IO server to a Flask application. Use it when you want Flask request and session integration plus realtime events, rooms, broadcasts, acknowledgements, and compatibility with Socket.IO clients.

If you do not need Flask integration, use `python-socketio` directly. If you are building a new ASGI-native app, compare it with FastAPI or Starlette plus the lower-level Socket.IO server.

## Install

Pin the package version when you need the documented behavior exactly:

```bash
python -m pip install "Flask-SocketIO==5.6.1"
```

Pick an async model deliberately instead of letting dependency auto-detection surprise you:

```bash
# Threaded worker with WebSocket support
python -m pip install "Flask-SocketIO==5.6.1" simple-websocket gunicorn

# gevent
python -m pip install "Flask-SocketIO==5.6.1" gevent gevent-websocket gunicorn

# eventlet
python -m pip install "Flask-SocketIO==5.6.1" eventlet gunicorn
```

Practical guidance:

- `threading` plus `simple-websocket` is the safest default when your app or dependencies do not tolerate green threads.
- `gevent` is the preferred green-thread option in current upstream guidance.
- `eventlet` still works, but upstream now calls out that it is not actively maintained.
- If more than one async backend is installed, set `async_mode` explicitly so upgrades do not silently switch runtimes.

## Initialize It Correctly

Use an extension-style singleton and start the app with `socketio.run(app)` in local or simple deployments.

```python
from flask import Flask
from flask_socketio import SocketIO

socketio = SocketIO(
    cors_allowed_origins=["https://app.example.com"],
    async_mode="threading",
    logger=True,
    engineio_logger=False,
)

def create_app() -> Flask:
    app = Flask(__name__)
    app.config["SECRET_KEY"] = "replace-me"
    socketio.init_app(app, manage_session=True)
    return app

app = create_app()

if __name__ == "__main__":
    socketio.run(app, debug=True)
```

Important setup rules:

- Set `SECRET_KEY` before initializing the extension if you rely on Flask sessions.
- Prefer `socketio.init_app(app)` in app-factory projects.
- Use `socketio.run(app)` instead of `flask run`; the Flask CLI server does not provide production WebSocket support for this package.
- Keep `path="socket.io"` unless your reverse proxy or client uses a custom Engine.IO path.

## Core Event Patterns

### Connect, authenticate, join rooms, and acknowledge work

```python
from flask import request, session
from flask_socketio import emit, join_room

@socketio.on("connect")
def on_connect(auth):
    user_id = session.get("user_id")
    token = (auth or {}).get("token")

    if not user_id and token != "dev-token":
        return False

    personal_room = f"user:{user_id or request.sid}"
    join_room(personal_room)
    emit("connected", {"sid": request.sid, "room": personal_room})

@socketio.on("chat:send")
def on_chat_send(data):
    room = data["room"]
    body = data["body"]
    emit("chat:message", {"body": body}, to=room)
    return {"ok": True}
```

What matters:

- `connect(auth)` receives the optional auth payload sent by modern Socket.IO clients.
- Returning `False` from `connect` rejects the connection.
- `request.sid` is the connection id and the built-in personal room for that client.
- Returning a value from a handler sends an acknowledgement payload to the client callback.
- Use `to=room` for room broadcasts; omit it when replying only on the current namespace.

### Emit outside a handler

Inside an event handler, use `emit()` or `send()`. Outside the event context, call methods on the `socketio` instance.

```python
from flask import request

@socketio.on("jobs:start")
def start_job(data):
    room = request.sid
    socketio.start_background_task(run_job, room, data)
    return {"started": True}

def run_job(room: str, data: dict) -> None:
    socketio.sleep(2)
    socketio.emit("jobs:done", {"input": data, "result": "ok"}, to=room)
```

Use `socketio.start_background_task()` and `socketio.sleep()` instead of raw thread or greenlet primitives. They adapt to the selected async backend.

### Rooms and namespaces

- Use `join_room()`, `leave_room()`, and `close_room()` for room lifecycle.
- Keep room names application-defined and deterministic, such as `project:123` or `user:42`.
- Only create custom namespaces when you need separate event surfaces or auth rules. Rooms are enough for most multitenant realtime features.

## Config, Session, And Auth

The most important `SocketIO()` and `init_app()` options in real projects are:

- `async_mode`: `threading`, `eventlet`, `gevent`, or `gevent_uwsgi`.
- `cors_allowed_origins`: same-origin is the default; use an explicit allow-list in production.
- `message_queue`: required for multi-process or multi-host broadcasting.
- `logger` and `engineio_logger`: turn these on when debugging handshakes, transport upgrades, or disconnects.

### Session semantics are not the same as HTTP

- the Socket.IO side starts from the session state that existed when the connection was created
- if a Socket.IO handler mutates `session`, later Socket.IO events see that change
- normal Flask routes do not automatically see those Socket.IO-only mutations

If you use server-side sessions and avoid mutating the session inside Socket.IO handlers, later HTTP-side session updates can still be visible to Socket.IO handlers.

### Auth patterns that work

Use one of these patterns:

- authenticate over normal Flask HTTP routes first, then read `session` or `current_user` in Socket.IO handlers
- send an `auth` dictionary during the connection handshake and validate it in `connect`

Do not use Flask route decorators such as `login_required` directly on Socket.IO event handlers. If you need handler-level enforcement, write a small decorator that checks `current_user.is_authenticated` and calls `disconnect()` when the client is unauthorized.

## Deployment And Scaling

### Embedded server

For local development or simple deployments:

```python
socketio.run(app)
```

If `eventlet` or `gevent` is installed, `socketio.run(app)` uses a production-capable server for that backend. Without them, it falls back to Flask's development server.

### Gunicorn

Current upstream deployment examples include:

```bash
gunicorn --worker-class eventlet -w 1 module:app
gunicorn -k gevent -w 1 module:app
gunicorn -k geventwebsocket.gunicorn.workers.GeventWebSocketWorker -w 1 module:app
gunicorn -w 1 --threads 100 module:app
```

Practical guidance:

- Keep Gunicorn at `-w 1` for a single Flask-SocketIO process; Gunicorn's normal load balancing is not compatible with Socket.IO's stickiness requirements.
- Use the threaded worker plus `simple-websocket` when green threads are a poor fit.
- Use `gevent-websocket` only when you specifically want Gunicorn plus gevent-based WebSocket handling.

### Multiple workers or hosts

Horizontal scaling needs both:

- sticky sessions at the load balancer
- a message queue such as Redis, RabbitMQ, or Kafka

Typical Redis setup:

```python
socketio = SocketIO(
    app,
    cors_allowed_origins=["https://app.example.com"],
    message_queue="redis://",
)
```

If you use `eventlet` or `gevent` together with a queue client, patch early in process startup so queue I/O cooperates with the async runtime:

```python
# eventlet
import eventlet
eventlet.monkey_patch()
```

```python
# gevent
from gevent import monkey
monkey.patch_all()
```

Apply monkey patching before imports that open sockets or initialize queue clients.

## Testing

Use the built-in Socket.IO test client. Pass Flask's test client when you need cookies or Flask session state to carry into the Socket.IO connection.

```python
from flask import Flask
from flask_socketio import SocketIO

socketio = SocketIO()

def create_app():
    app = Flask(__name__)
    app.config["SECRET_KEY"] = "test-secret"
    socketio.init_app(app)
    return app

def test_ping_roundtrip():
    app = create_app()

    @socketio.on("ping")
    def handle_ping(data):
        return {"pong": data["value"]}

    flask_client = app.test_client()
    socket_client = socketio.test_client(app, flask_test_client=flask_client)

    assert socket_client.is_connected()
    assert socket_client.emit("ping", {"value": 1}, callback=True) == {"pong": 1}
    socket_client.disconnect()
```

Use `socketio.test_client()` instead of spinning up a real server for normal handler tests.

## Common Pitfalls

- `flask run` works for plain Flask routes, not for a production Flask-SocketIO server.
- Installing multiple async backends without setting `async_mode` can change behavior between environments.
- Running multiple Gunicorn workers without a message queue and sticky sessions causes missed broadcasts and broken rooms.
- Emitting from background jobs or external workers requires the `socketio` object plus a configured `message_queue`; context-local `emit()` only works inside an event handler.
- Cross-origin failures are often just missing `cors_allowed_origins` or a proxy that does not forward WebSocket upgrade headers.
- Mixing blocking code with `eventlet` or `gevent` can stall the whole process. Use `threading` when your code or libraries are not green-thread-friendly.

## Version-Sensitive Notes For 5.6.1

- `5.6.1` is in the Flask-SocketIO `5.x` line and uses the newer Socket.IO / Engine.IO protocol generation documented in the upstream version-compatibility table. Pair it with modern JavaScript Socket.IO clients, not old `2.x` clients.
- Upgrading from Flask-SocketIO `4.x` is not drop-in. The upstream upgrade guide calls out protocol and behavior changes, including explicit namespace connection behavior in newer Socket.IO clients.
- The official docs root is `latest`, not a patch-pinned docs tree. For `5.6.1`, use the PyPI version plus the upgrade notes together when checking older examples from blogs or issue threads.
- Current maintainer docs and PyPI metadata both require Python `3.8+`. Older `5.x` examples written for Python `3.6` or `3.7` are stale for `5.6.1`.

## Official Sources

- Docs root: https://flask-socketio.readthedocs.io/en/latest/
- Introduction and version compatibility: https://flask-socketio.readthedocs.io/en/latest/intro.html
- Getting started: https://flask-socketio.readthedocs.io/en/latest/getting_started.html
- Implementation notes: https://flask-socketio.readthedocs.io/en/latest/implementation_notes.html
- Deployment: https://flask-socketio.readthedocs.io/en/latest/deployment.html
- API reference: https://flask-socketio.readthedocs.io/en/latest/api.html
- Upgrade notes: https://flask-socketio.readthedocs.io/en/latest/upgrading.html
- PyPI: https://pypi.org/project/Flask-SocketIO/
- Repository: https://github.com/miguelgrinberg/Flask-SocketIO
