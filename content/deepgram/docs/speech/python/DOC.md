---
name: speech
description: "Deepgram Python SDK coding guidelines for speech recognition, text-to-speech, and audio intelligence"
metadata:
  languages: "python"
  versions: "5.1.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "deepgram,speech,transcription,tts,audio"
---

# Deepgram Python SDK Coding Guidelines

You are a Deepgram Python SDK coding expert. Help me with writing code using the Deepgram API calling the official Python SDK.

Please follow the following guidelines when generating code.

You can find the official SDK documentation and code samples here:
https://developers.deepgram.com/docs

## Golden Rule: Use the Correct and Current SDK

Always use the official Deepgram Python SDK for all Deepgram API interactions. <cite/>

- **Library Name:** Deepgram Python SDK
- **Python Package:** `deepgram-sdk`
- **Repository:** https://github.com/deepgram/deepgram-python-sdk

**Installation:**

- **Correct:** `pip install deepgram-sdk`

**APIs and Usage:**

- **Correct:** `from deepgram import DeepgramClient`
- **Correct:** `from deepgram import SpeakOptions, PrerecordedOptions, LiveOptions`
- **Correct:** `from deepgram import ReadOptions, SettingsOptions`

## Initialization and API Key

The `deepgram-sdk` library requires creating a client object for all API calls. <cite/>

- Always use `client = DeepgramClient()` to create a client object
- Set `DEEPGRAM_API_KEY` environment variable, which will be picked up automatically
- Alternatively, pass the API key directly: `client = DeepgramClient(api_key="YOUR_API_KEY")`

```python
from deepgram import DeepgramClient

# Using environment variable DEEPGRAM_API_KEY
client = DeepgramClient()

# Or direct API key
client = DeepgramClient(api_key="YOUR_API_KEY")
```

## Authentication Methods

The Deepgram Python SDK supports multiple authentication methods:

### API Key Authentication (Traditional)
```python
from deepgram import DeepgramClient

# Direct API key
client = DeepgramClient(api_key="YOUR_API_KEY")

# Or using environment variable DEEPGRAM_API_KEY
client = DeepgramClient()  # Auto-detects from environment
```

### Bearer Token Authentication (OAuth 2.0)
```python
from deepgram import DeepgramClient

# Direct access token
client = DeepgramClient(access_token="YOUR_ACCESS_TOKEN")

# Or using environment variable DEEPGRAM_ACCESS_TOKEN
client = DeepgramClient()  # Auto-detects from environment
```

## Models

By default, use the following models when using the Deepgram SDK: <cite/>

- **Speech-to-Text Tasks:** `nova-3` (latest general model)
- **Text-to-Speech Tasks:** `aura-2-thalia-en` (default TTS model)
- **Text Intelligence Tasks:** `nova-3`

## Speech-to-Text (Transcription)

### Pre-Recorded Audio (Synchronous)

For transcribing pre-recorded audio files:

```python
from deepgram import DeepgramClient, PrerecordedOptions, FileSource

# Create client
client = DeepgramClient()

# Configure options
options = PrerecordedOptions(
    model="nova-3",
    smart_format=True,
    language="en"
)

# From URL
url_source = {"url": "https://example.com/audio.wav"}
response = client.listen.rest.v("1").transcribe_url(url_source, options)

# From local file
with open("audio.wav", "rb") as file:
    buffer_data = file.read()
    
payload = {"buffer": buffer_data}
response = client.listen.rest.v("1").transcribe_file(payload, options)

print(response.results.channels[0].alternatives[0].transcript)
```

### Pre-Recorded Audio (Asynchronous)

For asynchronous transcription with callbacks:

```python
import asyncio
from deepgram import DeepgramClient, PrerecordedOptions

async def main():
    client = DeepgramClient()
    
    options = PrerecordedOptions(
        model="nova-3",
        smart_format=True
    )
    
    url_source = {"url": "https://example.com/audio.wav"}
    response = await client.listen.asyncrest.v("1").transcribe_url(url_source, options)
    
    print(response.results.channels[0].alternatives[0].transcript)

asyncio.run(main())
```

### Streaming Audio (Real-time)

For real-time audio transcription:

