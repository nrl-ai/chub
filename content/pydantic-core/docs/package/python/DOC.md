---
name: package
description: "pydantic-core package guide for low-level Python validation and serialization with core schemas"
metadata:
  languages: "python"
  versions: "2.42.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pydantic-core,python,validation,serialization,json,schema"
---

# pydantic-core Python Package Guide

## What It Is

`pydantic-core` is the low-level validation and serialization engine used by `pydantic` v2. Use it directly when you need to:

- compile and reuse validators or serializers in a hot path
- validate raw Python objects, JSON bytes, or string-heavy inputs against a core schema
- work with the engine layer instead of `BaseModel` or `TypeAdapter`

For most application code, prefer `pydantic`. The maintainer README explicitly notes that most users should not need to use `pydantic-core` directly.

## Installation

`pydantic-core 2.42.0` requires Python `>=3.9`.

Install the package from PyPI:

```bash
pip install "pydantic-core==2.42.0"
```

With `uv`:

```bash
uv add "pydantic-core==2.42.0"
```

The PyPI package name uses a hyphen, but the Python import name uses an underscore:

```python
import pydantic_core
from pydantic_core import SchemaSerializer, SchemaValidator, ValidationError, core_schema
```

## Initialization And Environment

There is no auth flow, network client, or package-specific required environment variable.

Initialization means compiling a schema into one of these objects:

- `SchemaValidator(schema)` for validation
- `SchemaSerializer(schema)` for serialization

In normal application code, create these once at import time or app startup and reuse them.

## Build And Reuse A Validator

The most direct workflow is:

1. define a core schema with helpers from `pydantic_core.core_schema`
2. compile it into a `SchemaValidator`
3. reuse that validator for each input

```python
from pydantic_core import SchemaValidator, core_schema

user_schema = core_schema.typed_dict_schema(
    {
        "id": core_schema.typed_dict_field(core_schema.int_schema(ge=1)),
        "email": core_schema.typed_dict_field(
            core_schema.str_schema(min_length=3, strip_whitespace=True)
        ),
        "is_admin": core_schema.typed_dict_field(
            core_schema.bool_schema(),
            required=False,
        ),
    },
    extra_behavior="forbid",
)

validator = SchemaValidator(user_schema)

user = validator.validate_python(
    {
        "id": "123",
        "email": "  dev@example.com  ",
    }
)

assert user == {
    "id": 123,
    "email": "dev@example.com",
}
```

Use the `core_schema` helpers unless you specifically need to build raw schema dictionaries by hand. They are easier to read and less error-prone.

## Choose The Right Validation Entry Point

`SchemaValidator` has separate methods for different input shapes.

### `validate_python(...)`

Use this when you already have Python objects such as `dict`, `list`, or application objects.

```python
user = validator.validate_python(
    {
        "id": "123",
        "email": "dev@example.com",
        "is_admin": False,
    }
)
```

### `validate_json(...)`

Use this for raw request bodies or cached JSON bytes. The official API docs note that this avoids constructing intermediate Python objects and can be significantly faster than `json.loads(...)` followed by `validate_python(...)`.

```python
user = validator.validate_json(
    b'{"id": 123, "email": "dev@example.com", "is_admin": false}'
)
```

### `validate_strings(...)`

Use this for query params, form posts, CLI arguments, or environment-derived mappings where every value starts as text.

```python
from pydantic_core import SchemaValidator, core_schema

event_schema = core_schema.typed_dict_schema(
    {
        "at": core_schema.typed_dict_field(core_schema.datetime_schema()),
        "count": core_schema.typed_dict_field(core_schema.int_schema(ge=1)),
    },
    extra_behavior="forbid",
)

event_validator = SchemaValidator(event_schema)

event = event_validator.validate_strings(
    {
        "at": "2032-06-01T12:00:00Z",
        "count": "3",
    }
)
```

This produces Python values such as `datetime` and `int`, not just cleaned strings.

## Parse JSON Without A Schema

If you only need fast JSON deserialization, use `from_json(...)`.

```python
from pydantic_core import from_json

payload = from_json(b'{"ids": [1, 2, 3], "name": "demo"}')

assert payload == {
    "ids": [1, 2, 3],
    "name": "demo",
}
```

`from_json(...)` parses JSON into Python objects. It does not validate that data against a schema. If you need both parsing and validation, call `validate_json(...)` on a `SchemaValidator` instead.

## Serialize Data

Use `SchemaSerializer` when you want serialization behavior tied to a compiled schema.

```python
from pydantic_core import SchemaSerializer

serializer = SchemaSerializer(user_schema)

python_value = serializer.to_python(
    {
        "id": 123,
        "email": "dev@example.com",
        "is_admin": False,
    }
)

json_bytes = serializer.to_json(
    {
        "id": 123,
        "email": "dev@example.com",
        "is_admin": False,
    }
)

print(json_bytes.decode("utf-8"))
```

For standalone helpers, use `to_json(...)` and `to_jsonable_python(...)`:

```python
from datetime import datetime, timezone

from pydantic_core import to_json, to_jsonable_python

json_bytes = to_json({"id": 123, "email": "dev@example.com"})

json_ready = to_jsonable_python(
    {"created_at": datetime(2032, 6, 1, 12, 0, tzinfo=timezone.utc)}
)

assert json_ready == {"created_at": "2032-06-01T12:00:00Z"}
```

`SchemaSerializer.to_json(...)` and `to_json(...)` return `bytes`, not `str`.

## Handle Validation Errors

Validation failures raise `ValidationError`.

```python
from pydantic_core import ValidationError

try:
    validator.validate_python(
        {
            "id": 0,
            "email": "x",
            "role": "admin",
        }
    )
except ValidationError as exc:
    for error in exc.errors():
        print(error["loc"], error["type"], error["msg"])
```

Use these methods depending on how you want to report failures:

- `exc.errors()` for structured Python data
- `exc.json()` for a JSON string
- `str(exc)` for a readable multi-line message

## When To Reach For `pydantic` Instead

Reach for `pydantic` instead of `pydantic-core` when you want:

- `BaseModel` classes for request and domain models
- `TypeAdapter` for high-level validation of arbitrary type hints
- JSON Schema generation
- settings management through `pydantic-settings`
- a more stable and ergonomic public API for normal application code

Use `pydantic-core` directly when low-level schema control or validator reuse is the actual goal.

## Common Pitfalls

- Import `pydantic_core`, not `pydantic-core`.
- Build validators and serializers once. Recompiling schemas in a request loop wastes the main benefit of the package.
- Use `validate_json(...)` for raw JSON input and `validate_strings(...)` for string-heavy mappings. They solve different problems.
- `from_json(...)` only parses. It does not enforce a schema.
- `to_json(...)` and `SchemaSerializer.to_json(...)` return `bytes`; call `.decode("utf-8")` if you need a string.
- If you mainly want typed models or settings, using `pydantic-core` directly usually adds complexity with little payoff.

## Version-Sensitive Notes

- This guide targets `pydantic-core 2.42.0`.
- PyPI uses the project name `pydantic-core`, while Python code imports `pydantic_core`.
- PyPI metadata for this package line requires Python `>=3.9`.

## Official Sources

- Repository and maintainer README: https://github.com/pydantic/pydantic-core
- API docs root: https://docs.pydantic.dev/latest/api/pydantic_core/
- PyPI project page: https://pypi.org/project/pydantic-core/
