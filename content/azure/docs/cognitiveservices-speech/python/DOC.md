---
name: cognitiveservices-speech
description: "Azure Cognitive Services Speech SDK for Python for speech recognition, synthesis, translation, and language identification"
metadata:
  languages: "python"
  versions: "1.48.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,speech,speech-to-text,text-to-speech,translation,language-identification,pronunciation"
---

# Azure Cognitive Services Speech SDK for Python

## Golden Rule

Use `azure-cognitiveservices-speech` for real-time Azure Speech work in Python, import it as `azure.cognitiveservices.speech`, and configure it with a speech resource key plus either a region or a custom endpoint. This package ships native binaries, so install failures are often platform or system-library problems, not Python import problems.

## Install

Pin the version your project expects:

```bash
python -m pip install "azure-cognitiveservices-speech==1.48.2"
```

Common alternatives:

```bash
uv add "azure-cognitiveservices-speech==1.48.2"
poetry add "azure-cognitiveservices-speech==1.48.2"
```

Important install notes:

- Microsoft Learn currently documents Python `3.8` or later for the SDK setup flow. PyPI metadata still reports `Requires: >=3.7`; follow the stricter Microsoft Learn requirement if the two disagree.
- PyPI publishes platform wheels and currently no source distribution. Treat this as a native SDK package, not a pure-Python fallback package.
- On Linux, stay within the supported distro and architecture matrix from Microsoft Learn. If audio capture or playback fails, verify system audio dependencies before debugging your Python code.
- For compressed audio input such as MP3, OGG, or FLAC, the SDK relies on GStreamer. WAV/PCM works without that extra dependency.

Ubuntu or Debian systems that need compressed input support commonly need:

```bash
sudo apt install libgstreamer1.0-0 \
  gstreamer1.0-plugins-base \
  gstreamer1.0-plugins-good \
  gstreamer1.0-plugins-bad \
  gstreamer1.0-plugins-ugly
```

## Authentication And Setup

Most official Python examples use a speech resource key with either:

- `SPEECH_KEY` + `SPEECH_REGION`
- `SPEECH_KEY` + `ENDPOINT`

Microsoft Learn also has some translation examples that use double-underscore names such as `SPEECH__SUBSCRIPTION__KEY` and `SPEECH__SERVICE__REGION`. If you are working in an existing codebase, support both forms instead of assuming one naming convention.

Use a helper that normalizes the environment:

```python
import os
import azure.cognitiveservices.speech as speechsdk

def make_speech_config() -> speechsdk.SpeechConfig:
    key = os.getenv("SPEECH_KEY") or os.getenv("SPEECH__SUBSCRIPTION__KEY")
    region = os.getenv("SPEECH_REGION") or os.getenv("SPEECH__SERVICE__REGION")
    endpoint = os.getenv("ENDPOINT") or os.getenv("SPEECH_ENDPOINT")

    if not key:
        raise RuntimeError("Set SPEECH_KEY or SPEECH__SUBSCRIPTION__KEY.")

    if endpoint:
        return speechsdk.SpeechConfig(subscription=key, endpoint=endpoint)

    if region:
        return speechsdk.SpeechConfig(subscription=key, region=region)

    raise RuntimeError("Set either SPEECH_REGION or ENDPOINT.")
```

Recommended environment variables:

```bash
export SPEECH_KEY="..."
export SPEECH_REGION="eastus"
```

If your resource or scenario requires a custom domain endpoint instead of a region, set:

```bash
export SPEECH_KEY="..."
export ENDPOINT="https://<resource-or-custom-domain>.cognitiveservices.azure.com/"
```

## Core Usage

### Recognize one utterance from the default microphone

Use `recognize_once_async()` for short, single-utterance interactions:

```python
import azure.cognitiveservices.speech as speechsdk

speech_config = make_speech_config()
speech_config.speech_recognition_language = "en-US"

audio_config = speechsdk.audio.AudioConfig(use_default_microphone=True)
recognizer = speechsdk.SpeechRecognizer(
    speech_config=speech_config,
    audio_config=audio_config,
)

result = recognizer.recognize_once_async().get()

if result.reason == speechsdk.ResultReason.RecognizedSpeech:
    print(result.text)
elif result.reason == speechsdk.ResultReason.NoMatch:
    raise RuntimeError("No speech could be recognized.")
elif result.reason == speechsdk.ResultReason.Canceled:
    details = result.cancellation_details
    raise RuntimeError(f"{details.reason}: {details.error_details}")
```

