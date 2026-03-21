---
name: login
description: "Flask-Login guide for user loading, login state, remember cookies, and route protection in Flask apps"
metadata:
  languages: "python"
  versions: "0.6.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,flask-login,authentication,session,login,remember-cookie"
---

# Flask-Login for Flask Apps

## Install

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "Flask-Login==0.6.3"
```

PyPI metadata for `0.6.3` requires:

- `Flask >=1.0.4`
- `Werkzeug >=1.0.1`

`0.6.3` specifically adds compatibility with Flask 3 and Werkzeug 3.

## Minimal Setup

```python
from flask import Flask, redirect, url_for
from flask_login import LoginManager, UserMixin, current_user, login_required

app = Flask(__name__)
app.config["SECRET_KEY"] = "replace-me"

login_manager = LoginManager()
login_manager.login_view = "login"
login_manager.init_app(app)

class User(UserMixin):
    def __init__(self, user_id: str, email: str) -> None:
        self.id = user_id
        self.email = email

USERS = {
    "1": User("1", "user@example.com"),
}

@login_manager.user_loader
def load_user(user_id: str):
    return USERS.get(user_id)

@app.get("/")
def index():
    if current_user.is_authenticated:
        return f"Hello, {current_user.email}"
    return redirect(url_for("login"))

@app.get("/settings")
@login_required
def settings():
    return "protected"
```

Important contract for your user class:

- `is_authenticated`
- `is_active`
- `is_anonymous`
- `get_id()`

If a user should not be allowed to sign in, make `is_active` return `False`.

## Login And Logout Flow

You validate credentials yourself, then call `login_user()`:

```python
from flask import abort, redirect, render_template, request, url_for
from flask_login import login_required, login_user, logout_user

def is_safe_redirect_target(target: str | None) -> bool:
    return not target or target.startswith("/")

@app.route("/login", methods=["GET", "POST"])
def login():
    if request.method == "GET":
        return render_template("login.html")

    email = request.form["email"]
    password = request.form["password"]
    remember = request.form.get("remember") == "on"

    user = authenticate(email, password)
    if user is None:
        abort(401)

    login_user(user, remember=remember)

    next_url = request.args.get("next")
    if not is_safe_redirect_target(next_url):
        abort(400)

    return redirect(next_url or url_for("index"))

@app.post("/logout")
@login_required
def logout():
    logout_user()
    return redirect(url_for("index"))
```

Do not blindly redirect to `request.args["next"]`. Validate it first or you create an open redirect.

If you want Flask-Login to keep the target URL in the session rather than the query string, set:

```python
app.config["USE_SESSION_FOR_NEXT"] = True
```

## Request Loading And API Behavior

### Session-backed login

`user_loader` is the normal path for browser sessions. It receives the string returned by `get_id()` and must return the matching user object or `None`.

Returning `None` matters: if the user no longer exists, Flask-Login removes the invalid ID from the session.

### Request-based login

For request-authenticated endpoints, use `request_loader`:

```python
import base64
from flask import request

@login_manager.request_loader
def load_user_from_request(request):
    api_key = request.args.get("api_key")
    if api_key:
        user = lookup_user_by_api_key(api_key)
        if user:
            return user

    auth_header = request.headers.get("Authorization")
    if auth_header and auth_header.startswith("Basic "):
        encoded = auth_header.removeprefix("Basic ")
        try:
            decoded = base64.b64decode(encoded).decode()
        except Exception:
            return None
        return lookup_user_by_api_key(decoded)

    return None
```

`header_loader` still exists in `0.6.3`, but the docs mark it deprecated. Prefer `request_loader` for new code.

### Unauthorized responses

If you set `login_view`, unauthorized browser requests redirect to that view. That is usually wrong for APIs, so add `unauthorized_handler`:

```python
from http import HTTPStatus
from flask import jsonify, redirect, request, url_for

@login_manager.unauthorized_handler
def unauthorized():
    if request.blueprint == "api":
        return jsonify(error="authentication required"), HTTPStatus.UNAUTHORIZED
    return redirect(url_for("login"))
```

If your app has different auth entry points per blueprint, use `login_manager.blueprint_login_views` instead of forcing one global login route.

## Fresh Logins For Sensitive Actions

Remembered sessions are not always "fresh". Use `@fresh_login_required` when a user must re-authenticate before a sensitive action:

```python
from flask_login import confirm_login, fresh_login_required, login_user

login_manager.refresh_view = "accounts.reauthenticate"
login_manager.needs_refresh_message = (
    "Please reauthenticate before changing sensitive settings."
)
login_manager.needs_refresh_message_category = "info"

@app.post("/account/reauth")
def reauth():
    user = authenticate_again(...)
    if user is None:
        return {"error": "bad credentials"}, 401
    confirm_login()
    return {"ok": True}

@app.get("/account/security")
@fresh_login_required
def account_security():
    return "fresh session required"
