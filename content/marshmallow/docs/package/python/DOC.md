---
name: package
description: "marshmallow 4.2.2 package guide for Python schema validation, serialization, and deserialization"
metadata:
  languages: "python"
  versions: "4.2.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "marshmallow,python,serialization,validation,schema,json"
---

# marshmallow Python Package Guide

## Golden Rule

Use explicit `Schema` subclasses with declared fields, validate incoming data with `load()` or `validate()`, and assume `marshmallow 4` behavior. `dump()` serializes objects but does not validate them, so do not treat a successful dump as input validation.

## Install

Pin the package version your project expects:

```bash
python -m pip install "marshmallow==4.2.2"
```

Common alternatives:

```bash
uv add "marshmallow==4.2.2"
poetry add "marshmallow==4.2.2"
```

Notes:

- The PyPI package is `marshmallow`, and the main import is also `marshmallow`.
- `marshmallow` itself has no runtime service credentials or network setup.
- PyPI exposes extras such as `dev`, `docs`, and `tests` for upstream development workflows; most applications should install the base package only.

## Initialize A Schema

The main setup step is defining a schema class with explicit fields.

```python
from marshmallow import Schema, fields

class UserSchema(Schema):
    id = fields.UUID(dump_only=True)
    email = fields.Email(required=True)
    display_name = fields.String(required=True)
    created_at = fields.DateTime(dump_only=True)
    is_admin = fields.Boolean(load_default=False)

user_schema = UserSchema()
users_schema = UserSchema(many=True)
```

Key patterns:

- Use `required=True` for mandatory input fields.
- Use `load_default=...` for default values during deserialization.
- Use `dump_only=True` and `load_only=True` for output-only and input-only fields.
- Use `many=True` when working with lists of objects.

## Core Usage

### Deserialize and validate input

`load()` validates and converts incoming data into Python-native values.

```python
from marshmallow import Schema, ValidationError, fields

class UserInputSchema(Schema):
    email = fields.Email(required=True)
    age = fields.Integer(required=True)

schema = UserInputSchema()

payload = {"email": "dev@example.com", "age": "42"}

try:
    data = schema.load(payload)
    print(data)  # {'email': 'dev@example.com', 'age': 42}
except ValidationError as err:
    print(err.messages)
```

If you only need validation errors without deserialized output, use `schema.validate(payload)`.

### Serialize Python objects or dicts

`dump()` converts Python objects into plain dicts. It does not run the same validation path as `load()`.

```python
from dataclasses import dataclass
from datetime import datetime, UTC
from uuid import uuid4

@dataclass
class User:
    id: str
    email: str
    display_name: str
    created_at: datetime
    is_admin: bool

user = User(
    id=str(uuid4()),
    email="dev@example.com",
    display_name="Dev",
    created_at=datetime.now(UTC),
    is_admin=False,
)

result = UserSchema().dump(user)
print(result)
```

### Build objects with `post_load`

Use `@post_load` when you want `load()` to return application objects instead of plain dicts.

```python
from dataclasses import dataclass

from marshmallow import Schema, fields, post_load

@dataclass
class User:
    email: str
    display_name: str

class UserSchema(Schema):
    email = fields.Email(required=True)
    display_name = fields.String(required=True)

    @post_load
    def make_user(self, data, **kwargs):
        return User(**data)
```

### Handle nested structures

Use `fields.Nested` for embedded objects and `fields.Pluck` when you only need one field from a nested schema.

```python
from marshmallow import Schema, fields

class AuthorSchema(Schema):
    id = fields.Integer(required=True)
    name = fields.String(required=True)

class ArticleSchema(Schema):
    title = fields.String(required=True)
    author = fields.Nested(AuthorSchema, required=True)
    author_name = fields.Pluck(AuthorSchema, "name", dump_only=True, attribute="author")
```

For self-referential or recursive schemas in `marshmallow 4`, prefer a callable:

```python
class CategorySchema(Schema):
    name = fields.String(required=True)
    children = fields.List(fields.Nested(lambda: CategorySchema()))
```

### Rename wire-format keys

Use `data_key` when the external payload name differs from the Python field name.

```python
from marshmallow import Schema, fields

class EventSchema(Schema):
    occurred_at = fields.DateTime(data_key="occurredAt", required=True)
```

## Configuration And Input-Control Choices

`marshmallow` does not have auth config. The important configuration surface is schema behavior.

### Unknown fields

The safest default for API request parsing is usually to reject or explicitly exclude unknown input.

```python
from marshmallow import EXCLUDE, Schema, fields

class UserSchema(Schema):
    name = fields.String(required=True)

    class Meta:
        unknown = EXCLUDE
```

Options:

- `RAISE`: fail on unknown fields
- `EXCLUDE`: ignore unknown fields during `load()`
- `INCLUDE`: keep unknown fields in output from `load()`

You can also override this per call:

```python
result = UserSchema().load(payload, unknown=EXCLUDE)
```

### Partial updates

Use `partial=True` for PATCH-style payloads.

