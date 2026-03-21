---
name: package
description: "PyAV Python bindings for FFmpeg for decoding, encoding, remuxing, and frame conversion"
metadata:
  languages: "python"
  versions: "16.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pyav,av,ffmpeg,video,audio,media,encoding,decoding"
---

# PyAV Python Package Guide

## Golden Rule

Use `av` as Python bindings around FFmpeg, and treat the current PyAV docs root as `https://pyav.basswood-io.com/docs/stable/`. The docs URL `https://pyav.org/docs/stable/` is still live, but on 2026-03-12 it serves older `9.0.2` docs and should not be used as the canonical source for `16.1.0`.

## Install

Pin the version your project expects:

```bash
python -m pip install "av==16.1.0"
```

Common alternatives:

```bash
uv add "av==16.1.0"
poetry add "av==16.1.0"
```

For frame conversion helpers you will often also want:

```bash
python -m pip install "numpy" "pillow"
```

Source-build notes:

- PyAV `16.1.0` supports Python `3.10` through `3.13`.
- Building from source requires FFmpeg `7.0` or newer plus development headers and `pkg-config`.
- Prefer prebuilt wheels unless you need a custom FFmpeg build or a platform without wheels.

## Initialization And Environment

PyAV has no package-level authentication model. It opens local files, Python file-like objects, or media/network URLs through FFmpeg.

The core entry point is `av.open(...)`:

```python
import av

container = av.open("input.mp4")
container.close()
```

Use a context manager so containers always close cleanly:

```python
import av

with av.open("input.mp4") as container:
    print(container.format.name)
    print(container.streams.video)
```

Useful `av.open(...)` controls:

- `mode="r"` or `"w"` when the input is not obvious
- `format="mp4"` or another container name when FFmpeg cannot infer it
- `options={...}` for demuxer/protocol settings
- `container_options={...}` and `stream_options=[...]` for finer-grained FFmpeg control
- `timeout=` as a float or `(open_timeout, read_timeout)` tuple for network inputs
- `buffer_size=` when wrapping a Python file-like object

Example for a network stream:

```python
import av

with av.open(
    "rtsp://example.invalid/live",
    options={"rtsp_transport": "tcp"},
    timeout=(5.0, 30.0),
) as container:
    for frame in container.decode(video=0):
        print(frame.pts)
        break
```

If you pass a Python file object, open it in binary mode:

```python
import av

with open("input.mp4", "rb") as fh, av.open(fh, mode="r") as container:
    frame = next(container.decode(video=0))
    print(frame)
```

## Core Usage

### Decode frames from a video stream

```python
import av

with av.open("input.mp4") as container:
    video_stream = container.streams.video[0]

    for frame in container.decode(video_stream):
        print(
            "pts=", frame.pts,
            "time_base=", frame.time_base,
            "size=", (frame.width, frame.height),
        )
```

`container.decode(...)` yields decoded `VideoFrame` or `AudioFrame` objects. Use `container.streams.video[0]` or `container.streams.audio[0]` rather than assuming the first stream is the right type.

### Convert video frames to NumPy arrays

```python
import av

with av.open("input.mp4") as container:
    frame = next(container.decode(video=0))
    rgb = frame.to_ndarray(format="rgb24")
    print(rgb.shape)
```

`VideoFrame.to_ndarray(...)` is the most direct way to hand frames to NumPy/OpenCV-style code. Choose the pixel format explicitly when downstream code expects RGB rather than FFmpeg-native YUV layouts.

### Create frames from NumPy and encode a video

```python
import av
import numpy as np

width = 640
height = 360

with av.open("output.mp4", mode="w") as container:
    stream = container.add_stream("libx264", rate=30)
    stream.width = width
    stream.height = height
    stream.pix_fmt = "yuv420p"

    for i in range(60):
        rgb = np.zeros((height, width, 3), dtype=np.uint8)
        rgb[:, :, 0] = (i * 4) % 255
        frame = av.VideoFrame.from_ndarray(rgb, format="rgb24")

        for packet in stream.encode(frame):
            container.mux(packet)

    for packet in stream.encode(None):
        container.mux(packet)
```

Two rules matter here:

- Set codec properties such as `width`, `height`, and `pix_fmt` before encoding.
- Flush the encoder with `stream.encode(None)` or the file may be truncated.

### Remux packets without re-encoding

Use remuxing when you want a new container but the encoded media should stay unchanged:

```python
import av

with av.open("input.mp4") as input_container, av.open("output.mkv", mode="w") as output_container:
    in_video = input_container.streams.video[0]
    out_video = output_container.add_stream_from_template(in_video)

    for packet in input_container.demux(in_video):
        if packet.dts is None:
            continue

        packet.stream = out_video
        output_container.mux(packet)
```

