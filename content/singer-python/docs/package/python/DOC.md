---
name: package
description: "Singer Python utility library for building Singer taps and targets that emit schemas, records, state, and metadata"
metadata:
  languages: "python"
  versions: "6.4.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "singer,etl,elt,tap,target,json-schema,state"
---

# singer-python Python Package Guide

## Golden Rule

Use `singer-python` as a helper library inside a Singer tap or target, import it as `import singer`, and keep the Singer message stream clean on stdout. Use `singer` write helpers for `SCHEMA`, `RECORD`, `STATE`, and version messages, and send logs through `singer.get_logger()` so they go to stderr instead of corrupting the stream.

## Install

Use an isolated environment and pin the version you actually want:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "singer-python==6.4.0"
```

Common alternatives:

```bash
uv add "singer-python==6.4.0"
poetry add "singer-python==6.4.0"
```

The upstream README still shows a source checkout plus `make install`, but for normal tap or target development the PyPI package is the practical starting point.

## Standard CLI Setup

`parse_args(required_config_keys)` is the main setup helper for Singer taps. It handles the standard Singer CLI flags, loads JSON files for you, and validates that required config keys are present.

Supported standard flags:

- `--config` / `-c`: required JSON config file
- `--state` / `-s`: optional JSON state file, defaulting to an empty dict
- `--catalog`: optional catalog file, loaded into a `Catalog` object
- `--properties` / `-p`: deprecated legacy selection file
- `--discover` / `-d`: discovery mode
- `--dev`: dev mode flag

Minimal tap skeleton:

```python
import singer
from singer import metadata

LOGGER = singer.get_logger()
REQUIRED_CONFIG_KEYS = ["api_token", "start_date"]

def user_schema() -> dict:
    return {
        "type": "object",
        "properties": {
            "id": {"type": ["null", "integer"]},
            "email": {"type": ["null", "string"]},
            "updated_at": {"type": ["null", "string"], "format": "date-time"},
        },
    }

def discover() -> None:
    schema = user_schema()
    singer.write_schema(
        "users",
        schema,
        key_properties=["id"],
        bookmark_properties=["updated_at"],
    )

def sync(config: dict, state: dict) -> None:
    schema = user_schema()
    raw_metadata = metadata.get_standard_metadata(
        schema=schema,
        key_properties=["id"],
        valid_replication_keys=["updated_at"],
        replication_method="INCREMENTAL",
    )
    compiled_metadata = metadata.to_map(raw_metadata)

    bookmark = singer.get_bookmark(state, "users", "updated_at") or config["start_date"]

    for raw_record in fetch_users_since(config["api_token"], bookmark):
        record = singer.transform(raw_record, schema, metadata=compiled_metadata)
        singer.write_record("users", record)

        if record.get("updated_at"):
            singer.set_bookmark(state, "users", "updated_at", record["updated_at"])

    singer.write_state(state)

@singer.utils.handle_top_exception(LOGGER)
def main() -> None:
    args = singer.parse_args(REQUIRED_CONFIG_KEYS)

    if args.discover:
        discover()
        return

    sync(args.config, args.state)

if __name__ == "__main__":
    main()
```

Implementation notes:

- `args.config` is already parsed JSON, not a file path string.
- `args.state` is already parsed JSON and defaults to `{}` when no state file is provided.
- `args.catalog` is already loaded as a `Catalog` object when `--catalog` is passed.

## Core Message Writing

The most common helpers are exported at the package top level:

- `singer.write_schema(stream_name, schema, key_properties, bookmark_properties=None)`
- `singer.write_record(stream_name, record, stream_alias=None, time_extracted=None)`
- `singer.write_records(stream_name, records)`
- `singer.write_state(state_dict)`
- `singer.write_version(stream_name, version)`

Typical flow:

1. Emit one `SCHEMA` message per stream before records.
2. Emit `RECORD` messages as dictionaries.
3. Emit periodic `STATE` messages, especially after successful bookmark updates.

Example:

```python
import singer

schema = {
    "type": "object",
    "properties": {
        "id": {"type": ["null", "integer"]},
        "name": {"type": ["null", "string"]},
    },
}

singer.write_schema("users", schema, ["id"])
singer.write_record("users", {"id": 1, "name": "Ada"})
singer.write_state({"bookmarks": {"users": {"updated_at": "2026-03-12T00:00:00Z"}}})
```

If you need experimental versioned streams, `RecordMessage` supports a `version` field, but `write_record()` itself does not take a `version` argument. In that case, build a `RecordMessage` and pass it to `write_message()` directly.

## Metadata And Catalog Selection

The metadata helpers live in the `singer.metadata` module, not as top-level functions.

The common pattern is:

1. Build raw metadata with `metadata.get_standard_metadata(...)`
2. Convert it with `metadata.to_map(...)`
3. Pass the compiled metadata into `singer.transform(...)`

Example:

```python
from singer import metadata
import singer

schema = {
    "type": "object",
    "properties": {
        "id": {"type": ["null", "integer"]},
        "email": {"type": ["null", "string"]},
    },
}

raw_metadata = metadata.get_standard_metadata(
    schema=schema,
    key_properties=["id"],
)
compiled_metadata = metadata.to_map(raw_metadata)

