---
name: package
description: "vLLM Python package guide for high-throughput LLM inference and OpenAI-compatible serving"
metadata:
  languages: "python"
  versions: "0.17.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "vllm,llm,inference,serving,openai-compatible,huggingface,gpu"
---

# vLLM Python Package Guide

## What It Covers

`vllm` is a Python package for high-throughput LLM inference. In practice you use it in one of two ways:

- embed it in Python with `LLM(...)` for local or service-side generation
- run `vllm serve ...` to expose an OpenAI-compatible HTTP API

This guide targets PyPI package version `0.17.1`.

## Installation

Official docs list Python `3.10` through `3.13` as supported.

```bash
pip install "vllm==0.17.1"
```

If you use `uv`, the install docs recommend:

```bash
uv pip install --torch-backend=auto vllm
```

Important backend note:

- `pip install vllm` currently installs the default wheel for Linux `x86_64` with CUDA `12.8` and PyTorch `2.7.1`
- if you need CPU, AMD ROCm, Intel XPU, TPU, or a different CUDA stack, use the backend-specific install pages from the official docs instead of assuming the default wheel is correct

## Setup And Model Access

Choose a Hugging Face model ID that vLLM supports, for example `Qwen/Qwen2.5-1.5B-Instruct`.

Model downloads are cached in the Hugging Face cache directory. By default that is:

```bash
~/.cache/huggingface
```

Set `HF_HOME` if you need the cache elsewhere:

```bash
export HF_HOME=/mnt/models/hf
```

Only enable remote model code when the upstream repository is trusted:

```python
llm = LLM(model="some/model", trust_remote_code=True)
```

The CLI/server equivalent is `--trust-remote-code`.

## Embedded Python Usage

```python
from vllm import LLM, SamplingParams

prompts = [
    "Name three practical uses for speculative decoding.",
    "Write a two-line summary of vLLM.",
]

sampling_params = SamplingParams(
    temperature=0.7,
    top_p=0.95,
    max_tokens=128,
)

llm = LLM(model="Qwen/Qwen2.5-1.5B-Instruct")
outputs = llm.generate(prompts, sampling_params)

for output in outputs:
    print(output.prompt)
    print(output.outputs[0].text)
```

Use this path when your Python process owns model lifecycle and inference directly.

## OpenAI-Compatible Server

Start the server:

```bash
vllm serve Qwen/Qwen2.5-1.5B-Instruct --api-key token-abc123
```

Then call it with the OpenAI Python client:

```bash
pip install openai
```

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:8000/v1",
    api_key="token-abc123",
)

completion = client.chat.completions.create(
    model="Qwen/Qwen2.5-1.5B-Instruct",
    messages=[
        {"role": "system", "content": "You are a concise assistant."},
        {"role": "user", "content": "Explain tensor parallelism in one paragraph."},
    ],
)

print(completion.choices[0].message.content)
```

Auth and access control:

- server auth can be set with `--api-key` or `VLLM_API_KEY`
- the HTTP surface is OpenAI-compatible, but you still need to secure the process, host, and network boundary yourself

## Common Configuration Notes

- Prefer passing an explicit model ID everywhere instead of relying on examples copied from unrelated model families.
- For multimodal or audio models, install the matching extras when the official model docs require them, for example:

```bash
pip install "vllm[audio]==0.17.1"
```

- Keep an eye on local disk usage. vLLM commonly pulls multi-GB model weights into the Hugging Face cache.

## Common Pitfalls

- The official docs URL for this package is `/en/latest/`, so some install details can move ahead of the exact package version in your environment.
- The OpenAI-compatible server uses a model repository's `generation_config.json` by default. If you want vLLM defaults instead, start the server with:

```bash
vllm serve Qwen/Qwen2.5-1.5B-Instruct --generation-config vllm
```

- Do not enable `trust_remote_code` unless the model repository is trusted; it allows execution of model-provided Python code.
- Do not assume the default `pip install vllm` wheel matches your accelerator stack. Backend-specific install docs matter for non-default environments.

## Version-Sensitive Notes For `0.17.1`

- PyPI package version covered here: `0.17.1`
- Official install docs currently state Python `3.10` to `3.13`
- Official latest install docs currently state the default pre-built wheel targets Linux `x86_64`, CUDA `12.8`, and PyTorch `2.7.1`
- Because the docs root is `latest`, verify backend and deployment details again if you are debugging behavior on a different `vllm` minor release

## Official Sources

- Docs root: https://docs.vllm.ai/en/latest/
- Quickstart: https://docs.vllm.ai/en/latest/getting_started/quickstart.html
- Installation overview: https://docs.vllm.ai/en/latest/getting_started/installation/index.html
- OpenAI-compatible server: https://docs.vllm.ai/en/latest/serving/openai_compatible_server.html
- Supported models: https://docs.vllm.ai/en/latest/models/supported_models.html
- PyPI: https://pypi.org/project/vllm/
