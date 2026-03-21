---
name: package
description: "pluggy package guide for building and validating hook-based plugin systems in Python"
metadata:
  languages: "python"
  versions: "1.6.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pluggy,python,plugins,hooks,extension-points"
---

# pluggy Python Package Guide

## What It Is

`pluggy` is the small hook and plugin framework used by `pytest`, but it is also a general-purpose library for any Python application that wants extension points.

The host application defines hook specifications, plugins implement those hooks, and `pluggy.PluginManager` validates registration and dispatches calls.

This doc is pinned to `pluggy` `1.6.0`. The official Read the Docs site is the maintainer source of truth, but the `stable` pages are not a frozen release snapshot, so check the `1.6.0` changelog when behavior looks version-sensitive.

## Install

```bash
pip install pluggy==1.6.0
```

`pluggy` `1.6.0` requires Python `>=3.9`.

## Core Objects

- `pluggy.HookspecMarker("myapp")` marks host hook specifications.
- `pluggy.HookimplMarker("myapp")` marks plugin implementations.
- `pluggy.PluginManager("myapp")` owns registration, validation, tracing, and hook dispatch.
- `pm.add_hookspecs(...)` registers the hook specification namespace.
- `pm.register(plugin, name=...)` registers a plugin object or module.
- `pm.hook.<hook_name>(...)` calls a hook across all registered implementations.

The project name string must match across `HookspecMarker`, `HookimplMarker`, and `PluginManager`.

## Minimal Working Setup

```python
import pluggy

hookspec = pluggy.HookspecMarker("myapp")
hookimpl = pluggy.HookimplMarker("myapp")

class Spec:
    @hookspec
    def transform(self, text: str) -> str:
        """Return a transformed string."""

class UpperPlugin:
    @hookimpl
    def transform(self, text: str) -> str:
        return text.upper()

class SuffixPlugin:
    @hookimpl
    def transform(self, text: str) -> str:
        return f"{text}-done"

pm = pluggy.PluginManager("myapp")
pm.add_hookspecs(Spec)
pm.register(UpperPlugin(), name="upper")
pm.register(SuffixPlugin(), name="suffix")

results = pm.hook.transform(text="hello")
print(results)  # ['hello-done', 'HELLO']
```

Hook calls use keyword arguments. By default, non-wrapper implementations run in LIFO registration order, so the most recently registered plugin runs first unless `tryfirst=True` or `trylast=True` changes that order.

## Recommended Host Startup Pattern

Use this sequence in application startup:

1. Create markers and a `PluginManager`.
2. Register hook specifications with `add_hookspecs(...)`.
3. Register built-in plugins directly.
4. Optionally load third-party plugins from entry points.
5. Call `check_pending()` so mismatches fail early.

```python
import pluggy

hookspec = pluggy.HookspecMarker("myapp")
hookimpl = pluggy.HookimplMarker("myapp")

class Spec:
    @hookspec(firstresult=True)
    def load_config(self, path: str):
        """Return the first plugin that can load this file."""

class YamlPlugin:
    @hookimpl(tryfirst=True)
    def load_config(self, path: str):
        if path.endswith((".yaml", ".yml")):
            return {"kind": "yaml", "path": path}
        return None

pm = pluggy.PluginManager("myapp")
pm.add_hookspecs(Spec)
pm.register(YamlPlugin(), name="yaml")
pm.load_setuptools_entrypoints("myapp")
pm.check_pending()

config = pm.hook.load_config(path="settings.yaml")
print(config)  # {'kind': 'yaml', 'path': 'settings.yaml'}
```

Use `firstresult=True` when the host wants one answer instead of a list. Pluggy stops at the first non-`None` implementation result.

## Registration And Discovery

### Register plugin objects directly

```python
plugin = MyPlugin()
pm.register(plugin, name="my-plugin")
```

### Unregister or block a plugin

```python
pm.unregister(plugin)
pm.set_blocked("legacy-plugin")
pm.unblock("legacy-plugin")
```

Blocking is useful when the host wants to reject a plugin name even if a package is installed.

### Load installed plugins from entry points

```python
loaded = pm.load_setuptools_entrypoints("myapp")
print(f"loaded {loaded} entry-point plugins")
```

Use a stable entry-point group name for your application. `pluggy` only loads those plugins when you call `load_setuptools_entrypoints(...)`; discovery is not automatic.

## Designing Hook Specifications

Define hook specs on a class or module and register them once:

```python
class Spec:
    @hookspec
    def render(self, document: str, style: str = "plain") -> str:
        """Render a document."""
```

Important spec options:

