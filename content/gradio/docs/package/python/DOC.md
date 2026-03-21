---
name: package
description: "Gradio Python package for building web UIs, chat apps, and MCP-enabled demos around Python functions"
metadata:
  languages: "python"
  versions: "6.9.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "gradio,python,ui,chatbot,ml,fastapi,mcp"
---

# Gradio Python Package Guide

## Golden Rule

Use `gradio` as the UI layer around ordinary Python callables, import it as `import gradio as gr`, and pick the highest-level abstraction that fits the app:

- `gr.Interface(...)` for one function with clear inputs and outputs
- `gr.ChatInterface(...)` for chatbot-style apps
- `gr.Blocks()` when you need custom layout, multiple events, shared state, or tighter control over queuing and launch behavior

For Gradio 6.x, assume launch-time settings now belong on `.launch()` instead of the `Blocks` constructor, and do not rely on old `/predict` or `/chat` API endpoint names unless you set `api_name` explicitly.

## Install

Pin the package version your project expects:

```bash
python -m pip install "gradio==6.9.0"
```

Common alternatives:

```bash
uv add "gradio==6.9.0"
poetry add "gradio==6.9.0"
```

Optional extras:

```bash
python -m pip install "gradio[mcp]==6.9.0"
python -m pip install "gradio[oauth]==6.9.0"
```

Use the `mcp` extra if you want `.launch(mcp_server=True)` support without manually resolving MCP dependencies.

## Fast Start

### `Interface` for a single function

Use `Interface` when one Python function maps neatly to inputs and outputs:

```python
import gradio as gr

def classify_text(text: str) -> str:
    if "error" in text.lower():
        return "negative"
    return "positive"

demo = gr.Interface(
    fn=classify_text,
    inputs=gr.Textbox(label="Text"),
    outputs=gr.Label(label="Prediction"),
    title="Simple classifier",
)

if __name__ == "__main__":
    demo.launch()
```

`fn` parameters line up with the input components in order, and the return value must match the output components you declared.

### `Blocks` for custom layouts and events

Use `Blocks` once you need multiple components, custom layout, or event wiring:

```python
import gradio as gr

def reverse_text(text: str) -> str:
    return text[::-1]

with gr.Blocks() as demo:
    gr.Markdown("## Reverse text")
    with gr.Row():
        inp = gr.Textbox(label="Input")
        out = gr.Textbox(label="Output")
    run = gr.Button("Reverse")
    run.click(reverse_text, inputs=inp, outputs=out)

if __name__ == "__main__":
    demo.launch(theme="soft")
```

In Gradio 6.x, app-level settings such as `theme`, `css`, and related launch behavior should be treated as launch concerns, not old `Blocks(...)` constructor-era behavior from 5.x examples.

### `ChatInterface` for chatbot UIs

`ChatInterface` expects a function shaped like `fn(message, history, ...)`. The current history format is a list of OpenAI-style message dictionaries with `role` and `content`.

```python
import gradio as gr

def respond(message: str, history: list[dict]) -> str:
    turns = len([item for item in history if item["role"] == "assistant"])
    return f"Turn {turns + 1}: you said {message!r}"

demo = gr.ChatInterface(
    fn=respond,
    title="Echo bot",
    save_history=True,
)

if __name__ == "__main__":
    demo.launch()
```

`save_history=True` stores chat history in the browser's local storage, which is convenient for demos but worth remembering if the app handles sensitive conversations.

Streaming is just a generator:

```python
import time
import gradio as gr

def slow_echo(message: str, history: list[dict]):
    for index in range(len(message)):
        time.sleep(0.05)
        yield f"You typed: {message[:index + 1]}"

gr.ChatInterface(slow_echo).launch()
```

If you enable `multimodal=True`, the first parameter is no longer a plain string. It becomes a dictionary with `"text"` and `"files"` keys:

```python
import gradio as gr

def count_uploads(message: dict, history: list[dict]) -> str:
    return f"text={message['text']!r}, files={len(message['files'])}"

demo = gr.ChatInterface(
    fn=count_uploads,
    multimodal=True,
    textbox=gr.MultimodalTextbox(file_count="multiple", file_types=["image"]),
)

demo.launch()
```

## Launch, Hosting, And Environment

`launch()` is where Gradio runtime behavior is controlled.

Local-network access:

```python
demo.launch(server_name="0.0.0.0", server_port=8000)
```

Equivalent environment variables:

```bash
export GRADIO_SERVER_NAME="0.0.0.0"
export GRADIO_SERVER_PORT=8000
```

Useful environment variables:

- `GRADIO_SERVER_NAME`: defaults to `127.0.0.1`; set to `0.0.0.0` for LAN/container access
- `GRADIO_SERVER_PORT`: defaults to `7860`
- `GRADIO_ROOT_PATH`: use when deployed behind a reverse proxy or non-root mount path
- `GRADIO_ALLOWED_PATHS` and `GRADIO_BLOCKED_PATHS`: comma-separated absolute paths controlling what Gradio may serve
- `GRADIO_SSR_MODE`: enables server-side rendering; Gradio docs note SSR requires Node 20+
- `GRADIO_ANALYTICS_ENABLED`: opt out of telemetry if your environment requires it
- `GRADIO_DEBUG=1`: keeps the main thread alive for easier debugging in notebook-style environments

If you need HTTPS locally, `launch()` also accepts `ssl_keyfile` and `ssl_certfile`.

## Authentication, Sharing, And Reverse Proxies

### Built-in auth

For a quick password gate:

```python
demo.launch(auth=("admin", "pass1234"))
```

