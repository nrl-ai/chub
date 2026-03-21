---
name: axes
description: "django-axes login lockout and failed-auth tracking for Django projects"
metadata:
  languages: "python"
  versions: "8.3.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,axes,authentication,security,login,lockout"
---

# Django Axes Python Package Guide

## Golden Rule

Use `django-axes` only in authentication flows that call Django `authenticate()` and `login()` or emit the same auth signals themselves. Put `axes.backends.AxesStandaloneBackend` first in `AUTHENTICATION_BACKENDS`, put `axes.middleware.AxesMiddleware` last in `MIDDLEWARE`, and make client IP resolution explicit when you run behind proxies.

## Install

Install the package with the optional `ipware` extra unless you already plan to provide a custom client-IP callable:

```bash
python -m pip install "django-axes[ipware]==8.3.1"
```

Without the extra:

```bash
python -m pip install "django-axes==8.3.1"
```

Use the plain package only if you will set `AXES_CLIENT_IP_CALLABLE` yourself.

## Minimal Setup

`django-axes` becomes active through Django settings plus a database migration.

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "axes",
]

AUTHENTICATION_BACKENDS = [
    "axes.backends.AxesStandaloneBackend",
    "django.contrib.auth.backends.ModelBackend",
]

MIDDLEWARE = [
    "django.middleware.security.SecurityMiddleware",
    "django.contrib.sessions.middleware.SessionMiddleware",
    "django.middleware.common.CommonMiddleware",
    "django.middleware.csrf.CsrfViewMiddleware",
    "django.contrib.auth.middleware.AuthenticationMiddleware",
    "django.contrib.messages.middleware.MessageMiddleware",
    "django.middleware.clickjacking.XFrameOptionsMiddleware",
    "axes.middleware.AxesMiddleware",
]
```

Then validate the config and create the tables:

```bash
python manage.py check
python manage.py migrate
```

Notes:

- `AxesStandaloneBackend` should be first in `AUTHENTICATION_BACKENDS`.
- `AxesMiddleware` should be last in `MIDDLEWARE`.
- `AxesBackend` still exists for backward compatibility, but upstream recommends `AxesStandaloneBackend` if you have custom permission logic.
- `python manage.py check` is worth running in CI because Axes includes system checks for security-sensitive misconfiguration.

## Auth Flow And Core Usage

Axes monitors Django authentication by listening to:

- `user_logged_in`
- `user_logged_out`
- `user_login_failed`

If your login flow uses Django’s normal `authenticate()` and `login()`, those signals are handled for you. If you use custom JSON login views or a nonstandard auth layer, you must emit the expected signals yourself or Axes will not track failures correctly.

With the default database handler, Axes stores attempts and lockouts in the database and can be reset from admin or management commands:

```bash
python manage.py axes_reset
python manage.py axes_reset_ip 203.0.113.10
python manage.py axes_reset_username alice
python manage.py axes_reset_ip_username 203.0.113.10 alice
python manage.py axes_reset_logs 30
```

If you switch to a non-database handler such as a cache-based handler, the docs note that you must implement custom reset commands for that handler.

## Configuration That Matters In Real Projects

### Lockout identity

For modern Axes configuration, prefer `AXES_LOCKOUT_PARAMETERS` instead of the older boolean-style flags:

```python
# Default-style lockout by IP
AXES_LOCKOUT_PARAMETERS = ["ip_address"]

# Lock out by username only
AXES_LOCKOUT_PARAMETERS = ["username"]