### Recognize speech from a local audio file

```python
import azure.cognitiveservices.speech as speechsdk

speech_config = make_speech_config()
speech_config.speech_recognition_language = "en-US"

audio_config = speechsdk.audio.AudioConfig(filename="input.wav")
recognizer = speechsdk.SpeechRecognizer(
    speech_config=speech_config,
    audio_config=audio_config,
)

result = recognizer.recognize_once_async().get()
print(result.text)
```

Use WAV/PCM first when you want the least setup friction. If you switch to MP3, FLAC, OGG, or MP4 containers, install and expose GStreamer correctly.

### Continuous recognition for longer sessions

`recognize_once_async()` stops after one utterance. For longer audio or streaming UX, use the continuous APIs and event callbacks:

```python
import azure.cognitiveservices.speech as speechsdk

speech_config = make_speech_config()
speech_config.speech_recognition_language = "en-US"

recognizer = speechsdk.SpeechRecognizer(
    speech_config=speech_config,
    audio_config=speechsdk.audio.AudioConfig(filename="meeting.wav"),
)

done = False

def stop_cb(evt):
    global done
    done = True

recognizer.recognized.connect(lambda evt: print("RECOGNIZED:", evt.result.text))
recognizer.canceled.connect(lambda evt: print("CANCELED:", evt))
recognizer.session_stopped.connect(stop_cb)

recognizer.start_continuous_recognition()
while not done:
    pass
recognizer.stop_continuous_recognition()
```

For production code, replace the busy wait with proper synchronization such as `threading.Event`.

### Synthesize speech to the default speaker

```python
import azure.cognitiveservices.speech as speechsdk

speech_config = make_speech_config()
speech_config.speech_synthesis_voice_name = "en-US-Ava:DragonHDLatestNeural"

synthesizer = speechsdk.SpeechSynthesizer(speech_config=speech_config)
result = synthesizer.speak_text_async("Hello from Azure Speech.").get()

if result.reason != speechsdk.ResultReason.SynthesizingAudioCompleted:
    details = result.cancellation_details
    raise RuntimeError(f"{details.reason}: {details.error_details}")
```

Use SSML with `speak_ssml_async()` when you need fine-grained voice, style, rate, pitch, or pronunciation control instead of plain text synthesis.

### Synthesize to a file

```python
import azure.cognitiveservices.speech as speechsdk

speech_config = make_speech_config()
audio_config = speechsdk.audio.AudioOutputConfig(filename="output/hello.wav")

synthesizer = speechsdk.SpeechSynthesizer(
    speech_config=speech_config,
    audio_config=audio_config,
)
synthesizer.speak_text_async("Saved to disk.").get()
```

The parent directory must already exist before you construct `AudioOutputConfig`.

### Translate speech while recognizing it

Use the translation namespace, not `SpeechRecognizer`, when you need text translations from speech input:

```python
import os
import azure.cognitiveservices.speech as speechsdk

translation_config = speechsdk.translation.SpeechTranslationConfig(
    subscription=os.environ["SPEECH_KEY"],
    region=os.environ["SPEECH_REGION"],
)
translation_config.speech_recognition_language = "en-US"
translation_config.add_target_language("de")

recognizer = speechsdk.translation.TranslationRecognizer(
    translation_config=translation_config,
    audio_config=speechsdk.audio.AudioConfig(use_default_microphone=True),
)

result = recognizer.recognize_once_async().get()

if result.reason == speechsdk.ResultReason.TranslatedSpeech:
    print("Source:", result.text)
    print("German:", result.translations["de"])
```

### Auto-detect the spoken language

This is the SDK path for multilingual front doors and voice assistants:

```python
import azure.cognitiveservices.speech as speechsdk

speech_config = make_speech_config()
auto_detect = speechsdk.languageconfig.AutoDetectSourceLanguageConfig(
    languages=["en-US", "de-DE", "fr-FR"]
)

recognizer = speechsdk.SourceLanguageRecognizer(
    speech_config=speech_config,
    auto_detect_source_language_config=auto_detect,
    audio_config=speechsdk.audio.AudioConfig(use_default_microphone=True),
)

result = recognizer.recognize_once_async().get()
print(result.text)
```

