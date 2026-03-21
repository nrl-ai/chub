---
name: package
description: "instructor package guide for Python: structured outputs, validation-driven retries, streaming partials, and provider client patching for LLM apps"
metadata:
  languages: "python"
  versions: "1.14.5"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "instructor,llm,structured-outputs,pydantic,openai,anthropic,gemini,validation"
---

# instructor Python Package Guide

## What It Does

`instructor` wraps provider SDK clients so you can ask for structured outputs as Pydantic models instead of manually parsing JSON or tool-call payloads yourself. In practice, that means:

- define a `BaseModel`
- patch or construct a provider client with `instructor`
- pass `response_model=...` on generation calls
- get back validated Python objects

It is most useful when you need reliable extraction, classification, routing, or multi-step agent code that should fail on schema violations instead of silently accepting malformed model output.

## Version Scope

- Package: `instructor`
- Language: `python`
- Frontmatter version: `1.14.5`
- Docs root: `https://python.useinstructor.com/`
- Registry: `https://pypi.org/project/instructor/`

Important version note: the current docs site is not pinned to `1.14.5` and already includes newer migration guidance for `v2` and the newer `from_provider(...)` setup flow. For code that must match `1.14.5`, prefer the stable provider-specific wrappers such as `from_openai(...)` unless you have verified the newer helper exists in the installed version.

## Installation

Install `instructor` plus the SDK for the model provider you are actually calling.

```bash
python -m pip install "instructor==1.14.5" openai
```

```bash
uv add "instructor==1.14.5" openai
```

```bash
poetry add "instructor==1.14.5" openai
```

Common provider packages:

- OpenAI-style examples: `openai`
- Anthropic: `anthropic`
- Google Gemini: use the provider package shown in the current maintainer docs for the integration you need

If a project already pins a provider SDK, keep that version aligned with the project lockfile rather than copying a random blog example.

## Core Mental Model

`instructor` does not replace the provider SDK. It augments it.

1. Create the normal provider client.
2. Wrap it with `instructor`.
3. Pass a Pydantic model as `response_model`.
4. Receive a validated model instance instead of raw text.

That also means authentication, base URLs, organization or project IDs, timeouts, and transport settings still come from the underlying provider SDK.

## Initialization And Auth

### OpenAI sync client

```python
import os

import instructor
from openai import OpenAI

client = instructor.from_openai(
    OpenAI(api_key=os.environ["OPENAI_API_KEY"])
)
```

### OpenAI async client

```python
import os
import asyncio

import instructor
from openai import AsyncOpenAI

async def main() -> None:
    client = instructor.from_openai(
        AsyncOpenAI(api_key=os.environ["OPENAI_API_KEY"])
    )

    # Use the patched async client exactly where you would normally make a request.
    print(client)

asyncio.run(main())
```

### Other providers

The docs also cover provider-specific wrappers and newer generic provider setup flows. The safe rule is:

- authenticate the provider client exactly as that provider SDK expects
- then wrap that client with the `instructor` integration documented for that provider

`instructor` does not introduce a separate universal auth environment variable.

## Basic Structured Output

```python
import os

import instructor
from openai import OpenAI
from pydantic import BaseModel

class UserInfo(BaseModel):
    name: str
    age: int

client = instructor.from_openai(
    OpenAI(api_key=os.environ["OPENAI_API_KEY"])
)

user = client.chat.completions.create(
    model="gpt-4.1-mini",
    messages=[
        {
            "role": "user",
            "content": "Jane Doe is 34 years old.",
        }
    ],
    response_model=UserInfo,
)

print(user)
print(user.name)
print(user.age)
```

The return value is the validated `UserInfo` instance, not a JSON string you still need to parse.

## Lists And Repeated Objects

When the task is “extract many records,” use the iterable helper instead of asking for free-form JSON and then trying to coerce it afterward.

```python
import os

import instructor
from openai import OpenAI
from pydantic import BaseModel

class Person(BaseModel):
    name: str
    age: int

client = instructor.from_openai(
    OpenAI(api_key=os.environ["OPENAI_API_KEY"])
)

people = client.chat.completions.create_iterable(
    model="gpt-4.1-mini",
    messages=[
        {
            "role": "user",
            "content": "Extract Jane Doe, 34, and Bob Smith, 41.",
        }
    ],
    response_model=Person,
)

for person in people:
    print(person)
```

Prefer `create_iterable(...)` when you want repeated model instances, progressive consumption, or a cleaner control flow than “ask for a list and hope the JSON is perfect.”

## Streaming Partial Structured Output

