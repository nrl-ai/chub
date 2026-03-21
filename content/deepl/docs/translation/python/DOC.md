---
name: translation
description: "DeepL API client for Python with text and document translation, glossary management, and language detection"
metadata:
  languages: "python"
  versions: "1.23.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "deepl,translation,language,localization,api"
---

# DeepL API Coding Guidelines (Python)

You are a DeepL API coding expert. Help me with writing code using the DeepL API calling the official Python SDK.

You can find the official SDK documentation and code samples here:
https://www.deepl.com/docs-api/

## Golden Rule: Use the Correct and Current SDK

Always use the official DeepL Python SDK to call the DeepL API, which is the standard library for all DeepL API interactions.

**Library Name:** DeepL Python SDK
**PyPI Package:** `deepl`
**Current Version:** 1.23.0

**Installation:**
- **Correct:** `pip install deepl`
- **Incorrect:** `pip install pydeepl`, `pip install deepl-api`, `pip install deepl.py`

**APIs and Usage:**
- **Correct:** `import deepl`
- **Correct:** `translator = deepl.DeepLClient(auth_key)`
- **Incorrect:** `from deepl import Translator` (old API)
- **Incorrect:** `import pydeepl`

## Initialization and API Key

Set the `DEEPL_AUTH_KEY` environment variable or pass the API key directly.

```python
import deepl

# Using environment variable
import os
auth_key = os.environ.get('DEEPL_AUTH_KEY')
translator = deepl.DeepLClient(auth_key)

# Or pass API key directly (not recommended for production)
# translator = deepl.DeepLClient("your-api-key-here")
```

### Getting an API Key

Create a DeepL API account to receive your authentication key. With a DeepL API Free account, you can translate up to 500,000 characters per month for free.

## Text Translation

### Basic Translation

Translate a single text string:

```python
import deepl

auth_key = os.environ.get('DEEPL_AUTH_KEY')
translator = deepl.DeepLClient(auth_key)

result = translator.translate_text("Hello, world!", target_lang="FR")
print(result.text)  # "Bonjour, le monde !"
```

The `translate_text` method:
- First parameter: text to translate (string or list of strings)
- `target_lang` parameter: target language code (required)
- `source_lang` parameter: source language code (optional, auto-detects if not specified)

### Auto-Detection of Source Language

```python
result = translator.translate_text("Hello, world!", target_lang="DE")
print(result.text)  # "Hallo, Welt!"
print(result.detected_source_lang)  # "EN"
```

### Specifying Source Language

```python
result = translator.translate_text(
    "Hello, world!",
    source_lang="EN",
    target_lang="ES"
)
print(result.text)  # "¡Hola, mundo!"
```

### Multiple Texts Translation

Translate multiple texts in a single API call:

```python
texts = [
    "Hello, world!",
    "How are you?",
    "Good morning"
]

results = translator.translate_text(texts, target_lang="JA")
for result in results:
    print(result.text)
# こんにちは、世界！
# お元気ですか？
# おはようございます
```

### TextResult Properties

Each translation result contains:

```python
result = translator.translate_text("Hello", target_lang="DE")

print(result.text)                    # Translated text: "Hallo"
print(result.detected_source_lang)    # Source language: "EN"
print(result.billed_characters)       # Characters charged
print(result.model_type_used)         # Model type if specified
```

## Translation Options

### Formality Control

Control the formality level of translations for supported languages (DE, FR, IT, ES, NL, PL, PT-BR, PT-PT, JA, RU):

```python
# Informal translation
informal = translator.translate_text(
    "How are you?",
    target_lang="DE",
    formality="less"
)
print(informal.text)  # "Wie geht es dir?"

# Formal translation
formal = translator.translate_text(
    "How are you?",
    target_lang="DE",
    formality="more"
)
print(formal.text)  # "Wie geht es Ihnen?"
```

Formality options:
- `"less"` - informal language
- `"more"` - formal language
- `"default"` - standard formality
- `"prefer_less"` - informal if available, otherwise default
- `"prefer_more"` - formal if available, otherwise default

### Context Parameter

Provide additional context to improve translation accuracy (not translated, not billed):

```python
result = translator.translate_text(
    "bank",
    source_lang="EN",
    target_lang="DE",
    context="financial institution"
)
print(result.text)  # "Bank" (financial context)

result2 = translator.translate_text(
    "bank",
    source_lang="EN",
    target_lang="DE",
    context="riverside"
)
print(result2.text)  # "Ufer" (riverbank)
```

