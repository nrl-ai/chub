---
name: wtf
description: "Flask-WTF form handling, CSRF protection, file uploads, and reCAPTCHA for Flask applications"
metadata:
  languages: "python"
  versions: "1.2.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,flask-wtf,wtforms,forms,csrf,file-upload,recaptcha"
---

# Flask-WTF Python Package Guide

## Install

Pin the package to the version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "Flask-WTF==1.2.2"
```

If you use WTForms email validation, install the published extra:

```bash
python -m pip install "Flask-WTF[email]==1.2.2"
```

Common alternatives:

```bash
uv add "Flask-WTF==1.2.2"
poetry add "Flask-WTF==1.2.2"
```

## Initialize In A Flask App

Use Flask-WTF with an application factory and a real secret key. CSRF uses Flask's `SECRET_KEY` unless you set `WTF_CSRF_SECRET_KEY`.

```python
from flask import Flask
from flask_wtf.csrf import CSRFProtect

csrf = CSRFProtect()

def create_app() -> Flask:
    app = Flask(__name__)
    app.config.from_mapping(
        SECRET_KEY="replace-in-production",
    )

    csrf.init_app(app)

    return app
```

Important:

- Any view that uses `FlaskForm` already gets CSRF handling for that form.
- Register `CSRFProtect` when you also need to protect non-form POST/PUT/PATCH/DELETE endpoints or JavaScript requests.
- Do not disable CSRF globally unless you have a narrow, explicit reason.

## Define And Validate Forms

Import fields and validators from `wtforms`, not from `flask_wtf`.

```python
from flask import Blueprint, redirect, render_template, url_for
from flask_wtf import FlaskForm
from wtforms import PasswordField, StringField, SubmitField
from wtforms.validators import DataRequired, Email, Length

bp = Blueprint("auth", __name__)

class LoginForm(FlaskForm):
    email = StringField("Email", validators=[DataRequired(), Email()])
    password = PasswordField("Password", validators=[DataRequired(), Length(min=8)])
    submit = SubmitField("Sign in")

@bp.route("/login", methods=["GET", "POST"])
def login():
    form = LoginForm()

    if form.validate_on_submit():
        email = form.email.data
        return redirect(url_for("auth.success", email=email))

    return render_template("login.html", form=form)
```

`validate_on_submit()` is the common path. It is equivalent to checking that the request was submitted with a mutating HTTP method and then running `validate()`.

## Render Forms In Templates

Render the hidden CSRF field or use `hidden_tag()` to render all hidden fields in one place.

```html
<form method="post" action="{{ url_for('auth.login') }}">
  {{ form.hidden_tag() }}

  <div>
    {{ form.email.label }}
    {{ form.email() }}
    {% for error in form.email.errors %}
      <p>{{ error }}</p>
    {% endfor %}
  </div>

  <div>
    {{ form.password.label }}
    {{ form.password() }}
    {% for error in form.password.errors %}
      <p>{{ error }}</p>
    {% endfor %}
  </div>

  {{ form.submit() }}
</form>
```

If you forget the hidden CSRF field, `validate_on_submit()` will fail even though the form otherwise looks correct.

## File Uploads

Use Flask-WTF's file helpers instead of plain WTForms file fields when you want Flask-aware `FileStorage` handling and validators.

```python
from pathlib import Path

from flask import Blueprint, current_app, redirect, render_template, url_for
from flask_wtf import FlaskForm
from flask_wtf.file import FileAllowed, FileField, FileRequired
from werkzeug.utils import secure_filename
from wtforms import SubmitField

bp = Blueprint("uploads", __name__)

class UploadForm(FlaskForm):
    photo = FileField(
        "Photo",
        validators=[
            FileRequired(),
            FileAllowed(["jpg", "jpeg", "png", "webp"], "Images only."),
        ],
    )
    submit = SubmitField("Upload")

@bp.route("/upload", methods=["GET", "POST"])
def upload():
    form = UploadForm()

    if form.validate_on_submit():
        storage = form.photo.data
        filename = secure_filename(storage.filename)
        destination = Path(current_app.instance_path) / "uploads" / filename
        destination.parent.mkdir(parents=True, exist_ok=True)
        storage.save(destination)
        return redirect(url_for("uploads.upload"))

    return render_template("upload.html", form=form)
```

Template requirement:

```html
<form method="post" enctype="multipart/form-data">
  {{ form.hidden_tag() }}
  {{ form.photo() }}
  {{ form.submit() }}
</form>
```

Notes:

- `enctype="multipart/form-data"` is required or `request.files` will be empty.
- If you manually instantiate a form with explicit request data, combine `request.files` and `request.form`; otherwise file fields will not populate.
- `MultipleFileField` is available in the 1.2.x line when you need multiple uploads.
- `FileRequired`, `FileAllowed`, and `FileSize` work with both single and multiple file fields in 1.2.x.

## Global CSRF Protection For HTML And JavaScript Requests

If you submit requests outside a `FlaskForm`, protect them with `CSRFProtect` and send the token explicitly.

Server setup:

```python
from flask import Flask, jsonify
from flask_wtf.csrf import CSRFError, CSRFProtect

csrf = CSRFProtect()

def create_app() -> Flask:
    app = Flask(__name__)
    app.config["SECRET_KEY"] = "replace-in-production"
    csrf.init_app(app)

    @app.errorhandler(CSRFError)
    def handle_csrf_error(e):
        return jsonify(error="csrf_failed", reason=e.description), 400

    return app
