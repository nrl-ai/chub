---
name: text-to-speech
description: "ElevenLabs Python SDK for text-to-speech synthesis, voice cloning, audio generation, and real-time streaming"
metadata:
  languages: "python"
  versions: "2.18.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "elevenlabs,text-to-speech,tts,audio,voice"
---

# ElevenLabs Python SDK Coding Guidelines

You are an ElevenLabs API coding expert. Help me with writing code using the ElevenLabs Python SDK and API.

Please follow the following guidelines when generating code.

You can find the official SDK documentation and code samples here:
https://elevenlabs.io/docs/api-reference

## Golden Rule: Use the Correct and Current SDK

Always use the official ElevenLabs Python SDK for all ElevenLabs API interactions. This is the standard and recommended library for accessing ElevenLabs services.

- **Library Name:** ElevenLabs Python SDK
- **Python Package:** `elevenlabs`

**Installation:**

- **Correct:** `pip install elevenlabs`

**APIs and Usage:**

- **Correct:** `from elevenlabs.client import ElevenLabs`
- **Correct:** `from elevenlabs.client import AsyncElevenLabs`
- **Correct:** `from elevenlabs import play, save, stream`

## Initialization and API Key

The `elevenlabs` library requires creating a client object for all API calls.

- Always use `client = ElevenLabs()` to create a client object.
- Set `ELEVENLABS_API_KEY` environment variable, which will be picked up automatically.
- Alternatively, pass the API key directly: `client = ElevenLabs(api_key="YOUR_API_KEY")`

## Main Models

By default, use the following models when using the ElevenLabs SDK:

- **General Text-to-Speech (Recommended):** `eleven_multilingual_v2`
    - Excels in stability, language diversity, and accent accuracy
    - Supports 29 languages
    - Recommended for most use cases

- **Ultra-Low Latency:** `eleven_flash_v2_5`
    - Supports 32 languages
    - Faster model, 50% lower price per character

- **Balanced Quality and Speed:** `eleven_turbo_v2_5`
    - Good balance of quality and latency
    - Ideal for developer use cases where speed is crucial
    - Supports 32 languages

## Basic Text-to-Speech Conversion

Here's how to convert text to speech using a voice of your choice:

```python
from elevenlabs.client import ElevenLabs
from elevenlabs import play

client = ElevenLabs()

audio = client.text_to_speech.convert(
    text="The first move is what sets everything in motion.",
    voice_id="JBFqnCBsd6RMkjVDRZzb",
    model_id="eleven_multilingual_v2",
    output_format="mp3_44100_128",
)

play(audio)
```

The `convert` method returns an iterator of bytes that represents the generated audio file.

## Audio Output Formats

The SDK supports various audio formats specified as `codec_sample_rate_bitrate`:

- **MP3 formats:** `mp3_22050_32`, `mp3_44100_32`, `mp3_44100_64`, `mp3_44100_96`, `mp3_44100_128`, `mp3_44100_192`
- **PCM formats:** `pcm_8000`, `pcm_16000`, `pcm_22050`, `pcm_24000`, `pcm_44100`, `pcm_48000`
- **Other formats:** `ulaw_8000`, `alaw_8000`, `opus_48000_32`, `opus_48000_64`, etc.

**Note:** MP3 with 192kbps bitrate requires Creator tier or above. PCM with 44.1kHz sample rate requires Pro tier or above.

## Audio Handling

The SDK provides three main functions for handling generated audio:

### Play Audio
```python
from elevenlabs import play

# Play audio directly
play(audio)

# Play in Jupyter notebook
play(audio, notebook=True)

# Use alternative audio backend
play(audio, use_ffmpeg=False)
```

### Save Audio
```python
from elevenlabs import save

save(audio, "output.mp3")
```

### Stream Audio
```python
from elevenlabs import stream

# For streamed audio
stream(audio_stream)
```

## Voice Management

### List Available Voices
```python
client = ElevenLabs(api_key="YOUR_API_KEY")

# Get all voices
response = client.voices.get_all()
print(response.voices)

# Search voices with pagination
response = client.voices.search()
print(response.voices)
```

## Voice Cloning

Clone a voice using audio samples:

