---
name: package
description: "PySpark package guide for Python projects using Apache Spark 4.1.1"
metadata:
  languages: "python"
  versions: "4.1.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pyspark,spark,dataframe,sql,etl,distributed-computing,big-data"
---

# PySpark Python Package Guide

## Golden Rule

Use `pyspark` for Python access to Apache Spark, start from `SparkSession`, prefer DataFrame and Spark SQL APIs over low-level RDD code for new work, and keep the PySpark minor version aligned with the Spark runtime you connect to.

## Install

Pin the package version your project expects:

```bash
python -m pip install "pyspark==4.1.1"
```

Common alternatives:

```bash
uv add "pyspark==4.1.1"
poetry add "pyspark==4.1.1"
```

Useful extras from the official install guide:

```bash
python -m pip install "pyspark[sql]==4.1.1"
python -m pip install "pyspark[pandas_on_spark]==4.1.1" plotly
python -m pip install "pyspark[connect]==4.1.1"
```

Important install notes:

- PySpark on PyPI is mainly for local development or as a client to an existing cluster, not for provisioning a Spark cluster by itself.
- PySpark requires Java 17 or later and a valid `JAVA_HOME`.
- The default pip distribution uses Hadoop 3.3 and Hive 2.3.
- `PYSPARK_HADOOP_VERSION=3` is the default pip distribution choice; `without` is also supported, but that install path is explicitly marked experimental by Spark.
- Conda packages exist, but the official docs note they are maintained separately by the community and are not synchronized exactly with Spark releases.

Basic environment setup for local use:

```bash
export JAVA_HOME=/path/to/jdk-17
python -m pip install "pyspark==4.1.1"
```

## Initialize A Session

### Local or classic Spark session

Use an explicit `master` in scripts so behavior is predictable outside the `pyspark` shell:

```python
from pyspark.sql import SparkSession

spark = (
    SparkSession.builder
    .master("local[*]")
    .appName("example-job")
    .config("spark.sql.session.timeZone", "UTC")
    .getOrCreate()
)

print(spark.version)
```

The interactive `pyspark` shell creates `spark` for you automatically, but normal Python code does not.

### Spark Connect session

Use Spark Connect when your Python process should talk to a remote Spark server over `sc://...`:

```python
from pyspark.sql import SparkSession

spark = (
    SparkSession.builder
    .remote("sc://localhost:15002")
    .appName("connect-client")
    .getOrCreate()
)
```

If you switch from a local/classic session to Spark Connect in the same process, stop the existing regular session first.

### Connect-focused package variants

The install docs distinguish three relevant package choices:

- `pyspark`: full PySpark package
- `pyspark-connect`: installs `pyspark` and the Spark Connect dependencies, and supports both `spark.master` and `spark.remote`
- `pyspark-client`: pure Python Spark Connect client for remote `spark.remote` usage only; it does not rely on JARs or a local JRE

## Core Usage

### Create a DataFrame

```python
from datetime import date
from pyspark.sql import SparkSession

spark = SparkSession.builder.master("local[*]").appName("people").getOrCreate()

df = spark.createDataFrame(
    [
        (1, "Ada", date(2026, 3, 12)),
        (2, "Linus", date(2026, 3, 11)),
    ],
    schema="id long, name string, created_at date",
)

df.show()
```

PySpark can infer schema from Python rows, dictionaries, pandas DataFrames, and RDDs, but explicit schemas reduce type surprises in real jobs.

### Transform with DataFrame APIs

PySpark DataFrames are lazily evaluated: transformations build a plan, and execution starts only when you trigger an action such as `show()`, `count()`, `collect()`, or a write.

```python
from pyspark.sql import functions as F

result = (
    df
    .withColumn("name_upper", F.upper("name"))
    .filter(F.col("id") >= 1)
    .groupBy("name_upper")
    .agg(F.count("*").alias("row_count"))
)

result.show()
```

### Mix DataFrames and SQL

```python
df.createOrReplaceTempView("people")

summary = spark.sql("""
    SELECT name, COUNT(*) AS row_count
    FROM people
    GROUP BY name
    ORDER BY row_count DESC
""")

summary.show()
```

Spark SQL and DataFrame APIs share the same execution engine, so it is normal to move between them in one job.

### Read and write data

```python
events = spark.read.parquet("data/events/")

(
    events
    .filter("event_type = 'click'")
    .write
    .mode("overwrite")
    .parquet("out/click-events/")
)
```

For small local jobs, file paths can be local. For real clusters, use storage paths that are reachable from executors, not just from your laptop.