`add_stream_from_template(...)` preserves codec parameters from the input stream. For remuxing, do not decode frames and do not invent new timestamps unless you really intend to re-encode.

### Work with audio streams

```python
import av

with av.open("input.mp4") as container:
    audio_stream = container.streams.audio[0]

    for frame in container.decode(audio_stream):
        print(frame.samples, frame.sample_rate, frame.layout.name)
        break
```

PyAV also exposes audio resampling and FIFO helpers, but simple decode/read flows start exactly like video: select the correct stream, decode frames, and inspect sample metadata before transforming anything.

## Configuration Notes

- There is no separate credential object. For HTTP, RTSP, S3-compatible gateways, and similar inputs, credentials typically live in the URL or in FFmpeg/protocol options passed to `av.open(...)`.
- `options={...}` is the first place to put protocol knobs such as transport choice, reconnect flags, codec hints, or demuxer-specific options.
- Use `timeout=` for network sources; otherwise a bad remote source can block longer than your application expects.
- PyAV does not replace FFmpeg concepts. Container format, codec name, pixel format, sample format, and timestamps still matter and often must be set explicitly.

## Errors And Debugging

PyAV raises FFmpeg-backed exceptions such as `av.FFmpegError` and more specific subclasses like `av.DecoderNotFoundError`.

```python
import av

try:
    with av.open("missing.mp4") as container:
        next(container.decode(video=0))
except av.FFmpegError as exc:
    print(type(exc).__name__)
    print(exc)
```

During local debugging you can enable FFmpeg logging:

```python
import av

av.logging.set_level(av.logging.VERBOSE)
```

Do not leave verbose logging on by default in threaded or embedded interpreter environments. The upstream caveats page warns that FFmpeg logging can interact badly with Python sub-interpreters and has caused lockups in Cython shutdown paths.

## Common Pitfalls

- Always close containers. Use `with av.open(...) as container:` instead of relying on garbage collection.
- Do not confuse remuxing with transcoding. `demux` plus `mux` keeps encoded packets; `decode` plus `encode` produces new media.
- When remuxing, skip packets with `packet.dts is None`; the upstream cookbook example does this because flush packets are not valid for muxing.
- Be explicit about stream selection. Containers can have multiple video, audio, subtitle, or data streams.
- Frame timing is not seconds by default. `frame.pts` and `packet.pts` are expressed in units of `time_base`.
- If colors look wrong after conversion, check `pix_fmt` and the array format you requested from `to_ndarray(...)` or `from_ndarray(...)`.
- If installation falls back to a source build and fails, you are usually missing FFmpeg development libraries or `pkg-config`.
- Older PyAV blog posts and the old `pyav.org` docs can be materially stale. Validate examples against the current `basswood-io.com` docs and the `16.1.0` changelog.

## Time And Timestamps

PyAV follows FFmpeg timing rules:

- `Packet.pts` / `Packet.dts` are in the stream time base
- `Frame.pts` is in the frame time base
- `time_base` is the unit conversion factor, not a frame rate

Practical rule: do not fabricate timestamps unless you understand the target codec and container. When reusing an existing stream template, preserve upstream timing. When encoding new content, make the stream rate and frame cadence consistent from the start.

## Version-Sensitive Notes For 16.1.0

- PyPI and the current maintained docs both reflect `16.1.0` as of 2026-03-12.
- The maintained docs root is `https://pyav.basswood-io.com/docs/stable/`; the docs URL `https://pyav.org/docs/stable/` currently serves old `9.0.2` documentation and should be treated as stale.
- PyAV `16.x` supports Python `3.10+` and source builds against FFmpeg `7.0+`.
- The `16.1.0` changelog adds more modern media support, including AMD AMF hardware decoding, AV1/H.264 multiview decoding, `CodecContext.colorspace`, and alpha-aware `libx264rgb` encoding. If you are copying older examples, check whether they predate these capabilities.

## Official Links

- Docs root: `https://pyav.basswood-io.com/docs/stable/`
- Installation: `https://pyav.basswood-io.com/docs/stable/overview/installation.html`
- Basics cookbook: `https://pyav.basswood-io.com/docs/stable/cookbook/basics.html`
- Timing guide: `https://pyav.basswood-io.com/docs/stable/api/time.html`
- API reference: `https://pyav.basswood-io.com/docs/stable/api/_globals.html`
- Changelog: `https://github.com/PyAV-Org/PyAV/blob/main/CHANGELOG.rst`
- PyPI: `https://pypi.org/project/av/`
