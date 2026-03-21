---
name: package
description: "LaunchDarkly Python server-side SDK for evaluating feature flags, targeting contexts, and sending analytics events"
metadata:
  languages: "python"
  versions: "9.15.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "launchdarkly,feature-flags,feature-management,experimentation,python,sdk"
---

# LaunchDarkly Python Server-side SDK

## Golden Rule

Use `launchdarkly-server-sdk` for server-side Python evaluation, authenticate with a server-side SDK key, initialize the client once per process, and evaluate flags against `Context` objects rather than legacy user dictionaries. Always provide an explicit fallback value in code because LaunchDarkly will return that fallback when the client is offline, uninitialized, or the flag cannot be evaluated.

## Install

Pin the version your project expects:

```bash
python -m pip install "launchdarkly-server-sdk==9.15.0"
```

Common alternatives:

```bash
uv add "launchdarkly-server-sdk==9.15.0"
poetry add "launchdarkly-server-sdk==9.15.0"
```

Useful extras published by the package:

```bash
python -m pip install "launchdarkly-server-sdk[redis]==9.15.0"
python -m pip install "launchdarkly-server-sdk[dynamodb]==9.15.0"
python -m pip install "launchdarkly-server-sdk[consul]==9.15.0"
python -m pip install "launchdarkly-server-sdk[test-filesource]==9.15.0"
```

Optional observability support is a separate package:

```bash
python -m pip install launchdarkly-observability
```

## Authentication And Initialization

Server-side SDKs use an SDK key from the LaunchDarkly environment you want to read from. Keep it in a secret manager or environment variable.

```bash
export LAUNCHDARKLY_SDK_KEY="sdk-xxxxxxxxxxxxxxxx"
```

Initialize once during process startup and reuse the singleton client:

```python
import os

import ldclient
from ldclient.config import Config

ldclient.set_config(
    Config(
        sdk_key=os.environ["LAUNCHDARKLY_SDK_KEY"],
    )
)

client = ldclient.get()

if not client.is_initialized():
    # The SDK will still return your fallback values.
    # Decide whether to continue in degraded mode or fail startup.
    raise RuntimeError("LaunchDarkly client did not initialize")
```

If your app can keep serving when LaunchDarkly is unavailable, log the failure and continue with conservative fallback values instead of crashing.

### Add common configuration

```python
import os

import ldclient
from ldclient.config import Config

ldclient.set_config(
    Config(
        sdk_key=os.environ["LAUNCHDARKLY_SDK_KEY"],
        offline=False,
        send_events=True,
        all_attributes_private=False,
        private_attributes={"email", "phone", "ssn"},
    )
)

client = ldclient.get()
```

Notes:

- `private_attributes` and `all_attributes_private` control which context attributes are omitted from analytics payloads.
- Use `offline=True` only when you intentionally want the SDK to skip network access and return fallbacks.
- Close the client during shutdown so pending analytics events flush cleanly.

```python
client.flush()
client.close()
```

## Core Usage

### Build a context

`Context` is the current targeting model. The simplest form is a key-only user context:

```python
from ldclient import Context

context = Context.builder("user-123").name("Ada Lovelace").set("plan", "pro").build()
```

Use custom kinds for non-user entities such as organizations, tenants, or devices:

```python
from ldclient import Context

org = Context.builder("org-42").kind("organization").name("Acme").build()
```

Use a multi-context when flag rules depend on more than one entity:

```python
from ldclient import Context

user = Context.builder("user-123").name("Ada Lovelace").build()
org = Context.builder("org-42").kind("organization").name("Acme").build()

multi = Context.create_multi(user, org)
```

### Evaluate a flag

Use the typed variation method that matches the fallback value you want:

```python
enabled = client.variation("new-checkout", context, False)

if enabled:
    run_new_checkout()
else:
    run_legacy_checkout()
```

If you need the evaluation reason for debugging or audits, use `variation_detail`:

```python
detail = client.variation_detail("new-checkout", context, False)

print(detail.value)
print(detail.reason)
```

### Track custom events

Use `track` for business events that should appear in LaunchDarkly experiments or analytics:

```python
client.track(
    "checkout-completed",
    context,
    data={"cart_value": 125.50, "currency": "USD"},
    metric_value=125.50,
)
```

### Get multiple flag values

`all_flags_state` is useful when you need a snapshot of flag values for server-rendered pages or bootstrapping a client:

```python
state = client.all_flags_state(context)
flag_values = state.to_json_dict()
```

## Configuration Patterns

### Keep sensitive attributes private

LaunchDarkly contexts can carry application data, but private fields should not be sent in events.

Global config:

```python
ldclient.set_config(
    Config(
        sdk_key=os.environ["LAUNCHDARKLY_SDK_KEY"],
        private_attributes={"email", "ssn"},
    )
)
```

