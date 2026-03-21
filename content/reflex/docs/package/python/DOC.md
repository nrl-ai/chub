---
name: package
description: "Reflex Python package guide for building full-stack web apps in pure Python"
metadata:
  languages: "python"
  versions: "0.8.27"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "reflex,python,web,framework,ui,full-stack"
---

# Reflex Python Package Guide

## Golden Rule

Use the official `reflex` package, import it as `import reflex as rx`, start new apps with `reflex init`, and let Reflex compile Python state and components into the frontend/backend runtime it manages for you. Do not mix in ad hoc React files unless you are intentionally extending Reflex with custom components.

## Install

Pin the package version your project expects:

```bash
python -m pip install "reflex==0.8.27"
```

Common alternatives:

```bash
uv add "reflex==0.8.27"
poetry add "reflex==0.8.27"
```

PyPI currently publishes optional extras for database and monitoring features:

```bash
python -m pip install "reflex[db]==0.8.27"
python -m pip install "reflex[monitoring]==0.8.27"
```

## Initialize A New App

The official installation flow uses `uv`, but the CLI also works from a normal virtualenv.

Using `uv`:

```bash
mkdir my_app
cd my_app
uv init
uv add "reflex==0.8.27"
uv run reflex init
uv run reflex run
```

Using `venv` + `pip`:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "reflex==0.8.27"
reflex init
reflex run
```

Important generated files:

- `rxconfig.py`: app-level Reflex configuration
- `<app_name>/<app_name>.py`: default app entry module created by `reflex init`
- `.web/`: generated frontend build artifacts; treat this as generated output

## Core Usage

Reflex apps are built from three main pieces:

- components such as `rx.text`, `rx.button`, `rx.vstack`
- state classes derived from `rx.State`
- pages added to an `rx.App()`

Minimal counter example:

```python
import reflex as rx

class CounterState(rx.State):
    count: int = 0

    @rx.var
    def parity(self) -> str:
        return "even" if self.count % 2 == 0 else "odd"

    def increment(self):
        self.count += 1

def index() -> rx.Component:
    return rx.vstack(
        rx.heading("Counter"),
        rx.text("Count: ", CounterState.count),
        rx.text("Parity: ", CounterState.parity),
        rx.button("Increment", on_click=CounterState.increment),
        spacing="3",
        padding="2rem",
    )

app = rx.App()
app.add_page(index, route="/")
```

Patterns the official docs lean on:

- Use `@rx.var` for computed values derived from state.
- Use event handlers on `rx.State` methods to mutate state.
- Use `rx.cond(...)` for conditional rendering and `rx.foreach(...)` for list rendering instead of raw Python `if`/`for` inside the component tree.
- Use the `@rx.page(...)` decorator when page metadata or `on_load` behavior belongs next to the page function.

## Configuration, Environment Variables, And Auth

Reflex configuration lives in `rxconfig.py`:

```python
import reflex as rx

config = rx.Config(
    app_name="my_app",
    api_url="http://localhost:8000",
    backend_port=8000,
    frontend_port=3000,
    db_url="sqlite:///reflex.db",
    env_file=".env",
)
```

Practical rules:

- Treat `rxconfig.py` as the source of truth for ports, URLs, database connection, and deployment-facing config.
- Keep secrets out of source control. Put them in `.env` locally and in your hosting secret store for deployed apps.
- The config reference says config values can be overridden with environment variables prefixed by `REFLEX_`, for example `REFLEX_API_URL` or `REFLEX_DB_URL`.
- If you deploy to Reflex Cloud, authenticate the CLI with `reflex cloud login` before deploying or managing secrets.

Reflex itself is not your identity provider. For application auth, keep credentials in environment variables and run any session/bootstrap checks from page load hooks or state methods rather than hardcoding secrets into components.

## Database Workflow

If your app uses the built-in database workflow, define models with `rx.Model` and run the database CLI explicitly.

Example model:

```python
import reflex as rx

class Todo(rx.Model, table=True):
    title: str
    done: bool = False
```

Typical commands:

```bash
reflex db init
reflex db makemigrations --message "create todo table"
reflex db migrate
```

Use a real database URL in `rxconfig.py` or via environment variables before running migrations in shared environments.

## Common Commands

```bash
reflex init
reflex run
reflex run --env prod
reflex export
reflex db init
reflex db makemigrations
reflex db migrate
reflex cloud login
reflex deploy
```

## Common Pitfalls

- Do not instantiate `rx.State` directly. Define a subclass and let Reflex manage it.
- Do not mutate state from arbitrary helper code; mutate it from state event handlers so the framework can track updates correctly.
- Raw Python control flow does not replace Reflex rendering helpers inside component trees. Use `rx.cond` and `rx.foreach`.
- `reflex init` generates project structure and config. Running `reflex run` in an uninitialized directory is a common mistake.
- `.web/` is generated output. Do not hand-edit it and expect changes to persist.
- If pages or models are not discovered, make sure the relevant modules are imported by the app entrypoint before assuming the framework is broken.
- Database migrations are not automatic. Define the model, then run `reflex db makemigrations` and `reflex db migrate`.
- Self-hosting and exported frontend deployments need the frontend/backend URLs aligned, or browser calls will point at the wrong backend.

## Version-Sensitive Notes For 0.8.27

- PyPI currently lists `0.8.27` as the package version for `reflex`, with Python support `>=3.10,<4.0`.
- The public docs site is a rolling documentation set rather than a version-pinned doc tree. For exact `0.8.27` behavior, prefer the generated project files and `reflex --help` over older blog posts or issue comments.
- PyPI advertises `db` and `monitoring` extras. Keep install commands explicit when your project depends on those optional surfaces.
- Inference from current upstream docs: configuration naming is slightly inconsistent across pages. The config reference documents `REFLEX_<FIELD>` overrides, while self-hosting examples still show legacy bare names such as `API_URL` and `FRONTEND_PORT`. Prefer the `REFLEX_` form in new code unless the specific deployment guide for your setup says otherwise.

## Official Links

- API reference: `https://reflex.dev/docs/api-reference/`
- Getting started: `https://reflex.dev/docs/getting-started/introduction/`
- Installation: `https://reflex.dev/docs/getting-started/installation/`
- State overview: `https://reflex.dev/docs/state/overview/`
- Config reference: `https://reflex.dev/docs/api-reference/config/reflex-conf/`
- CLI reference: `https://reflex.dev/docs/api-reference/cli/`
- Deploy quick start: `https://reflex.dev/docs/hosting/deploy-quick-start/`
- Self hosting: `https://reflex.dev/docs/hosting/self-hosting/`
