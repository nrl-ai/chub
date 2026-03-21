---
name: package
description: "Textual framework for building terminal and browser UIs in Python"
metadata:
  languages: "python"
  versions: "8.1.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "textual,python,tui,terminal-ui,async,css,widgets"
---

# Textual Python Package Guide

## Golden Rule

Build Textual apps by subclassing `App`, declaring widgets in `compose()`, styling with Textual CSS, and moving slow I/O into workers. If you update the UI in response to network or blocking calls directly inside a message handler, the app will feel frozen.

## Install

Pin the package version your project expects:

```bash
python -m pip install "textual==8.1.1"
```

For local app development, install the separate devtools package too:

```bash
python -m pip install "textual==8.1.1" textual-dev
```

Useful variants:

```bash
python -m pip install "textual[syntax]==8.1.1"
uv add "textual==8.1.1"
poetry add "textual==8.1.1"
```

Notes:

- `textual-dev` provides the `textual` CLI used for `textual run`, `textual serve`, and the dev console.
- The `syntax` extra enables syntax highlighting in `TextArea`.
- Textual runs on Linux, macOS, and Windows. On macOS, the official docs recommend a modern terminal such as iTerm2, Ghostty, Kitty, or WezTerm instead of the default Terminal app.

## Minimal App

This is a practical starting point that shows the core Textual patterns: `App`, `compose()`, CSS, DOM queries, and reactivity.

```python
from textual.app import App, ComposeResult
from textual.containers import Vertical
from textual.reactive import reactive
from textual.widgets import Footer, Header, Input, Static

class Greeting(Static):
    name = reactive("world")

    def watch_name(self, name: str) -> None:
        self.update(f"Hello, {name or 'world'}!")

class GreetingApp(App[None]):
    CSS = """
    Screen {
        align: center middle;
    }

    Vertical {
        width: 40;
    }

    Input {
        margin-bottom: 1;
    }

    Greeting {
        height: 3;
        border: round $accent;
        content-align: center middle;
    }
    """

    def compose(self) -> ComposeResult:
        yield Header()
        with Vertical():
            yield Input(placeholder="Type your name")
            yield Greeting()
        yield Footer()

    def on_mount(self) -> None:
        self.query_one(Input).focus()

    def on_input_changed(self, event: Input.Changed) -> None:
        self.query_one(Greeting).name = event.value.strip()

if __name__ == "__main__":
    GreetingApp().run()
```

Run it with Python:

```bash
python greeting_app.py
```

Or with devtools:

```bash
textual run greeting_app.py
textual run --dev greeting_app.py
```

## Core Usage Patterns

### App structure

Most Textual apps follow this shape:

1. Subclass `App`.
2. Yield widgets from `compose()`.
3. Handle events with `on_<widget>_<message>()` methods such as `on_button_pressed` or `on_input_changed`.
4. Use `query_one(...)` or `query(...)` to find widgets in the DOM when you need to update them.

`App.run()` enters terminal application mode and Textual handles keyboard and mouse input until the app exits. Textual defines default bindings including `ctrl+q` to quit.

### Styling with Textual CSS

Prefer `CSS_PATH` for real apps and inline `CSS` only for short scripts or examples.

```python
from textual.app import App, ComposeResult
from textual.widgets import Button, Label

class QuestionApp(App[str]):
    CSS_PATH = "question.tcss"

    def compose(self) -> ComposeResult:
        yield Label("Do you love Textual?", id="question")
        yield Button("Yes", id="yes", variant="primary")
        yield Button("No", id="no", variant="error")
```

Example `question.tcss`:

```css
#question {
    text-style: bold;
    content-align: center middle;
}

Button {
    width: 1fr;
}

.danger {
    background: red;
    color: white;
}
```

What matters operationally:

- Textual selectors work on widget type names, `#id`, `.class`, and combinators.
- Widgets accept `id=` and `classes=` in constructors so CSS and DOM queries can target them consistently.
- Inline `CSS` is loaded after `CSS_PATH`, so inline rules win on equivalent specificity.
- `textual run --dev my_app.py` live-reloads CSS changes and connects to the dev console.

### Reactive state

Use `reactive(...)` for UI state that should automatically refresh or trigger watchers.

```python
from textual.reactive import reactive
from textual.widget import Widget

class Counter(Widget):
    count = reactive(0)

    def watch_count(self, old_value: int, new_value: int) -> None:
        self.log(f"count changed from {old_value} to {new_value}")

    def render(self) -> str:
        return f"Count: {self.count}"
```

Practical guidance:

- `watch_<name>` is the simplest way to keep dependent widgets in sync.
- `validate_<name>` is useful for clamping or normalizing state before it is stored.
- If a reactive holds a mutable object like `list` or `dict`, mutating it in place does not trigger the reactive machinery; use `mutate_reactive(...)` or assign a new value.

### Workers for async and blocking work

Do not wait on remote I/O inside input handlers, button handlers, or other high-frequency message handlers. Use workers.

Async API pattern:

```python
import httpx

from textual import work
from textual.app import App, ComposeResult
from textual.widgets import Input, Static

class SearchApp(App[None]):
    def compose(self) -> ComposeResult:
        yield Input(placeholder="Search")
        yield Static(id="results")

    def on_input_changed(self, event: Input.Changed) -> None:
        self.fetch_results(event.value)

    @work(exclusive=True)
    async def fetch_results(self, query: str) -> None:
        if not query:
            self.query_one("#results", Static).update("")
            return

        async with httpx.AsyncClient() as client:
            response = await client.get("https://example.com/search", params={"q": query})
        self.query_one("#results", Static).update(response.text)
```