Use partial streaming when you want progressive model state before the final object is complete. Make fields optional or provide defaults so intermediate partials can validate.

```python
import os

import instructor
from openai import OpenAI
from pydantic import BaseModel

class TicketDraft(BaseModel):
    title: str | None = None
    summary: str | None = None
    priority: str | None = None

client = instructor.from_openai(
    OpenAI(api_key=os.environ["OPENAI_API_KEY"])
)

for partial in client.chat.completions.create_partial(
    model="gpt-4.1-mini",
    messages=[
        {
            "role": "user",
            "content": "Draft a bug ticket for a flaky login timeout issue.",
        }
    ],
    response_model=TicketDraft,
):
    print(partial)
```

Use this for streaming UIs or long extraction jobs where waiting for one final object would hurt latency or user feedback.

## Validation And Retries

Validation is the main reason to use `instructor`. Put real constraints in the model and let the wrapper retry when the provider returns something that does not satisfy the schema.

```python
import os

import instructor
from openai import OpenAI
from pydantic import BaseModel, field_validator

class Customer(BaseModel):
    email: str
    age: int

    @field_validator("age")
    @classmethod
    def age_must_be_positive(cls, value: int) -> int:
        if value <= 0:
            raise ValueError("age must be positive")
        return value

client = instructor.from_openai(
    OpenAI(api_key=os.environ["OPENAI_API_KEY"])
)

customer = client.chat.completions.create(
    model="gpt-4.1-mini",
    messages=[
        {
            "role": "user",
            "content": "Extract the customer profile from: Sam, age 27, sam@example.com",
        }
    ],
    response_model=Customer,
    max_retries=3,
)
```

Practical guidance:

- keep validators focused on business constraints that matter
- use retries sparingly because each retry adds cost and latency
- prefer clear field names and descriptions so the model has a better chance of satisfying the schema on the first attempt

## Raw Completion Access

If you need both the validated object and the raw provider response for logging, token accounting, or finish-reason debugging, use the maintainer-documented helper that returns structured output together with the completion payload.

Treat this as the debugging and observability path. For normal application code, the plain structured-return flow is simpler.

## Configuration Notes

The main configuration surface lives on the underlying provider SDK client:

- API keys and auth
- custom base URLs
- organization or project identifiers
- timeout and retry transport settings
- HTTP client or proxy settings

The main configuration surface on `instructor` itself is how you request structure:

- `response_model=...`
- streaming helpers such as partial or iterable output
- validation-driven retry count
- provider-specific integration or mode choices from the docs

When a project uses Azure OpenAI, OpenRouter, LiteLLM, local gateways, or other OpenAI-compatible endpoints, configure those details on the provider client first and then wrap that configured client.

## Common Pitfalls

### Forgetting the provider SDK dependency

`instructor` is not a standalone LLM transport. Installing only `instructor` is not enough for real requests.

### Treating auth as an Instructor concern

Set auth on `OpenAI(...)`, `Anthropic(...)`, or the provider client you use. `instructor` wraps the client; it does not replace provider authentication.

### Writing overly strict partial models

`create_partial(...)` emits incomplete intermediate states. If every field is required, partial streaming will be brittle. Make partial fields optional or defaulted.

### Assuming every docs example matches `1.14.5`

The docs site currently includes newer `v2` migration content and `from_provider(...)` examples. For `1.14.5`, stay with the provider-specific wrappers unless you have confirmed otherwise in your installed package.

### Using retries without thinking about cost

Validation retries improve correctness, but they also create extra model calls. Keep retry counts small for hot paths.

### Falling back to raw JSON parsing anyway

If you are already using `response_model`, let Pydantic own validation. Do not ask the model for JSON strings and then write fragile post-processing unless you need a provider feature that the wrapper cannot express.

## Version-Sensitive Notes

- The version used here for this package is `1.14.5`.
- The maintainer docs root is newer than that version and already references migration guidance for `v2`.
- For compatibility-focused work on `1.14.5`, favor the documented `from_openai(...)` style and the `chat.completions.create(...)` family shown in the main getting-started flow.
- If you want to use the newer `from_provider(...)` helper, OpenAI Responses API integrations, or recently added provider abstractions, verify them against the installed package version first.

## Official Sources

- Docs root: https://python.useinstructor.com/
- Getting started: https://python.useinstructor.com/getting-started/
- Patching and provider setup: https://python.useinstructor.com/concepts/patching/
- Validation and retry concepts: https://python.useinstructor.com/concepts/validation/
- Hooks and observability: https://python.useinstructor.com/concepts/hooks/
- Registry: https://pypi.org/project/instructor/