```

Use this for actions such as email changes, MFA setup, or billing changes.

## Remember-Me And Cookie Configuration

`login_user(..., remember=True)` issues a persistent remember cookie. Important settings from the `0.6.3` docs:

- `REMEMBER_COOKIE_NAME`
- `REMEMBER_COOKIE_DURATION`
- `REMEMBER_COOKIE_DOMAIN`
- `REMEMBER_COOKIE_PATH`
- `REMEMBER_COOKIE_SECURE`
- `REMEMBER_COOKIE_HTTPONLY`
- `REMEMBER_COOKIE_REFRESH_EACH_REQUEST`
- `REMEMBER_COOKIE_SAMESITE`

Example:

```python
from datetime import timedelta

app.config.update(
    REMEMBER_COOKIE_DURATION=timedelta(days=14),
    REMEMBER_COOKIE_SECURE=True,
    REMEMBER_COOKIE_HTTPONLY=True,
    REMEMBER_COOKIE_SAMESITE="Lax",
    REMEMBER_COOKIE_REFRESH_EACH_REQUEST=False,
)
```

Operational notes:

- `REMEMBER_COOKIE_SECURE` should be `True` in production behind HTTPS.
- `REMEMBER_COOKIE_HTTPONLY` is `True` by default and should usually stay that way.
- `REMEMBER_COOKIE_REFRESH_EACH_REQUEST=True` extends the lifetime on every request; leave it off unless you explicitly want sliding expiration.

## Session Protection

Flask-Login can bind a session to a client fingerprint based on IP address and user agent:

```python
login_manager.session_protection = "strong"
```

Modes in `0.6.3`:

- `"basic"`: identifier changes mark the session non-fresh
- `None`: disables session protection

The docs say the default is `"basic"`. You can also configure this via `app.config["SESSION_PROTECTION"]`.

Be careful with `"strong"` behind proxies, mobile networks, or other environments where IP/user-agent characteristics shift often.

## API Requests Without Writing A Session Cookie

If you authenticate via `request_loader`, you may not want Flask to write a session cookie for API calls. The official docs recommend a custom session interface keyed off `user_loaded_from_request`:

```python
from flask import g
from flask.sessions import SecureCookieSessionInterface
from flask_login import user_loaded_from_request

@user_loaded_from_request.connect
def mark_login_via_request(app, user=None):
    g.login_via_request = True

class APISessionInterface(SecureCookieSessionInterface):
    def save_session(self, *args, **kwargs):
        if g.get("login_via_request"):
            return
        return super().save_session(*args, **kwargs)

app.session_interface = APISessionInterface()
```

Use this when request-authenticated API traffic should stay stateless.

## Testing

For test code, use the built-in `FlaskLoginClient` instead of manually building login session state:

```python
from flask_login import FlaskLoginClient

app.test_client_class = FlaskLoginClient

def test_settings_page(app, user):
    with app.test_client(user=user, fresh_login=True) as client:
        response = client.get("/settings")
        assert response.status_code == 200
```

Two details from the docs matter:

- Pass `user=` as a keyword argument, not a positional argument.
- If session protection is enabled, test-client requests may become non-fresh in `"basic"` mode or fail in `"strong"` mode. Disable session protection in tests if needed.

`LOGIN_DISABLED` still exists in `0.6.3` and can bypass login checks in tests, but it is not a good primary testing strategy when you want realistic auth behavior.

## Common Pitfalls

- Missing `SECRET_KEY`: session-backed auth will not work correctly without it.
- Returning non-string IDs: keep `get_id()` string-based.
- Forgetting `user_loader`: `current_user` cannot be restored from the session without it.
- Using `@login_required` on JSON routes without `unauthorized_handler`: you get HTML redirects instead of `401`.
- Treating Flask-Login as a full auth system: it does not hash passwords, register users, or handle permissions.
- Using deprecated `header_loader` in new code: prefer `request_loader`.
- Copying examples from `latest`: the `latest` docs are `0.7.0`, not `0.6.3`.

## Version-Sensitive Notes

- `0.6.3` is the package version covered here and is the release that adds Flask 3 / Werkzeug 3 compatibility.
- `0.6.2` already warned that `header_loader` is deprecated and that `request_loader` should replace it.
- The current `0.7.0` changelog removes `LOGIN_DISABLED` and previously deprecated code. If you rely on deprecated paths in `0.6.3`, expect upgrade cleanup.
- When behavior and docs disagree, prefer the version-pinned `0.6.3` docs and the package's changelog over `latest`.

## Official Sources

- Version-pinned docs: https://flask-login.readthedocs.io/en/0.6.3/
- Latest docs root: https://flask-login.readthedocs.io/en/latest/
- PyPI release page: https://pypi.org/project/Flask-Login/0.6.3/
- PyPI JSON metadata: https://pypi.org/pypi/Flask-Login/0.6.3/json
- Source repository: https://github.com/maxcountryman/flask-login
- Changelog: https://github.com/maxcountryman/flask-login/blob/main/CHANGES.md
