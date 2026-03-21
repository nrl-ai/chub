---
name: allauth
description: "django-allauth package guide for Django account, email, MFA, and social authentication in Python projects"
metadata:
  languages: "python"
  versions: "65.15.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,authentication,accounts,social-login,oauth,oidc,mfa"
---

# django-allauth Python Package Guide

## What This Package Is

`django-allauth` adds account management, email flows, MFA, and social login on top of Django's auth system.

- Package: `django-allauth`
- Main apps: `allauth`, `allauth.account`, `allauth.socialaccount`
- Version covered here: `65.15.0`
- Docs root: `https://docs.allauth.org/en/latest/`
- Registry URL: `https://pypi.org/project/django-allauth/`

Use it when you want Django-managed signup, login, logout, password reset, email confirmation, and OAuth/OIDC provider login without building those flows yourself.

## Install

For account management only:

```bash
pip install django-allauth==65.15.0
```

For social login support:

```bash
pip install "django-allauth[socialaccount]==65.15.0"
```

Upstream drift matters here:

- PyPI currently lists `65.15.0` as the latest release and requires Python `>=3.10`.
- The latest docs requirements page still says Python `3.8+` and Django `4.2+`.
- The `65.15.0` release notes explicitly say support for Python `3.8` and `3.9` was dropped.

For installation constraints, trust PyPI metadata and the `65.15.0` release notes over older docs pages.

## Minimal Django Setup

Start with the official quickstart wiring. The current setup needs the request context processor, the allauth authentication backend, and `AccountMiddleware`.

```python
# settings.py
INSTALLED_APPS = [
    "django.contrib.admin",
    "django.contrib.auth",
    "django.contrib.contenttypes",
    "django.contrib.sessions",
    "django.contrib.messages",
    "django.contrib.staticfiles",
    "allauth",
    "allauth.account",
]

MIDDLEWARE = [
    "django.middleware.security.SecurityMiddleware",
    "django.contrib.sessions.middleware.SessionMiddleware",
    "django.middleware.common.CommonMiddleware",
    "django.middleware.csrf.CsrfViewMiddleware",
    "django.contrib.auth.middleware.AuthenticationMiddleware",
    "django.contrib.messages.middleware.MessageMiddleware",
    "django.middleware.clickjacking.XFrameOptionsMiddleware",
    "allauth.account.middleware.AccountMiddleware",
]

TEMPLATES = [
    {
        "BACKEND": "django.template.backends.django.DjangoTemplates",
        "DIRS": [],
        "APP_DIRS": True,
        "OPTIONS": {
            "context_processors": [
                "django.template.context_processors.request",
                "django.contrib.auth.context_processors.auth",
                "django.contrib.messages.context_processors.messages",
            ],
        },
    },
]

AUTHENTICATION_BACKENDS = [
    "django.contrib.auth.backends.ModelBackend",
    "allauth.account.auth_backends.AuthenticationBackend",
]

LOGIN_REDIRECT_URL = "/"
```

Add the URLconf:

```python
# urls.py
from django.urls import include, path

urlpatterns = [
    path("accounts/", include("allauth.urls")),
]
```

Then run migrations:

```bash
python manage.py migrate
```

Notes:

- If you only need standard browser-based account flows, this setup is enough to start with `/accounts/login/`, `/accounts/signup/`, `/accounts/logout/`, and password reset views.

## Core Usage Pattern

For normal Django apps, let allauth own the account flow and keep app code centered on `request.user`.

```python
# views.py
from django.contrib.auth.decorators import login_required
from django.shortcuts import render

@login_required
def dashboard(request):
    return render(request, "dashboard.html", {"user": request.user})
```

Typical flow:

1. User signs up or logs in through `/accounts/`.
2. allauth authenticates the Django user and manages email/password flows.
3. Your app uses Django auth normally: `request.user`, permissions, groups, and decorators.
4. Override templates under `templates/account/` or `templates/socialaccount/` instead of replacing allauth views unless you need a custom auth protocol.

## Account Configuration That Matters First

The current docs use newer setting names. Prefer these settings instead of older blog posts and pre-65.4 examples.

```python
# settings.py
ACCOUNT_LOGIN_METHODS = {"email"}
ACCOUNT_SIGNUP_FIELDS = ["email*", "password1*", "password2*"]
ACCOUNT_EMAIL_VERIFICATION = "mandatory"
ACCOUNT_RATE_LIMITS = {
    "login": "30/m/ip",
}
```

Guidance:

- `ACCOUNT_LOGIN_METHODS` replaced the older `ACCOUNT_AUTHENTICATION_METHOD` setting.
- `ACCOUNT_SIGNUP_FIELDS` replaced older boolean-style signup field settings. Fields marked with `*` are required.
- `ACCOUNT_EMAIL_VERIFICATION = "mandatory"` is the simplest production default if login should depend on confirmed email.
- `ACCOUNT_RATE_LIMITS` is enabled by default; override it only when you need different limits for login, signup, password reset, or verification code flows.
- Keep login methods and signup fields aligned. If you only allow email login, collect email at signup.

## Social Login Setup

Add social auth only after the base account flow works.

```python
# settings.py
INSTALLED_APPS += [
    "allauth.socialaccount",
    "allauth.socialaccount.providers.google",
]
```

Current provider configuration can live in settings or in the Django admin `SocialApp` model.

