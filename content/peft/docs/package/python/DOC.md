---
name: package
description: "PEFT Python package guide for parameter-efficient fine-tuning, adapter loading, and adapter-based inference with Hugging Face models"
metadata:
  languages: "python"
  versions: "0.18.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "peft,huggingface,transformers,lora,adapters,fine-tuning,ml"
---

# PEFT Python Package Guide

## Golden Rule

Use `peft` to add, train, save, and load adapters on top of a separately loaded base model. Do not treat a PEFT adapter as a standalone model checkpoint unless you explicitly merge it back into the base model. As of March 12, 2026, PyPI publishes `peft 0.18.1`, while the Hugging Face stable docs are still labeled `v0.18.0`; use the stable docs for API shape and the `0.18.1` release notes for patch-level drift.

## Install

Pin the package version your project expects:

```bash
python -m pip install "peft==0.18.1"
```

Typical LLM fine-tuning stack:

```bash
python -m pip install "peft==0.18.1" transformers accelerate torch
```

If you are doing 4-bit or 8-bit adapter training, add quantization support explicitly:

```bash
python -m pip install "peft==0.18.1" transformers accelerate torch bitsandbytes
```

Notes:

- The PEFT install docs still say the library is tested on Python 3.9+, but PyPI metadata for `0.18.1` requires Python 3.10 or newer. For installation constraints, prefer PyPI.
- PEFT examples often assume recent `transformers`, `accelerate`, and `torch` versions. If an example fails unexpectedly, check dependency drift before changing the PEFT code.

## Initialize A LoRA Adapter

The common workflow is:

1. Load the base model with `transformers`
2. Define a PEFT config such as `LoraConfig`
3. Wrap the model with `get_peft_model`
4. Train only the adapter weights
5. Save the adapter with `save_pretrained`

```python
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import LoraConfig, TaskType, get_peft_model

model_id = "Qwen/Qwen2.5-3B-Instruct"

tokenizer = AutoTokenizer.from_pretrained(model_id)
base_model = AutoModelForCausalLM.from_pretrained(
    model_id,
    device_map="auto",
)

peft_config = LoraConfig(
    task_type=TaskType.CAUSAL_LM,
    inference_mode=False,
    r=16,
    lora_alpha=32,
    lora_dropout=0.05,
    target_modules=["q_proj", "v_proj"],
)

model = get_peft_model(base_model, peft_config)
model.print_trainable_parameters()
```

Notes:

- `target_modules` is model-architecture specific. Reuse a known-good example for the exact backbone you are adapting instead of assuming `q_proj` and `v_proj` exist everywhere.
- `print_trainable_parameters()` is the fastest sanity check that PEFT actually froze the base model and only exposed adapter parameters for training.

## Train And Save The Adapter

After wrapping the model, train it with your normal training loop or `transformers.Trainer`. Save the adapter artifacts when training completes:

```python
model.save_pretrained("artifacts/my-lora-adapter")
tokenizer.save_pretrained("artifacts/my-lora-adapter")
```

What gets saved:

- PEFT config
- Adapter weights
- Optional tokenizer files if you save them separately

What does not get saved by default:

- The full base model weights

That distinction matters when you reload the adapter later.

## Load An Existing Adapter For Inference

The explicit pattern is to load the base model first, then attach the adapter:

```python
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import PeftModel

base_model_id = "Qwen/Qwen2.5-3B-Instruct"
adapter_path = "artifacts/my-lora-adapter"

tokenizer = AutoTokenizer.from_pretrained(base_model_id)
base_model = AutoModelForCausalLM.from_pretrained(
    base_model_id,
    device_map="auto",
)

model = PeftModel.from_pretrained(base_model, adapter_path)
model.eval()

inputs = tokenizer("Write a haiku about adapters.", return_tensors="pt").to(model.device)
outputs = model.generate(**inputs, max_new_tokens=40)
print(tokenizer.decode(outputs[0], skip_special_tokens=True))
```

If the adapter repository already contains the PEFT config that points at its base model, `AutoPeftModelForCausalLM` is the shortest load path:

```python
from transformers import AutoTokenizer
from peft import AutoPeftModelForCausalLM

adapter_repo = "ybelkada/opt-350m-lora"

model = AutoPeftModelForCausalLM.from_pretrained(
    adapter_repo,
    device_map="auto",
)
tokenizer = AutoTokenizer.from_pretrained("facebook/opt-350m")
```

