---
name: package
description: "Pydantic 2.12.5 package guide for Python data validation, settings, and schema generation"
metadata:
  languages: "python"
  versions: "2.12.5"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "pydantic,python,validation,typing,json-schema,settings"
---

# Pydantic Python Package Guide

## What It Is

`pydantic` validates Python data against type hints and turns it into typed objects. In v2, the main surfaces to reach for are:

- `BaseModel` for structured request, config, and domain models
- `TypeAdapter` for validating arbitrary types without creating a model
- `Field(...)` for defaults and field constraints
- `ConfigDict(...)` for model behavior such as extra-field handling, strictness, aliases, and attribute-based loading

`pydantic` itself does not do network calls or authentication. If you need environment-backed application settings, use the separate `pydantic-settings` package.

## Install

```bash
pip install "pydantic==2.12.5"
```

If you use `uv`:

```bash
uv add "pydantic==2.12.5"
```

If you need `EmailStr` and related email validation helpers:

```bash
pip install "pydantic[email]==2.12.5"
```

## Core Model Pattern

Use `BaseModel` for typed inputs and outputs. Default to explicit field types and fail closed on unknown input unless you have a reason not to.

```python
from pydantic import BaseModel, ConfigDict, Field

class UserCreate(BaseModel):
    model_config = ConfigDict(extra="forbid", str_strip_whitespace=True)

    id: int
    email: str
    is_admin: bool = False
    display_name: str = Field(min_length=1, max_length=80)

payload = {
    "id": "123",
    "email": "dev@example.com",
    "display_name": "Ada",
}

user = UserCreate.model_validate(payload)

assert user.id == 123
assert user.model_dump() == {
    "id": 123,
    "email": "dev@example.com",
    "is_admin": False,
    "display_name": "Ada",
}
```

Useful model methods in v2:

- `model_validate(obj)` validates Python objects like `dict`
- `model_validate_json(raw_json)` validates JSON bytes or strings directly
- `model_validate_strings(data)` is useful when every incoming value is a string
- `model_dump()` returns Python data
- `model_dump_json()` returns serialized JSON
- `model_json_schema()` generates JSON Schema for APIs, forms, and tool definitions

## JSON And String Inputs

Use the right entry point for the data shape you actually have:

```python
from datetime import datetime

from pydantic import BaseModel

class Event(BaseModel):
    at: datetime
    count: int

event = Event.model_validate_json('{"at":"2032-06-01T12:00:00Z","count":3}')

query_data = {"at": "2032-06-01T12:00:00Z", "count": "3"}
event_from_strings = Event.model_validate_strings(query_data)
```

For request bodies that are already JSON text, `model_validate_json()` avoids a separate `json.loads(...)` step.

## Validate Arbitrary Types With `TypeAdapter`

Use `TypeAdapter` when you do not want a wrapper model:

```python
from typing import Annotated

from pydantic import Field, TypeAdapter

UserIds = TypeAdapter(list[Annotated[int, Field(ge=1)]])

ids = UserIds.validate_python(["1", 2, 3])
assert ids == [1, 2, 3]

raw = "[1, 2, 3]"
ids_from_json = UserIds.validate_json(raw)
```

This is the right tool for:

- validating response fragments
- parsing CLI or env-derived values
- validating nested types reused across multiple models

If validation runs in a hot path, instantiate the adapter once and reuse it.

## Config And Settings

### Model Configuration

Set model behavior with `model_config = ConfigDict(...)`:

```python
from pydantic import BaseModel, ConfigDict, Field

class Account(BaseModel):
    model_config = ConfigDict(
        extra="forbid",
        validate_assignment=True,
        from_attributes=True,
        validate_by_name=True,
        validate_by_alias=True,
    )

    account_id: int = Field(alias="accountId")
    owner_name: str
```

Common config choices:

- `extra="forbid"` rejects undeclared input keys
- `validate_assignment=True` re-validates values when attributes change after model creation
- `from_attributes=True` replaces the v1 `from_orm` pattern for loading from objects with attributes
- `validate_by_alias=True` and `validate_by_name=True` let you accept both API aliases and Python field names
- `strict=True` disables coercion globally if you need exact types

### Environment-Backed App Settings

`BaseSettings` moved out of `pydantic` in v2. Use `pydantic-settings`:

```bash
pip install pydantic-settings
```