- `firstresult=True`: return the first non-`None` result.
- `historic=True`: remember calls and replay them to plugins registered later.
- `warn_on_impl=...`: warn whenever a plugin implements that hook.
- `warn_on_impl_args={...}`: warn only when a plugin implements specific arguments.

Pluggy validates implementations against the spec, but it is intentionally forward-compatible about added arguments. A hook implementation can accept fewer named arguments than the spec, which lets the host evolve hook signatures without breaking older plugins immediately.

```python
class Spec:
    @hookspec
    def render(self, document: str, style: str):
        """Render a document."""

class OldPlugin:
    @hookimpl
    def render(self, document: str):
        return document.upper()
```

That implementation is still valid because it accepts a subset of the spec arguments.

If you use `historic=True`, remember that the host must replay past calls with `call_historic(...)`, and historic hooks cannot be combined with `firstresult=True`.

## Hook Implementations And Wrappers

```python
class Plugin:
    @hookimpl(trylast=True)
    def render(self, document: str, style: str = "plain") -> str:
        return document.upper()
```

Useful implementation options:

- `tryfirst=True`: run earlier than normal implementations.
- `trylast=True`: run later than normal implementations.
- `optionalhook=True`: allow the implementation even if the host has not registered that spec yet.
- `wrapper=True`: new-style wrapper API.
- `hookwrapper=True`: old-style wrapper API kept for compatibility.

Prefer `wrapper=True` for new code:

```python
class TimingPlugin:
    @hookimpl(wrapper=True)
    def render(self, document: str, style: str = "plain"):
        print("before render")
        result = yield
        print("after render")
        return result
```

Legacy `hookwrapper=True` plugins still exist in the ecosystem, but wrappers must yield exactly once, and older teardown patterns are easier to get wrong.

## Validation And Debugging

### Fail fast on mismatches

```python
pm = pluggy.PluginManager("myapp")
pm.add_hookspecs(Spec)
pm.register(MyPlugin())
pm.check_pending()
```

`check_pending()` is the simplest way to surface unknown or invalid hook implementations during startup.

### Inspect hook calls

```python
undo = pm.add_hookcall_monitoring(
    before=lambda name, hook_impls, kwargs: print("before", name, kwargs),
    after=lambda outcome, name, hook_impls, kwargs: print("after", name, outcome),
)

try:
    pm.hook.render(document="hello", style="plain")
finally:
    undo()
```

You can also use `pm.enable_tracing()` for built-in tracing through the manager's trace root.

## Configuration And Security Model

`pluggy` does not define authentication, network transport, sandboxing, or a plugin config format.

Your application must decide:

- how plugins are discovered
- which plugin packages are trusted
- whether users can enable or disable plugins dynamically
- what objects and data are passed into hooks
- how plugin configuration is loaded and validated

For production systems, treat third-party plugins as application code with full process access unless you provide your own isolation.

## Common Pitfalls

- Project-name mismatch: `PluginManager("myapp")`, `HookspecMarker("myapp")`, and `HookimplMarker("myapp")` must use the same name.
- Forgetting `add_hookspecs(...)`: register specs before plugins, then call `check_pending()` during startup.
- Assuming positional calls work: hook invocations should use keyword arguments like `pm.hook.render(document="x")`.
- Assuming registration order is FIFO: default call order is LIFO for normal implementations.
- Forgetting entry-point loading: installed plugin packages are ignored until you call `load_setuptools_entrypoints(...)`.
- Overusing wrappers: wrappers are powerful, but they complicate control flow and error handling.
- Treating `stable` docs as version-frozen: confirm release-sensitive details against the `1.6.0` changelog and PyPI metadata.

## Version-Sensitive Notes For 1.6.0

- `1.6.0` drops Python `3.8`; the supported range is Python `>=3.9`.
- `1.6.0` adds `PluginManager.unblock(name)` so hosts can remove a previously blocked plugin name.
- `1.6.0` adds `Result.force_exception()` for wrapper and result-manipulation flows.
- `1.5.0` added `warn_on_impl_args`, so do not rely on that option if you still support `1.4.x` or older pluggy releases.

## Official Sources

- Docs root: https://pluggy.readthedocs.io/en/stable/
- API reference: https://pluggy.readthedocs.io/en/stable/api_reference.html
- Changelog: https://pluggy.readthedocs.io/en/stable/changelog.html
- PyPI package: https://pypi.org/project/pluggy/
- PyPI `1.6.0` metadata: https://pypi.org/pypi/pluggy/1.6.0/json
- Maintainer repository: https://github.com/pytest-dev/pluggy
