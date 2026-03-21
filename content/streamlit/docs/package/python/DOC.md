---
name: package
description: "Streamlit package guide for Python apps with installation, execution flow, configuration, connections, and auth"
metadata:
  languages: "python"
  versions: "1.55.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "streamlit,python,web,ui,dashboard,data-app,chat"
---

# Streamlit Python Package Guide

## Golden Rule

Use `streamlit` for Python-first interactive apps, run the app with the Streamlit CLI instead of `python app.py`, and design with Streamlit's rerun model in mind. Every widget interaction reruns your script unless you deliberately batch work with forms or isolate work with fragments.

## Install

Pin the version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "streamlit==1.55.0"
```

Common alternatives:

```bash
uv add "streamlit==1.55.0"
poetry add "streamlit==1.55.0"
```

Useful CLI commands from the official docs:

```bash
streamlit hello
streamlit config show
streamlit version
```

If you want Streamlit to scaffold a new app:

```bash
streamlit init
```

## Create And Run An App

Minimal app:

```python
import streamlit as st

st.set_page_config(
    page_title="Demo",
    page_icon=":material/rocket_launch:",
    layout="wide",
)

st.title("Hello, Streamlit")
name = st.text_input("Name", value="Ada")

if st.button("Greet"):
    st.success(f"Hello, {name}!")
```

Run it:

```bash
streamlit run app.py
```

Notes:

- If you run `streamlit run` with no file argument, Streamlit looks for `streamlit_app.py` in the current working directory.
- Your working directory matters because `.streamlit/config.toml` and `.streamlit/secrets.toml` are resolved relative to wherever you call `streamlit run`.
- `streamlit run` also accepts a directory; in that case Streamlit looks for `streamlit_app.py` inside that directory.

## Core Usage Patterns

### 1. Accept the rerun model

By default, Streamlit reruns the whole script on each interaction. This is normal, not a bug.

Use these tools when reruns become expensive or awkward:

- `st.form` to batch related widget updates behind a submit button
- `st.fragment` to rerun only part of the app
- `st.rerun()` to trigger an immediate rerun
- `st.stop()` to end the current run early

Example with a form:

```python
import streamlit as st

with st.form("filters"):
    city = st.text_input("City")
    limit = st.number_input("Rows", min_value=1, max_value=100, value=10)
    submitted = st.form_submit_button("Apply")

if submitted:
    st.write({"city": city, "limit": limit})
```

Example with an independently rerunning fragment:

```python
import streamlit as st
import pandas as pd
import numpy as np

@st.fragment(run_every="10s")
def live_chart() -> None:
    df = pd.DataFrame(np.random.randn(20, 3), columns=["a", "b", "c"])
    st.line_chart(df)

live_chart()
```

### 2. Keep per-user state in `st.session_state`

Use Session State for values that must survive reruns for one user session.

```python
import streamlit as st

if "messages" not in st.session_state:
    st.session_state.messages = []

prompt = st.chat_input("Say something")
if prompt:
    st.session_state.messages.append({"role": "user", "text": prompt})

for message in st.session_state.messages:
    with st.chat_message(message["role"]):
        st.write(message["text"])
```

Important behavior:

- Session State persists across reruns and across pages in a multipage app.
- Session State is tied to the WebSocket session, so a browser reload or Markdown-link navigation resets it.
- Inside forms, only `st.form_submit_button` supports callbacks.
- Do not mutate a widget's value through Session State after that widget has already been instantiated in the same run.

### 3. Cache data and resources intentionally

Use `@st.cache_data` for data results that should be copied per caller:

```python
import streamlit as st
import pandas as pd

@st.cache_data(ttl="10m")
def load_csv(path: str) -> pd.DataFrame:
    return pd.read_csv(path)
```

Use `@st.cache_resource` for shared global resources such as clients, engines, or models:

```python
import streamlit as st
from sqlalchemy import create_engine

@st.cache_resource
def get_engine():
    return create_engine(st.secrets.database.url)
```

Rules that matter in practice:

- `@st.cache_data` stores return values in pickled form, so returns must be pickleable.
- `@st.cache_resource` returns shared objects and those objects must be thread-safe if they are global.
- If an argument should not participate in cache hashing, prefix that parameter name with `_`.
- Widgets inside cached functions are supported but still experimental and can create excessive cache growth.
- Async objects are not a safe caching target; keep cached values sync-friendly.

### 4. Use Streamlit-native UI and data primitives

The practical center of most apps is:

- display: `st.write`, `st.markdown`, `st.dataframe`, `st.metric`, `st.json`
- input: `st.text_input`, `st.selectbox`, `st.multiselect`, `st.slider`, `st.file_uploader`, `st.chat_input`
- layout: `st.sidebar`, `st.columns`, `st.tabs`, `st.expander`, `st.container`, `st.popover`
- long-running work: `st.spinner`, `st.status`, `st.progress`, `st.toast`

For LLM or assistant-style apps, the native chat elements are usually the right default:

```python
import streamlit as st

with st.chat_message("assistant"):
    st.write("How can I help?")

prompt = st.chat_input("Ask a question")
if prompt:
    with st.chat_message("user"):
        st.write(prompt)
```

### 5. Prefer built-in navigation for multipage apps

Modern multipage apps should prefer `st.Page` and `st.navigation` in the entrypoint file:

```python
import streamlit as st

def home():
    st.title("Home")

def reports():
    st.title("Reports")

page = st.navigation(
    [
        st.Page(home, title="Home", icon=":material/home:", default=True),
        st.Page(reports, title="Reports", icon=":material/assessment:"),
    ],
    position="sidebar",
)