### Model Type Selection

Choose between quality-optimized and latency-optimized models:

```python
# Quality-optimized (higher quality, slower)
quality_result = translator.translate_text(
    "Complex technical document",
    source_lang="EN",
    target_lang="DE",
    model_type="quality_optimized"
)

# Latency-optimized (faster, still high quality)
fast_result = translator.translate_text(
    "Simple message",
    source_lang="EN",
    target_lang="DE",
    model_type="latency_optimized"
)
```

### Split Sentences Option

Control how text is split into sentences:

```python
# Split by newlines and punctuation (default)
result1 = translator.translate_text(
    "First sentence. Second sentence.",
    target_lang="DE",
    split_sentences="1"
)

# No splitting
result2 = translator.translate_text(
    "First sentence. Second sentence.",
    target_lang="DE",
    split_sentences="0"
)

# Split by punctuation only (not newlines)
result3 = translator.translate_text(
    "First sentence.\nSecond sentence.",
    target_lang="DE",
    split_sentences="nonewlines"
)
```

### Preserve Formatting

Prevent automatic format correction:

```python
result = translator.translate_text(
    "Hello,    world!",
    target_lang="DE",
    preserve_formatting=True
)
# Preserves extra spaces
```

### HTML and XML Tag Handling

Parse and preserve markup tags:

```python
# HTML tag handling
html_result = translator.translate_text(
    "<p>Hello, <strong>world</strong>!</p>",
    target_lang="DE",
    tag_handling="html"
)
print(html_result.text)  # "<p>Hallo, <strong>Welt</strong>!</p>"

# XML tag handling
xml_result = translator.translate_text(
    "<text>Hello, <emphasis>world</emphasis>!</text>",
    target_lang="DE",
    tag_handling="xml"
)
```

### XML-Specific Options

Advanced XML handling options:

```python
result = translator.translate_text(
    "<root><p>First paragraph</p><ignore>Do not translate</ignore></root>",
    source_lang="EN",
    target_lang="DE",
    tag_handling="xml",
    outline_detection=False,           # Disable automatic tag detection
    splitting_tags=["p", "div"],       # Tags that split sentences
    non_splitting_tags=["span"],       # Tags that don't split sentences
    ignore_tags=["ignore", "code"]     # Tags with untranslated content
)
```

## Text Improvement (Write API)

The Write API allows you to rephrase and improve text quality.

### Basic Text Rephrasing

```python
result = translator.rephrase_text(
    "A rainbouw has seven colours.",
    target_lang="EN-US"
)
print(result.text)  # "A rainbow has seven colors."
```

### Style Options

Apply different writing styles:

```python
# Business style
result = translator.rephrase_text(
    "Thanks for your email. I'll get back to you soon.",
    target_lang="EN",
    style="business"
)

# Academic style
result = translator.rephrase_text(
    "This study shows that...",
    target_lang="EN",
    style="academic"
)

# Casual style
result = translator.rephrase_text(
    "I am writing to inform you...",
    target_lang="EN",
    style="casual"
)

# Simple style
result = translator.rephrase_text(
    "The implementation of the aforementioned protocol...",
    target_lang="EN",
    style="simple"
)
```

Available styles: `"business"`, `"academic"`, `"casual"`, `"simple"`, `"default"`

### Tone Options

Apply different tones:

```python
# Friendly tone
result = translator.rephrase_text(
    "Please submit your report by Friday.",
    target_lang="EN",
    tone="friendly"
)

# Confident tone
result = translator.rephrase_text(
    "I think this approach might work.",
    target_lang="EN",
    tone="confident"
)

# Diplomatic tone
result = translator.rephrase_text(
    "This is wrong.",
    target_lang="EN",
    tone="diplomatic"
)

# Enthusiastic tone
result = translator.rephrase_text(
    "This is a good result.",
    target_lang="EN",
    tone="enthusiastic"
)
```

Available tones: `"friendly"`, `"confident"`, `"diplomatic"`, `"enthusiastic"`, `"default"`

### Optional Style/Tone Application

Use `"prefer_"` prefix for optional application:

```python
result = translator.rephrase_text(
    "The meeting is scheduled for tomorrow.",
    target_lang="EN",
    tone="prefer_diplomatic"
)
# Applies diplomatic tone if suitable, otherwise uses default
```