Settings-based configuration is the easiest path for agents:

```python
# settings.py
SOCIALACCOUNT_PROVIDERS = {
    "google": {
        "APPS": [
            {
                "client_id": "your-client-id",
                "secret": "your-client-secret",
                "settings": {
                    "scope": ["profile", "email"],
                    "auth_params": {"access_type": "online"},
                },
            }
        ]
    }
}

SOCIALACCOUNT_LOGIN_ON_GET = False
```

Practical rules:

- Register the callback URL exactly as allauth expects, for example `/accounts/google/login/callback/`.
- Keep `SOCIALACCOUNT_LOGIN_ON_GET = False` so login initiation is not a GET side effect.
- If you use the Django admin `SocialApp` model instead of settings, add `django.contrib.sites`, set `SITE_ID`, and attach the app to the correct site.
- Do not configure the same provider in both `SOCIALACCOUNT_PROVIDERS` and `SocialApp` for the same site unless you want lookup conflicts such as `MultipleObjectsReturned`.

## Adapters and Custom Rules

Use adapters when you need business rules around signup, redirects, or provider data.

```python
# users/adapters.py
from allauth.account.adapter import DefaultAccountAdapter

class AccountAdapter(DefaultAccountAdapter):
    def is_open_for_signup(self, request):
        return request.get_host().endswith(".example.com")
```

```python
# settings.py
ACCOUNT_ADAPTER = "users.adapters.AccountAdapter"
```

Use:

- `ACCOUNT_ADAPTER` for account signup and email flow customization
- `SOCIALACCOUNT_ADAPTER` for provider-specific account linking and social signup rules

## Common Pitfalls

- Missing `allauth.account.middleware.AccountMiddleware`. Current allauth expects it.
- Missing `django.template.context_processors.request`. The quickstart includes it because templates and flows depend on request context.
- Copying older examples that use `ACCOUNT_AUTHENTICATION_METHOD` or older signup field flags. Current releases use `ACCOUNT_LOGIN_METHODS` and `ACCOUNT_SIGNUP_FIELDS`.
- Adding social login before the base account flow works. Debug local email/password first, then provider auth.
- Registering the wrong callback URL with Google, GitHub, or another provider.
- Configuring the provider in both Django settings and `SocialApp` for the same site.
- If you use `SocialApp`, forgetting `django.contrib.sites`, `SITE_ID`, or site association.
- Assuming Django admin login is automatically protected by allauth's MFA and rate limiting. The admin login view is separate; use allauth's `secure_admin_login` support if you need that coverage.
- Using a dummy cache or incorrect proxy/IP setup with rate limits. Rate limiting depends on cache and correct client IP detection.

## Version-Sensitive Notes For `65.15.0`

- `65.15.0` dropped support for Python `3.8` and `3.9`. PyPI now requires Python `>=3.10`.
- The docs requirements page still says Python `3.8+`, so treat that page as stale for this release line.
- `65.15.0` changed the human-facing authentication code format to RFC 8628 style (`ABC-DEF`) and added `ACCOUNT_LOGIN_BY_CODE_MAX_ATTEMPTS`.
- `65.14.2` changed proxy/IP handling to distrust `X-Forwarded-For` by default. If you deploy behind proxies or load balancers, review client IP configuration before relying on rate limits.
- `65.13.0` made the headless feature require the `headless` extra.
- `65.5.0` introduced `ACCOUNT_SIGNUP_FIELDS` and code-based password reset settings.
- `65.4.0` replaced `ACCOUNT_AUTHENTICATION_METHOD` with `ACCOUNT_LOGIN_METHODS`.
- The latest release notes page currently labels the `65.15.0` section with `2025-03-09`, while the PyPI release page shows the release on March 9, 2026. Treat PyPI as authoritative for release timing and install metadata.

## Recommended Agent Workflow

1. Confirm whether the project needs plain Django account flows, social login, MFA, or headless APIs.
2. Wire the minimal Django setup first and verify `/accounts/login/` works locally.
3. Add `ACCOUNT_LOGIN_METHODS`, `ACCOUNT_SIGNUP_FIELDS`, and email verification settings before custom templates.
4. Add one provider at a time and verify its callback URL and site binding before layering on more providers.
5. If deployment uses proxies, validate rate-limit client IP behavior before assuming brute-force protections are working.
6. When examples disagree, prefer the current release notes and PyPI metadata over older docs pages and third-party tutorials.

## Official Sources Used

- Docs root: `https://docs.allauth.org/en/latest/`
- Quickstart: `https://docs.allauth.org/en/latest/installation/quickstart.html`
- Requirements: `https://docs.allauth.org/en/latest/installation/requirements.html`
- Account configuration: `https://docs.allauth.org/en/latest/account/configuration.html`
- Account rate limits: `https://docs.allauth.org/en/latest/account/rate_limits.html`
- Social account configuration: `https://docs.allauth.org/en/latest/socialaccount/configuration.html`
- Social provider configuration: `https://docs.allauth.org/en/latest/socialaccount/provider_configuration.html`
- Google provider docs: `https://docs.allauth.org/en/latest/socialaccount/providers/google.html`
- Admin security docs: `https://docs.allauth.org/en/latest/common/admin.html`
- Recent release notes: `https://docs.allauth.org/en/latest/release-notes/recent.html`
- PyPI package page: `https://pypi.org/project/django-allauth/`