page.run()
```

Important behavior:

- The entrypoint file becomes a router and reruns on every app interaction.
- Once any session executes `st.navigation`, the app ignores the `pages/` directory across all sessions.
- In `1.55.0`, `st.Page` adds a `visibility` parameter so pages can be hidden from navigation while staying routable.

## Configuration, Secrets, And Connections

### `config.toml`

Per-project config lives at `.streamlit/config.toml` in the working directory:

```toml
[server]
port = 8501
headless = true
maxUploadSize = 400

[theme]
primaryColor = "#ff4b4b"
base = "light"
```

Global config lives at `~/.streamlit/config.toml`.

Precedence is:

1. CLI flags like `--server.port=8502`
2. `STREAMLIT_*` environment variables
3. Per-project `.streamlit/config.toml`
4. Global `~/.streamlit/config.toml`

Useful facts:

- Theme changes in `config.toml` apply live while the app is running.
- Non-theme config changes usually require a server restart.
- `streamlit config show` prints available config keys and current values.
- `server.maxUploadSize` defaults to 200 MB unless you raise it.

### `secrets.toml`

Per-project secrets live at `.streamlit/secrets.toml`:

```toml
OPENAI_API_KEY = "..."

[database]
url = "sqlite:///app.db"

[connections.analytics]
url = "postgresql+psycopg://user:pass@host/db"
```

Access them from code:

```python
import streamlit as st

api_key = st.secrets["OPENAI_API_KEY"]
db_url = st.secrets.database.url
```

Global secrets live at `~/.streamlit/secrets.toml`. Project secrets override duplicate global keys.

### `st.connection`

Use `st.connection()` when a built-in or custom connection fits your data source:

```python
import streamlit as st

conn = st.connection("analytics", type="sql")
df = conn.query("select * from events limit 50")
st.dataframe(df)
```

Notes:

- Streamlit reads connection secrets from `[connections.<name>]` in `secrets.toml`.
- Returned connections are internally cached with `st.cache_resource`.
- Built-in connection types include SQL and Snowflake.

## Authentication

Streamlit has native OIDC auth. For package version `1.55.0`, the relevant API is `st.login()`, `st.user`, and `st.logout()`.

Install the auth extra when needed:

```bash
python -m pip install "streamlit[auth]==1.55.0"
```

Or install the required dependency directly:

```bash
python -m pip install "Authlib>=1.3.2"
```

Example `.streamlit/secrets.toml`:

```toml
[auth]
redirect_uri = "http://localhost:8501/oauth2callback"
cookie_secret = "replace-with-a-long-random-value"
client_id = "your-client-id"
client_secret = "your-client-secret"
server_metadata_url = "https://accounts.google.com/.well-known/openid-configuration"
```

Example app code:

```python
import streamlit as st

if not st.user.is_logged_in:
    if st.button("Log in"):
        st.login()
else:
    st.write(f"Signed in as {st.user.name}")
    if st.button("Log out"):
        st.logout()
```

Auth constraints that matter:

- Streamlit supports OIDC providers, not generic OAuth providers.
- `redirect_uri` and `cookie_secret` are required in `[auth]`.
- URLs in auth settings must be absolute.
- Auth is not supported for embedded apps.
- The identity cookie lasts 30 days if the user closes the app without logging out.
- `st.user` is read at session start, so other open tabs will not update until they start a new session.

## Common Pitfalls

- Do not launch a Streamlit app with `python app.py`. Use `streamlit run`.
- Do not assume widgets mutate state in place. They trigger reruns and you must rebuild the UI from current state.
- Do not put non-thread-safe global clients inside `@st.cache_resource`.
- Do not store secrets in code or commit `.streamlit/secrets.toml`.
- Do not expect Session State to survive browser reloads or Markdown-link navigation.
- Do not mix older `pages/` directory routing with `st.navigation` and expect both to be active.
- Do not copy old examples that use `st.experimental_user` or `st.experimental_*_query_params`; those are removed in current releases.
- If a widget change should not rerun expensive work immediately, move the widgets into `st.form` or isolate the expensive section in `st.fragment`.
- `st.file_uploader` and `st.camera_input` are not supported inside cached functions.

## Version-Sensitive Notes For `1.55.0`

- PyPI lists `streamlit 1.55.0` as released on March 3, 2026.
- `1.55.0` adds dynamic container reruns: `st.tabs`, `st.popover`, and `st.expander` can use `on_change` so open and close events can trigger reruns.
- `1.55.0` adds widget binding so most non-trigger widgets can use a `bind` parameter to sync more directly with query parameters.
- `st.Page` in `1.55.0` adds `visibility`, which is useful for hidden-but-routable pages.
- Since `1.54.0`, `st.experimental_user` is removed. Use `st.user`.
- Since `1.54.0`, `st.experimental_get_query_params` and `st.experimental_set_query_params` are removed. Use `st.query_params`.
- Since modern `1.4x+` releases, `st.set_page_config()` is additive and can be called multiple times. Older blog posts that require a single first call are outdated.
- Since `1.51.0`, Python 3.9 is no longer supported. `1.55.0` requires Python 3.10+.

## Official Sources

- Streamlit API reference: `https://docs.streamlit.io/develop/api-reference`
- Installation guide: `https://docs.streamlit.io/get-started/installation`
- CLI reference: `https://docs.streamlit.io/develop/api-reference/cli`
- Release notes: `https://docs.streamlit.io/develop/quick-reference/release-notes/2026`
- PyPI package page: `https://pypi.org/project/streamlit/`
