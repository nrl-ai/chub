---
name: package
description: "Flax for Python on JAX with NNX modules, Linen interop, training, randomness, and checkpointing guidance"
metadata:
  languages: "python"
  versions: "0.12.5"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flax,jax,nnx,linen,deep-learning,neural-networks"
---

# Flax Python Package Guide

## Golden Rule

Use `from flax import nnx` for new Flax code, keep `from flax import linen as nn` only for existing Linen-style projects, and treat JAX installation as a separate platform decision. `flax==0.12.5` is the current PyPI release as of March 12, 2026, but GPU and TPU projects should install the correct JAX build first instead of assuming the default `pip install flax` path is enough.

## Install

For CPU-only local development:

```bash
python -m pip install "flax==0.12.5"
```

For projects that train models, checkpoint state, or follow the official tutorials, install companion libraries explicitly:

```bash
python -m pip install "flax==0.12.5" optax orbax-checkpoint
```

Important install note:

- `flax` depends on JAX, but accelerator-specific JAX installation is not a Flax concern. For CUDA, ROCm, or TPU environments, install the correct JAX build first using the JAX installation matrix, then install `flax`.

## Choose The Right API Surface

### NNX for new code

Use NNX when you are starting fresh. NNX modules are Python objects that hold state directly, work with `nnx.Module`, and use explicit RNG containers like `nnx.Rngs(...)`.

### Linen for existing codebases

Use Linen when the project already relies on `init/apply`, `@nn.compact`, `TrainState`, or older tutorials. Linen is still supported, but the main docs site now prioritizes NNX and keeps Linen material in the legacy docs site plus the NNX bridge guide.

## Initialize A Model With NNX

NNX modules create parameter state at construction time:

```python
import jax.numpy as jnp
from flax import nnx

class MLP(nnx.Module):
    def __init__(self, din: int, hidden: int, dout: int, *, rngs: nnx.Rngs):
        self.linear1 = nnx.Linear(din, hidden, rngs=rngs)
        self.linear2 = nnx.Linear(hidden, dout, rngs=rngs)

    def __call__(self, x):
        x = self.linear1(x)
        x = nnx.relu(x)
        return self.linear2(x)

model = MLP(din=784, hidden=256, dout=10, rngs=nnx.Rngs(0))
batch = jnp.ones((32, 784))
logits = model(batch)
print(logits.shape)
```

What changes compared with Linen:

- NNX does not start from `variables = model.init(...)`
- The model instance already owns parameter state
- You mutate or update stateful objects through NNX transforms and utilities instead of passing a variables tree into `apply`

## Training With Optax

`optax` is the standard optimizer library used in Flax examples. In NNX, attach the optimizer to the model and update parameters through gradients:

```python
import jax.numpy as jnp
import optax
from flax import nnx

class Regressor(nnx.Module):
    def __init__(self, *, rngs: nnx.Rngs):
        self.linear = nnx.Linear(4, 1, rngs=rngs)

    def __call__(self, x):
        return self.linear(x)

model = Regressor(rngs=nnx.Rngs(0))
optimizer = nnx.Optimizer(model, optax.adam(1e-3), wrt=nnx.Param)

@nnx.jit
def train_step(model, optimizer, x, y):
    def loss_fn(model):
        preds = model(x)
        return jnp.mean((preds - y) ** 2)

    loss, grads = nnx.value_and_grad(loss_fn)(model)
    optimizer.update(model, grads)
    return loss

x = jnp.ones((8, 4))
y = jnp.ones((8, 1))
loss = train_step(model, optimizer, x, y)
print(loss)
```

Use `flax.training.train_state.TrainState` only when you are staying in Linen-style training loops.

## Randomness And Configuration

Flax is a local library, so there is no service authentication step. The important runtime configuration is how you manage JAX and RNG state.

Named RNG streams are the normal NNX pattern:

```python
import jax.numpy as jnp
from flax import nnx

dropout = nnx.Dropout(rate=0.5)
rngs = nnx.Rngs(params=0, dropout=1)

output = dropout(jnp.ones((4, 4)), rngs=rngs)
```

Practical rules:

- Use separate named streams such as `params`, `dropout`, and any data-augmentation stream your code needs.
- Do not silently reuse the same RNG stream everywhere. Reused dropout keys are a common source of suspiciously deterministic behavior.
- When batching or vectorizing stochastic code, follow the randomness guide and fork streams instead of manually juggling raw JAX keys everywhere.
- Install JAX for the correct platform before debugging Flax errors. Many apparent Flax failures are really mismatched `jax` or `jaxlib` installs.

## Checkpointing

The current Flax docs point to Orbax for checkpointing. For NNX modules, save and restore state objects, then update the live module:

```python
from flax import nnx
import orbax.checkpoint as ocp

state = nnx.state(model)
checkpointer = ocp.StandardCheckpointer()
checkpointer.save("/tmp/flax-demo", state, force=True)

abstract_model = nnx.eval_shape(lambda: Regressor(rngs=nnx.Rngs(0)))
abstract_state = nnx.state(abstract_model)
restored_state = checkpointer.restore("/tmp/flax-demo", abstract_state)
nnx.update(model, restored_state)
```

Checkpointing rules that matter:

- Prefer Orbax APIs from current docs instead of older `flax.training.checkpoints` examples.
- Restore against an abstract target tree when shapes or sharding matter.
- Keep checkpoint structure stable across refactors or write explicit migration code.

## Linen Interop And Migration

If you are migrating an existing Linen project, use the NNX bridge guide instead of rewriting everything at once.

Useful mental model:

- Linen: functional `init/apply`, variables collections, `@nn.compact`
- NNX: object-style modules with explicit state and RNG containers
- Bridge tools exist so you can convert or wrap pieces gradually instead of doing a flag day rewrite

When reading external examples:

- If the example imports `from flax import linen as nn`, it is not written in the current NNX style
- If the example uses `TrainState`, `model.init`, or `variables["params"]`, it is a Linen example
- Do not paste Linen examples into an NNX training loop without translating the state model

## Common Pitfalls

- Treating `flax` as a standalone deep-learning runtime. Flax sits on top of JAX, so the JAX install and backend still determine whether code runs on CPU, GPU, or TPU.
- Mixing NNX and Linen idioms in the same function without understanding the bridge boundary.
- Assuming tutorial dependencies are bundled with `flax`. Training and checkpoint examples commonly also need `optax`, `orbax-checkpoint`, and dataset libraries.
- Copying older blog posts that still target pre-NNX Flax APIs.
- Restoring checkpoints directly into a changed model structure without checking shapes, names, or collection layout.

## Version-Sensitive Notes For 0.12.5

- PyPI lists `flax 0.12.5` as the latest release and marks `Requires: Python >=3.11`.
- The latest docs site is now NNX-first, while legacy Linen docs live at `flax-linen.readthedocs.io`.
- The PyPI project description still contains older prose that says Flax works on Python 3.8 or later. For `0.12.5`, trust the PyPI package metadata instead of the stale prose.
- If you are starting from an older Flax tutorial, verify whether it predates the NNX transition before copying module and training patterns.

## Official Links

- NNX docs root: `https://flax.readthedocs.io/en/latest/`
- NNX basics and tutorials: `https://flax.readthedocs.io/en/latest/basics.html`
- Randomness guide: `https://flax.readthedocs.io/en/latest/guides/randomness.html`
- Checkpointing guide: `https://flax.readthedocs.io/en/latest/guides/checkpointing.html`
- NNX bridge guide: `https://flax.readthedocs.io/en/latest/guides/bridge_guide.html`
- NNX optimizer reference: `https://flax.readthedocs.io/en/latest/api_reference/flax.nnx/training/optimizer.html`
- Legacy Linen docs: `https://flax-linen.readthedocs.io/en/latest/`
- PyPI package: `https://pypi.org/project/flax/`
