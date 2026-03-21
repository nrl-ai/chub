---
name: package
description: "torchaudio for Python: PyTorch-native audio transforms, datasets, and pretrained speech pipelines"
metadata:
  languages: "python"
  versions: "2.10.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "torchaudio,pytorch,audio,speech,ml,asr"
---

# torchaudio Python Package Guide

## What It Is

`torchaudio` is the PyTorch audio library. In current releases it is primarily for:

- audio transforms and signal features
- speech and audio models/pipelines
- dataset wrappers for common public audio corpora
- PyTorch-native tensor workflows on CPU or CUDA

Do not treat it as a general-purpose media I/O library anymore. Since `torchaudio` 2.9, audio decode/encode convenience APIs are implemented through TorchCodec, and TorchAudio is in a maintenance phase focused on its core ML strengths.

## Installation

## Golden Rule

Install `torchaudio` and `torch` from the same release line. Do not mix `torchaudio==2.10.0` with an older or newer PyTorch build.

The official install page still links to the PyTorch install selector and correctly says `torchaudio` is compiled against a specific `torch` version, but the compatibility table visible under `/audio/stable/installation.html` is stale and only lists versions through `2.6.0`. For `2.10.0`, confirm the package version on PyPI and keep the `torch` release aligned.

### Typical install

```bash
pip install torch==2.10.0 torchaudio==2.10.0
```

If you need the correct CUDA or platform-specific wheel, start from the PyTorch selector:

- `https://pytorch.org/get-started/locally/`

### Optional dependencies called out by the official docs

```bash
pip install sentencepiece
pip install deep-phonemizer
```

- `sentencepiece` is required for Emformer RNN-T ASR examples.
- `deep-phonemizer` is required for Tacotron2 text-to-speech examples.

### Audio I/O in 2.9+

If you call `torchaudio.load()` or `torchaudio.save()` in new code, also install TorchCodec:

```bash
pip install torchcodec
```

`torchaudio.load()` and `torchaudio.save()` now delegate to TorchCodec. TorchCodec uses FFmpeg under the hood for media decode/encode, so keep that dependency in mind when debugging runtime failures.

## Basic Setup

```python
import torch
import torchaudio

device = torch.device("cuda" if torch.cuda.is_available() else "cpu")

print(torch.__version__)
print(torchaudio.__version__)
print(device)
```

For package-sensitive code paths, fail early on mismatched versions:

```python
import torch
import torchaudio

if not torch.__version__.startswith("2.10."):
    raise RuntimeError(f"Expected torch 2.10.x, got {torch.__version__}")

if not torchaudio.__version__.startswith("2.10."):
    raise RuntimeError(f"Expected torchaudio 2.10.x, got {torchaudio.__version__}")
```

## Core Usage

## Load a waveform

```python
import torchaudio

waveform, sample_rate = torchaudio.load("speech.wav")
print(waveform.shape)  # [channels, time]
print(sample_rate)
```

Current behavior to remember:

- `torchaudio.load()` returns `float32` tensors.
- `normalize=False` no longer restores old integer semantics; TorchCodec always returns normalized float32.
- `backend` and `buffer_size` are accepted for compatibility but ignored.
- `uri` can be a path, URL, or file-like object.

For new performance-sensitive decode paths, prefer native TorchCodec decoders instead of treating `torchaudio.load()` as the long-term primary API.

## Resample audio

Use the functional API for one-off resampling:

```python
import torchaudio

waveform_16k = torchaudio.functional.resample(waveform, sample_rate, 16_000)
```

Use the transform when you will reuse the same sample-rate conversion repeatedly:

```python
import torchaudio

resample = torchaudio.transforms.Resample(orig_freq=48_000, new_freq=16_000)
waveform_16k = resample(waveform)
```

Important precision note from the official `Resample` docs:

- `torchaudio.transforms.Resample` caches its kernel and may lose a small amount of precision for inputs above `float32`.
- If high precision matters, use `torchaudio.functional.resample(...)` or configure the transform to cache a higher-precision kernel.

## Extract audio features

```python
import torchaudio

mel = torchaudio.transforms.MelSpectrogram(
    sample_rate=16_000,
    n_fft=400,
    hop_length=160,
    n_mels=80,
)

features = mel(waveform_16k)  # [channels, n_mels, time]
```

`MelSpectrogram` is one of the core transforms that remains squarely in TorchAudio’s supported ML-focused surface area.

## Run a pretrained ASR pipeline

```python
import torch
import torchaudio

device = torch.device("cuda" if torch.cuda.is_available() else "cpu")

bundle = torchaudio.pipelines.WAV2VEC2_ASR_BASE_960H
model = bundle.get_model().to(device)

waveform, sample_rate = torchaudio.load("speech.wav")
waveform = waveform.to(device)

if sample_rate != bundle.sample_rate:
    waveform = torchaudio.functional.resample(
        waveform, sample_rate, bundle.sample_rate
    )

with torch.inference_mode():
    emissions, _ = model(waveform)

labels = bundle.get_labels()
print(bundle.sample_rate)
print(labels[:8])
print(emissions.shape)
```

