---
name: bigquery
description: "Google Cloud BigQuery Python client for queries, table loads, and dataset setup"
metadata:
  languages: "python"
  versions: "3.40.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,bigquery,google-cloud,gcp,sql,analytics,data-warehouse,python"
---

# google-cloud-bigquery Python Package Guide

## Golden Rule

Use the official `google-cloud-bigquery` package with `from google.cloud import bigquery`, and authenticate with Google Cloud Application Default Credentials (ADC).

For predictable behavior, pass `project=` explicitly and set `location=` when your datasets or jobs are region-specific. BigQuery client code should not rely on API keys.

## Install

Pin the package version you want to reason about:

```bash
python -m pip install "google-cloud-bigquery==3.40.1"
```

If you want pandas helpers such as `to_dataframe()` and `load_table_from_dataframe()`:

```bash
python -m pip install "google-cloud-bigquery[pandas]==3.40.1"
```

Other common package managers:

```bash
uv add "google-cloud-bigquery==3.40.1"
poetry add "google-cloud-bigquery==3.40.1"
```

## Authentication And Setup

Recommended local setup:

```bash
gcloud auth application-default login
export GOOGLE_CLOUD_PROJECT="your-project-id"
```

Service account key fallback:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
export GOOGLE_CLOUD_PROJECT="your-project-id"
```

Practical setup notes:

- Use ADC for local development, CI, and deployed Google Cloud workloads when possible.
- `GOOGLE_CLOUD_PROJECT` helps when project discovery is ambiguous or missing.
- Pick a `location` such as `US`, `EU`, or a regional location that matches the datasets and jobs you intend to use.

## Initialize A Client

Typical client initialization:

```python
import os

from google.cloud import bigquery

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]

client = bigquery.Client(
    project=project_id,
    location="US",
)
```

Explicit service account credentials:

```python
from google.cloud import bigquery
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)

client = bigquery.Client(
    project="your-project-id",
    credentials=credentials,
    location="US",
)
```

## Common Workflows

### Run A Parameterized Query

Use `QueryJobConfig` and query parameters instead of string interpolation.

```python
from google.cloud import bigquery

client = bigquery.Client(project="your-project-id", location="US")

sql = """
SELECT name, SUM(number) AS total
FROM `bigquery-public-data.usa_names.usa_1910_2013`
WHERE state = @state
GROUP BY name
ORDER BY total DESC
LIMIT @limit
"""

job_config = bigquery.QueryJobConfig(
    query_parameters=[
        bigquery.ScalarQueryParameter("state", "STRING", "TX"),
        bigquery.ScalarQueryParameter("limit", "INT64", 10),
    ]
)

query_job = client.query(sql, job_config=job_config)

for row in query_job.result():
    print(row.name, row.total)
```

BigQuery uses standard SQL by default. Only set `use_legacy_sql=True` when you intentionally need legacy SQL behavior.

### Read Query Results Into A DataFrame

Install the pandas extra first, then convert the result iterator:

```python
from google.cloud import bigquery

client = bigquery.Client(project="your-project-id", location="US")

sql = """
SELECT name, state, year, number
FROM `bigquery-public-data.usa_names.usa_1910_2013`
WHERE state = 'CA'
LIMIT 100
"""

rows = client.query(sql).result()
df = rows.to_dataframe()

print(df.head())
```

### Create A Dataset And Table

Create the dataset in the same location you plan to query and load.

```python
from google.cloud import bigquery

client = bigquery.Client(project="your-project-id", location="US")

dataset = bigquery.Dataset("your-project-id.analytics")
dataset.location = "US"
client.create_dataset(dataset)

schema = [
    bigquery.SchemaField("user_id", "STRING", mode="REQUIRED"),
    bigquery.SchemaField("amount_cents", "INT64"),
    bigquery.SchemaField("event_ts", "TIMESTAMP"),
]

