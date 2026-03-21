---
name: package
description: "Apache Beam Python SDK for building batch and streaming pipelines locally or on distributed runners"
metadata:
  languages: "python"
  versions: "2.71.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "apache-beam,beam,python,data-processing,etl,streaming,batch,dataflow"
---

# Apache Beam Python Package Guide

## Golden Rule

Use `apache-beam` to define the pipeline and make the runner, extras, and remote dependency packaging explicit. Most Beam failures are not in the transforms themselves; they come from using the wrong extra (`[gcp]`, `[aws]`, `[azure]`, `[dataframe]`, `[yaml]`), assuming local dependencies exist on remote workers, or submitting a job without the runner-specific options and credentials it needs.

## Install

Pin the Beam version your project expects:

```bash
python -m pip install "apache-beam==2.71.0"
```

Common extras:

```bash
python -m pip install "apache-beam[gcp]==2.71.0"
python -m pip install "apache-beam[aws]==2.71.0"
python -m pip install "apache-beam[azure]==2.71.0"
python -m pip install "apache-beam[dataframe]==2.71.0"
python -m pip install "apache-beam[yaml]==2.71.0"
```

Use a virtual environment so Beam and connector dependencies do not fight with unrelated project packages:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "apache-beam==2.71.0"
```

## Initialize A Pipeline

Start with a local pipeline on `DirectRunner` before moving to a remote runner:

```python
import apache_beam as beam
from apache_beam.options.pipeline_options import PipelineOptions

options = PipelineOptions(
    runner="DirectRunner",
)

with beam.Pipeline(options=options) as pipeline:
    (
        pipeline
        | "Create" >> beam.Create(["to be or not to be", "beam is portable"])
        | "Split" >> beam.FlatMap(str.split)
        | "PairWithOne" >> beam.Map(lambda word: (word, 1))
        | "Count" >> beam.CombinePerKey(sum)
        | "Print" >> beam.Map(print)
    )
```

Beam structure to keep straight:

- `PCollection`: the data flowing through the graph
- `PTransform`: operations like `Map`, `FlatMap`, `ParDo`, `CombinePerKey`, `ReadFromText`, `WriteToText`
- `Pipeline`: the graph and execution context
- `PipelineOptions`: runner and environment configuration

Use `with beam.Pipeline(...) as pipeline:` for simple cases; it runs the pipeline automatically when the block exits.

## Core Usage Patterns

### Local file I/O

```python
import apache_beam as beam

with beam.Pipeline() as pipeline:
    (
        pipeline
        | "ReadLines" >> beam.io.ReadFromText("input.txt")
        | "Uppercase" >> beam.Map(str.upper)
        | "WriteLines" >> beam.io.WriteToText("output/result")
    )
```

### Explicit pipeline options from CLI arguments

For real jobs, let Beam parse pipeline flags instead of hard-coding every option:

```python
import apache_beam as beam
from apache_beam.options.pipeline_options import PipelineOptions

pipeline_options = PipelineOptions()

with beam.Pipeline(options=pipeline_options) as pipeline:
    (
        pipeline
        | beam.Create([1, 2, 3, 4])
        | beam.Map(lambda x: x * 10)
        | beam.Map(print)
    )
```

Then run it with flags such as:

```bash
python my_pipeline.py --runner=DirectRunner
```

## Runner Configuration And Auth

### DirectRunner

Use `DirectRunner` for local development, model validation, and tests. It intentionally checks things that are easy to get wrong, including element encodability, function serializability, and assumptions about ordering.

Do not treat a successful `DirectRunner` run as proof that production configuration is correct. Remote runners still need their own options, dependencies, and credentials.

### DataflowRunner

For Google Cloud Dataflow, install the GCP extra and provide the runner-specific options:

```bash
python -m pip install "apache-beam[gcp]==2.71.0"
```

Typical launch command:

```bash
python my_pipeline.py \
  --runner=DataflowRunner \
  --project=my-gcp-project \
  --region=us-central1 \
  --temp_location=gs://my-bucket/tmp \
  --staging_location=gs://my-bucket/staging