## Document Translation

### Basic Document Translation

Translate entire documents while preserving formatting:

```python
import deepl

auth_key = os.environ.get('DEEPL_AUTH_KEY')
translator = deepl.DeepLClient(auth_key)

# Using file paths
translator.translate_document_from_filepath(
    "/path/to/input.docx",
    "/path/to/output.docx",
    target_lang="DE"
)
```

### Document Translation with Options

```python
translator.translate_document_from_filepath(
    "Manual.pdf",
    "Anleitung.pdf",
    target_lang="DE",
    formality="more"
)
```

### Using File Objects

```python
with open("input.docx", "rb") as in_file, \
     open("output.docx", "wb") as out_file:
    translator.translate_document(
        in_file,
        out_file,
        target_lang="DE",
        formality="more"
    )
```

### Supported Document Formats

DeepL supports various document formats:
- **Text documents:** docx, pptx, pdf, htm, html, txt
- **Spreadsheets:** xlsx
- **Others:** Check DeepL API documentation for complete list

### Document Translation Error Handling

Handle errors and retrieve document handle for recovery:

```python
try:
    translator.translate_document_from_filepath(
        "input.pdf",
        "output.pdf",
        target_lang="FR"
    )
except deepl.DocumentTranslationException as error:
    doc_id = error.document_handle.id
    doc_key = error.document_handle.key
    print(f"Document ID: {doc_id}")
    print(f"Document Key: {doc_key}")
except deepl.DeepLException as error:
    print(f"Error: {error}")
```

### Lower-Level Document Translation Methods

For advanced control over the document translation process:

```python
# Upload document
with open("input.docx", "rb") as in_file:
    handle = translator.translate_document_upload(
        in_file,
        target_lang="DE",
        formality="more"
    )

# Check translation status
status = translator.translate_document_get_status(handle)
print(f"Status: {status.status}")
print(f"Billed characters: {status.billed_characters}")

# Wait until translation is complete
translator.translate_document_wait_until_done(handle)

# Download translated document
with open("output.docx", "wb") as out_file:
    translator.translate_document_download(handle, out_file)
```

### Polling Document Status

```python
import time

with open("input.pdf", "rb") as in_file:
    handle = translator.translate_document_upload(in_file, target_lang="FR")

while True:
    status = translator.translate_document_get_status(handle)

    if status.status == "done":
        with open("output.pdf", "wb") as out_file:
            translator.translate_document_download(handle, out_file)
        break
    elif status.status == "error":
        raise Exception("Document translation failed")

    time.sleep(5)  # Wait 5 seconds before checking again
```

## Multilingual Glossaries (v3 API)

Glossaries allow you to specify custom translations for specific terms and phrases.

### Creating Glossaries

```python
from deepl import MultilingualGlossaryDictionaryEntries

# Define glossary entries
entries = {"artist": "Maler", "prize": "Gewinn"}
dictionaries = [
    MultilingualGlossaryDictionaryEntries("EN", "DE", entries)
]

# Create glossary
my_glossary = translator.create_multilingual_glossary(
    "My glossary",
    dictionaries
)

print(f"Glossary ID: {my_glossary.glossary_id}")
print(f"Created: {my_glossary.creation_time}")
```

### Creating Glossary from CSV

```python
# CSV format:
# source,target
# artist,Maler
# prize,Gewinn

with open("glossary.csv", "r", encoding="utf-8") as f:
    csv_data = f.read()

glossary = translator.create_multilingual_glossary_from_csv(
    "CSV glossary",
    source_lang="EN",
    target_lang="DE",
    csv_data=csv_data
)
```

### Listing Glossaries

```python
glossaries = translator.list_multilingual_glossaries()

for g in glossaries:
    print(f"Name: {g.name}")
    print(f"ID: {g.glossary_id}")
    print(f"Created: {g.creation_time}")
    for dict_entry in g.dictionaries:
        print(f"  {dict_entry.source_lang} -> {dict_entry.target_lang}: {dict_entry.entry_count} entries")
```

### Retrieving Specific Glossary

```python
# Get by ID
glossary = translator.get_multilingual_glossary("glossary-id-here")

# Find by name
glossaries = translator.list_multilingual_glossaries()
my_glossary = next((g for g in glossaries if g.name == "My glossary"), None)
```

### Viewing Glossary Entries