Or provide a validation callback:

```python
def check_login(username: str, password: str) -> bool:
    return username == password

demo.launch(auth=check_login)
```

This is convenient but basic. The official guide calls out that it is not a substitute for stronger controls such as MFA, rate limiting, or account lockout.

### Share links

```python
demo.launch(share=True)
```

Gradio share links are proxied public URLs for your local app. The official docs state that Gradio’s share servers do not store app data and that share links expire after 1 week.

### Mount inside FastAPI

If your application already uses FastAPI, mount a Gradio app instead of running a separate server:

```python
from fastapi import FastAPI
import gradio as gr

app = FastAPI()

def greet(name: str) -> str:
    return f"Hello {name}"

demo = gr.Interface(greet, "textbox", "textbox")
app = gr.mount_gradio_app(app, demo, path="/gradio")
```

Important mount options:

- `root_path`: required when a reverse proxy strips the public prefix before forwarding requests
- `auth`: basic username/password auth for the mounted app
- `auth_dependency`: use this instead of `auth` when you already have external auth in FastAPI or OAuth middleware
- `allowed_paths` and `blocked_paths`: absolute file-system paths controlling what mounted apps may serve
- `ssr_mode=True`: enables server-side rendering and requires Node 20+

For external auth:

```python
from fastapi import FastAPI, Request
import gradio as gr

def current_user(request: Request) -> str | None:
    user = getattr(request.state, "user", None)
    return getattr(user, "id", None)

app = FastAPI()
demo = gr.Interface(lambda text: text.upper(), "textbox", "textbox")
app = gr.mount_gradio_app(app, demo, path="/gradio", auth_dependency=current_user)
```

## Queues, API Exposure, And Performance

Every Gradio event listener has a queue. By default, each listener processes one request at a time.

Use per-listener concurrency when a function can safely run more than one request in parallel:

```python
import gradio as gr

def generate(prompt: str) -> str:
    return f"generated: {prompt}"

with gr.Blocks() as demo:
    prompt = gr.Textbox()
    output = gr.Textbox()
    button = gr.Button("Run")
    button.click(generate, prompt, output, concurrency_limit=4)

demo.queue(default_concurrency_limit=2, max_size=32)
demo.launch()
```

Key rules:

- `concurrency_limit=None` removes the limit for that event listener
- `concurrency_id="gpu_queue"` lets multiple listeners share the same queue and concurrency budget
- `demo.queue(max_size=...)` caps queued events and fails fast when the queue is full
- `api_open=True` on `Blocks.queue()` allows direct backend REST routes to skip the queue; only use this intentionally

## File Access And Output Handling

Gradio can serve files back to the browser, which makes file exposure settings important.

Prefer explicit allowlists:

```python
demo.launch(
    allowed_paths=["/srv/gradio/public-assets"],
    blocked_paths=["/srv/gradio/public-assets/secrets"],
)
```

Rules that matter:

- `allowed_paths` and `blocked_paths` must be absolute paths
- `blocked_paths` takes precedence over `allowed_paths`
- exposing a directory means all files inside it and its subdirectories can be reachable by app users
- `GRADIO_TEMP_DIR` controls where temporary files are stored
- `max_file_size` can enforce upload limits when you mount a Gradio app or configure launch options

## MCP Support

Gradio 6.9.0 can expose documented functions as MCP tools.

Minimal MCP server:

```python
import gradio as gr

def letter_counter(word: str, letter: str) -> int:
    """Count how many times a letter appears in a word."""
    return word.lower().count(letter.lower())

demo = gr.Interface(
    fn=letter_counter,
    inputs=[gr.Textbox("strawberry"), gr.Textbox("r")],
    outputs=gr.Number(),
    api_name="predict",
)

demo.launch(mcp_server=True)
```

Practical notes:

- install with `gradio[mcp]`
- Gradio uses your function docstring and type hints to describe the tool
- the MCP endpoint is served under `/gradio_api/mcp/`
- `GRADIO_MCP_SERVER=True` can enable MCP mode via environment variable instead of code

## Common Pitfalls

- Do not copy Gradio 5.x launch snippets blindly. Gradio 6 moved or renamed several parameters.
- Do not assume chat APIs still default to `/chat` or `/predict`. In 6.x, `Interface` and `ChatInterface` use the function name as the default API route unless you set `api_name`.
- If `multimodal=True`, your chat function’s first argument must accept a dictionary, not a string.
- If your app is behind a reverse proxy and file URLs break, set `root_path` explicitly.
- If users cannot see uploaded or generated files, check `allowed_paths`, `blocked_paths`, and temp-directory behavior before debugging component code.
- Built-in `auth` is a convenience gate, not a full security layer.
- If you enable SSR without Node 20+, rendering will fail.
- `api_open=True` can bypass queue limits; that is useful for some internal systems and surprising in public deployments.
- If you expose directories with `allowed_paths`, you are exposing every file under those directories unless a matching blocked path overrides them.

## Version-Sensitive Notes For 6.9.0

- PyPI lists `gradio 6.9.0` as the latest release on March 12, 2026, with Python `>=3.10`.
- The official Gradio 6 migration guide says Gradio 6 is the actively maintained major line and recommends upgrading through `gradio==5.50` first if you need deprecation warnings before a 6.x migration.
- `show_api` in launch/event listeners is replaced by `footer_links` and `api_visibility`.
- Old `Blocks(...)` constructor-era app settings moved to `launch()`.
- `Interface` and `ChatInterface` now default API names to the Python function name, which matters if external callers depend on the old endpoint paths.