table = bigquery.Table("your-project-id.analytics.events", schema=schema)
client.create_table(table)
```

### Load A CSV File From Cloud Storage

Use this for normal batch ingestion from `gs://...` objects.

```python
from google.cloud import bigquery

client = bigquery.Client(project="your-project-id", location="US")

job_config = bigquery.LoadJobConfig(
    source_format=bigquery.SourceFormat.CSV,
    skip_leading_rows=1,
    autodetect=True,
    write_disposition=bigquery.WriteDisposition.WRITE_APPEND,
)

load_job = client.load_table_from_uri(
    "gs://my-bucket/orders.csv",
    "your-project-id.analytics.orders",
    job_config=job_config,
)

load_job.result()
table = client.get_table("your-project-id.analytics.orders")
print(table.num_rows)
```

### Load A pandas DataFrame

Use this when your data already exists in memory as a DataFrame.

```python
import pandas as pd
from google.cloud import bigquery

client = bigquery.Client(project="your-project-id", location="US")

df = pd.DataFrame(
    [
        {"user_id": "u_1", "amount_cents": 4999},
        {"user_id": "u_2", "amount_cents": 1299},
    ]
)

job_config = bigquery.LoadJobConfig(
    write_disposition=bigquery.WriteDisposition.WRITE_TRUNCATE,
)

load_job = client.load_table_from_dataframe(
    df,
    "your-project-id.analytics.daily_revenue",
    job_config=job_config,
)

load_job.result()
```

### Insert JSON Rows

Use `insert_rows_json()` when you need row-oriented inserts from application code and you want per-row error details.

```python
from google.cloud import bigquery

client = bigquery.Client(project="your-project-id", location="US")

rows = [
    {
        "user_id": "u_1",
        "amount_cents": 4999,
        "event_ts": "2026-03-12T12:00:00Z",
    },
    {
        "user_id": "u_2",
        "amount_cents": 1299,
        "event_ts": "2026-03-12T12:05:00Z",
    },
]

errors = client.insert_rows_json(
    "your-project-id.analytics.events",
    rows,
)

if errors:
    raise RuntimeError(errors)
```

## Important Notes

### Authentication Is Credential-Based, Not API-Key-Based

For normal Python client usage, use ADC or explicit service account credentials. Do not build BigQuery client initialization around API keys.

### Project Discovery Can Be Missing

If your code runs in local scripts, CI, or mixed-cloud environments, do not assume the project ID will always be inferred. Pass `project=` explicitly when correctness matters.

### Location Mismatches Cause Job Errors

Queries, loads, and table operations must use the same BigQuery location as the datasets they touch. If your dataset is in `EU`, do not create the client or run the job with `location="US"`.

### Check Insert Errors Explicitly

`insert_rows_json()` returns a list of row-level errors. Treat a non-empty return value as a failed write path.

### Prefer Parameterized Queries

Use `ScalarQueryParameter`, `ArrayQueryParameter`, and related query parameter helpers instead of building SQL with untrusted strings.

## Version Notes

- This guide targets `google-cloud-bigquery==3.40.1`.
- The maintained BigQuery Python reference for the `latest` line and the PyPI package page both pointed to `3.40.1` when this guide was updated.
- If you are upgrading older `2.x` code, check the official `3.0.0` migration guide before reusing deprecated patterns.

## Official Source URLs

- `https://cloud.google.com/python/docs/reference/bigquery/latest`
- `https://cloud.google.com/python/docs/reference/bigquery/latest/google.cloud.bigquery.client.Client`
- `https://cloud.google.com/python/docs/reference/bigquery/latest/3.0.0_migration_guide`
- `https://cloud.google.com/python/docs/reference/bigquery/latest/changelog`
- `https://cloud.google.com/docs/authentication/provide-credentials-adc`
- `https://cloud.google.com/docs/authentication/set-up-adc-local-dev-environment`
- `https://pypi.org/project/google-cloud-bigquery/`