```python
entries = translator.get_multilingual_glossary_entries(
    glossary,
    "EN",
    "DE"
)
print(entries.dictionaries[0])
```

### Using Glossaries in Translation

```python
# Using glossary ID
result = translator.translate_text(
    "The artist won the prize",
    source_lang="EN",
    target_lang="DE",
    glossary="glossary-id-here"
)

# Using glossary object
glossaries = translator.list_multilingual_glossaries()
my_glossary = glossaries[0]

result = translator.translate_text(
    "The artist won the prize",
    source_lang="EN",
    target_lang="DE",
    glossary=my_glossary
)

print(result.text)  # Uses custom translations from glossary
```

### Editing Glossaries

#### Update Glossary Dictionary (Merge)

```python
from deepl import MultilingualGlossaryDictionaryEntries

# Merge new entries with existing ones
new_entries = {"hello": "hallo", "prize": "Gewinn"}
glossary_dict = MultilingualGlossaryDictionaryEntries(
    "EN", "DE", new_entries
)
translator.update_multilingual_glossary_dictionary(
    glossary,
    glossary_dict
)
```

#### Replace Glossary Dictionary

```python
# Replace all entries completely
replacement = {"goodbye": "Auf Wiedersehen"}
glossary_dict = MultilingualGlossaryDictionaryEntries(
    "EN", "DE", replacement
)
translator.replace_multilingual_glossary_dictionary(
    glossary,
    glossary_dict
)
```

#### Update from CSV

```python
with open("new.csv", "r", encoding="utf-8") as f:
    csv_data = f.read()

translator.update_multilingual_glossary_dictionary_from_csv(
    glossary="glossary-id-here",
    source_lang="EN",
    target_lang="DE",
    csv_data=csv_data
)
```

#### Rename Glossary

```python
glossary = translator.update_multilingual_glossary_name(
    glossary,
    "New name"
)
```

### Deleting Glossaries

```python
# Delete entire glossary
translator.delete_multilingual_glossary(glossary)

# Delete specific dictionary from glossary
translator.delete_multilingual_glossary_dictionary(
    glossary,
    glossary.dictionaries[0]
)
```

### Glossary Supported Languages

Check which language pairs support glossaries:

```python
glossary_languages = translator.get_glossary_languages()

for lang_pair in glossary_languages:
    print(f"{lang_pair.source_lang} -> {lang_pair.target_lang}")
```

## Language Detection and Information

### Detecting Language

Detect the language of text without translating:

```python
# Detect single text
detected = translator.detect_language("Bonjour, comment allez-vous?")
print(detected)  # 'fr'

# Detect multiple texts
texts = ["Hello", "Bonjour", "Hola"]
languages = translator.detect_language(texts)
for text, lang in zip(texts, languages):
    print(f'"{text}" is {lang}')
# "Hello" is en
# "Bonjour" is fr
# "Hola" is es
```

### Getting Supported Languages

```python
# Get all supported languages
languages = translator.get_languages()
for lang in languages:
    print(f"{lang.code}: {lang.name}")

# Get source languages
source_languages = translator.get_source_languages()

# Get target languages
target_languages = translator.get_target_languages()
for lang in target_languages:
    print(f"{lang.code}: {lang.name}")
    if hasattr(lang, 'supports_formality') and lang.supports_formality:
        print("  Supports formality")

# Get glossary-supported languages
glossary_languages = translator.get_glossary_languages()
```

Language object properties:
- `code` - ISO 639-1 language code (e.g., 'en', 'de', 'fr')
- `name` - Display name (e.g., 'English', 'German', 'French')
- `supports_formality` - Boolean (target languages only)

## Usage Monitoring

### Checking Account Usage

Monitor your character usage and account limits:

```python
usage = translator.get_usage()

print(f"Characters used: {usage.character.count}")
print(f"Character limit: {usage.character.limit}")

if usage.character.limit_reached:
    print("Character limit reached!")

# Calculate remaining characters
remaining = usage.character.limit - usage.character.count
print(f"Remaining characters: {remaining}")

# Document usage (if available)
if hasattr(usage, 'document'):
    print(f"Documents translated: {usage.document.count}")
    print(f"Document limit: {usage.document.limit}")
```

### Account Tiers

- **DeepL API Free:** 500,000 characters/month, up to 2 active API keys
- **DeepL API Pro:** Pay-as-you-go, up to 25 active API keys, cost control limits

## Error Handling

### Common Error Patterns

