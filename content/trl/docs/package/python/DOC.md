---
name: package
description: "Hugging Face TRL for Python post-training workflows: SFT, preference optimization, reward modeling, and RL trainers"
metadata:
  languages: "python"
  versions: "0.29.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "trl,huggingface,rlhf,alignment,fine-tuning,llm,training"
---

# trl Python Package Guide

## What It Is

`trl` is Hugging Face's training library for LLM post-training workflows in Python. It sits on top of the Hugging Face stack (`transformers`, `datasets`, `accelerate`, optional `peft`) and gives you trainer classes and CLIs for:

- supervised fine-tuning with `SFTTrainer`
- preference optimization with trainers such as `DPOTrainer`
- reward modeling
- RL-style trainers such as GRPO, PPO, RLOO, and KTO

Use it when you already have a Transformers-compatible model and want training loops that match current alignment and post-training patterns.

## Installation

Pin to the version you are documenting or reproducing:

```bash
pip install "trl==0.29.0"
```

Common surrounding packages in real projects:

```bash
pip install "transformers" "datasets" "accelerate"
```

If you use parameter-efficient tuning or quantization, install the matching extras in your project environment as well, for example `peft` and hardware-specific dependencies.

PyPI classifiers for `0.29.0` indicate Python `3.10` through `3.14`.

## Setup Checklist

Before writing code, confirm these pieces:

1. A supported base model or checkpoint is available through `transformers`.
2. Your dataset shape matches the trainer you plan to use.
3. `accelerate` is configured if you need multi-GPU or distributed runs.
4. Any optional integrations you rely on, such as PEFT or experiment tracking, are installed and configured in the environment.

For distributed training, TRL follows the Hugging Face `accelerate` workflow:

```bash
accelerate config
```

## Trainer Picker

- Use `SFTTrainer` for instruction tuning or standard supervised fine-tuning.
- Use `DPOTrainer` when your dataset contains paired preference examples such as `chosen` and `rejected`.
- Use reward trainers when you are fitting a reward model from preference data.
- Use GRPO, PPO, RLOO, or KTO only when you specifically need RL-style post-training and understand the data and rollout requirements.

If the task is ordinary instruction tuning, start with `SFTTrainer`.

## Minimal SFT Example

This is the safest starting point for most coding tasks:

```python
from datasets import load_dataset
from trl import SFTConfig, SFTTrainer

dataset = load_dataset("trl-lib/Capybara", split="train")

trainer = SFTTrainer(
    model="Qwen/Qwen2-0.5B-Instruct",
    train_dataset=dataset,
    args=SFTConfig(
        output_dir="qwen2-sft",
    ),
)

trainer.train()
trainer.save_model()
```

What this assumes:

- the model identifier can be loaded by `transformers`
- the dataset format is accepted by `SFTTrainer`
- your runtime has the right CUDA, PyTorch, and accelerator setup if you are training on GPU

## Common Dataset Shapes

TRL documents several supported dataset formats. The ones that matter most in practice are:

- plain text examples for language modeling
- prompt/completion style examples
- conversational examples using message arrays
- preference datasets with `prompt`, `chosen`, and `rejected`
- tool-calling datasets that include tool schemas and tool interactions

For coding agents, the main rule is: match your dataset columns to the trainer's expected format instead of forcing one trainer to accept another trainer's schema.

Examples:

- `SFTTrainer`: use text, prompt/completion, or conversational records
- `DPOTrainer`: use paired preference records
- reward training: use labeled preference data appropriate for a reward model

## Conversational Training Notes

If you are fine-tuning an instruction or chat model:

- prefer chat-capable base models
- keep dataset examples in the conversational structure the tokenizer expects
- ensure the tokenizer's chat template is compatible with your data

If generations look malformed, inspect the rendered prompt after chat templating before changing training code.

## CLI Workflows

TRL ships command-line entry points for common training flows. The docs list commands including:

- `trl sft`
- `trl dpo`
- `trl grpo`
- `trl reward`
- `trl kto`
- `trl rloo`
- `trl env`
- `trl vllm-serve`

Start with:

```bash
trl sft --help
```

Use the CLI when you want a reproducible config-driven run. Use the Python API when you need custom callbacks, preprocessing, or trainer composition inside your codebase.

## Distributed and Large-Model Training

TRL relies on the Hugging Face training stack for scaling:

- `accelerate` for launching distributed runs
- `transformers` training arguments and model loading
- optional PEFT adapters for lighter-weight fine-tuning
- optional vLLM integrations for some workflows

Practical rule: if the launch strategy, mixed precision mode, or sharding approach is unclear, resolve that in `accelerate` and `transformers` first. TRL usually layers on top of those systems rather than replacing them.

## Auth and Configuration

TRL does not introduce a separate package-specific authentication model. In practice:

- public Hub models and datasets can often be loaded directly by name
- private Hugging Face Hub assets require the usual Hugging Face authentication available to the process
- experiment tracking integrations, if enabled, require their own environment setup

Treat auth as coming from the surrounding Hugging Face and tooling ecosystem, not from `trl` itself.

Useful environment and setup checks:

- confirm the process can read the model and dataset you reference
- confirm output directories are writable
- confirm tracker environment variables are present before enabling reporting
- confirm `accelerate` config matches the hardware you intend to use

## Saving and Reusing a Trained Model

After training, save the output and load it through the normal Transformers APIs:

```python
from transformers import AutoModelForCausalLM, AutoTokenizer

model = AutoModelForCausalLM.from_pretrained("qwen2-sft")
tokenizer = AutoTokenizer.from_pretrained("qwen2-sft")
```

For chat models, apply the tokenizer's chat template during inference instead of manually concatenating role prefixes unless you know the model format exactly.

## Common Pitfalls

- Installing `trl` without aligning the surrounding `transformers`, `datasets`, `accelerate`, and PyTorch environment.
- Using the wrong trainer for the dataset shape.
- Copying examples from `main` docs into a `0.29.0` environment without checking for API drift.
- Passing conversational data without validating the chat template the tokenizer uses.
- Treating `trl` as a complete training platform when launch behavior still depends on `accelerate` and `transformers`.
- Expecting preference or RL trainers to work with ordinary SFT text datasets.
- Forgetting that large-model workflows may also require PEFT, quantization, or launcher configuration outside TRL.

For vision-language training, the official docs call out a specific gotcha: set `max_length=None` in `SFTConfig` to avoid truncating image tokens.

## Version-Sensitive Notes For 0.29.0

- The Hugging Face docs expose both `main` and versioned documentation. For `trl==0.29.0`, prefer the `v0.29.0` docs when available.
- Trainer names and configuration fields can drift across releases. Do not assume examples from older blog posts still match `0.29.0`.
- CLI coverage is broad in `0.29.0`, but command options can change across releases. Check `--help` against the installed package before scripting runs.

## Recommended Workflow For Agents

1. Pin `trl` to the project's expected version.
2. Choose the trainer from the dataset shape, not from the model family.
3. Validate model loading and tokenization with a tiny sample before starting a long run.
4. Resolve `accelerate`, PEFT, quantization, and tracker setup outside the training loop first.
5. Save and reload the trained output with standard Transformers APIs.

## Official Sources Used

- TRL docs root: https://huggingface.co/docs/trl/
- TRL installation docs: https://huggingface.co/docs/trl/installation
- TRL SFT trainer docs: https://huggingface.co/docs/trl/sft_trainer
- TRL dataset formats docs: https://huggingface.co/docs/trl/dataset_formats
- TRL distributed training docs: https://huggingface.co/docs/trl/distributing_training
- TRL CLI docs: https://huggingface.co/docs/trl/clis
- PyPI package page: https://pypi.org/project/trl/
