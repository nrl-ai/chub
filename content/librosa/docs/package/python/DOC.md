---
name: package
description: "librosa Python package guide for audio loading, feature extraction, beat tracking, and streaming analysis"
metadata:
  languages: "python"
  versions: "0.11.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "librosa,python,audio,music,dsp,signal-processing,feature-extraction"
---

# librosa Python Package Guide

## Golden Rule

Use `librosa` for offline music and audio analysis, but be explicit about sample rate, channel handling, and frame settings. The most common agent mistake is calling `librosa.load()` and then forgetting that it resamples to `22050` Hz by default and mixes audio to mono unless you override that behavior.

## Install

Pin the package version your project expects:

```bash
python -m pip install "librosa==0.11.0"
```

If you want plotting helpers from `librosa.display`, install the display extra:

```bash
python -m pip install "librosa[display]==0.11.0"
```

If you prefer conda, the maintainers also publish packages on conda-forge:

```bash
conda install -c conda-forge librosa
```

Notes:

- `librosa` depends on `soundfile` for audio decoding in the supported path. On some Linux systems, `soundfile` may need system `libsndfile` packages installed first.
- If you need codecs that `soundfile` cannot decode, you may still see projects rely on `audioread`, but `audioread` support is deprecated in `0.10.x` and scheduled for removal in `1.0`.

## Initialize And Inspect The Environment

Basic import and version check:

```python
import librosa

print(librosa.__version__)
librosa.show_versions()
```

`show_versions()` is useful in bug reports because audio backends, NumPy/SciPy versions, and optional dependencies affect behavior.

## Core Usage

### Load audio safely

Use `sr=None` when you need the file's native sample rate, and use `mono=False` when channel structure matters:

```python
import librosa

y, sr = librosa.load("audio.wav", sr=None, mono=True)
print(y.shape, sr)
```

Useful options:

- `sr=None`: keep the original sample rate instead of resampling to `22050`
- `mono=False`: preserve channels instead of downmixing
- `offset=` and `duration=`: read only part of a file
- `res_type=`: choose a different resampler when you do want resampling

### Compute a mel spectrogram

```python
import librosa

y, sr = librosa.load("audio.wav", sr=None)
mel = librosa.feature.melspectrogram(y=y, sr=sr, n_fft=2048, hop_length=512, n_mels=128)
mel_db = librosa.power_to_db(mel, ref="max")

print(mel_db.shape)
```

This is the standard entry point for many classification, similarity, and visualization pipelines.

### Estimate tempo and beats

```python
import librosa

y, sr = librosa.load("audio.wav", sr=None)
tempo, beat_frames = librosa.beat.beat_track(y=y, sr=sr)
beat_times = librosa.frames_to_time(beat_frames, sr=sr)

print(tempo, beat_times[:10])
```

If you need times in seconds, convert frames explicitly with `frames_to_time`.

### Work from example audio in the docs

The maintainers provide downloadable example recordings:

```python
import librosa

path = librosa.util.example("brahms")
y, sr = librosa.load(path, sr=None)
```

This is useful for tests, debugging, and reproducible examples.

### Stream long files instead of loading everything

For large files, stream blocks and process each frame window incrementally:

```python
import soundfile as sf
import librosa

with sf.SoundFile("long_recording.wav") as f:
    for y_block in librosa.stream(
        f,
        block_length=256,
        frame_length=2048,
        hop_length=512,
        mono=True,
    ):
        rms = librosa.feature.rms(y=y_block, frame_length=2048, hop_length=512)
        print(rms.shape)
```

When using `librosa.stream`, pair it with frame-based feature extraction and disable centered analysis where applicable. Centered framing assumes full random access to the signal and is usually incompatible with streamed blocks.

## Configuration And Runtime Notes

There is no auth or remote service configuration. The main runtime choices are audio backend behavior, cache/example data location, and numerical settings.

Useful environment and setup notes:

- `LIBROSA_DATA_DIR` controls where example audio files downloaded by `librosa.util.example()` are cached.
- Plotting with `librosa.display` requires the optional display dependencies.
- Resampling quality and speed depend on the selected backend and `res_type`.
- Numerical results can shift slightly across NumPy/SciPy/audio-backend versions, so pin dependencies for reproducible pipelines.

## Common Pitfalls

- `librosa.load()` defaults to `sr=22050`. If your model or analysis expects the source sample rate, pass `sr=None`.
- `librosa.load()` defaults to mono mixing. Pass `mono=False` for stereo or multichannel workflows.
- Old blog posts often use positional arguments that no longer match current keyword-only signatures cleanly. Prefer explicit keyword arguments such as `y=y, sr=sr`.
- `librosa.stream()` is not a drop-in replacement for `load()`. Features that depend on centered windows or future context need different settings.
- `librosa.beat.beat_track()` in current docs returns tempo together with beat frames; downstream code should not assume only one return value.
- If decoding fails for compressed formats, check whether the file is supported by `soundfile` before reaching for deprecated `audioread` workarounds.
- If you import `librosa.display` without the extra plotting dependencies installed, plotting code will fail even though the core package imported successfully.

## Version-Sensitive Notes For 0.11.0

- `0.11.0` is the current stable release on PyPI as of March 12, 2026, and the versioned docs root `https://librosa.org/doc/0.11.0/` is safer than relying on the floating `latest/` URL.
- The project supports NumPy 2.0 in the modern `0.11.0` line, so very old compatibility advice around pinning pre-2.0 NumPy is often stale.
- The changelog notes that the default FFT backend now uses `scipy.fft`, and the old `librosa.set_fftlib` path is deprecated in favor of SciPy backend control.
- `audioread` support was deprecated in the `0.10` series and is planned for removal in `1.0`; prefer `soundfile`-based workflows when writing new code.

## Official Source URLs

- Docs root: `https://librosa.org/doc/0.11.0/`
- Installation: `https://librosa.org/doc/0.11.0/install.html`
- Changelog: `https://librosa.org/doc/0.11.0/changelog.html`
- `librosa.load`: `https://librosa.org/doc/0.11.0/generated/librosa.load.html`
- Mel spectrograms: `https://librosa.org/doc/0.11.0/generated/librosa.feature.melspectrogram.html`
- Beat tracking: `https://librosa.org/doc/0.11.0/generated/librosa.beat.beat_track.html`
- Streaming: `https://librosa.org/doc/0.11.0/generated/librosa.stream.html`
- Example audio helper: `https://librosa.org/doc/0.11.0/generated/librosa.util.example.html`
- PyPI: `https://pypi.org/project/librosa/0.11.0/`