What the bundle gives you:

- `bundle.sample_rate`
- `bundle.get_labels()`
- `bundle.get_model()`

The official tutorial also shows a simple greedy CTC decoder built from `bundle.get_labels()`.

## Use built-in datasets

```python
import torchaudio

dataset = torchaudio.datasets.YESNO("./data/yesno", download=True)
waveform, sample_rate, labels = dataset[0]
```

Dataset wrappers are still a first-class use case. The dataset APIs typically return waveform tensors plus metadata such as sample rate, labels, transcripts, or speaker identifiers depending on the dataset.

## Save audio

```python
import torch
import torchaudio

audio = torch.clamp(waveform_16k, -1.0, 1.0).to(torch.float32)
torchaudio.save("out.wav", audio, sample_rate=16_000)
```

Current save behavior:

- `torchaudio.save()` uses TorchCodec’s `AudioEncoder`.
- input must be `float32` in `[-1, 1]`
- output format is determined by the filename extension
- old parameters like `format`, `encoding`, `bits_per_sample`, and `backend` are compatibility shims and should not be the basis of new code

## Config And Auth

There is no package-level auth or service configuration model.

What usually matters instead:

- model bundles may download weights on first use
- dataset helpers may download public archives when `download=True`
- your runtime needs outbound network access and writable cache/storage paths for those downloads
- pretrained models and datasets can have separate licenses or usage restrictions; check the dataset/model notes before shipping them in a product

## Common Pitfalls

- Version mismatch: `torchaudio` wheels are built against specific PyTorch releases. If import or extension loading fails, check that `torch` and `torchaudio` are on the same release line first.
- Assuming old backend behavior: code that depends on `sox_io`, `soundfile`, explicit backend selection, or old integer decode behavior is likely from pre-2.9 assumptions.
- Relying on ignored parameters: in current `load()` and `save()` implementations, several legacy parameters are accepted but ignored because TorchCodec is underneath.
- Forgetting TorchCodec: a missing `torchcodec` installation now causes `ImportError` for `torchaudio.load()` and `torchaudio.save()`.
- Forgetting sample-rate alignment: many pretrained pipelines expect a fixed sample rate. Resample before inference.
- Treating it like a general multimedia toolkit: the project is intentionally narrower now. Prefer TorchCodec for media decode/encode primitives and keep TorchAudio for transforms, models, pipelines, and datasets.

## Version-Sensitive Notes For 2.10.0

- PyPI shows `torchaudio 2.10.0` released on `2026-01-21`.
- The maintainers’ 2026-01-22 update on issue `pytorch/audio#3902` says `2.10` completes the migration to the maintenance-phase scope.
- Most of `transforms`, `functional`, `compliance.kaldi`, `models`, and `pipelines` remain part of the supported surface.
- The same maintainer update says `lfilter`, `RNNTLoss`, `CUCT`, `forced_align`, and `overdrive` were preserved even though they were originally planned for removal.
- Parts of the official docs site chrome under `/audio/stable/` are inconsistent: some pages still display stale "Old version" or `2.8.0a0` navigation labels while their API bodies document the newer behavior. Trust the page body plus PyPI release history when checking `2.10.0` specifics.

## Official Sources

- Docs root: `https://docs.pytorch.org/audio/stable/`
- Install guide: `https://docs.pytorch.org/audio/stable/installation.html`
- `torchaudio.load`: `https://docs.pytorch.org/audio/stable/generated/torchaudio.load.html`
- `torchaudio.save`: `https://docs.pytorch.org/audio/stable/generated/torchaudio.save.html`
- `torchaudio.transforms.Resample`: `https://docs.pytorch.org/audio/stable/generated/torchaudio.transforms.Resample.html`
- `torchaudio.transforms.MelSpectrogram`: `https://docs.pytorch.org/audio/stable/generated/torchaudio.transforms.MelSpectrogram.html`
- Speech recognition tutorial: `https://docs.pytorch.org/audio/stable/tutorials/speech_recognition_pipeline_tutorial.html`
- Audio datasets tutorial: `https://docs.pytorch.org/audio/stable/tutorials/audio_datasets_tutorial.html`
- YESNO dataset reference: `https://docs.pytorch.org/audio/stable/generated/torchaudio.datasets.YESNO.html`
- PyPI package page: `https://pypi.org/project/torchaudio/`
- Upstream repository: `https://github.com/pytorch/audio`
- Maintainer migration note: `https://github.com/pytorch/audio/issues/3902`
- TorchCodec install docs: `https://github.com/pytorch/torchcodec#installing-torchcodec`
