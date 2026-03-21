---
name: crispy-forms
description: "django-crispy-forms 2.6 for Python - practical guide to template-pack setup and form rendering"
metadata:
  languages: "python"
  versions: "2.6"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,forms,templates,bootstrap,pypi,crispy"
---

# django-crispy-forms 2.6 for Python

`django-crispy-forms` keeps Django's normal forms, validation, and view flow, but gives you helper objects and layout primitives so you can control HTML rendering without hand-writing every field template.

This entry is for `django-crispy-forms==2.6`, the current PyPI release as of March 1, 2026.

## Install

Install the core package plus one template-pack package that matches your frontend stack. Since crispy-forms `2.x`, template packs are not bundled in the core package.

```bash
pip install django-crispy-forms crispy-bootstrap5
```

If you use `uv`:

```bash
uv add django-crispy-forms crispy-bootstrap5
```

If you use Poetry:

```bash
poetry add django-crispy-forms crispy-bootstrap5
```

Bootstrap 5 is the current default choice for new projects. If your project is still on Bootstrap 4, use `crispy-bootstrap4` instead and set the corresponding template-pack values.

## Django Setup

Add the core app and the template-pack app to `INSTALLED_APPS`, then set the allowed and default template pack.

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "crispy_forms",
    "crispy_bootstrap5",
]

CRISPY_ALLOWED_TEMPLATE_PACKS = "bootstrap5"
CRISPY_TEMPLATE_PACK = "bootstrap5"
```

There is no package-specific auth flow. Configuration is ordinary Django settings plus your project's normal form security rules such as CSRF protection.

One upstream gotcha: the core install page still shows `uni_form` as a placeholder setting example. Do not copy that into a new Bootstrap-based project; use the value required by the template pack you actually installed.

## Basic Usage

Use the `{% crispy %}` tag when you want helper-driven rendering and layout objects.

```python
from django import forms
from crispy_forms.helper import FormHelper
from crispy_forms.layout import Field, Layout, Submit

class ContactForm(forms.Form):
    name = forms.CharField(max_length=100)
    email = forms.EmailField()
    message = forms.CharField(widget=forms.Textarea)

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.helper = FormHelper()
        self.helper.form_method = "post"
        self.helper.layout = Layout(
            Field("name"),
            Field("email"),
            Field("message", rows=4),
            Submit("submit", "Send"),
        )
```

```django
{% load crispy_forms_tags %}

<form method="post">
  {% csrf_token %}
  {% crispy form %}
</form>
```

If you just want default pack styling without a custom layout, the filter form is shorter:

```django
{% load crispy_forms_tags %}
{{ form|crispy }}
```

Use `{{ form|crispy }}` for quick rendering. Use `{% crispy form %}` when you need a `FormHelper`, custom layout objects, helper options, or custom submit controls.

## FormHelper Options That Matter

Most real setup issues are `FormHelper` issues.

```python
self.helper = FormHelper()
self.helper.form_method = "post"
self.helper.form_action = "/contact/"
self.helper.form_tag = False
self.helper.disable_csrf = True
self.helper.include_media = False
self.helper.attrs = {"data-controller": "contact"}
```

Important behavior:

- `form_method` and `form_action` control the rendered `<form>` tag when `form_tag` is enabled.
- `form_tag = False` is useful when the template already owns the outer `<form>`.
- `disable_csrf = True` only affects crispy's injected token; it does not disable Django's CSRF protection.
- `include_media = False` prevents duplicate widget media when the page template handles media separately.
- `attrs` lets you add arbitrary HTML attributes. Upstream converts underscores in keys to hyphens.
- `render_unmentioned_fields = True` can save you from accidentally hiding fields that were left out of a custom layout.

If `form_tag = False`, crispy-forms will not render `<form>` tags for you, and you still need to provide `{% csrf_token %}` in the template unless you deliberately handle that elsewhere.

## Layout Primitives

The main value of crispy-forms is composing Python layout objects instead of duplicating field HTML in templates.

```python
from crispy_forms.layout import Div, Field, HTML, Layout, Submit