```python
client = ElevenLabs(api_key="YOUR_API_KEY")

voice = client.voices.ivc.create(
    name="Alex",
    description="An old American male voice with a slight hoarseness in his throat. Perfect for news",
    files=["./sample_0.mp3", "./sample_1.mp3", "./sample_2.mp3"],
)
```

The `create` method accepts multiple audio file paths and optional parameters like `remove_background_noise` and `description`.

## Streaming Audio

Stream audio in real-time as it's being generated:

```python
from elevenlabs import stream
from elevenlabs.client import ElevenLabs

client = ElevenLabs()

audio_stream = client.text_to_speech.stream(
    text="This is a test",
    voice_id="JBFqnCBsd6RMkjVDRZzb",
    model_id="eleven_multilingual_v2"
)

# Option 1: Play the streamed audio locally
stream(audio_stream)

# Option 2: Process the audio bytes manually
for chunk in audio_stream:
    if isinstance(chunk, bytes):
        print(chunk)
```

The `stream` method returns an iterator of bytes for real-time audio processing.

## Async Client

Use `AsyncElevenLabs` for asynchronous API calls:

```python
import asyncio
from elevenlabs.client import AsyncElevenLabs

client = AsyncElevenLabs(api_key="YOUR_API_KEY")

async def generate_speech():
    audio = await client.text_to_speech.convert(
        text="Hello, world!",
        voice_id="JBFqnCBsd6RMkjVDRZzb",
        model_id="eleven_multilingual_v2"
    )
    return audio

asyncio.run(generate_speech())
```

## Advanced Features

### Text-to-Speech with Timestamps
```python
response = client.text_to_speech.convert_with_timestamps(
    voice_id="21m00Tcm4TlvDq8ikWAM",
    text="This is a test for the API of ElevenLabs.",
)
```

### Voice Settings Customization
You can override voice settings for individual requests by passing a `VoiceSettings` object:

### Latency Optimization
Control streaming latency with the `optimize_streaming_latency` parameter (0-4, where 4 is maximum optimization):

## Error Handling

The SDK provides specific error types for different API responses:

## Useful Links

- **Documentation:** https://elevenlabs.io/docs/api-reference
- **Models:** https://elevenlabs.io/docs/models
- **GitHub Repository:** https://github.com/elevenlabs/elevenlabs-python
- **API Pricing:** https://elevenlabs.io/pricing

## Notes

- The SDK automatically handles API authentication when the `ELEVENLABS_API_KEY` environment variable is set
- Voice cloning requires an API key and appropriate subscription tier
- Some output formats and features require specific subscription tiers
- The SDK supports both synchronous and asynchronous operations
- Audio files are returned as byte iterators for efficient memory usage

### Citations

```bash
pip install elevenlabs
```

1. **Eleven Multilingual v2** (`eleven_multilingual_v2`)

    - Excels in stability, language diversity, and accent accuracy
    - Supports 29 languages
    - Recommended for most use cases

2. **Eleven Flash v2.5** (`eleven_flash_v2_5`)

    - Ultra-low latency
    - Supports 32 languages
    - Faster model, 50% lower price per character

2. **Eleven Turbo v2.5** (`eleven_turbo_v2_5`)

    - Good balance of quality and latency
    - Ideal for developer use cases where speed is crucial
    - Supports 32 languages


```python
from dotenv import load_dotenv
from elevenlabs.client import ElevenLabs
from elevenlabs import play

load_dotenv()

client = ElevenLabs()

audio = client.text_to_speech.convert(
    text="The first move is what sets everything in motion.",
    voice_id="JBFqnCBsd6RMkjVDRZzb",
    model_id="eleven_multilingual_v2",
    output_format="mp3_44100_128",
)

play(audio)


```python
from elevenlabs.client import ElevenLabs

client = ElevenLabs(
  api_key="YOUR_API_KEY",
)

response = client.voices.search()
print(response.voices)

```python
from elevenlabs.client import ElevenLabs
from elevenlabs import play

client = ElevenLabs(
  api_key="YOUR_API_KEY",
)

voice = client.voices.ivc.create(
    name="Alex",
    description="An old American male voice with a slight hoarseness in his throat. Perfect for news", # Optional
    files=["./sample_0.mp3", "./sample_1.mp3", "./sample_2.mp3"],
)
```