```python
class ProfileSchema(Schema):
    email = fields.Email(required=True)
    bio = fields.String(required=True)

schema = ProfileSchema()
data = schema.load({"bio": "Updated"}, partial=True)
```

### Read-only and write-only fields

Mark fields explicitly instead of filtering dicts by hand:

```python
class LoginSchema(Schema):
    email = fields.Email(required=True)
    password = fields.String(required=True, load_only=True)
    access_token = fields.String(dump_only=True)
```

## Schema Hooks And Custom Behavior

### Normalize input with hooks

`pre_load`, `post_load`, `pre_dump`, and `post_dump` let you adapt data before or after marshmallow's main processing.

```python
from marshmallow import Schema, fields, post_load, pre_load

class SignupSchema(Schema):
    email = fields.Email(required=True)
    display_name = fields.String(required=True)

    @pre_load
    def normalize_email(self, data, **kwargs):
        if "email" in data:
            data["email"] = data["email"].strip().lower()
        return data

    @post_load
    def finalize(self, data, **kwargs):
        data["display_name"] = data["display_name"].strip()
        return data
```

If you are upgrading old code, decorator arguments renamed in `4.x`: use `pass_collection`, not `pass_many`.

### Cross-field validation

Use `@validates_schema` for checks that depend on more than one field.

```python
from marshmallow import Schema, ValidationError, fields, validates_schema

class DateRangeSchema(Schema):
    starts_at = fields.DateTime(required=True)
    ends_at = fields.DateTime(required=True)

    @validates_schema
    def validate_range(self, data, **kwargs):
        if data["ends_at"] <= data["starts_at"]:
            raise ValidationError(
                "ends_at must be after starts_at",
                field_name="ends_at",
            )
```

### Computed or custom fields

Prefer `fields.Method` or `fields.Function` when you only need computed serialization or deserialization logic.

```python
from marshmallow import Schema, fields

class UserSummarySchema(Schema):
    first_name = fields.String(required=True)
    last_name = fields.String(required=True)
    full_name = fields.Method("get_full_name", dump_only=True)

    def get_full_name(self, obj):
        return f"{obj['first_name']} {obj['last_name']}"
```

Create a custom field subclass only when built-in fields, `Method`, and `Function` are not enough.

## Common Pitfalls

- `dump()` is serialization, not validation. Run `load()` or `validate()` on untrusted input.
- In `marshmallow 4`, implicit field creation is removed. Every schema field must be declared explicitly unless another library generates it for you.
- `Schema.context` was removed in `4.x`. Do not rely on `self.context` from old examples; pass values another way or evaluate the experimental context API in the upgrading guide.
- `fields.Field`, `fields.Number`, and `fields.Mapping` are no longer meant to be dropped directly into schemas as concrete fields. Use specific field classes like `String`, `Integer`, `Dict`, or subclass `fields.Field`.
- Custom validators should raise `ValidationError`; old examples that return `False` are outdated.
- `dump_only` fields are treated as unknown during `load()`. This matters if you switch to `unknown=INCLUDE`, because those fields can then pass through as unvalidated data.
- For recursive schemas, avoid old `"self"` nesting patterns from older blog posts. Use a callable such as `lambda: CategorySchema()`.
- `DateTime` uses Python's standard-library ISO parsing in `4.x`. Inputs like `YYYY-MM-DD` may deserialize differently than older `3.x` assumptions.

## Version-Sensitive Notes For 4.2.2

- `4.2.2` is the current PyPI release and the stable docs version as of March 12, 2026.
- PyPI currently requires Python `>=3.10`. `marshmallow 4.1.0` dropped Python 3.9 support, so do not copy older CI matrices blindly.
- `4.0.0` removed several long-deprecated patterns that still appear in third-party examples: implicit field creation, `Schema.context`, and old decorator argument names such as `pass_many`.
- `4.0.0` changed `DateTime` parsing to use `datetime.fromisoformat()` for `"iso"` handling and removed the `TimeDelta` `precision` argument.
- `4.2.2` includes a fix for `fields.Constant(None)` so it deserializes back to `None` consistently. If your project uses constant-valued fields, prefer `4.2.2` or newer in the `4.2.x` line.

## Official Sources

- Docs root: `https://marshmallow.readthedocs.io/en/stable/`
- Quickstart: `https://marshmallow.readthedocs.io/en/stable/quickstart.html`
- Nesting guide: `https://marshmallow.readthedocs.io/en/stable/nesting.html`
- Custom fields guide: `https://marshmallow.readthedocs.io/en/stable/custom_fields.html`
- Hooks guide: `https://marshmallow.readthedocs.io/en/stable/extending/pre_and_post_processing_methods.html`
- Upgrading guide: `https://marshmallow.readthedocs.io/en/stable/upgrading.html`
- Changelog: `https://marshmallow.readthedocs.io/en/stable/changelog.html`
- PyPI project: `https://pypi.org/project/marshmallow/`
- Repository: `https://github.com/marshmallow-code/marshmallow`