# Lock out by username + IP combination
AXES_LOCKOUT_PARAMETERS = [["username", "ip_address"]]
```

You can also combine factors:

```python
AXES_LOCKOUT_PARAMETERS = ["ip_address", ["username", "user_agent"]]
```

This supersedes older settings such as `AXES_ONLY_USER_FAILURES`, `AXES_LOCK_OUT_BY_COMBINATION_USER_AND_IP`, `AXES_LOCK_OUT_BY_USER_OR_IP`, and `AXES_USE_USER_AGENT`.

### Reverse proxies and client IPs

If your Django app runs behind Nginx, a load balancer, or another proxy, do not rely on defaults blindly. Axes uses `django-ipware` when available and exposes proxy settings such as:

- `AXES_IPWARE_PROXY_COUNT`
- `AXES_IPWARE_META_PRECEDENCE_ORDER`
- `AXES_IPWARE_PROXY_ORDER`

If you do not install the `ipware` extra, provide your own IP lookup:

```python
AXES_CLIENT_IP_CALLABLE = "project.security.get_client_ip"
```

### Username normalization

If your login identifiers are transformed before authentication, teach Axes the same transformation:

```python
AXES_USERNAME_CALLABLE = "project.security.get_axes_username"
```

Axes does not apply those transformations for you before `authenticate()`. Your auth code and Axes configuration need to agree on the final username value.

### API-friendly lockout responses

If your app needs JSON or another custom response format on lockout, use a callable:

```python
# settings.py
AXES_LOCKOUT_CALLABLE = "project.views.lockout"
```

```python
# project/views.py
from django.http import JsonResponse

def lockout(request, response, credentials, *args, **kwargs):
    return JsonResponse(
        {"detail": "Too many failed login attempts"},
        status=403,
    )
```

## Integration Notes

### Django Allauth

Allauth does not pass login data in the shape Axes expects by default. The upstream integration guide requires:

- `AXES_USERNAME_FORM_FIELD = "login"`
- a custom Allauth login form that duplicates the login identifier into the credentials dict
- decorating Allauth `LoginView.dispatch` with `axes_dispatch`
- decorating `LoginView.form_invalid` with `axes_form_invalid`

If you skip those patches, failed Allauth logins can bypass Axes tracking.

### Django REST Framework

The upstream DRF guidance only applies to authentication schemes that use Django `authenticate()`. `TokenAuthentication` is explicitly called out as unsupported in that guide.

For API lockouts, connect the `axes.signals.user_locked_out` signal to an exception or response your API stack understands, for example raising DRF `PermissionDenied`.

## Common Pitfalls

- Wrong backend order: if `AxesStandaloneBackend` is not first, failed logins can bypass Axes.
- Wrong middleware placement: `AxesMiddleware` belongs at the end of `MIDDLEWARE`.
- Proxy misconfiguration: without `django-ipware` or a correct `AXES_CLIENT_IP_CALLABLE`, every user can appear to come from the proxy IP.
- Custom login views that never emit Django auth signals: Axes will not see failures unless you send the signals yourself.
- Using pre-6.x config flags from old blog posts instead of `AXES_LOCKOUT_PARAMETERS` and `AXES_IPWARE_*`.
- Assuming reset commands work for every handler. The docs only describe them for the default database handler.
- Trusting the docs requirements page for Python support. For `8.3.1`, PyPI metadata is the authoritative compatibility source.

## Version-Sensitive Notes

- `8.3.1` was released on February 11, 2026 and fixes configuration JSON serialization errors for Celery.
- `8.1.0` added Django `6.0` support and Python `3.14` support, and removed Django `5.1` plus Python `3.9`.
- `8.0.0` moved all database-related logic to the default `axes.handlers.database.AxesDatabaseHandler`.
- If you upgraded from Axes 6 to 7 and use a callable for `AXES_COOLOFF_TIME`, the callable must now accept a `request`.
- If you upgraded from Axes 5-era config, rename proxy settings to `AXES_IPWARE_*` and migrate older lockout booleans to `AXES_LOCKOUT_PARAMETERS`.

## Official Sources

- Docs root: `https://django-axes.readthedocs.io/en/latest/`
- Installation: `https://django-axes.readthedocs.io/en/latest/2_installation.html`
- Usage: `https://django-axes.readthedocs.io/en/latest/3_usage.html`
- Configuration: `https://django-axes.readthedocs.io/en/latest/4_configuration.html`
- Customization: `https://django-axes.readthedocs.io/en/latest/5_customization.html`
- Integration: `https://django-axes.readthedocs.io/en/latest/6_integration.html`
- Changelog: `https://django-axes.readthedocs.io/en/latest/10_changelog.html`
- PyPI: `https://pypi.org/project/django-axes/`
