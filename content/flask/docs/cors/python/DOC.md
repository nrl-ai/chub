---
name: cors
description: "Flask-CORS guide for Python Flask apps, selective CORS policies, and credentialed browser requests"
metadata:
  languages: "python"
  versions: "6.0.2"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "python,flask,cors,http,web,security"
---

# flask-cors Python Package Guide

## What It Is

`flask-cors` adds Cross-Origin Resource Sharing headers to Flask responses so browser apps on one origin can call a Flask app on another origin.

- PyPI package: `flask-cors`
- Import path: `flask_cors`
- Main APIs: `CORS` for app or blueprint setup, `cross_origin` for route-level overrides
- Version covered here: `6.0.2`
- PyPI requirement for this release: Python `>=3.9`

Use it only when a browser must make cross-origin requests. It does not replace authentication, authorization, or CSRF protection.

## Install

```bash
pip install flask-cors==6.0.2
```

If the project already pins Flask and related middleware, keep the package version aligned with the lockfile instead of upgrading ad hoc.

## Minimal Setup

Enable CORS for the whole Flask app:

```python
from flask import Flask, jsonify
from flask_cors import CORS

app = Flask(__name__)
CORS(app)

@app.get("/ping")
def ping():
    return jsonify(ok=True)
```

This is convenient for local development, but it is usually too broad for production APIs because it allows every origin on every route by default.

## Restrict CORS To API Routes

Prefer a scoped policy for API paths:

```python
from flask import Flask
from flask_cors import CORS

app = Flask(__name__)

CORS(
    app,
    resources={
        r"/api/*": {
            "origins": [
                "https://app.example.com",
                "https://admin.example.com",
            ],
            "methods": ["GET", "POST", "PATCH", "DELETE"],
            "allow_headers": ["Content-Type", "Authorization"],
        }
    },
)
```

The official configuration docs say resource regexes are sorted longest-first before matching, so the most specific pattern should win.

## Route-Level Control With `cross_origin`

Use the decorator when only a few routes need cross-origin access:

```python
from flask import Flask, jsonify
from flask_cors import cross_origin

app = Flask(__name__)

@app.get("/public")
@cross_origin(origins="*")
def public():
    return jsonify(public=True)

@app.post("/api/token")
@cross_origin(
    origins=["https://app.example.com"],
    methods=["POST"],
    allow_headers=["Content-Type", "Authorization"],
)
def issue_token():
    return jsonify(token="example")
```

Decorator kwargs override the broader app-level defaults for that route.

## App Factory And Config Keys

`flask-cors` also reads Flask config keys prefixed with `CORS_`, which is useful in app-factory setups:

```python
from flask import Flask
from flask_cors import CORS

def create_app():
    app = Flask(__name__)
    app.config.update(
        CORS_ORIGINS=["https://app.example.com"],
        CORS_RESOURCES={r"/api/*": {"origins": ["https://app.example.com"]}},
        CORS_ALLOW_HEADERS=["Content-Type", "Authorization"],
        CORS_METHODS=["GET", "POST", "PATCH", "DELETE"],
        CORS_SUPPORTS_CREDENTIALS=True,
        CORS_MAX_AGE=600,
        CORS_VARY_HEADER=True,
    )
    CORS(app)
    return app
```

The documented precedence order is:

1. Resource-level settings
2. Keyword arguments passed to `CORS(...)` or `@cross_origin(...)`
3. App config keys such as `CORS_ORIGINS`
4. Package defaults

Common options that matter in real apps:

- `origins`: allowed origin string, list, or regex
- `resources`: route-regex to options mapping
- `methods`: methods advertised on preflight responses
- `allow_headers`: request headers accepted during preflight
- `expose_headers`: response headers readable from browser code
- `supports_credentials`: allow cookies or auth-bearing cross-origin requests
- `max_age`: cache successful preflight responses
- `send_wildcard`: send `*` instead of echoing the caller origin when `origins="*"`
- `vary_header`: keep `Vary: Origin` when origin handling is dynamic
- `allow_private_network`: control the `Access-Control-Allow-Private-Network` response header

## Blueprints

You can attach CORS to a blueprint instead of the whole app:

```python
from flask import Blueprint
from flask_cors import CORS

api = Blueprint("api", __name__, url_prefix="/api")
CORS(api, origins=["https://app.example.com"])
```

Register the blueprint on the Flask app as usual.

## Credentials, Cookies, And Auth

If the frontend sends cookies or other credentials, use explicit origins and enable credentials:

```python
from flask import Flask
from flask_cors import CORS

app = Flask(__name__)

CORS(
    app,
    resources={r"/api/*": {"origins": ["https://app.example.com"]}},
    supports_credentials=True,
)
```

Important constraints:

- Do not combine `supports_credentials=True` with `origins="*"`. The official API docs explicitly disallow it.
- The browser client must also opt in to credentials mode.
- CORS only controls browser access to responses. It does not implement login, token validation, or CSRF defense.
- If cookie-based auth crosses origins, verify Flask session cookie settings and CSRF protections together with the CORS policy.

## Preflight-Friendly JSON APIs

Browsers preflight requests that use JSON, custom headers, or non-simple methods. A common setup for API routes is:

```python
from flask import Flask
from flask_cors import CORS

app = Flask(__name__)

CORS(
    app,
    resources={r"/api/*": {"origins": ["https://app.example.com"]}},
    allow_headers=["Content-Type", "Authorization"],
    methods=["GET", "POST", "PUT", "PATCH", "DELETE"],
    max_age=600,
)
```

If a browser reports a CORS error on a JSON request, check the exact path, request method, and `allow_headers` first. Flask route failures often surface as generic browser CORS errors.

## Common Pitfalls

- Install name and import name differ: install `flask-cors`, import `flask_cors`.
- Origins must match scheme, host, and port exactly.
- `CORS(app)` is easy to overuse. Prefer scoped `resources` or route decorators in production.
- Dynamic origin behavior should usually keep `vary_header=True` so shared caches do not reuse the wrong response.
- If a frontend sends `Authorization` or JSON, missing `allow_headers` is a common reason for failed preflights.
- If you use overlapping regexes in `resources`, keep the specific ones longer and test them explicitly.
- A backend `404`, `405`, or `500` can look like a CORS problem in the browser. Reproduce with `curl` before changing the policy.

## Version-Sensitive Notes For 6.0.2

- PyPI currently lists `6.0.2` as the latest release and requires Python `>=3.9`.
- The current official docs are served from `https://flask-cors.corydolphin.com/en/latest/`. The older Read the Docs URL still resolves, but the custom domain is the canonical docs root advertised on PyPI.
- The maintainer release notes for `6.0.1` mention a fix for incorrect regex-length sorting. If you rely on multiple overlapping `resources` patterns, prefer `6.0.1+` behavior and avoid older examples that assume different match ordering.
- The maintainer release notes for `6.0.0` mention security fixes. Treat older third-party snippets carefully if they assume pre-`6.x` defaults.
- The public docs are not obviously version-pinned per package release. When a behavior matters for production security, verify it against the exact installed version instead of assuming the `latest` docs describe `6.0.2` perfectly.

## Official Sources

- PyPI package page: https://pypi.org/project/flask-cors/
- Version-specific PyPI page: https://pypi.org/project/flask-cors/6.0.2/
- Canonical docs root: https://flask-cors.corydolphin.com/en/latest/
- API docs: https://flask-cors.corydolphin.com/en/latest/api.html
- Configuration docs: https://flask-cors.corydolphin.com/en/latest/configuration.html
- Maintainer release notes: https://github.com/corydolphin/flask-cors/releases