Blocking library pattern:

```python
from textual import work

@work(exclusive=True, thread=True)
def load_blocking(self) -> None:
    result = some_blocking_call()
    self.call_from_thread(self.query_one("#status").update, result)
```

Rules that prevent hard-to-debug UI bugs:

- Use `exclusive=True` for "latest input wins" workloads such as search boxes and autocomplete.
- `@work` on a regular `def` requires `thread=True`; Textual raises an exception otherwise.
- Thread workers must not update the UI directly. Use `call_from_thread(...)` or post messages back to the app.
- By default, worker exceptions exit the app. Set `exit_on_error=False` only when you are explicitly handling worker failures.

### Screens, modes, and bindings

For multi-view apps, define persistent screens on `SCREENS` and switch with `push_screen(...)` or modes.

```python
from textual.app import App
from textual.binding import Binding
from textual.screen import Screen

class HelpScreen(Screen[None]):
    pass

class MyApp(App[None]):
    BINDINGS = [Binding("ctrl+s", "save", "Save")]
    SCREENS = {
        "help": HelpScreen,
    }

    def action_save(self) -> None:
        ...
```

This is cleaner than conditionally mounting and removing large parts of the UI by hand.

## Devtools, Browser Serving, and Testing

If you installed `textual-dev`, the most useful commands are:

```bash
textual --help
textual run my_app.py
textual run --dev my_app.py
textual serve my_app.py
```

Use `textual serve` when you want to access the app in a browser. That serves the Textual app over the web, but Textual itself does not provide application-level authentication, TLS termination, or deployment policy for you. Treat browser exposure as a deployment concern, not a widget concern.

For automated tests, use `run_test()` and the `Pilot` object:

```python
import pytest

@pytest.mark.asyncio
async def test_submit():
    app = GreetingApp()
    async with app.run_test() as pilot:
        await pilot.press("A", "l", "i", "c", "e")
        assert app.query_one(Greeting).name == "Alice"
```

Useful test capabilities from the official guide:

- `pilot.press(...)` simulates keyboard input.
- `pilot.click(...)` simulates widget clicks by selector.
- `app.run_test(size=(100, 50))` lets you verify layout-sensitive behavior at a specific terminal size.
- `pytest-textual-snapshot` is the official snapshot plugin if you need visual regression coverage.

## Configuration and Environment

Textual itself does not require credentials or environment variables. Configuration is mostly Python class attributes and your own app settings.

Common app-level configuration surfaces:

- `CSS_PATH`: stylesheet files for production apps.
- `CSS`: inline stylesheet for small examples.
- `BINDINGS`: keyboard shortcuts and command labels.
- `SCREENS` and `MODES`: screen routing and multi-view navigation.
- `AUTO_FOCUS`: selector for what should gain focus when a screen activates.

If your Textual app talks to external services, handle API keys and environment-specific configuration exactly as you would in any other Python app. Textual does not wrap secret management for you.

## Common Pitfalls

- Missing `textual` CLI: install `textual-dev`; the framework package alone does not guarantee the devtools command is on your path.
- Querying widgets too early: `query_one(...)` raises `NoMatches` if the widget is not mounted yet.
- Mounting is async: if you `mount()` and immediately query descendants in the same handler, make the handler `async` and `await self.mount(...)`.
- Blocking the UI loop: direct network or filesystem work inside message handlers makes input laggy; use `run_worker(...)` or `@work`.
- Threaded workers touching UI state: use `call_from_thread(...)` or messages, not direct widget mutation from the worker thread.
- Mutable reactives: changing a list or dict in place will not trigger watchers or refresh logic unless you explicitly notify Textual.
- Old examples using `renderable=`: modern Textual uses `content` for `Label` and `Static` APIs after the `6.0.0` breaking change.
- Browser exposure assumptions: `textual serve` is not a substitute for your auth, proxy, and deployment controls.

## Version-Sensitive Notes

- `8.1.1` is the current PyPI release as of `2026-03-12`; the version used here is current.
- PyPI currently lists support for Python `3.9` through `3.14` and requires `>=3.9,<4.0`.
- The Textual `6.3.0` release dropped Python `3.8`. If you are migrating an older app or CI image, upgrade Python first.
- The Textual `6.0.0` release renamed `Static.renderable` to `Static.content` and changed the `Label` constructor argument from `renderable` to `content`. Old blog posts and gists may still use the pre-6.0 names.
- If you upgrade Textual and rely on devtools, update `textual-dev` at the same time to avoid mismatched CLI behavior.

## Official Sources

- Documentation root: https://textual.textualize.io/
- Getting started: https://textual.textualize.io/getting_started/
- App guide: https://textual.textualize.io/guide/app/
- CSS guide: https://textual.textualize.io/guide/CSS/
- Reactivity guide: https://textual.textualize.io/guide/reactivity/
- Workers guide: https://textual.textualize.io/guide/workers/
- Testing guide: https://textual.textualize.io/guide/testing/
- Devtools guide: https://textual.textualize.io/guide/devtools/
- App API reference: https://textual.textualize.io/api/app/
- PyPI package page: https://pypi.org/project/textual/
- Official release notes: https://github.com/Textualize/textual/releases