record = singer.transform(
    {"id": "123", "email": "user@example.com"},
    schema,
    metadata=compiled_metadata,
)
```

`metadata.get_standard_metadata(...)` marks key fields as `automatic` and regular fields as `available`, which is what `singer.transform(...)` uses when it filters fields by selection metadata.

## Record Transformation And Type Coercion

`singer.transform(...)` and `Transformer` are useful when your source payload needs to be coerced to the schema you are about to emit.

Useful behaviors in `6.4.0`:

- strings can be coerced to `integer`, `number`, and `boolean`
- schema `anyOf` handling is supported
- arrays and nested objects are transformed recursively
- unsupported or unselected fields can be filtered via metadata
- `format: singer.decimal` is handled specially

Example:

```python
import singer

schema = {
    "type": "object",
    "properties": {
        "id": {"type": ["null", "integer"]},
        "active": {"type": ["null", "boolean"]},
        "amount": {"type": ["null", "number"]},
    },
}

record = singer.transform(
    {"id": "42", "active": "false", "amount": "19.95"},
    schema,
)

assert record == {"id": 42, "active": False, "amount": 19.95}
```

## State And Bookmark Management

For incremental syncs, use the exported state helpers instead of mutating nested state structures ad hoc.

Common helpers:

- `singer.get_bookmark(state, stream, key)`
- `singer.set_bookmark(state, stream, key, value)`
- `singer.clear_bookmark(state, stream, key)`
- `singer.reset_stream(state, stream)`
- `singer.get_offset(state, stream)`
- `singer.set_offset(state, stream, value)`
- `singer.clear_offset(state, stream)`
- `singer.get_currently_syncing(state)`
- `singer.set_currently_syncing(state, stream)`
- `singer.get_version(state, stream)`
- `singer.set_version(state, stream, version)`
- `singer.clear_version(state, stream)`

Typical incremental pattern:

```python
bookmark = singer.get_bookmark(state, "users", "updated_at") or config["start_date"]

for record in fetch_users_since(config["api_token"], bookmark):
    singer.write_record("users", record)
    singer.set_bookmark(state, "users", "updated_at", record["updated_at"])

singer.write_state(state)
```

## Config And Auth

`singer-python` does not implement authentication for you. Your tap owns auth, retries, request signing, and API-specific pagination behavior. The library helps with argument parsing, state, metadata, and message emission.

Practical config rules:

- keep credentials in the JSON config file or inject them into that file from environment variables before execution
- validate required keys through `parse_args(REQUIRED_CONFIG_KEYS)`
- never emit credentials in `RECORD`, `STATE`, or log output
- keep transient request headers and tokens out of Singer state unless they are part of a durable resume strategy

Typical config shape:

```json
{
  "api_token": "env-or-secret-manager-value",
  "start_date": "2026-01-01T00:00:00Z",
  "base_url": "https://api.example.com"
}
```

## Logging, Errors, And Output Hygiene

Use `singer.get_logger()` and `@singer.utils.handle_top_exception(LOGGER)` for top-level error handling. That decorator logs each exception line at critical level and re-raises the error.

Important rule:

- do not use plain `print()` for diagnostics in a tap or target unless you are deliberately writing Singer messages

Singer messages belong on stdout. Operational logs, warnings, and tracebacks must stay off the data stream.

## Common Pitfalls

- `--config` must be JSON. `parse_args()` uses `json.load`, so YAML or dotenv files will fail unless your own wrapper converts them first.
- `--properties` is deprecated. Prefer `--catalog` for field selection.
- `metadata.get_standard_metadata(...)` returns a list form. Convert it with `metadata.to_map(...)` before passing it into `singer.transform(...)`.
- `write_schema()` requires `key_properties` to be a string or list of strings. `bookmark_properties` follows the same expectation.
- If you pass `time_extracted`, it must be a timezone-aware `datetime`.
- Do not interleave log lines with Singer messages on stdout. That is one of the easiest ways to break downstream targets.
- `singer.strptime()` is deprecated in the utility module. Prefer `singer.utils.strptime_to_utc(...)` or your own timezone-safe parsing.
- `write_record()` does not expose the experimental `version` field. Use `RecordMessage` plus `write_message()` if you truly need versioned streams.

## Version-Sensitive Notes

For installable `6.4.0`:

- `clear_offset` behavior was updated in `6.4.0` to remove the offset key from the bookmark structure.
- `allow_nan` support in message JSON output landed in `6.3.0`.
- JSON schema generation landed in `6.2.0`, with several fixes in `6.2.1` through `6.2.3` for dates, empty arrays, `anyOf`, and non-standard data types.

For newer code that may be visible in the maintainer repo but not yet on PyPI:

- `6.5.0`: `bookmarks.py` is deprecated and functions move to `state.py`; activate-version state helpers are added.
- `6.6.0`: state helper functions are exported from `singer`, and `write_bookmark` is renamed to `set_bookmark` with a backward-compatible alias.
- `6.7.0`: `set_version`, `get_version`, and `clear_version` remove the `key` parameter.
- `6.8.0`: the state key `activate_versions` is renamed to `versions`.

If you are working against a project pin that really is newer than `6.4.0`, verify the installed package version before copying state-management examples from older taps.

## Official Sources

- Maintainer repository: `https://github.com/singer-io/singer-python`
- Package registry: `https://pypi.org/project/singer-python/`
- Singer specification context: `https://github.com/singer-io/getting-started`