### Use offline mode intentionally

Offline mode disables LaunchDarkly network access. The SDK then serves only your code-level fallback values.

```python
ldclient.set_config(
    Config(
        sdk_key="ignored-in-offline-mode",
        offline=True,
    )
)
```

This is useful for local development, deterministic tests, or emergency kill-switch behavior, but it is not a substitute for real flag data.

### Load flag data from files or test data

For local development or tests, the SDK supports file-based data and an in-memory test data source.

```python
import os

import ldclient
from ldclient.config import Config
from ldclient.integrations import Files

data_source = Files.new_data_source(paths=["tests/flags.json"], auto_update=False)

ldclient.set_config(
    Config(
        sdk_key=os.environ["LAUNCHDARKLY_SDK_KEY"],
        update_processor_class=data_source,
        send_events=False,
    )
)
```

For programmatic tests:

```python
import ldclient
from ldclient.config import Config
from ldclient.integrations.test_data import TestData

td = TestData.data_source()
td.update(td.flag("new-checkout").variation_for_all(True))

ldclient.set_config(Config("sdk-test-key", update_processor_class=td))
client = ldclient.get()
```

### Use a persistent store or Relay Proxy when needed

If you need shared flag state across processes, LaunchDarkly supports persistent feature stores such as Redis, DynamoDB, and Consul. If your application should read from a Relay Proxy daemon instead of LaunchDarkly directly, configure a feature store and set `use_ldd=True`.

This is the pattern to reach for when:

- many worker processes would otherwise open their own streaming connections
- you want daemon mode through Relay Proxy
- you need a shared cache across short-lived processes

### Call `postfork()` in worker-process servers

The SDK is not fork-safe. In servers that fork after initialization, call `postfork()` in each child so background threads and connections are recreated correctly.

```python
def post_fork(server, worker):
    ldclient.get().postfork()
```

LaunchDarkly documents this for worker-based servers such as uWSGI and requires Relay Proxy `8.11+` if you use `postfork()` together with daemon mode.

## Common Pitfalls

- Do not use a client-side ID or mobile key on the server. The server SDK requires an SDK key.
- Do not create a new LaunchDarkly client per request. Initialize once per process and reuse it.
- Do not keep using legacy user dictionaries in new code. Use `Context` and multi-context targeting.
- `variation()` returns the fallback value when the client is offline, uninitialized, the flag is missing, or the context is invalid. Pick fallback values deliberately.
- `offline=True` does not serve remote flags. It only returns local fallback values unless you also wire in a file or test data source.
- If you use file data for tests, disable events unless you intentionally want analytics emitted from those runs.
- In pre-fork servers, initialize carefully and call `postfork()` in child workers. Reusing a pre-fork client without that step can produce broken background threads or stale connections.
- Remember to `flush()` and `close()` the client during shutdown so analytics are not dropped.

## Version-Sensitive Notes For 9.15.0

- `9.15.0` drops Python 3.9 support. LaunchDarkly's high-level Python SDK page still says `9.12+` requires Python `3.9+`, but the `9.15.0` PyPI metadata and upstream `pyproject.toml` require `>=3.10`.
- Modern LaunchDarkly Python code should use `Context`. If you are migrating older code, treat "users" examples from pre-v8 blog posts as legacy guidance.
- The observability plugin support on the Python server-side docs requires SDK `9.12+`.
- The `9.15.0` changelog marks breaking changes only for the early-access file data system (`Files.new_data_source_v2`, `fdv2`, and related APIs). Avoid those v2 APIs in general-purpose agent code unless you are intentionally targeting the early-access data-system model.

## Official Source URLs

- `https://launchdarkly.com/docs/sdk/server-side/python`
- `https://launchdarkly.com/docs/home/flags/context-attributes`
- `https://launchdarkly.com/docs/sdk/features/config#python`
- `https://launchdarkly.com/docs/sdk/features/offline-mode#python`
- `https://launchdarkly.com/docs/sdk/features/flags-from-files#python`
- `https://launchdarkly.com/docs/sdk/features/storing-data#python`
- `https://launchdarkly.com/docs/sdk/server-side/python#worker-based-servers`
- `https://launchdarkly-python-sdk.readthedocs.io/en/latest/api-main.html`
- `https://launchdarkly-python-sdk.readthedocs.io/en/latest/api-integrations.html`
- `https://launchdarkly-python-sdk.readthedocs.io/en/latest/api-testing.html`
- `https://pypi.org/project/launchdarkly-server-sdk/`
- `https://github.com/launchdarkly/python-server-sdk`
- `https://github.com/launchdarkly/python-server-sdk/blob/main/CHANGELOG.md`