Use `PeftModel.from_pretrained(...)` when your code already manages the base model lifecycle. Use `AutoPeftModel...` when you want PEFT to reconstruct the adapted model from the adapter repository metadata. In either case, keep the tokenizer aligned with the same base model family used during fine-tuning.

## Merge The Adapter Back Into The Base Model

If you need a single merged checkpoint for deployment or export, merge the adapter and unload the PEFT wrappers:

```python
from transformers import AutoModelForCausalLM
from peft import PeftModel

base_model = AutoModelForCausalLM.from_pretrained("Qwen/Qwen2.5-3B-Instruct")
model = PeftModel.from_pretrained(base_model, "artifacts/my-lora-adapter")

merged_model = model.merge_and_unload(safe_merge=True)
merged_model.save_pretrained("artifacts/merged-model")
```

Use merging deliberately:

- It removes the adapter indirection
- It is useful for export or serving systems that expect a normal `transformers` model
- It gives up the flexibility of swapping adapters at runtime

## Hub Authentication And Runtime Configuration

PEFT commonly loads both base models and adapters from the Hugging Face Hub. For private or gated repositories, authenticate before calling `from_pretrained(...)`:

```bash
# In CI or containers
export HF_TOKEN=hf_your_token_here

# Or in local development
hf auth login
```

Useful environment variables:

- `HF_TOKEN`: access token for private or gated Hub assets
- `HF_HOME`: root directory for Hugging Face local state
- `HF_HUB_CACHE`: explicit Hub cache directory

Practical guidance:

- In local development, `hf auth login` is usually enough.
- In CI or containers, set `HF_TOKEN` through your secret manager instead of hard-coding credentials.
- If a model or adapter is gated, authenticate before debugging PEFT itself. Many apparent load failures are Hub permission failures.

## Common Pitfalls

- `save_pretrained()` saves adapter artifacts, not a standalone base model checkpoint. Reload the compatible base model before attaching the adapter again.
- Use `low_cpu_mem_usage=True` for adapter loading paths such as `from_pretrained()` and `load_adapter()`, not as a substitute for correctly constructing a fresh trainable adapter.
- If you added tokens or resized embeddings during fine-tuning, recreate the same tokenizer and embedding shape before loading the adapter. The PEFT troubleshooting guide calls out embedding mismatches and `save_embedding_layers` as common fixes.
- Mixed-precision training can break if trainable parameters stay in `float16`. PEFT 0.12.0 and newer automatically promotes adapter weights to `float32`; only disable that behavior if you understand the tradeoff.
- Multi-package version drift is common. If examples break, check the exact installed versions of `peft`, `transformers`, `accelerate`, and `torch` before rewriting the code path.
- `set_adapter()` makes the activated adapter trainable unless you set `inference_mode=True`. That matters when you load multiple adapters and expect inference-only behavior.
- The stable docs are still versioned as `v0.18.0`. For `0.18.1`, use the same API docs but also read the `v0.18.1` release notes for patch fixes.

## Version-Sensitive Notes For 0.18.x

- `0.18.0` dropped Python 3.9 support and added compatibility work for `transformers v5`.
- `0.18.1` is a patch release on top of that line. The official release notes call out fixes for upcoming `transformers v5` special cases, an AMD ROCm issue in `BoneLayer`, and a regression that incorrectly required `transformers>=4.52`.
- Because the docs site stable branch is still `v0.18.0`, patch-level behavior that changed in `0.18.1` may only be visible in release notes or source history.

## Official Sources

- PEFT docs root: `https://huggingface.co/docs/peft/index`
- PEFT installation guide: `https://huggingface.co/docs/peft/installation`
- PEFT quicktour: `https://huggingface.co/docs/peft/quicktour`
- PEFT troubleshooting: `https://huggingface.co/docs/peft/developer_guides/troubleshooting`
- PEFT model merging guide: `https://huggingface.co/docs/peft/developer_guides/model_merging`
- PEFT package reference for model loading: `https://huggingface.co/docs/peft/package_reference/auto_class`
- Hugging Face Hub environment variables: `https://huggingface.co/docs/huggingface_hub/package_reference/environment_variables`
- Hugging Face Hub authentication: `https://huggingface.co/docs/huggingface_hub/package_reference/authentication`
- Hugging Face Hub CLI auth command: `https://huggingface.co/docs/huggingface_hub/package_reference/cli`
- PyPI package page: `https://pypi.org/project/peft/`
- PEFT `v0.18.0` release notes: `https://github.com/huggingface/peft/releases/tag/v0.18.0`
- PEFT `v0.18.1` release notes: `https://github.com/huggingface/peft/releases/tag/v0.18.1`
