---
name: generativeai
description: "Legacy Google Gemini API Python SDK (`google-generativeai`) for text generation, chat, files, embeddings, and basic tool use"
metadata:
  languages: "python"
  versions: "0.8.6"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,gemini,generative-ai,llm,python,pypi,legacy"
---

# google-generativeai Python Package Guide

## Golden Rule

`google-generativeai` is the legacy Gemini Python SDK. Keep using it only when a project is already pinned to it. For new work, migrate to `google-genai`.

- Package covered here: `google-generativeai==0.8.6`
- Import path: `import google.generativeai as genai`
- Python requirement on PyPI: `>=3.9`
- Lifecycle status as of March 12, 2026: support ended on November 30, 2025, the package is marked inactive on PyPI, and the upstream repo was archived on December 16, 2025

## Install

```bash
pip install "google-generativeai==0.8.6"
```

If you are starting a new integration, install `google-genai` instead. The public Gemini docs now default to the new SDK.

## Auth And Setup

Create an API key in Google AI Studio, then configure the SDK before creating models.

```bash
export GEMINI_API_KEY="your-api-key"
```

```python
import os
import google.generativeai as genai

genai.configure(api_key=os.environ["GEMINI_API_KEY"])
```

Notes:

- Google’s key guide says Gemini API libraries can pick up `GEMINI_API_KEY` or `GOOGLE_API_KEY`; if both are set, `GOOGLE_API_KEY` takes precedence.
- For this legacy package, prefer calling `genai.configure(...)` explicitly so the active key is unambiguous.
- Keep the API key server-side only. Google’s API key guide warns against exposing Gemini API keys in browser code.

## Core Text Generation

The legacy SDK does not use a top-level client object. You create a `GenerativeModel` and call methods on it directly.

```python
import os
import google.generativeai as genai

genai.configure(api_key=os.environ["GEMINI_API_KEY"])

model = genai.GenerativeModel("gemini-2.0-flash")
response = model.generate_content("Tell me a story in 300 words.")
print(response.text)
```

Use `response.text` for the common case.

## Configure Generation

Attach generation settings when you create the model, or pass them into calls as needed.

```python
import os
import google.generativeai as genai

genai.configure(api_key=os.environ["GEMINI_API_KEY"])

model = genai.GenerativeModel(
    "gemini-2.0-flash",
    system_instruction="You are a terse assistant for production incident summaries.",
    generation_config=genai.GenerationConfig(
        temperature=0.2,
        top_p=0.95,
        top_k=40,
        max_output_tokens=512,
        stop_sequences=["\n\n"],
    ),
)

response = model.generate_content("Summarize this incident: ...")
print(response.text)
```

Common knobs:

- `temperature`, `top_p`, `top_k` for sampling behavior
- `max_output_tokens` to cap output size
- `stop_sequences` to stop on delimiters
- `system_instruction` for persistent model behavior

## Structured JSON Output

`google-generativeai` supports schema-constrained JSON output through `GenerationConfig`.

```python
import os
import typing_extensions as typing
import google.generativeai as genai

genai.configure(api_key=os.environ["GEMINI_API_KEY"])

class CountryInfo(typing.TypedDict):
    name: str
    capital: str
    population: int

model = genai.GenerativeModel("gemini-1.5-flash")
response = model.generate_content(
    "Give me information about Japan.",
    generation_config=genai.GenerationConfig(
        response_mime_type="application/json",
        response_schema=CountryInfo,
    ),
)

print(response.text)
```

This legacy SDK generally leaves you with JSON text to parse yourself. The newer `google-genai` SDK adds stronger typed parsing helpers such as `response.parsed`.

## Multi-Turn Chat

```python
import os
import google.generativeai as genai

genai.configure(api_key=os.environ["GEMINI_API_KEY"])

model = genai.GenerativeModel("gemini-2.0-flash")
chat = model.start_chat()

print(chat.send_message("Tell me a story in 100 words.").text)
print(chat.send_message("What happened after that?").text)
```

Use chat objects when conversation state should remain server-managed across turns. For one-off tasks, plain `generate_content(...)` is simpler.

## Function Calling

Automatic function calling in the legacy SDK is chat-only. New-SDK examples that pass tools directly to `client.models.generate_content(...)` do not translate 1:1.

