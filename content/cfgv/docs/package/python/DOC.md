---
name: package
description: "cfgv package guide for Python configuration schema validation"
metadata:
  languages: "python"
  versions: "3.5.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "cfgv,python,configuration,validation"
---

# cfgv Python Package Guide

`cfgv` validates Python data structures against a schema and raises nested, human-readable `ValidationError` messages when configuration is wrong.

It does not parse configuration formats by itself. You load JSON, YAML, or another text format into Python objects, then validate those objects with `cfgv`.

## Install

```bash
pip install cfgv
```

- Import name: `cfgv`
- Environment variables: none
- Authentication: none
- Client initialization: none

## Define a schema

The core building blocks are:

- `cfgv.Map(object_name, id_key, *items)` for dictionaries
- `cfgv.Array(of, allow_empty=True)` for lists of nested objects
- `cfgv.Required(...)` and `cfgv.Optional(...)` for keys in a map
- `cfgv.NoAdditionalKeys(...)` to reject unexpected keys

```python
import cfgv

SERVICE_SCHEMA = cfgv.Map(
    "Service",
    "name",
    cfgv.Required("name", cfgv.check_string),
    cfgv.Optional("enabled", cfgv.check_bool, True),
    cfgv.NoAdditionalKeys(("name", "enabled")),
)

APP_CONFIG_SCHEMA = cfgv.Map(
    "Config",
    None,
    cfgv.Required("version", cfgv.check_int),
    cfgv.RequiredRecurse("service", SERVICE_SCHEMA),
    cfgv.Optional("mode", cfgv.check_one_of({"dev", "prod"}), "dev"),
    cfgv.NoAdditionalKeys(("version", "service", "mode")),
)
```

Choose `object_name` and `id_key` carefully. They are included in error output, so readable values make failures much easier to debug.

## Validate data and apply defaults

`cfgv.validate(value, schema)` checks the data and returns the original value on success. It does not add defaults.

Use `cfgv.apply_defaults(value, schema)` when you want a new object with optional defaults populated.

Using `APP_CONFIG_SCHEMA` from the previous example:

```python
import cfgv

config = {
    "version": 1,
    "service": {"name": "api"},
}

try:
    cfgv.validate(config, APP_CONFIG_SCHEMA)
except cfgv.ValidationError as error:
    print(error)
    raise

config_with_defaults = cfgv.apply_defaults(config, APP_CONFIG_SCHEMA)

print(config_with_defaults)
# {'version': 1, 'service': {'name': 'api', 'enabled': True}, 'mode': 'dev'}
```

If you need to serialize a config without values that only exist because of defaults, use `cfgv.remove_defaults(value, schema)`.

```python
cleaned = cfgv.remove_defaults(config_with_defaults, APP_CONFIG_SCHEMA)
print(cleaned)
# {'version': 1, 'service': {'name': 'api'}}
```

## Load and validate a file

`cfgv.load_from_filename()` reads a UTF-8 text file, passes the file contents to your loader function, validates the parsed data, and returns a new object with defaults applied.

Using `APP_CONFIG_SCHEMA` from the earlier schema definition:

```python
import functools
import json

import cfgv


class InvalidConfigError(Exception):
    pass


load_app_config = functools.partial(
    cfgv.load_from_filename,
    schema=APP_CONFIG_SCHEMA,
    load_strategy=json.loads,
    exc_tp=InvalidConfigError,
)

config = load_app_config("config.json")
```

Your `load_strategy` must accept the file contents as a string and return Python data structures such as dictionaries, lists, strings, integers, and booleans.

If you want friendlier file paths in error messages, pass `display_filename=`:

```python
config = cfgv.load_from_filename(
    "./config.json",
    APP_CONFIG_SCHEMA,
    json.loads,
    display_filename="config.json",
)
```

## Common patterns

### Arrays of nested objects

Use `cfgv.Array()` together with a nested `cfgv.Map()` when a key contains a list of structured items.

```python
import cfgv

HOOK_SCHEMA = cfgv.Map(
    "Hook",
    "id",
    cfgv.Required("id", cfgv.check_string),
    cfgv.OptionalNoDefault("pattern", cfgv.check_regex),
    cfgv.NoAdditionalKeys(("id", "pattern")),
)

HOOKS_CONFIG_SCHEMA = cfgv.Map(
    "Config",
    None,
    cfgv.RequiredRecurse("hooks", cfgv.Array(HOOK_SCHEMA, allow_empty=False)),
    cfgv.NoAdditionalKeys(("hooks",)),
)
```

### Conditional keys

Use `cfgv.Conditional()` when one field is only valid when another field has a particular value.

```python
import cfgv

AUTH_SCHEMA = cfgv.Map(
    "Auth",
    None,
    cfgv.Required("type", cfgv.check_one_of({"anonymous", "token"})),
    cfgv.Conditional(
        "token",
        cfgv.check_string,
        "type",
        "token",
        ensure_absent=True,
    ),
    cfgv.NoAdditionalKeys(("type", "token")),
)
```

With `ensure_absent=True`, `cfgv` rejects `token` when `type` is not `"token"`.

## Useful built-in helpers

- Scalar checks: `cfgv.check_bool`, `cfgv.check_bytes`, `cfgv.check_int`, `cfgv.check_string`, `cfgv.check_text`
- Choice checks: `cfgv.check_one_of(...)`, `cfgv.In(...)`, `cfgv.NotIn(...)`, `cfgv.Not(...)`
- Composition: `cfgv.check_array(inner_check)`, `cfgv.check_and(*checks)`
- Nested schemas: `cfgv.RequiredRecurse(...)`, `cfgv.OptionalRecurse(...)`, `cfgv.ConditionalRecurse(...)`
- Extra-key handling: `cfgv.NoAdditionalKeys(...)`, `cfgv.WarnAdditionalKeys(keys, callback)`

## Pitfalls

- `cfgv` validates Python objects, not raw YAML or JSON text. Parse the file first or use `load_from_filename()` with an appropriate loader.
- `cfgv.validate()` does not mutate data and does not fill in optional defaults.
- `cfgv.apply_defaults()` and `cfgv.remove_defaults()` return new values instead of modifying the input in place.
- Default values are inserted exactly as provided to `cfgv.Optional(...)` or `cfgv.OptionalRecurse(...)`. Avoid mutable defaults such as `[]` or `{}` unless you control later mutation.
- `cfgv.NoAdditionalKeys(...)` is strict: any undeclared key raises `ValidationError`.
- Error readability depends on your schema names. A good `object_name` and `id_key` make nested failures much easier to trace.

## Official sources

- Maintainer repository: https://github.com/asottile/cfgv
- Package registry: https://pypi.org/project/cfgv/
