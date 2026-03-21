---
name: package
description: "jsonschema Python package guide for validating data with JSON Schema drafts, format checks, and explicit reference handling"
metadata:
  languages: "python"
  versions: "4.26.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "jsonschema,json-schema,validation,python,schemas,referencing"
---

# jsonschema Python Package Guide

## Golden Rule

Use `jsonschema` for JSON Schema validation in Python, declare the schema draft with `$schema`, and instantiate a draft-specific validator when you will validate repeatedly or need explicit control over formats and references. In `4.26.0`, new code should use `referencing.Registry` for `$ref` handling instead of the deprecated `RefResolver` APIs.

## Install

Pin the version your project expects:

```bash
python -m pip install "jsonschema==4.26.0"
```

Common alternatives:

```bash
uv add "jsonschema==4.26.0"
poetry add "jsonschema==4.26.0"
```

If you need built-in format checkers with their optional dependencies, install an extra:

```bash
python -m pip install "jsonschema[format]==4.26.0"
python -m pip install "jsonschema[format-nongpl]==4.26.0"
```

Notes:

- `jsonschema` itself is a library, not a service client, so there is no auth setup.
- The extras only install dependencies. They do not enable format validation by themselves.
- If you need a CLI, the project recommends `check-jsonschema` instead of relying on the deprecated `jsonschema` CLI.

## Initialize And Choose A Draft

For one-off checks, `validate()` is fine. For repeated validation, instantiate a validator once and reuse it.

Include `$schema` in schemas so the correct draft is selected:

```python
from jsonschema import validate

schema = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "type": "object",
    "properties": {
        "name": {"type": "string"},
        "age": {"type": "integer", "minimum": 0},
    },
    "required": ["name"],
    "additionalProperties": False,
}

instance = {"name": "Ada", "age": 37}

validate(instance=instance, schema=schema)
```

Why prefer an explicit validator for real code:

- `validate()` first checks that the schema itself is valid, which is useful but adds overhead.
- Reusing a validator is simpler when you need `is_valid()`, `iter_errors()`, `registry=...`, or `format_checker=...`.

Example:

```python
from jsonschema import Draft202012Validator

schema = {
    "$schema": Draft202012Validator.META_SCHEMA["$id"],
    "type": "object",
    "properties": {
        "email": {"type": "string", "format": "email"},
    },
    "required": ["email"],
}

Draft202012Validator.check_schema(schema)

validator = Draft202012Validator(schema)

assert validator.is_valid({"email": "user@example.com"})
assert not validator.is_valid({"email": 42})
```

## Core Usage

### Validate and raise on the first failure

```python
from jsonschema import Draft202012Validator
from jsonschema.exceptions import ValidationError

schema = {
    "$schema": Draft202012Validator.META_SCHEMA["$id"],
    "type": "array",
    "items": {"type": "integer"},
    "maxItems": 3,
}

validator = Draft202012Validator(schema)

try:
    validator.validate([1, 2, "x", 4])
except ValidationError as exc:
    print(exc.message)
    print(exc.json_path)
```

Use `validate()` / `validator.validate()` when you want exception-driven control flow.

### Collect every error instead of stopping at the first one

```python
from jsonschema import Draft202012Validator

schema = {
    "$schema": Draft202012Validator.META_SCHEMA["$id"],
    "type": "array",
    "items": {"enum": [1, 2, 3]},
    "maxItems": 2,
}

validator = Draft202012Validator(schema)
errors = sorted(validator.iter_errors([2, 3, 4]), key=lambda error: list(error.path))

for error in errors:
    print(error.message, error.json_path)
```

`iter_errors()` is the right API for batch reporting, APIs that need field-level feedback, and tests that should assert on structured error data instead of string messages.

### Inspect nested failures with `ErrorTree`

```python
from jsonschema import Draft202012Validator
from jsonschema.exceptions import ErrorTree

schema = {
    "$schema": Draft202012Validator.META_SCHEMA["$id"],
    "type": "array",
    "items": {"type": "number", "enum": [1, 2, 3]},
    "minItems": 3,
}

validator = Draft202012Validator(schema)
tree = ErrorTree(validator.iter_errors(["spam", 2]))

print(0 in tree)                     # True
print(sorted(tree[0].errors))        # ['enum', 'type']
print("minItems" in tree.errors)     # True
```

Use `ErrorTree` when you need to answer questions like "which field failed which keyword?" without parsing human-readable messages.