```python
from elevenlabs import stream
from elevenlabs.client import ElevenLabs

client = ElevenLabs()

audio_stream = client.text_to_speech.stream(
    text="This is a test",
    voice_id="JBFqnCBsd6RMkjVDRZzb",
    model_id="eleven_multilingual_v2"
)

# option 1: play the streamed audio locally
stream(audio_stream)

# option 2: process the audio bytes manually
for chunk in audio_stream:
    if isinstance(chunk, bytes):
        print(chunk)

```

```python
import asyncio

from elevenlabs.client import AsyncElevenLabs

eleven = AsyncElevenLabs(
  api_key="MY_API_KEY"
)

async def print_models() -> None:
    models = await eleven.models.list()
    print(models)

asyncio.run(print_models())
```

```python
from .errors import BadRequestError, ForbiddenError, NotFoundError, TooEarlyError, UnprocessableEntityError
```

```python
from .client import AsyncElevenLabs, ElevenLabs
from .environment import ElevenLabsEnvironment
from .history import HistoryListRequestSource
from .play import play, save, stream
```

```python
    def __init__(
        self,
        *,
        base_url: typing.Optional[str] = None,
        environment: ElevenLabsEnvironment = ElevenLabsEnvironment.PRODUCTION,
        api_key: typing.Optional[str] = os.getenv("ELEVENLABS_API_KEY"),
        timeout: typing.Optional[float] = 60,
        httpx_client: typing.Optional[httpx.Client] = None
```

```python
    def convert(
        self,
        voice_id: str,
        *,
        text: str,
        enable_logging: typing.Optional[bool] = None,
        optimize_streaming_latency: typing.Optional[int] = None,
        output_format: typing.Optional[TextToSpeechConvertRequestOutputFormat] = None,
        model_id: typing.Optional[str] = OMIT,
        language_code: typing.Optional[str] = OMIT,
        voice_settings: typing.Optional[VoiceSettings] = OMIT,
        pronunciation_dictionary_locators: typing.Optional[
            typing.Sequence[PronunciationDictionaryVersionLocator]
        ] = OMIT,
        seed: typing.Optional[int] = OMIT,
        previous_text: typing.Optional[str] = OMIT,
        next_text: typing.Optional[str] = OMIT,
        previous_request_ids: typing.Optional[typing.Sequence[str]] = OMIT,
        next_request_ids: typing.Optional[typing.Sequence[str]] = OMIT,
        use_pvc_as_ivc: typing.Optional[bool] = OMIT,
        apply_text_normalization: typing.Optional[
            BodyTextToSpeechV1TextToSpeechVoiceIdPostApplyTextNormalization
        ] = OMIT,
        apply_language_text_normalization: typing.Optional[bool] = OMIT,
        request_options: typing.Optional[RequestOptions] = None,
    ) -> typing.Iterator[bytes]:
```

```python
        optimize_streaming_latency : typing.Optional[int]
            You can turn on latency optimizations at some cost of quality. The best possible final latency varies by model. Possible values:
            0 - default mode (no latency optimizations)
            1 - normal latency optimizations (about 50% of possible latency improvement of option 3)
            2 - strong latency optimizations (about 75% of possible latency improvement of option 3)
            3 - max latency optimizations
            4 - max latency optimizations, but also with text normalizer turned off for even more latency savings (best latency, but can mispronounce eg numbers and dates).

            Defaults to None.
```

```python
        output_format : typing.Optional[TextToSpeechConvertRequestOutputFormat]
            Output format of the generated audio. Formatted as codec_sample_rate_bitrate. So an mp3 with 22.05kHz sample rate at 32kbs is represented as mp3_22050_32. MP3 with 192kbps bitrate requires you to be subscribed to Creator tier or above. PCM with 44.1kHz sample rate requires you to be subscribed to Pro tier or above. Note that the μ-law format (sometimes written mu-law, often approximated as u-law) is commonly used for Twilio audio inputs.

```

```python
        voice_settings : typing.Optional[VoiceSettings]
            Voice settings overriding stored settings for the given voice. They are applied only on the given request.
```