```

Template or page script for JavaScript requests:

```html
<script>
  axios.defaults.headers.common["X-CSRFToken"] = "{{ csrf_token() }}";
</script>
```

Relevant defaults in 1.2.x:

- `WTF_CSRF_ENABLED=True`
- `WTF_CSRF_CHECK_DEFAULT=True`
- `WTF_CSRF_METHODS={"POST", "PUT", "PATCH", "DELETE"}`
- `WTF_CSRF_FIELD_NAME="csrf_token"`
- `WTF_CSRF_HEADERS=["X-CSRFToken", "X-CSRF-Token"]`
- `WTF_CSRF_TIME_LIMIT=3600`

Use `WTF_CSRF_SECRET_KEY` if you want CSRF signing separated from your app `SECRET_KEY`.

## Configuration

Common config you will actually use:

```python
app.config.update(
    SECRET_KEY="replace-in-production",
    WTF_CSRF_SECRET_KEY="separate-csrf-key-if-needed",
    WTF_CSRF_TIME_LIMIT=3600,
)
```

Useful rules:

- `SECRET_KEY` is mandatory for normal form CSRF protection.
- `WTF_CSRF_SECRET_KEY` is optional and overrides the signing key for CSRF tokens only.
- `WTF_CSRF_CHECK_DEFAULT=False` is only for special flows where you call `csrf.protect()` selectively, such as preprocessing OAuth callbacks.
- `WTF_CSRF_TIME_LIMIT=None` keeps a token valid for the life of the session.
- CSRF failures are logged to the `flask_wtf.csrf` logger at `INFO`; configure logging if you want to see them.

## Recaptcha

Flask-WTF ships a `RecaptchaField` for server-side validation and widget rendering.

```python
from flask_wtf import FlaskForm, RecaptchaField
from wtforms import SubmitField

class SignupForm(FlaskForm):
    recaptcha = RecaptchaField()
    submit = SubmitField("Create account")
```

Required config:

```python
app.config.update(
    RECAPTCHA_PUBLIC_KEY="...",
    RECAPTCHA_PRIVATE_KEY="...",
)
```

The config surface also includes `RECAPTCHA_PARAMETERS`, `RECAPTCHA_DATA_ATTRS`, `RECAPTCHA_SCRIPT`, `RECAPTCHA_DIV_CLASS`, and `RECAPTCHA_VERIFY_SERVER`, which matters if you need alternative captcha services or custom rendering.

## Testing

If you are testing code that instantiates a form outside an active request, pass `formdata=None` explicitly so Flask-WTF does not try to read `request.form` and `request.files`.

For unit tests around CSRF-protected flows:

- prefer Flask's test client for end-to-end form submissions
- disable CSRF per-form with `meta={"csrf": False}` only in narrow test cases that are not exercising CSRF behavior
- when you need the token directly, the API docs note that the signed token is available in `g.csrf_token` and the raw token in `session["csrf_token"]` during testing

## Common Pitfalls

- Forgetting `SECRET_KEY` or setting it too late. CSRF and session-backed forms rely on it.
- Forgetting `{{ form.hidden_tag() }}` or `{{ form.csrf_token }}` in the template.
- Assuming Flask-WTF handles JSON APIs automatically. For JavaScript and non-form endpoints, register `CSRFProtect` and send the header token.
- Disabling `WTF_CSRF_ENABLED` globally instead of exempting a narrow route or blueprint.
- Importing fields from `flask_wtf` instead of `wtforms`.
- Forgetting `multipart/form-data` for upload forms.
- Passing only `request.form` when manually constructing an upload form.
- Caching HTML pages longer than `WTF_CSRF_TIME_LIMIT`, which can surface expired-token errors from stale pages.
- Expecting Flask-WTF to handle authentication. It handles forms, validation, CSRF, uploads, and reCAPTCHA, not login/session policy by itself.

## Version-Sensitive Notes For 1.2.x

- GitHub releases and PyPI both show `Flask-WTF 1.2.2` on October 24, 2024.
- `1.2.2` moved the project to the `pallets-eco` GitHub organization.
- `1.2.2` dropped Python 3.8 support and added Python 3.13 support; PyPI now lists `Requires: Python >=3.9`.
- `1.2.1` fixed a bug where file validators could mutate file field content.
- `1.2.0` added `MultipleFileField`, and the file validators (`FileRequired`, `FileAllowed`, `FileSize`) now support multiple files.
- The versioned `1.2.x` docs are the right reference target for this package line. The floating `stable/latest` docs are fine for browsing but less precise for package-pinned work.
- The Read the Docs changelog page is incomplete for the latest `1.2.x` releases, so use GitHub releases and PyPI when you need exact `1.2.1` / `1.2.2` release details.

## Official Sources

- Docs root: https://flask-wtf.readthedocs.io/en/1.2.x/
- Installation: https://flask-wtf.readthedocs.io/en/latest/install/
- Quickstart: https://flask-wtf.readthedocs.io/en/1.2.x/quickstart/
- Forms guide: https://flask-wtf.readthedocs.io/en/1.2.x/form/
- CSRF guide: https://flask-wtf.readthedocs.io/en/1.2.x/csrf/
- Configuration: https://flask-wtf.readthedocs.io/en/1.2.x/config/
- API reference: https://flask-wtf.readthedocs.io/en/1.2.x/api/
- Changelog: https://flask-wtf.readthedocs.io/en/1.2.x/changes/
- PyPI: https://pypi.org/project/Flask-WTF/
- Source: https://github.com/pallets-eco/flask-wtf
- Releases: https://github.com/pallets-eco/flask-wtf/releases
