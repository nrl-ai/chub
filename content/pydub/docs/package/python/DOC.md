---
name: package
description: "pydub package guide for Python audio loading, editing, effects, export, and ffmpeg-backed transcoding"
metadata:
  languages: "python"
  versions: "0.25.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pydub,audio,ffmpeg,media,python"
---

# pydub Python Package Guide

## Golden Rule

Use `pydub` as a high-level wrapper around `ffmpeg` or `libav`, not as a standalone codec stack. `AudioSegment` handles slicing, mixing, fades, silence utilities, and export, but compressed formats such as MP3, AAC, OGG, MP4, and FLV still depend on external binaries being installed and discoverable.

## Install

Install the Python package first:

```bash
python -m pip install "pydub==0.25.1"
```

Common alternatives:

```bash
uv add "pydub==0.25.1"
poetry add "pydub==0.25.1"
```

Install an audio backend as well. For most real work, that means `ffmpeg` plus `ffprobe` on `PATH`:

```bash
brew install ffmpeg
sudo apt-get install ffmpeg
choco install ffmpeg
```

Optional helpers:

- `simpleaudio` for the cleanest local playback path
- `pyaudio` as a fallback playback backend
- `scipy` for `pydub.effects.speedup(...)`

## Initialize And Verify Setup

Check the system binaries before debugging Python code:

```bash
ffmpeg -version
ffprobe -version
```

Minimal import:

```python
from pydub import AudioSegment
```

If `ffmpeg` is installed outside `PATH`, point `pydub` at it explicitly:

```python
from pydub import AudioSegment

AudioSegment.converter = "/opt/homebrew/bin/ffmpeg"
AudioSegment.ffmpeg = AudioSegment.converter
```

`pydub` resolves the probing binary separately, so keep `ffprobe` or `avprobe` available on `PATH` as well.

## Core Usage

### Load audio

`AudioSegment` is immutable. Each operation returns a new segment.

```python
from pydub import AudioSegment

song = AudioSegment.from_mp3("input.mp3")
wav = AudioSegment.from_wav("input.wav")
clip = AudioSegment.from_file("input.m4a", format="m4a")
```

If the file format is ambiguous or the extension is unreliable, pass `format=...` explicitly.

### Slice and transform

All time values are milliseconds:

```python
from pydub import AudioSegment

song = AudioSegment.from_mp3("podcast.mp3")

intro = song[:10_000]
middle = song[30_000:45_000]
louder = middle + 6
quieter = middle - 3
fade = louder.fade_in(2_000).fade_out(3_000)
mono = fade.set_channels(1)
resampled = mono.set_frame_rate(16_000)
```

### Concatenate, crossfade, and overlay

```python
from pydub import AudioSegment

voice = AudioSegment.from_wav("voice.wav")
music = AudioSegment.from_mp3("music.mp3") - 12

episode = voice.append(voice, crossfade=1_500)
mix = music.overlay(voice, position=5_000)
```

Use `append(..., crossfade=...)` for linear sequencing and `overlay(...)` for mixing simultaneous audio.

### Export

```python
from pydub import AudioSegment

segment = AudioSegment.from_file("input.wav")

segment.export(
    "output.mp3",
    format="mp3",
    bitrate="192k",
    tags={"artist": "Example", "album": "Demo"},
)
```

You can pass raw encoder flags through `parameters`, and `pydub` does not validate them:

```python
segment.export(
    "output.mp3",
    format="mp3",
    parameters=["-q:a", "0"],
)
```

### Silence-driven chunking

Use the silence helpers when you need coarse speech segmentation:

```python
from pydub import AudioSegment
from pydub.silence import split_on_silence

audio = AudioSegment.from_file("meeting.wav")

chunks = split_on_silence(
    audio,
    min_silence_len=700,
    silence_thresh=audio.dBFS - 16,
    keep_silence=250,
)
```

Tune `min_silence_len` and `silence_thresh` against the actual recording instead of copying fixed values from old examples.

### Playback

```python
from pydub import AudioSegment
from pydub.playback import play

segment = AudioSegment.from_file("preview.mp3")
play(segment)
```

In `0.25.1`, playback tries `simpleaudio`, then `pyaudio`, then `ffplay`. Treat playback as a local convenience, not a deployment primitive.

## Configuration Notes

`pydub` has no auth model and no service configuration. The main runtime configuration is local:

- `ffmpeg` and `ffprobe` binary availability
- file formats and codecs supported by the installed backend
- sample rate, channels, and bit depth expected by downstream systems
- export flags passed through `parameters`

When agents generate audio for speech or ML pipelines, normalize explicitly instead of assuming the source file already matches the target format:

```python
from pydub import AudioSegment

segment = AudioSegment.from_file("input.mp3")
normalized = (
    segment.set_channels(1)
    .set_frame_rate(16_000)
    .set_sample_width(2)
)

normalized.export("speech.wav", format="wav")
```

## Common Pitfalls

- Missing `ffmpeg` or `ffprobe` is the most common failure mode for non-WAV formats. Decode or export errors often trace back to missing binaries, not bad Python code.
- Time indexes are milliseconds, not seconds. `song[:10]` means ten milliseconds, not ten seconds.
- `AudioSegment` objects are immutable. Methods such as `fade_in`, `set_frame_rate`, and `set_channels` return new segments.
- `export(..., parameters=[...])` forwards flags directly to `ffmpeg`. Invalid flags fail at runtime.
- `play(...)` depends on optional local playback backends and may behave differently across machines.
- Silence splitting is data-dependent. Reuse the structure, not the exact thresholds, from prior examples.
- For large transcodes or batch jobs, remember that `pydub` is a Python wrapper around command-line tooling. If you need streaming-scale media pipelines, direct `ffmpeg` orchestration may be a better fit.

## Version-Sensitive Notes

- `0.25.1` is still the current released version on both PyPI and the GitHub releases page as of 2026-03-12.
- The latest release was published on 2021-03-10, so many third-party posts are effectively documenting this same code line. Prefer the maintainer README and API docs over blogs when method signatures look inconsistent.
- The official API docs still describe `speedup(...)` as requiring `scipy`. Keep that dependency explicit if you use that effect in generated code.
- This release line is old enough that you should validate newer Python runtimes in CI before assuming compatibility from unsourced blog examples.

## Official Source URLs

- `https://pypi.org/project/pydub/`
- `https://github.com/jiaaro/pydub`
- `https://github.com/jiaaro/pydub/blob/master/README.markdown`
- `https://github.com/jiaaro/pydub/blob/master/API.markdown`
- `https://github.com/jiaaro/pydub/blob/master/pydub/audio_segment.py`
- `https://github.com/jiaaro/pydub/blob/master/pydub/utils.py`
- `https://github.com/jiaaro/pydub/blob/master/pydub/playback.py`
- `https://github.com/jiaaro/pydub/releases/tag/v0.25.1`