```python
    def convert_with_timestamps(
        self,
        voice_id: str,
        *,
        text: str,
        enable_logging: typing.Optional[bool] = None,
        optimize_streaming_latency: typing.Optional[int] = None,
        output_format: typing.Optional[TextToSpeechConvertWithTimestampsRequestOutputFormat] = None,
        model_id: typing.Optional[str] = OMIT,
        language_code: typing.Optional[str] = OMIT,
        voice_settings: typing.Optional[VoiceSettings] = OMIT,
        pronunciation_dictionary_locators: typing.Optional[
            typing.Sequence[PronunciationDictionaryVersionLocator]
        ] = OMIT,
        seed: typing.Optional[int] = OMIT,
        previous_text: typing.Optional[str] = OMIT,
        next_text: typing.Optional[str] = OMIT,
        previous_request_ids: typing.Optional[typing.Sequence[str]] = OMIT,
        next_request_ids: typing.Optional[typing.Sequence[str]] = OMIT,
        use_pvc_as_ivc: typing.Optional[bool] = OMIT,
        apply_text_normalization: typing.Optional[
            BodyTextToSpeechWithTimestampsV1TextToSpeechVoiceIdWithTimestampsPostApplyTextNormalization
        ] = OMIT,
        apply_language_text_normalization: typing.Optional[bool] = OMIT,
        request_options: typing.Optional[RequestOptions] = None,
    ) -> AudioWithTimestampsResponse:
        """
        Generate speech from text with precise character-level timing information for audio-text synchronization.

        Parameters
```

```python
    def stream(
        self,
        voice_id: str,
        *,
        text: str,
        enable_logging: typing.Optional[bool] = None,
        optimize_streaming_latency: typing.Optional[int] = None,
        output_format: typing.Optional[TextToSpeechStreamRequestOutputFormat] = None,
        model_id: typing.Optional[str] = OMIT,
        language_code: typing.Optional[str] = OMIT,
        voice_settings: typing.Optional[VoiceSettings] = OMIT,
        pronunciation_dictionary_locators: typing.Optional[
            typing.Sequence[PronunciationDictionaryVersionLocator]
        ] = OMIT,
        seed: typing.Optional[int] = OMIT,
        previous_text: typing.Optional[str] = OMIT,
        next_text: typing.Optional[str] = OMIT,
        previous_request_ids: typing.Optional[typing.Sequence[str]] = OMIT,
        next_request_ids: typing.Optional[typing.Sequence[str]] = OMIT,
        use_pvc_as_ivc: typing.Optional[bool] = OMIT,
        apply_text_normalization: typing.Optional[
            BodyTextToSpeechStreamingV1TextToSpeechVoiceIdStreamPostApplyTextNormalization
        ] = OMIT,
        apply_language_text_normalization: typing.Optional[bool] = OMIT,
        request_options: typing.Optional[RequestOptions] = None,
    ) -> typing.Iterator[bytes]:
```

```python
TextToSpeechStreamRequestOutputFormat = typing.Union[
    typing.Literal[
        "mp3_22050_32",
        "mp3_44100_32",
        "mp3_44100_64",
        "mp3_44100_96",
        "mp3_44100_128",
        "mp3_44100_192",
        "pcm_8000",
        "pcm_16000",
        "pcm_22050",
        "pcm_24000",
        "pcm_44100",
        "pcm_48000",
        "ulaw_8000",
        "alaw_8000",
        "opus_48000_32",
        "opus_48000_64",
        "opus_48000_96",
        "opus_48000_128",
        "opus_48000_192",
    ],
    typing.Any,
]
```

```python
def play(
    audio: Union[bytes, Iterator[bytes]], 
    notebook: bool = False, 
    use_ffmpeg: bool = True
) -> None:
```

```python
def save(audio: Union[bytes, Iterator[bytes]], filename: str) -> None:
    if isinstance(audio, Iterator):
        audio = b"".join(audio)
    with open(filename, "wb") as f:
        f.write(audio)
```

```python
def stream(audio_stream: Iterator[bytes]) -> bytes:
    if not is_installed("mpv"):
        message = (
            "mpv not found, necessary to stream audio. "
            "On mac you can install it with 'brew install mpv'. "
            "On linux and windows you can install it from https://mpv.io/"
        )
        raise ValueError(message)
```

 src/elevenlabs/voices/ivc/client.py
```python
    def create(
        self,
        *,
        name: str,
        files: typing.List[core.File],
        remove_background_noise: typing.Optional[bool] = OMIT,
        description: typing.Optional[str] = OMIT,
        labels: typing.Optional[str] = OMIT,
        request_options: typing.Optional[RequestOptions] = None,
    ) -> AddVoiceIvcResponseModel:
```