### Enable format validation explicitly

`format` checks are informational by default. They only run if you pass a format checker.

```python
from jsonschema import Draft202012Validator

schema = {
    "$schema": Draft202012Validator.META_SCHEMA["$id"],
    "type": "object",
    "properties": {
        "ip": {"type": "string", "format": "ipv4"},
    },
    "required": ["ip"],
}

validator = Draft202012Validator(
    schema,
    format_checker=Draft202012Validator.FORMAT_CHECKER,
)

validator.validate({"ip": "127.0.0.1"})
```

If format checks seem to be ignored, the missing piece is usually `format_checker=...`, not the schema.

### Resolve `$ref` with an explicit registry

New code should use the `referencing` integration instead of deprecated resolver APIs:

```python
from jsonschema import Draft202012Validator
from referencing import Registry
from referencing.jsonschema import DRAFT202012

registry = Registry().with_resource(
    "urn:example:nonnegative-int",
    DRAFT202012.create_resource({"type": "integer", "minimum": 0}),
)

schema = {
    "$schema": Draft202012Validator.META_SCHEMA["$id"],
    "type": "object",
    "properties": {
        "count": {"$ref": "urn:example:nonnegative-int"},
    },
    "required": ["count"],
}

validator = Draft202012Validator(schema, registry=registry)
validator.validate({"count": 3})
```

This is the safe default for local schema stores and app-controlled reference resolution. If you need custom lookup behavior, pass `Registry(retrieve=...)` with your own loader.

### Extend a validator when you need custom behavior

The common example is filling defaults, which `jsonschema` does not do automatically:

```python
from jsonschema import Draft202012Validator, validators

def extend_with_default(validator_class):
    validate_properties = validator_class.VALIDATORS["properties"]

    def set_defaults(validator, properties, instance, schema):
        for property_name, subschema in properties.items():
            if "default" in subschema and isinstance(instance, dict):
                instance.setdefault(property_name, subschema["default"])

        yield from validate_properties(validator, properties, instance, schema)

    return validators.extend(validator_class, {"properties": set_defaults})

DefaultValidatingValidator = extend_with_default(Draft202012Validator)
```

Use this only when mutation is a deliberate application behavior. Validation alone will not apply defaults.

## Configuration Notes

- No auth is required. The important configuration is draft choice, format-checker behavior, and how `$ref` URIs are resolved.
- Prefer explicit validator classes such as `Draft202012Validator` when the draft matters to correctness.
- Call `Draft202012Validator.check_schema(schema)` in code paths that accept user- or config-supplied schemas.
- When validating YAML or TOML, deserialize them first and pass the resulting Python objects into `jsonschema`.
- Keep input data JSON-like. Behavior is not well-defined for data that cannot exist in JSON, such as mappings with non-string keys.

## Common Pitfalls

- Forgetting `$schema`. Without it, `validate()` chooses the latest released draft by default, which may not match an older schema.
- Assuming `format` is enforced automatically. It is not.
- Relying on exact error message wording or error ordering in tests. The project treats both as non-stable; assert on structured attributes like `path`, `schema_path`, `validator`, `validator_value`, or `json_path` instead.
- Expecting `default` to mutate the instance. It does not unless you build a custom validator.
- Using deprecated `RefResolver` patterns or depending on implicit remote reference fetching. Use an explicit `Registry` and retrieval function instead.
- Treating YAML/TOML input as fully equivalent to JSON without normalization. Non-JSON constructs can make validation semantics ambiguous.
- Mutating library-owned `ErrorTree` objects. `ErrorTree.__setitem__` is deprecated.

## Version-Sensitive Notes For 4.26.0

- PyPI lists `4.26.0` as the latest release as of March 12, 2026. It requires Python `>=3.10`.
- The `4.26.0` release note is small: it reduces import time by delaying an `urllib.request` import. There are no new top-level validation APIs to adopt for this patch release.
- The major reference-resolution shift happened in `4.18.0`: `RefResolver` was deprecated in favor of `referencing.Registry`, and automatic remote retrieval was called out as deprecated and security-sensitive.
- Since `4.19.0`, importing the `Validator` protocol from the package root is deprecated. Prefer `jsonschema.protocols.Validator` in type-focused code.
- Since `4.24.0`, Python 3.8 is no longer supported. For `4.26.0`, target Python 3.10+ only.