```python
from deepgram import DeepgramClient, LiveTranscriptionEvents, LiveOptions

# Create client
client = DeepgramClient()

# Create connection
dg_connection = client.listen.websocket.v("1")

def on_message(self, result, **kwargs):
    sentence = result.channel.alternatives[0].transcript
    if len(sentence) == 0:
        return
    print(f"Transcript: {sentence}")

def on_error(self, error, **kwargs):
    print(f"Error: {error}")

# Register event handlers
dg_connection.on(LiveTranscriptionEvents.Transcript, on_message)
dg_connection.on(LiveTranscriptionEvents.Error, on_error)

# Configure options
options = LiveOptions(
    model="nova-3",
    language="en",
    smart_format=True
)

# Start connection
if dg_connection.start(options) is False:
    print("Failed to start connection")
    exit()

# Send audio data (example with microphone)
# dg_connection.send(audio_data)

# Close connection when done
dg_connection.finish()
```

## Text-to-Speech

### REST API (Batch Conversion)

For converting text to speech in batch mode:

```python
from deepgram import DeepgramClient, SpeakOptions

# Create client
client = DeepgramClient()

# Configure TTS options
options = SpeakOptions(
    model="aura-2-thalia-en",
    encoding="linear16",
    sample_rate=24000
)

# Input text
input_text = {"text": "Hello, world."}

# Convert text to speech and store in memory
response = client.speak.rest.v("1").stream_memory(input_text, options)

# Access the audio data
audio_data = response.stream_memory.getbuffer()

# Save to a file
with open("output.wav", "wb") as file:
    file.write(audio_data)
```

### REST API (Save to File)

```python
from deepgram import DeepgramClient, SpeakOptions

client = DeepgramClient()

options = SpeakOptions(
    model="aura-2-thalia-en",
    encoding="linear16",
    sample_rate=24000
)

input_text = {"text": "Hello, world."}

# Convert text to speech and save directly to file
response = client.speak.rest.v("1").save("output.wav", input_text, options)
```

### Asynchronous Text-to-Speech

```python
import asyncio
from deepgram import DeepgramClient, SpeakOptions

async def main():
    client = DeepgramClient()
    
    options = SpeakOptions(
        model="aura-2-thalia-en",
        encoding="linear16",
        sample_rate=24000
    )
    
    input_text = {"text": "Hello, world."}
    
    response = await client.speak.asyncrest.v("1").stream_memory(input_text, options)
    
    audio_data = response.stream_memory.getbuffer()
    
    with open("output.wav", "wb") as file:
        file.write(audio_data)

asyncio.run(main())
```

### WebSocket API (Streaming TTS)

For real-time streaming text-to-speech:

```python
from deepgram import DeepgramClient, SpeakWebSocketEvents, SpeakWSOptions

# Create client
client = DeepgramClient()

# Create websocket connection
dg_connection = client.speak.websocket.v("1")

def on_open(self, open, **kwargs):
    print(f"Connection opened: {open}")

def on_binary_data(self, data, **kwargs):
    print("Received audio data")
    # Process audio data here
    with open("output.wav", "ab") as f:
        f.write(data)

def on_close(self, close, **kwargs):
    print(f"Connection closed: {close}")

# Register event handlers
dg_connection.on(SpeakWebSocketEvents.Open, on_open)
dg_connection.on(SpeakWebSocketEvents.AudioData, on_binary_data)
dg_connection.on(SpeakWebSocketEvents.Close, on_close)

# Configure TTS options
options = SpeakWSOptions(
    model="aura-2-thalia-en",
    encoding="linear16",
    sample_rate=16000
)

# Start the connection
if dg_connection.start(options) is False:
    print("Failed to start connection")
    exit()

# Send text to be converted to speech
dg_connection.send_text("Hello, this is a text to speech example using Deepgram.")

# Flush the connection
dg_connection.flush()

# Wait for processing to complete
dg_connection.wait_for_complete()

# Close the connection
dg_connection.finish()
```

## Text Intelligence

Analyze text for insights and intelligence:

```python
from deepgram import DeepgramClient, ReadOptions

client = DeepgramClient()

# Configure read options
options = ReadOptions(
    model="nova-3",
    language="en"
)

# Process text for intelligence
response = client.read.rest.v("1").process(
    text="The quick brown fox jumps over the lazy dog.",
    options=options
)

print(response.results)
```

## Voice Agent

Configure a Voice Agent for conversational AI:

