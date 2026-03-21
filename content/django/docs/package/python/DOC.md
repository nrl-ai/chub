---
name: package
description: "Django package guide for Python projects using the official Django 6.0 docs"
metadata:
  languages: "python"
  versions: "6.0.3"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "django,python,web,framework,orm,templates,admin"
---

# Django Python Package Guide

## Golden Rule

Use the Django 6.0 docs for setup and behavior, and stay on the latest 6.0.x patch if your project is pinned to the 6.0 series. As of March 11, 2026, PyPI lists `Django 6.0.3`, while the docs URL pointed to the older Django 5.1 reference.

## Install

Use a virtual environment and pin the version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "Django==6.0.3"
```

Common alternatives:

```bash
uv add "Django==6.0.3"
poetry add "Django==6.0.3"
```

Optional password hasher extras published on PyPI:

```bash
python -m pip install "Django[argon2]==6.0.3"
python -m pip install "Django[bcrypt]==6.0.3"
```

## Initialize A Project

Create a project, apply built-in migrations, and run the dev server:

```bash
django-admin startproject mysite
cd mysite
python manage.py migrate
python manage.py runserver
```

The generated project includes these important files:

- `manage.py`: command entry point for migrations, dev server, shell, and admin tasks
- `mysite/settings.py`: installed apps, database config, middleware, templates, static config
- `mysite/urls.py`: root URL router
- `mysite/asgi.py`: ASGI application entry point
- `mysite/wsgi.py`: WSGI application entry point

## Core Workflow

### Create an app

```bash
python manage.py startapp polls
```

Add the app to `INSTALLED_APPS` in `settings.py` before expecting models, templates, or admin registration to work.

### Define a model and create migrations

```python
# polls/models.py
from django.db import models

class Question(models.Model):
    question_text = models.CharField(max_length=200)
    pub_date = models.DateTimeField("date published")

    def __str__(self) -> str:
        return self.question_text
```

```bash
python manage.py makemigrations polls
python manage.py migrate
```

Useful inspection commands:

```bash
python manage.py sqlmigrate polls 0001
python manage.py shell
```

### Add views and routes

```python
# polls/views.py
from django.http import HttpResponse

def index(request):
    return HttpResponse("Hello from polls")
```

```python
# polls/urls.py
from django.urls import path

from . import views

urlpatterns = [
    path("", views.index, name="index"),
]
```

```python
# mysite/urls.py
from django.contrib import admin
from django.urls import include, path

urlpatterns = [
    path("polls/", include("polls.urls")),
    path("admin/", admin.site.urls),
]
```

### Render templates instead of hard-coded responses

Keep app templates namespaced under the app name to avoid collisions:

```text
polls/
  templates/
    polls/
      index.html
```

```python
# polls/views.py
from django.shortcuts import render

def index(request):
    context = {"latest_question_list": []}
    return render(request, "polls/index.html", context)
```

### Register models in the admin

```python
# polls/admin.py
from django.contrib import admin

from .models import Question

admin.site.register(Question)
```

Create an admin user:

```bash
python manage.py createsuperuser
```

## Settings And Environment

Django loads settings from the module pointed to by `DJANGO_SETTINGS_MODULE`. The default `manage.py`, `asgi.py`, and `wsgi.py` created by `startproject` set this for you.

For production, keep secrets and environment-specific values outside source control. A common pattern is to split settings into `base.py`, `dev.py`, and `prod.py`, then select the active module with `DJANGO_SETTINGS_MODULE`.

Minimum production settings to review explicitly:

- `DEBUG = False`
- `ALLOWED_HOSTS = ["your-domain.example"]`
- `CSRF_TRUSTED_ORIGINS = ["https://your-domain.example"]` when needed behind HTTPS proxies or cross-origin admin flows
- `SECRET_KEY` from an environment variable
- `DATABASES` from environment-specific config
- `STATIC_ROOT` and static file serving strategy

Example environment-driven settings:

```python
import os

DEBUG = os.getenv("DJANGO_DEBUG", "").lower() == "true"
SECRET_KEY = os.environ["DJANGO_SECRET_KEY"]
ALLOWED_HOSTS = os.getenv("DJANGO_ALLOWED_HOSTS", "localhost,127.0.0.1").split(",")
CSRF_TRUSTED_ORIGINS = [
    origin for origin in os.getenv("DJANGO_CSRF_TRUSTED_ORIGINS", "").split(",") if origin
]
```

## ASGI vs WSGI

Use the generated application object with your server instead of `runserver` in production.

ASGI:

```python
from django.core.asgi import get_asgi_application

application = get_asgi_application()
```

WSGI:

```python
from django.core.wsgi import get_wsgi_application

application = get_wsgi_application()
```

Use ASGI if your stack needs async views, long-lived connections, or additional ASGI tooling. Use WSGI for conventional sync deployments.

## Common Commands

```bash
python manage.py runserver
python manage.py makemigrations
python manage.py migrate
python manage.py createsuperuser
python manage.py shell
python manage.py test
python manage.py collectstatic
python manage.py check --deploy
```

## Common Pitfalls

- Do not use `runserver` in production. Deploy the ASGI or WSGI application behind a real application server.
- `makemigrations` writes migration files; `migrate` applies them. Agents often do one and forget the other.
- New apps must be added to `INSTALLED_APPS` before model discovery, app configs, admin registration, and template loading behave as expected.
- Keep templates under `templates/<app_name>/...`; flat template names collide quickly in larger projects.
- Set `ALLOWED_HOSTS` before turning `DEBUG` off or you will get `DisallowedHost` errors.
- Keep `SECRET_KEY` out of the repo. If rotating keys, Django 6.0 still supports `SECRET_KEY_FALLBACKS` for staged rotation.
- The deployment checklist is not optional. Run `python manage.py check --deploy` against production settings before release.
- Async Django is not the same as "everything is non-blocking". Avoid calling blocking libraries directly from async views.

## Version-Sensitive Notes For Django 6.0

- Django 6.0 requires Python 3.12 or later.
- The Django 6.0 release adds built-in Content Security Policy support, template partials, and a task framework. These are new enough that many third-party blog posts will not cover them correctly.
- Django 6.0.3 is a patch release in the 6.0 line; check the 6.0.3 release notes before copying examples from early 6.0 articles, especially around security fixes and bugfixes.
- If a project still targets Python 3.10 or 3.11, it cannot move to Django 6.0 without a Python upgrade.

## Official Sources

- Django 6.0 docs root: https://docs.djangoproject.com/en/6.0/
- Installation guide: https://docs.djangoproject.com/en/6.0/intro/install/
- Tutorial part 1: https://docs.djangoproject.com/en/6.0/intro/tutorial01/
- Tutorial part 2: https://docs.djangoproject.com/en/6.0/intro/tutorial02/
- Settings topic guide: https://docs.djangoproject.com/en/6.0/topics/settings/
- Deployment checklist: https://docs.djangoproject.com/en/6.0/howto/deployment/checklist/
- ASGI deployment: https://docs.djangoproject.com/en/6.0/howto/deployment/asgi/
- WSGI deployment: https://docs.djangoproject.com/en/6.0/howto/deployment/wsgi/
- Django 6.0 release notes: https://docs.djangoproject.com/en/6.0/releases/6.0/
- Django 6.0.3 release notes: https://docs.djangoproject.com/en/6.0/releases/6.0.3/
- PyPI package page: https://pypi.org/project/django/