```

Practical requirements for Dataflow:

- authenticate with Google Cloud before job submission
- enable the required Google Cloud APIs
- use a valid GCS bucket for `temp_location`
- set `staging_location` explicitly when you want deterministic staging behavior
- set `--streaming` for unbounded pipelines

### Other cloud I/O

Beam itself does not have one global auth mechanism. Authentication depends on the runner and the I/O connectors you use:

- Google Cloud connectors: install `apache-beam[gcp]`
- AWS connectors: install `apache-beam[aws]`
- Azure connectors: install `apache-beam[azure]`

Make sure the credentials are available in the same environment that executes the worker process, not just on your laptop at submission time.

## Managing Dependencies For Remote Workers

Local imports are not automatically available on remote workers. Use the packaging mode that matches the pipeline shape.

### Single-file pipeline with public PyPI dependencies

Use a trimmed `requirements.txt` and pass it with `--requirements_file`:

```text
apache-beam[gcp]==2.71.0
orjson==3.11.1
```

```bash
python my_pipeline.py \
  --runner=DataflowRunner \
  --requirements_file requirements.txt
```

Beam stages the requirements cache at submission time. This is the simplest path when the code is mostly one Python entrypoint plus public packages.

### Multi-file pipeline package

If the pipeline spans multiple modules, package it and pass `--setup_file`:

```python
# setup.py
import setuptools

setuptools.setup(
    name="my-beam-job",
    version="0.1.0",
    install_requires=[
        "apache-beam[gcp]==2.71.0",
    ],
    packages=setuptools.find_packages(),
)
```

```bash
pip install -e .
python my_pipeline.py --setup_file ./setup.py
```

This stages your package, but package dependencies may still be installed from PyPI at runtime unless they are already present in the worker image.

### Non-Python system dependencies

If a dependency needs OS packages or slower runtime setup, prefer a custom container. Beam’s dependency guide explicitly recommends custom containers for non-Python dependencies and for avoiding repeated worker startup installs.

## Common Pitfalls

- Base `apache-beam` no longer implies every optional connector dependency. Install the extra you actually need.
- Remote workers do not inherit the full state of your local environment. If imports work locally and fail remotely, check `--requirements_file`, `--setup_file`, or your custom container first.
- `DirectRunner` runs locally and validates Beam-model semantics, but it does not replace real testing on the production runner.
- Beam may process elements in arbitrary order. Do not write transforms that depend on stable in-memory ordering unless the transform contract guarantees it.
- `--save_main_session` is not a universal fix. It is mainly relevant when using `dill`-based serialization or code living in `__main__`; older blog posts often add it blindly.
- Submission and runtime environments must agree on serialization-related dependencies. Version mismatches can show up as unpickling or coder errors.
- For Dataflow, a missing or non-`gs://` temp location is a common launch-time failure.
- Streaming jobs do not finish on their own. They must be cancelled explicitly.

## Version-Sensitive Notes For 2.71.0

- PyPI lists `2.71.0` as the current release for `apache-beam`, published on January 22, 2026.
- `apache-beam 2.71.0` requires Python `>=3.10`. PyPI classifiers list support for Python `3.10`, `3.11`, `3.12`, and `3.13`.
- Beam `2.69.0` added official Python 3.13 support. If you see older Beam pages or articles that stop at Python 3.12, prefer PyPI and current release notes.
- Beam `2.70.0` split some Python dependencies into extras. If an older tutorial assumes Dataflow, YAML, or cloud connector packages are included in the base install, translate it into the explicit extras form for `2.71.0`.
- Beam `2.65.0` switched the default pickler to `cloudpickle`; Beam `2.71.0` includes a Python bugfix so logical type and coder registries are saved correctly with the default pickler. Serialization advice from older pre-`2.65` examples is often outdated.

## Official Links

- Apache Beam docs root: `https://beam.apache.org`
- Python SDK overview: `https://beam.apache.org/documentation/sdks/python/`
- Python quickstart: `https://beam.apache.org/get-started/quickstart-py/`
- Managing Python dependencies: `https://beam.apache.org/documentation/sdks/python-pipeline-dependencies/`
- DirectRunner docs: `https://beam.apache.org/documentation/runners/direct/`
- Dataflow runner docs: `https://beam.apache.org/documentation/runners/dataflow/`
- PyPI package page: `https://pypi.org/project/apache-beam/`