```python
import google.generativeai as genai

def get_current_weather(city: str) -> str:
    return "23C"

model = genai.GenerativeModel(
    model_name="gemini-2.0-flash",
    tools=[get_current_weather],
)

chat = model.start_chat(enable_automatic_function_calling=True)
response = chat.send_message("What is the weather in San Francisco?")
print(response.text)
```

If you need the current tool surface from Google’s latest docs, migrate instead of trying to backport new `google-genai` examples to this package.

## Files And Multimodal Prompts

The legacy package exposes file helpers at module level.

```python
import os
import google.generativeai as genai

genai.configure(api_key=os.environ["GEMINI_API_KEY"])

uploaded = genai.upload_file(path="notes.txt")

model = genai.GenerativeModel("gemini-2.0-flash")
response = model.generate_content([
    "Summarize this file in 5 bullets.",
    uploaded,
])

print(response.text)
```

Related helpers:

- `genai.list_files()`
- `genai.get_file(name=...)`
- `genai.delete_file(name=...)`

## Token Counting And Embeddings

Count tokens before large prompts:

```python
import os
import google.generativeai as genai

genai.configure(api_key=os.environ["GEMINI_API_KEY"])

model = genai.GenerativeModel("gemini-2.0-flash")
count = model.count_tokens("The quick brown fox jumps over the lazy dog.")
print(count)
```

Create embeddings with the top-level helper:

```python
import os
import google.generativeai as genai

genai.configure(api_key=os.environ["GEMINI_API_KEY"])

embedding = genai.embed_content(
    model="models/gemini-embedding-001",
    content="Hello world",
)

print(embedding)
```

## Safety Settings

For basic safety overrides, pass a `safety_settings` mapping to `generate_content(...)`.

```python
response = model.generate_content(
    "say something bad",
    safety_settings={
        "HATE": "BLOCK_ONLY_HIGH",
        "HARASSMENT": "BLOCK_ONLY_HIGH",
    },
)
```

Keep the settings close to the specific call unless the whole model instance should share the same policy.

## Common Pitfalls

- Do not import `from google import genai` in code that is supposed to use this package. That import is for the newer `google-genai` SDK.
- The current Gemini docs mostly show new-SDK examples. Translate them carefully: `client.models.generate_content(...)` becomes `GenerativeModel(...).generate_content(...)`, and `client.chats.create(...)` becomes `model.start_chat()`.
- Legacy automatic function calling is only supported through `start_chat(enable_automatic_function_calling=True)`.
- Support for this SDK ended on November 30, 2025. Do not expect new Gemini features or active maintenance.
- The repository was archived on December 16, 2025. Treat bug reports and stale examples accordingly.
- The official Gemini libraries page says legacy libraries do not provide access to recent features such as Live API and Veo.
- Model names are server-side. Some newer model IDs may still work, but new SDK-only features and new helper types will not appear in this package.
- The old SDK typically returns raw text or lower-level response objects. If you need parsed JSON objects, richer config types, or a stable client abstraction, migrate.

## Version-Sensitive Notes

- PyPI shows `0.8.6` as the latest and final published release, uploaded on December 16, 2025.
- PyPI also marks the project as `Development Status :: 7 - Inactive`.
- The current Google docs treat `google-generativeai` as a legacy library and recommend `google-genai` for all new Gemini work.
- If your project is pinned below `0.8.6`, prefer exact-version installs and avoid copying examples written for `google-genai` without translating the client model, chat, file, and config APIs.
- If you need Live API, Veo, typed parsed responses, or the maintained client abstraction, migrate to `google-genai` instead of upgrading within this legacy line.

## Official Sources

- Gemini API docs: https://ai.google.dev/gemini-api/docs
- Gemini API libraries page: https://ai.google.dev/gemini-api/docs/libraries
- Migration guide: https://ai.google.dev/gemini-api/docs/migrate
- Gemini API key guide: https://ai.google.dev/gemini-api/docs/api-key
- PyPI package page for `0.8.6`: https://pypi.org/project/google-generativeai/0.8.6/
- Current PyPI project page: https://pypi.org/project/google-generativeai/
- Archived upstream repository: https://github.com/google-gemini/deprecated-generative-ai-python