## Configuration, Packaging, And Cluster Access

Use builder config for application-local settings:

```python
from pyspark.sql import SparkSession

spark = (
    SparkSession.builder
    .master("local[*]")
    .appName("etl-job")
    .config("spark.sql.shuffle.partitions", "8")
    .config("spark.jars.packages", "group:artifact:version")
    .getOrCreate()
)
```

Useful configuration patterns from the official Spark docs:

- Use `.config(...)` or `SparkConf` for per-application settings.
- Use `spark-submit --conf key=value` for runtime-specific overrides.
- Use `spark-defaults.conf` when the same settings should apply repeatedly.
- Use `spark.jars.packages` to resolve connector JARs for the driver and executors.
- Use `spark.hadoop.*` and `spark.hive.*` properties for Hadoop and Hive-side configuration instead of mutating cluster-wide config files per application.
- Use `spark.submit.pyFiles` or `spark-submit --py-files` to distribute your own Python modules to executors.

Typical cluster submission:

```bash
spark-submit \
  --master spark://host:7077 \
  --conf spark.sql.shuffle.partitions=200 \
  --py-files dist/my_job.zip \
  jobs/my_job.py
```

Auth and access note:

- `pyspark` itself is not an auth SDK.
- Access to S3, GCS, ADLS, Hive metastore, or secured clusters is handled by Spark/Hadoop connectors, cluster configuration, and runtime credentials.
- If storage access fails, check connector JARs, `spark.hadoop.*` settings, and cluster identity before assuming the Python code is wrong.

## Testing

The official testing guide recommends using PySpark's built-in test utilities for DataFrame and schema comparisons:

```python
from pyspark.testing.utils import assertDataFrameEqual, assertSchemaEqual
```

For larger suites, use `unittest` or `pytest` and create one shared `SparkSession` per test class or fixture instead of spinning up a new session for every assertion.

## Common Pitfalls

- A pip-installed PySpark client must match the Spark cluster minor version closely. The PyPI page warns that Spark standalone clusters can produce odd errors when versions do not match.
- `collect()` and `toPandas()` bring data back to the driver. Use them only for small result sets.
- `pyspark` requires Java 17+. Missing or wrong `JAVA_HOME` is still a common local setup failure.
- Spark Connect and a regular local session cannot coexist in the same process without stopping the existing session first.
- `pyspark-client` is remote-only. If you need local mode or `master("local[*]")`, use `pyspark` or `pyspark-connect` instead.
- Python package installation does not automatically solve cluster-side connector setup. Executors still need access to the same data sources and dependencies.
- Standalone cluster `cluster` deploy mode is not supported for Python applications; use `client` mode there or use another cluster manager that supports your deployment pattern.

## Version-Sensitive Notes For 4.1.1

- Spark 4.1 drops Python 3.9 support in PySpark. Use Python 3.10+.
- Spark 4.1 raises the minimum supported PyArrow version in PySpark from `11.0.0` to `15.0.0`.
- Spark 4.1 raises the minimum supported pandas version in PySpark from `2.0.0` to `2.2.0`.
- In Spark 4.1, `BinaryType` maps to Python `bytes` by default in PySpark instead of the older mixed `bytearray` behavior.
- In Spark 4.1, `spark.sql.execution.pandas.convertToArrowArraySafely` is enabled by default, so Arrow-backed conversions can now fail on unsafe casts that previously slipped through.
- If you are upgrading older pandas-on-Spark code, read the upstream upgrade guide before copying pre-4.x examples.

## Official Sources

- Apache Spark PySpark overview: `https://spark.apache.org/docs/latest/api/python/`
- Apache Spark install guide: `https://spark.apache.org/docs/latest/api/python/getting_started/install.html`
- Apache Spark DataFrame quickstart: `https://spark.apache.org/docs/latest/api/python/getting_started/quickstart_df.html`
- Apache Spark Spark Connect quickstart: `https://spark.apache.org/docs/latest/api/python/getting_started/quickstart_connect.html`
- Apache Spark testing guide: `https://spark.apache.org/docs/latest/api/python/getting_started/testing_pyspark.html`
- Apache Spark migration guides: `https://spark.apache.org/docs/latest/api/python/migration_guide/index.html`
- PySpark upgrade notes: `https://spark.apache.org/docs/latest/api/python/migration_guide/pyspark_upgrade.html`
- Spark configuration: `https://spark.apache.org/docs/latest/configuration.html`
- Spark submission guide: `https://spark.apache.org/docs/latest/submitting-applications.html`
- PyPI package page: `https://pypi.org/project/pyspark/`
