---
name: package
description: "Unsloth Python package guide for fast LLM fine-tuning, inference, and export workflows"
metadata:
  languages: "python"
  versions: "2026.3.4"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "unsloth,python,llm,fine-tuning,inference,lora,trl,transformers"
---

# Unsloth Python Package Guide

## What It Is

`unsloth` is a Python package for faster, lower-memory LLM workflows on top of the Hugging Face stack. The maintainer docs center on:

- supervised fine-tuning with LoRA or QLoRA
- accelerated inference for supported models
- reinforcement learning workflows such as GRPO, GSPO, DPO, ORPO, PPO, and reward modeling
- model export to merged Hugging Face weights or GGUF for `llama.cpp`-style runtimes

For `2026.3.4`, the practical reference points are the official docs site, the PyPI package metadata, and the maintainer GitHub README examples.

## Installation

Install the package directly from PyPI:

```bash
pip install unsloth==2026.3.4
```

If you are not pinning the version used here:

```bash
pip install unsloth
```

### Environment expectations

- The official docs are oriented around NVIDIA GPU setups and hosted notebook environments such as Colab, Kaggle, RunPod, AWS, Azure, and GCP.
- The official FAQ says Windows users should run Unsloth through WSL instead of native Windows.
- Plan the rest of the stack together: `unsloth` works with `transformers`, `trl`, `datasets`, and PEFT-style adapter training.

## Core Setup

The maintainer README uses `FastLanguageModel` as the stable entry point for text-model loading, adapter attachment, training, and inference.

```python
from unsloth import FastLanguageModel

max_seq_length = 2048

model, tokenizer = FastLanguageModel.from_pretrained(
    model_name="unsloth/Llama-3.2-1B",
    max_seq_length=max_seq_length,
    dtype=None,
    load_in_4bit=True,
    token=None,  # supply a Hugging Face token only for gated models
)
```

Important knobs from the official examples:

- `max_seq_length`: sets the target context window and is part of Unsloth's long-context handling.
- `dtype=None`: lets the loader auto-detect a sensible dtype for the current hardware.
- `load_in_4bit=True`: cuts VRAM use for QLoRA-style workflows.
- `token=...`: needed when the base model is gated on Hugging Face.

## Attach LoRA Adapters

After loading the base model, the maintainer workflow adds adapters with `get_peft_model`:

```python
model = FastLanguageModel.get_peft_model(
    model,
    r=16,
    target_modules=[
        "q_proj",
        "k_proj",
        "v_proj",
        "o_proj",
        "gate_proj",
        "up_proj",
        "down_proj",
    ],
    lora_alpha=16,
    lora_dropout=0,
    bias="none",
    use_gradient_checkpointing="unsloth",
)
```

Practical defaults from the official example:

- `r=16` and `lora_alpha=16` are the starter values shown by the maintainer.
- `lora_dropout=0` and `bias="none"` are the optimized path used in the reference flow.
- `use_gradient_checkpointing="unsloth"` is the maintainer-recommended setting for longer contexts and tighter memory budgets.

## Supervised Fine-Tuning

The common training path uses TRL's `SFTTrainer` and `SFTConfig`:

```python
from datasets import load_dataset
from trl import SFTConfig, SFTTrainer
from unsloth import is_bfloat16_supported

dataset = load_dataset("yahma/alpaca-cleaned", split="train")

trainer = SFTTrainer(
    model=model,
    tokenizer=tokenizer,
    train_dataset=dataset,
    args=SFTConfig(
        dataset_text_field="text",
        max_seq_length=max_seq_length,
        per_device_train_batch_size=2,
        gradient_accumulation_steps=4,
        warmup_steps=5,
        max_steps=60,
        learning_rate=2e-4,
        logging_steps=1,
        optim="adamw_8bit",
        weight_decay=0.01,
        lr_scheduler_type="linear",
        fp16=not is_bfloat16_supported(),
        bf16=is_bfloat16_supported(),
        output_dir="outputs",
        seed=3407,
    ),
)

trainer.train()
```

Use this as the base pattern when you need a minimal, reproducible SFT loop. Then swap in your own dataset and trainer settings.

## Chat Datasets And Templates