```python
from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict

class Settings(BaseSettings):
    model_config = SettingsConfigDict(
        env_prefix="APP_",
        env_file=".env",
        extra="ignore",
    )

    debug: bool = False
    database_url: str
    api_key: str = Field(alias="API_KEY")

settings = Settings()
```

Notes:

- `env_prefix` scopes environment variables such as `APP_DEBUG`
- `env_file` lets local development load a `.env`
- `secrets_dir` is available if you need mounted secret files
- `pydantic` has no auth model of its own; it only validates your config and secret values

## Validators

Use validators when type hints and `Field(...)` constraints are not enough:

```python
from pydantic import BaseModel, field_validator, model_validator

class Signup(BaseModel):
    password: str
    confirm_password: str

    @field_validator("password")
    @classmethod
    def check_length(cls, value: str) -> str:
        if len(value) < 12:
            raise ValueError("password must be at least 12 characters")
        return value

    @model_validator(mode="after")
    def passwords_match(self) -> "Signup":
        if self.password != self.confirm_password:
            raise ValueError("passwords do not match")
        return self
```

Reach for:

- `field_validator(...)` for one-field cleanup or checks
- `model_validator(mode="before")` to reshape raw input
- `model_validator(mode="after")` for cross-field checks after parsing

## JSON Schema Generation

Use JSON Schema output when integrating with OpenAPI generators, forms, or LLM tool schemas:

```python
schema = UserCreate.model_json_schema()
```

If you need schema for a non-model type, generate it from `TypeAdapter(...).json_schema()`.

## Common Pitfalls

- V2 renamed many v1 methods. Use `model_validate`, `model_dump`, and `model_json_schema` instead of `parse_obj`, `dict`, and `json_schema`.
- `from_orm` is gone as the primary pattern. Set `from_attributes=True` and call `model_validate(...)`.
- `BaseSettings` is no longer in `pydantic`; install `pydantic-settings`.
- Coercion is powerful but can hide bad input. Use `strict=True` or stricter field types when inputs must not be coerced.
- Alias handling changed in v2. If you expect both `snake_case` and `camelCase`, configure alias behavior explicitly.
- `model_construct()` skips validation. Use it only when the data is already trusted and validated elsewhere.
- For partial updates, prefer `model_copy(update=...)` over rebuilding ad hoc dictionaries.
- Catch `ValidationError` at system boundaries and return structured errors instead of raw tracebacks.

## Version-Sensitive Notes For 2.12.5

- `2.12.5` is the current stable release on PyPI as of `2026-03-11`.
- The official docs root `https://docs.pydantic.dev/latest/` currently documents the v2.12 line.
- PyPI also lists newer pre-release builds; do not assume those APIs are safe to target unless the project explicitly uses them.
- If you are maintaining v1-era code, the migration guide documents the method renames and the `pydantic.v1` compatibility namespace.

## Recommended Agent Workflow

1. Install the exact version used by the project or pin `2.12.5` when creating new examples.
2. Start with a `BaseModel` or `TypeAdapter`, not custom parsing code.
3. Add `extra="forbid"` unless the payload is intentionally open-ended.
4. Use `model_validate_json()` for raw JSON, `model_validate()` for Python objects, and `model_validate_strings()` for all-string maps.
5. Move settings loading into `pydantic-settings` instead of mixing env parsing into application code.
6. Check the migration guide before copying any pre-v2 snippets from blogs, Stack Overflow, or old internal code.

## Official Sources

- Docs: https://docs.pydantic.dev/latest/
- Install: https://docs.pydantic.dev/latest/install/
- Models: https://docs.pydantic.dev/latest/concepts/models/
- Validators: https://docs.pydantic.dev/latest/concepts/validators/
- JSON parsing: https://docs.pydantic.dev/latest/concepts/json/
- TypeAdapter: https://docs.pydantic.dev/latest/concepts/type_adapter/
- Configuration: https://docs.pydantic.dev/latest/concepts/config/
- Aliases: https://docs.pydantic.dev/latest/concepts/alias/
- Settings: https://docs.pydantic.dev/latest/concepts/pydantic_settings/
- Migration guide: https://docs.pydantic.dev/latest/migration/
- Changelog: https://docs.pydantic.dev/latest/changelog/
- PyPI: https://pypi.org/project/pydantic/