For continuous language identification, Microsoft Learn documents endpoint-based configuration and specific SDK/platform support limits. Do not assume the one-shot and continuous language-ID flows have identical configuration rules.

### Pronunciation assessment

`PronunciationAssessmentConfig` is the SDK path for scoring pronunciation, fluency, and completeness:

```python
import azure.cognitiveservices.speech as speechsdk

speech_config = make_speech_config()
audio_config = speechsdk.audio.AudioConfig(filename="student.wav")

recognizer = speechsdk.SpeechRecognizer(
    speech_config=speech_config,
    audio_config=audio_config,
)

assessment = speechsdk.PronunciationAssessmentConfig(
    reference_text="The quick brown fox jumps over the lazy dog.",
    grading_system=speechsdk.PronunciationAssessmentGradingSystem.HundredMark,
    granularity=speechsdk.PronunciationAssessmentGranularity.Phoneme,
)
assessment.apply_to(recognizer)

result = recognizer.recognize_once_async().get()
print(result.text)
```

If you need richer scoring output, parse the JSON result payload from the pronunciation assessment APIs rather than assuming the top-level recognition text is enough.

## Configuration Notes

- Set `speech_recognition_language` or `speech_synthesis_voice_name` explicitly. Defaults are easy to forget and produce the wrong locale.
- Region-based setup is simplest for standard speech resources. Endpoint-based setup is common for custom domains, some translation flows, and some continuous language-identification scenarios.
- Keep your key and endpoint or region outside source control. Environment variables are the safest default for agent-written code.
- Use the async SDK methods shown in the official docs even in synchronous scripts. The Python samples consistently call `..._async().get()`.

## Common Pitfalls

- `recognize_once_async()` is not a streaming API. It handles a single utterance and then stops.
- The package contains native code. If imports fail in CI or containers, verify wheel compatibility, CPU architecture, OpenSSL or CA packages, and audio dependencies before rewriting code.
- Compressed input formats require GStreamer. A missing GStreamer install often looks like an audio-format bug in Python.
- Translation uses `speechsdk.translation.*` types, not the base recognizer classes.
- Language identification is a separate recognizer flow. Do not expect `SpeechRecognizer` to infer the source language automatically.
- File output requires the destination directory to exist first.
- Batch transcription and some management tasks belong to the Speech REST APIs, not this SDK’s real-time client flow.

## Version-Sensitive Notes

- `SourceLanguageRecognizer` was added in `1.18.0`. If an older codebase does not have it, the project is pinned well below the current package version.
- `SpeechSynthesisRequest` was added in `1.37.0` and is still marked preview in the API reference. Avoid building critical abstractions around preview-only synthesis features without checking the exact installed version.
- Microsoft Learn notes that pronunciation assessment content assessment was retired in Speech SDK `1.46.0` and later. Do not rely on older examples that still reference that preview capability.
- The version used here for this package matched current PyPI at `1.48.2` on March 12, 2026, so the frontmatter uses the live registry version without drift correction.

## Official Sources

- API reference: `https://learn.microsoft.com/en-us/python/api/azure-cognitiveservices-speech/`
- SDK install guide: `https://learn.microsoft.com/en-us/azure/ai-services/speech-service/quickstarts/setup-platform`
- Speech to text quickstart: `https://learn.microsoft.com/en-us/azure/ai-services/speech-service/get-started-speech-to-text`
- Text to speech quickstart: `https://learn.microsoft.com/en-us/azure/ai-services/speech-service/get-started-text-to-speech`
- Speech recognition how-to: `https://learn.microsoft.com/en-us/azure/ai-services/speech-service/how-to-recognize-speech`
- Speech translation how-to: `https://learn.microsoft.com/en-us/azure/ai-services/speech-service/how-to-translate-speech`
- Language identification: `https://learn.microsoft.com/en-us/azure/ai-services/speech-service/language-identification`
- Pronunciation assessment: `https://learn.microsoft.com/en-us/azure/ai-services/speech-service/how-to-pronunciation-assessment`
- Compressed audio input: `https://learn.microsoft.com/en-us/azure/ai-services/speech-service/how-to-use-codec-compressed-audio-input-streams`
- PyPI package page: `https://pypi.org/project/azure-cognitiveservices-speech/`