For chat or instruction tuning, keep the tokenizer chat template aligned with your dataset format before training or inference.

The official docs show this pattern:

```python
from unsloth.chat_templates import get_chat_template

tokenizer = get_chat_template(
    tokenizer,
    chat_template="llama-3.1",
)
```

Then apply the template to your messages before generation:

```python
messages = [
    {"role": "user", "content": "Continue the Fibonacci sequence."},
]

inputs = tokenizer.apply_chat_template(
    messages,
    tokenize=True,
    add_generation_prompt=True,
    return_tensors="pt",
).to("cuda")
```

If you train on conversational data but only want loss on assistant outputs, the maintainer tooling also exposes `train_on_responses_only`.

## Inference

Before generation, switch the model into the inference-optimized path:

```python
FastLanguageModel.for_inference(model)

inputs = tokenizer(
    ["Continue the Fibonacci sequence: 1, 1, 2, 3, 5, 8,"],
    return_tensors="pt",
).to("cuda")

outputs = model.generate(**inputs, max_new_tokens=64)
print(tokenizer.batch_decode(outputs))
```

For instruct models, prefer `apply_chat_template(...)` over raw prompt strings so the runtime prompt matches the fine-tuning format.

## Saving And Export

The maintainer README documents three common persistence paths:

### Save LoRA adapters only

```python
model.save_pretrained("lora_model")
tokenizer.save_pretrained("lora_model")
```

### Save or push merged 16-bit weights

```python
model.save_pretrained_merged(
    "merged_model",
    tokenizer,
    save_method="merged_16bit",
)

model.push_to_hub_merged(
    "your-hf-user/merged_model",
    tokenizer,
    save_method="merged_16bit",
    token="hf_...",
)
```

### Export GGUF for `llama.cpp`

```python
model.save_pretrained_gguf(
    "model",
    tokenizer,
    quantization_method="q4_k_m",
)
```

Choose the output format based on the next runtime:

- adapters only for continued PEFT training
- merged weights for standard Hugging Face serving or conversion steps
- GGUF when targeting `llama.cpp`-compatible local inference

## Config And Auth Notes

- Hugging Face auth is not a general Unsloth API key flow. Use the `token` argument only when loading gated models or pushing artifacts to the Hugging Face Hub.
- Keep `max_seq_length` explicit in code instead of relying on hidden defaults.
- Decide early whether you want 4-bit loading. It affects VRAM usage, training strategy, and downstream export choices.
- Use `is_bfloat16_supported()` from Unsloth when copying the official trainer setup so BF16 and FP16 follow the current GPU capability.

## Common Pitfalls

- Native Windows is not the recommended path. The official FAQ points Windows users to WSL.
- Do not mix raw string prompting and chat-template prompting for the same instruct model. Keep the dataset template and inference template consistent.
- Many examples online assume a plain `transformers` or PEFT flow. When using Unsloth helpers such as `FastLanguageModel`, follow the maintainer method names and argument shapes rather than mixing in third-party wrappers.
- Newer official docs pages sometimes demonstrate broader abstractions such as `FastModel`, while the maintainer README still uses `FastLanguageModel` for text LLM workflows. For text fine-tuning on this package version, the README flow is the safest baseline.
- Export format matters. `save_pretrained`, `save_pretrained_merged`, and `save_pretrained_gguf` produce different artifacts for different runtimes.

## Version-Sensitive Notes For `2026.3.4`

- The package version is date-based, not semver-like. Pin the exact PyPI version when you need reproducible trainer behavior.
- PyPI metadata for `2026.3.4` declares Python support from `3.9` up to but excluding `3.14`.
- The official docs and README cover a broad surface area, including text, vision, TTS, and RL workflows. Start with the text-model loading and SFT path above unless your task clearly needs a different trainer family.

## Official Sources

- Docs root: `https://docs.unsloth.ai/`
- FAQ: `https://docs.unsloth.ai/basics/troubleshooting-and-faqs`
- Chat templates guide: `https://docs.unsloth.ai/basics/chat-templates`
- PyPI package page: `https://pypi.org/project/unsloth/`
- PyPI metadata JSON: `https://pypi.org/pypi/unsloth/json`
- Maintainer README: `https://github.com/unslothai/unsloth`