```python
try:
    result = translator.translate_text("Hello", target_lang="DE")
    print(result.text)
except deepl.AuthorizationException:
    print("Authentication failed. Check your API key.")
except deepl.QuotaExceededException:
    print("Quota exceeded. Monthly character limit reached.")
except deepl.TooManyRequestsException:
    print("Too many requests. Please slow down.")
except deepl.DeepLException as error:
    print(f"Translation error: {error}")
```

### Document Translation Errors

```python
try:
    translator.translate_document_from_filepath(
        "input.docx",
        "output.docx",
        target_lang="DE"
    )
except deepl.DocumentTranslationException as error:
    # Document was uploaded but translation failed
    doc_id = error.document_handle.id
    doc_key = error.document_handle.key
    print(f"Can retry with document ID: {doc_id}")
except deepl.DeepLException as error:
    # Upload or other error
    print(f"Error: {error}")
```

## Advanced Configuration

### Custom Server URL

For DeepL API Pro plans with custom endpoints:

```python
translator = deepl.DeepLClient(
    auth_key,
    server_url="https://api.deepl.com"  # Pro endpoint
    # or "https://api-free.deepl.com" for Free endpoint
)
```

### HTTP Options

Configure timeout and other HTTP options:

```python
translator = deepl.DeepLClient(
    auth_key,
    max_retries=5,
    timeout=30.0  # seconds
)
```

## Async Support

DeepL Python SDK does not currently provide native async/await support. For async operations, use a thread pool executor:

```python
import asyncio
from concurrent.futures import ThreadPoolExecutor

executor = ThreadPoolExecutor(max_workers=5)
translator = deepl.DeepLClient(auth_key)

async def translate_async(text, target_lang):
    loop = asyncio.get_event_loop()
    result = await loop.run_in_executor(
        executor,
        translator.translate_text,
        text,
        None,
        target_lang
    )
    return result.text

async def main():
    result = await translate_async("Hello, world!", "FR")
    print(result)

asyncio.run(main())
```

## Complete Translation Example

```python
import deepl
import os

auth_key = os.environ.get('DEEPL_AUTH_KEY')
translator = deepl.DeepLClient(auth_key)

def complete_example():
    try:
        # 1. Check usage
        usage = translator.get_usage()
        remaining = usage.character.limit - usage.character.count
        print(f"Remaining: {remaining} characters")

        # 2. Get supported languages
        target_langs = translator.get_target_languages()
        german_lang = next((l for l in target_langs if l.code == 'DE'), None)
        if german_lang and hasattr(german_lang, 'supports_formality'):
            print(f"German supports formality: {german_lang.supports_formality}")

        # 3. Translate with options
        result = translator.translate_text(
            "How are you doing today?",
            target_lang="DE",
            formality="more"
        )
        print(f"Translation: {result.text}")
        print(f"Detected source: {result.detected_source_lang}")

        # 4. Translate document
        translator.translate_document_from_filepath(
            "document.pdf",
            "dokument.pdf",
            target_lang="DE",
            formality="more"
        )

        print("Translation complete!")
    except deepl.DeepLException as error:
        print(f"Error: {error}")

complete_example()
```

## Type Hints Support

The DeepL Python SDK includes type hints for better IDE support:

```python
from typing import List
import deepl

auth_key: str = os.environ.get('DEEPL_AUTH_KEY')
translator: deepl.DeepLClient = deepl.DeepLClient(auth_key)

def translate_texts(texts: List[str], target: str) -> List[str]:
    results: List[deepl.TextResult] = translator.translate_text(
        texts,
        target_lang=target
    )
    return [result.text for result in results]

translations: List[str] = translate_texts(
    ["Hello", "Goodbye"],
    "FR"
)
```

## Python Version Support

The deepl Python package supports Python 3.9, 3.10, 3.11, 3.12, and 3.13.

## Legacy API (v1/v2) - Deprecated

If you're using the old `Translator` class from earlier versions, migrate to `DeepLClient`:

```python
# Old API (deprecated)
# translator = deepl.Translator(auth_key)

# New API (current)
translator = deepl.DeepLClient(auth_key)
```

The method names remain mostly the same, but `DeepLClient` is the recommended approach.

## Additional Resources

- Official Documentation: https://www.deepl.com/docs-api/
- GitHub Repository: https://github.com/DeepLcom/deepl-python
- PyPI Package: https://pypi.org/project/deepl/