```python
from deepgram import DeepgramClient, SettingsOptions

client = DeepgramClient()

# Create websocket connection
connection = client.agent.websocket.v("1")

# Configure agent settings
options = SettingsOptions()
options.language = "en"
options.agent.think.provider.type = "open_ai"
options.agent.think.provider.model = "gpt-4o-mini"
options.agent.think.prompt = "You are a helpful AI assistant."
options.agent.listen.provider.type = "deepgram"
options.agent.listen.provider.model = "nova-3"
options.agent.speak.provider.type = "deepgram"
options.agent.speak.provider.model = "aura-2-thalia-en"

options.greeting = "Hello, I'm your AI assistant."

# Start the connection
connection.start(options)

# Close the connection
connection.finish()
```

## Captions Generation

Convert transcription results to captions:

### WebVTT
```python
from deepgram_captions import DeepgramConverter, webvtt

transcription = DeepgramConverter(dg_response)
captions = webvtt(transcription)
```

### SRT
```python
from deepgram_captions import DeepgramConverter, srt

transcription = DeepgramConverter(dg_response)
captions = srt(transcription)
```

## Error Handling

Always implement proper error handling:

```python
from deepgram import DeepgramClient
from deepgram.errors import DeepgramError

try:
    client = DeepgramClient()
    response = client.listen.rest.v("1").transcribe_url(url_source, options)
    print(response.results.channels[0].alternatives[0].transcript)
except DeepgramError as e:
    print(f"Deepgram Error: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## Advanced Configuration

### Client Options

Configure advanced client settings:

```python
from deepgram import DeepgramClient, DeepgramClientOptions
from deepgram.utils import verboselogs

config = DeepgramClientOptions(
    options={
        "auto_flush_speak_delta": "500",  # Auto-flush after 500ms
        "speaker_playback": "true"        # Auto-play audio
    },
    verbose=verboselogs.DEBUG
)

client = DeepgramClient("", config)
```

### Integration Example

Combine Speech-to-Text and Text-to-Speech:

```python
from deepgram import DeepgramClient, SpeakOptions, PrerecordedOptions

client = DeepgramClient()

# Text to convert to speech
text = "Hello, world."

# Configure TTS options
tts_options = SpeakOptions(
    model="aura-2-thalia-en", 
    encoding="linear16", 
    sample_rate=24000
)

# Convert text to speech
tts_response = client.speak.rest.v("1").stream_memory({"text": text}, tts_options)

# Save the audio to a buffer
audio_buffer = tts_response.stream_memory.getbuffer()

# Configure STT options
stt_options = PrerecordedOptions(
    model="nova-3",
    smart_format=True
)

# Transcribe the generated audio
stt_response = client.listen.rest.v("1").transcribe_file(
    {"buffer": audio_buffer}, 
    stt_options
)

# Verify the transcription matches the original text
transcript = stt_response.results.channels[0].alternatives[0].transcript
print(f"Original: {text}")
print(f"Transcribed: {transcript}")
```

## Requirements

- Python 3.10 or higher
- `deepgram-sdk` package

## Useful Links

- Documentation: https://developers.deepgram.com/docs
- Python SDK Repository: https://github.com/deepgram/deepgram-python-sdk
- API Reference: https://developers.deepgram.com/reference
- Discord Community: https://discord.gg/xWRaCDBtW4

## Notes

This SDK provides comprehensive support for all Deepgram APIs including Speech-to-Text (both pre-recorded and streaming), Text-to-Speech (both REST and WebSocket), Text Intelligence, and Voice Agent functionality. <cite/> The SDK follows semantic versioning and maintains backward compatibility within major versions. For development and testing, the repository includes both daily tests (against real API endpoints) and unit tests (against mock endpoints).

Wiki pages you might want to explore:
- [Text-to-Speech API (deepgram/deepgram-python-sdk)](/wiki/deepgram/deepgram-python-sdk#3)

### Citations

## Requirements

[Python](https://www.python.org/downloads/) (version ^3.10)

```python
from deepgram_captions import DeepgramConverter, webvtt

transcription = DeepgramConverter(dg_response)
captions = webvtt(transcription)
```

### SRT

```python
from deepgram_captions import DeepgramConverter, srt

transcription = DeepgramConverter(dg_response)
captions = srt(transcription)
```

[See our stand alone captions library for more information.](https://github.com/deepgram/deepgram-python-captions).
```

