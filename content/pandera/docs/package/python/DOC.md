---
name: package
description: "Pandera dataframe validation library for pandas-first Python workflows, with backend support for Polars, Ibis, and PySpark"
metadata:
  languages: "python"
  versions: "0.29.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pandera,python,data-validation,pandas,polars,pyspark,ibis,dataframe"
---

# Pandera Python Package Guide

## Golden Rule

Use backend-specific Pandera imports and validate data at dataframe boundaries instead of sprinkling ad hoc assertions through the pipeline. For pandas workflows, follow the current docs and import `pandera.pandas as pa`, not the legacy top-level `import pandera as pa`.

## Install

For pandas projects, install the package with the pandas extra and pin the version your project expects:

```bash
python -m pip install "pandera[pandas]==0.29.0"
```

Common alternatives:

```bash
uv pip install "pandera[pandas]==0.29.0"
poetry add "pandera[pandas]==0.29.0"
```

Notes:

- The upstream install examples are pandas-first. If you are using a different dataframe engine, use the backend-specific docs and import path for that engine.
- The docs and PyPI README explicitly show backend-specific modules such as `pandera.pandas` and `pandera.polars`.
- If you only install `pandera` without the matching backend dependencies, validation code may import cleanly but fail later when a backend-specific dependency is missing.

## Initialize And Choose A Backend

### Pandas setup

```python
import pandas as pd
import pandera.pandas as pa
from pandera.typing import DataFrame, Series
```

### Other documented backends

Use the backend module that matches the dataframe library you are validating:

```python
import pandera.polars as pa
# or
import pandera.ibis as pa
# or
import pandera.pyspark as pa
```

Do not assume all backends expose identical behavior. Pandera’s pandas API is the most common path, but backend support is documented separately and can differ in available checks and typing behavior.

## Core Usage

### Object-based schema with `DataFrameSchema`

Use `DataFrameSchema` when you want an explicit schema object that you can reuse across validation calls.

```python
import pandas as pd
import pandera.pandas as pa

transactions_schema = pa.DataFrameSchema(
    {
        "user_id": pa.Column(int, pa.Check.ge(1)),
        "amount": pa.Column(float, pa.Check.gt(0)),
        "country": pa.Column(str),
    },
    strict=True,
    coerce=True,
)

df = pd.DataFrame(
    {
        "user_id": ["1", "2"],
        "amount": [10.5, 42.0],
        "country": ["us", "ca"],
    }
)

validated = transactions_schema.validate(df)
print(validated.dtypes)
```

Use `strict=True` when extra columns should fail validation. Use `coerce=True` when upstream data often arrives as strings or mixed types and you want Pandera to cast before checking constraints.

### Class-based schema with `DataFrameModel`

Use `DataFrameModel` when you want reusable schema definitions that also work well with function annotations.

```python
import pandera.pandas as pa
from pandera.typing import DataFrame, Series

class Transactions(pa.DataFrameModel):
    user_id: Series[int] = pa.Field(ge=1)
    amount: Series[float] = pa.Field(gt=0)
    country: Series[str]

    class Config:
        strict = True
        coerce = True

@pa.check_types
def normalize_country(df: DataFrame[Transactions]) -> DataFrame[Transactions]:
    return df.assign(country=df["country"].str.upper())
```

This is usually the cleanest pattern for ETL helpers and pipeline stages: annotate the input and output once, then let `@pa.check_types` enforce the schema at the boundary.

### Decorate untyped functions

If a function cannot use dataframe type annotations cleanly, validate with explicit decorators:

```python
import pandas as pd
import pandera.pandas as pa

input_schema = pa.DataFrameSchema(
    {"amount": pa.Column(float, pa.Check.gt(0))},
    coerce=True,
)

@pa.check_input(input_schema)
@pa.check_output(input_schema)
def keep_positive_amounts(df: pd.DataFrame) -> pd.DataFrame:
    return df.loc[df["amount"] > 0].copy()
```

Use `@pa.check_input`, `@pa.check_output`, and `@pa.check_io` when runtime validation matters more than static typing clarity.

## Error Handling And Debugging

By default, Pandera raises on the first failure. For debugging or batch diagnostics, use lazy validation so you can inspect all failures at once:

```python
import pandera.pandas as pa

try:
    transactions_schema.validate(df, lazy=True)
except pa.errors.SchemaErrors as exc:
    print(exc.failure_cases)
```

This is the right default for agent-written data cleanup code because it gives you a structured failure table instead of a single first error.

## Configuration And Runtime Controls

Pandera has no credentials or API keys. Configuration is about validation scope and runtime behavior.

Temporarily adjust validation behavior in code with `config_context`:

```python
from pandera.config import ValidationDepth, config_context

with config_context(validation_depth=ValidationDepth.SCHEMA_ONLY):
    transactions_schema.validate(df)
```

Useful controls from the official configuration docs:

- `validation_enabled`: disable validation in a controlled scope when benchmarking or bypassing checks intentionally
- `validation_depth`: reduce how much validation runs, for example schema-only checks
- `cache_dataframe`: cache dataframe state during validation when supported by the backend

Environment variables also exist for these runtime controls, including `PANDERA_VALIDATION_ENABLED` and `PANDERA_VALIDATION_DEPTH`.

## Common Pitfalls

- Do not keep using `import pandera as pa` for pandas schemas. The upstream docs treat `pandera.pandas` as the recommended import path, and the top-level path is documented as deprecated for these classes.
- Install the backend you actually use. Pandera covers more than pandas, but the backend import and dependencies must match the dataframe engine.
- `coerce=True` changes data before checks run. That is often useful, but it can hide upstream type issues if you expected a pure validation pass.
- `strict=True` rejects unexpected columns. Use it when the schema is an API contract; avoid it when your upstream source adds benign extra columns that you plan to ignore.
- `@pa.check_types` only helps if the function is annotated with Pandera dataframe types. If a function accepts plain `pd.DataFrame`, use explicit decorators or a direct `.validate(...)` call.
- Backend docs are not interchangeable. A pandas example copied into a Polars or PySpark project may fail because the import path and supported checks differ.
- For bulk debugging, prefer `lazy=True`; otherwise Pandera stops on the first failure and you lose the full error picture.

## Version-Sensitive Notes For `0.29.0`

- The version used here `0.29.0` matched live PyPI on March 12, 2026.
- The current stable docs still carry the import-path warning introduced in `0.24.0`: for pandas workflows, move code to `import pandera.pandas as pa` instead of relying on the top-level `pandera` namespace for `DataFrameSchema` and `DataFrameModel`.
- The `0.29.0` docs document `@pa.check_types` support for collection return and parameter types such as `list[...]`, `tuple[...]`, and `dict[..., ...]` containing dataframe-typed objects. This matters if your helpers split data into multiple validated frames.
- Use backend-specific docs when upgrading old examples. Much older blog posts often assume pandas-only imports from the top-level package and miss the current backend module split.