self.helper.layout = Layout(
    Div(
        Field("name", css_class="form-control-lg"),
        Field("email"),
        css_class="row g-3",
    ),
    Field("message", placeholder="What do you need?"),
    HTML("<p class='form-text'>We usually reply within one business day.</p>"),
    Submit("save", "Save"),
)
```

The layout objects you will reach for first:

- `Field` to set widget attributes or wrapper classes for one field
- `Div` to group fields and attach CSS classes
- `Fieldset` when you need a labeled section
- `HTML` for small inline template fragments
- `Submit`, `Button`, and `Reset` for actions

Prefer building helpers per form instance in `__init__` if you mutate layout or helper state. Shared class-level helpers are easy to accidentally reuse across requests.

## Practical Patterns

### ModelForm with helper

```python
from django.forms import ModelForm
from crispy_forms.helper import FormHelper
from crispy_forms.layout import Layout, Submit

class ProjectForm(ModelForm):
    class Meta:
        model = Project
        fields = ["name", "slug", "description"]

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.helper = FormHelper(self)
        self.helper.form_method = "post"
        self.helper.layout = Layout("name", "slug", "description")
        self.helper.add_input(Submit("submit", "Create project"))
```

### Re-render a bound form for AJAX or HTMX responses

When you need HTML for a form fragment after server-side validation, use `render_crispy_form`.

```python
from crispy_forms.utils import render_crispy_form
from django.template.context_processors import csrf

def render_form_fragment(request, form):
    return render_crispy_form(form, context=csrf(request))
```

## Config And Failure Behavior

Useful project-level settings:

- `CRISPY_FAIL_SILENTLY = not DEBUG` controls whether crispy template errors are swallowed or raised.
- `CRISPY_CLASS_CONVERTERS` lets you map Django's default widget classes to classes expected by your CSS framework.
- `CRISPY_ALLOWED_TEMPLATE_PACKS` should include only packs you intentionally support.
- `CRISPY_TEMPLATE_PACK` sets the default pack for forms that do not override `helper.template_pack`.

There is still no package-managed auth, session, or permission layer here. Keep validation, permissions, and persistence in ordinary Django forms, models, and views.

## Common Pitfalls

- Installing only `django-crispy-forms` and forgetting the separate template-pack package required in `2.x`.
- Adding `crispy_forms` to `INSTALLED_APPS` but forgetting the pack app such as `crispy_bootstrap5`.
- Copying old snippets that set `CRISPY_TEMPLATE_PACK = "bootstrap4"` or `uni_form` without matching installed packages.
- Using `{% crispy form %}` without `{% load crispy_forms_tags %}`.
- Setting `form_tag = False` and then forgetting to render the outer `<form>` element and CSRF token yourself.
- Building a custom layout and then assuming omitted fields will still render unless you set `render_unmentioned_fields = True`.
- Expecting crispy-forms to change validation, save behavior, or authorization. It only changes rendering.

## Version-Sensitive Notes For 2.6

- This doc targets `django-crispy-forms==2.6`.
- PyPI marks `2.6` as the latest stable release, published on March 1, 2026.
- The project metadata for `2.6` supports Python `3.10` through `3.14`.
- The project metadata for `2.6` advertises Django `5.2` and `6.0`; if your stack is older, check an earlier crispy-forms release instead of copying 2.6 setup.
- The big compatibility break is still the `2.0` line: template packs moved out of core, so many `1.x` blog posts are wrong for current installs.

## Official Sources

- Core docs root: https://django-crispy-forms.readthedocs.io/en/latest/
- Installation: https://django-crispy-forms.readthedocs.io/en/latest/install.html
- Crispy tag and filters: https://django-crispy-forms.readthedocs.io/en/latest/crispy_tag_forms.html
- Form helper API: https://django-crispy-forms.readthedocs.io/en/latest/form_helper.html
- Layout objects: https://django-crispy-forms.readthedocs.io/en/latest/layouts.html
- PyPI package: https://pypi.org/project/django-crispy-forms/
- Bootstrap 5 template pack: https://pypi.org/project/crispy-bootstrap5/