```python
from deepgram import (
    SettingsOptions
)

# Create websocket connection
connection = deepgram.agent.websocket.v("1")

# Configure agent settings
options = SettingsOptions()
options.language = "en"
options.agent.think.provider.type = "open_ai"
options.agent.think.provider.model = "gpt-4o-mini"
options.agent.think.prompt = "You are a helpful AI assistant."
options.agent.listen.provider.type = "deepgram"
options.agent.listen.provider.model = "nova-3"
options.agent.speak.provider.type = "deepgram"
options.agent.speak.provider.model ="aura-2-thalia-en"

options.greeting = "Hello, I'm your AI assistant."

# Start the connection
connection.start(options)

# Close the connection
connection.finish()
```
```

```python
from deepgram import ReadOptions

# Configure read options
options = ReadOptions(
    model="nova-3",
    language="en"
)

# Process text for intelligence
response = deepgram.read.rest.v("1").process(
    text="The quick brown fox jumps over the lazy dog.",
    options=options
)
```
```

The Deepgram Python SDK supports multiple authentication methods to provide flexibility and enhanced security for your applications.

### Authentication Methods

#### API Key Authentication (Traditional)

The traditional method using your Deepgram API key:

```python
from deepgram import DeepgramClient

# Direct API key
client = DeepgramClient(api_key="YOUR_API_KEY")

# Or using environment variable DEEPGRAM_API_KEY
client = DeepgramClient()  # Auto-detects from environment
```

#### Bearer Token Authentication (OAuth 2.0)

Use short-lived access tokens for enhanced security:

```python
from deepgram import DeepgramClient

# Direct access token
client = DeepgramClient(access_token="YOUR_ACCESS_TOKEN")

# Or using environment variable DEEPGRAM_ACCESS_TOKEN
client = DeepgramClient()  # Auto-detects from environment
```
```

If you are looking to use, run, contribute or modify to the daily/unit tests, then you need to install the following dependencies:

```bash
pip install -r requirements-dev.txt
```

### Daily Tests

The daily tests invoke a series of checks against the actual/real API endpoint and save the results in the `tests/response_data` folder. This response data is updated nightly to reflect the latest response from the server. Running the daily tests does require a `DEEPGRAM_API_KEY` set in your environment variables.

To run the Daily Tests:

```bash
make daily-test
```

#### Unit Tests

The unit tests invoke a series of checks against mock endpoints using the responses saved in `tests/response_data` from the daily tests. These tests are meant to simulate running against the endpoint without actually reaching out to the endpoint; running the unit tests does require a `DEEPGRAM_API_KEY` set in your environment variables, but you will not actually reach out to the server.

```bash
make unit-test
```
```

## Backwards Compatibility

We follow semantic versioning (semver) to ensure a smooth upgrade experience. Within a major version (like `4.*`), we will maintain backward compatibility so your code will continue to work without breaking changes. When we release a new major version (like moving from `3.*` to `4.*`), we may introduce breaking changes to improve the SDK. We'll always document these changes clearly in our release notes to help you upgrade smoothly.

Older SDK versions will receive Priority 1 (P1) bug support only. Security issues, both in our code and dependencies, are promptly addressed. Significant bugs without clear workarounds are also given priority attention.
```

```python
from deepgram import DeepgramClient, SpeakOptions, PrerecordedOptions, FileSource

from tests.utils import save_metadata_string

TTS_MODEL = "aura-2-thalia-en"
STT_MODEL = "general-nova-3"

# response constants
TEXT1 = "Hello, world."

# Create a list of tuples to store the key-value pairs
input_output = [
    (
        TEXT1,
        SpeakOptions(model=TTS_MODEL, encoding="linear16", sample_rate=24000),
        PrerecordedOptions(model="nova-3", smart_format=True),
        {"results.channels.0.alternatives.0.transcript": [TEXT1]},
    ),
]
```

```python
from deepgram import DeepgramClient, SpeakOptions, PrerecordedOptions, FileSource

from tests.utils import read_metadata_string, save_metadata_string

MODEL = "aura-2-thalia-en"

# response constants
TEXT1 = "Hello, world."

# Create a list of tuples to store the key-value pairs
input_output = [
    (
        TEXT1,
        SpeakOptions(model=MODEL, encoding="linear16", sample_rate=24000),
        {
            "content_type": ["audio/wav"],
            "model_name": ["aura-2-thalia-en"],
            "characters": ["13"],
        },
    ),
]
```
cture 