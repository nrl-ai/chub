---
name: package
description: "sqlparse Python package guide for parsing, splitting, formatting, and inspecting SQL statements"
metadata:
  languages: "python"
  versions: "0.5.5"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "sqlparse,sql,parser,formatter,python,linting"
---

# sqlparse Python Package Guide

## Golden Rule

Use `sqlparse` when you need lightweight SQL splitting, formatting, or token inspection inside Python. It is intentionally non-validating, so do not treat it as a full SQL parser, a query planner, or a safety check for untrusted SQL semantics.

## Install

Pin the package version your project expects:

```bash
python -m pip install "sqlparse==0.5.5"
```

Common alternatives:

```bash
uv add "sqlparse==0.5.5"
poetry add "sqlparse==0.5.5"
```

If you only need the CLI formatter in CI or pre-commit, the same package install is enough; there is no separate formatter package.

## Initialize And Setup

`sqlparse` is a pure Python library. There is no network setup, database connection, or authentication step.

Typical import:

```python
import sqlparse
```

Important behavior to remember before writing code around it:

- `sqlparse` is non-validating. It tries to analyze SQL-like text without enforcing full dialect correctness.
- The top-level API is small: `split()`, `format()`, and `parse()`.
- `parse()` returns `Statement` objects backed by token trees, not AST nodes with semantic validation.

## Core Usage

### Split a script into individual statements

Use `split()` when you want statement boundaries without writing your own semicolon logic:

```python
import sqlparse

raw_sql = """
BEGIN TRANSACTION;
INSERT INTO users (id, email) VALUES (1, 'a@example.com');
COMMIT;
SELECT * FROM users;
"""

statements = sqlparse.split(raw_sql)

for statement in statements:
    print(repr(statement))
```

If your downstream consumer does not want trailing semicolons, use `strip_semicolon=True`:

```python
statements = sqlparse.split(raw_sql, strip_semicolon=True)
```

### Format SQL for readability or normalization

`format()` is the main entry point for SQL pretty-printing:

```python
import sqlparse

sql = "select id,name from users where active=1 order by created_at desc"

formatted = sqlparse.format(
    sql,
    reindent=True,
    keyword_case="upper",
    use_space_around_operators=True,
)

print(formatted)
```

Common formatting options from the official API docs:

- `keyword_case="upper" | "lower" | "capitalize"`
- `identifier_case="upper" | "lower" | "capitalize"`
- `strip_comments=True`
- `truncate_strings=<int>`
- `truncate_char="[...]"`
- `reindent=True`
- `reindent_aligned=True`
- `use_space_around_operators=True`
- `indent_tabs=True`
- `indent_width=2`
- `wrap_after=<column_count>`
- `compact=True`
- `output_format="python" | "php"`
- `comma_first=True`

Example tuned for embedding SQL into Python source:

```python
import sqlparse

sql = "select id, email from users where active = 1 order by created_at desc"

python_literal = sqlparse.format(
    sql,
    reindent=True,
    keyword_case="upper",
    output_format="python",
)

print(python_literal)
```

### Parse SQL into statements and inspect tokens

Use `parse()` when you need structural inspection rather than just formatting:

```python
import sqlparse

statement = sqlparse.parse(
    "select u.id, u.email from users u where u.active = 1"
)[0]

print(statement.get_type())  # SELECT

for token in statement.tokens:
    print(type(token).__name__, repr(str(token)))
```

The official docs describe `Token` and `TokenList` as the base classes. Useful methods for agent-written code include:

- `Statement.get_type()`
- `TokenList.flatten()`
- `TokenList.token_first()`
- `TokenList.token_next()` / `token_prev()`
- `TokenList.get_alias()`
- `TokenList.get_name()`
- `TokenList.get_real_name()`
- `IdentifierList.get_identifiers()`
- `Token.match(...)`
- `Token.within(...)`

Example: walk identifiers from a simple `SELECT` list.

```python
import sqlparse
from sqlparse.sql import IdentifierList, Identifier

statement = sqlparse.parse(
    "select u.id, u.email, count(*) as total from users u"
)[0]

for token in statement.tokens:
    if isinstance(token, IdentifierList):
        for identifier in token.get_identifiers():
            print(identifier.get_name(), identifier.get_real_name())
    elif isinstance(token, Identifier):
        print(token.get_name(), token.get_real_name())
```

### Convert parsed fragments back to strings

Parsed objects preserve the original token values, so `str(...)` is the normal way to reconstruct fragments:

```python
import sqlparse

statement = sqlparse.parse(
    'select * from "public"."users" where id = 1'
)[0]

where_clause = statement.tokens[-1]

print(str(statement))
print(str(where_clause))
```

## CLI And Pre-commit Usage

The package ships the `sqlformat` CLI:

```bash
sqlformat --help
```

Format one or more SQL files in place:

```bash
sqlformat --in-place --reindent --keywords upper queries/report.sql
```

The docs also describe a `pre-commit` hook:

```yaml
repos:
  - repo: https://github.com/andialbrecht/sqlparse
    rev: 0.5.5
    hooks:
      - id: sqlformat
        args: [--in-place, --reindent, --keywords, upper]
```

If you override hook args, keep `--in-place` first or the hook will not rewrite files.

## Dialect Customization

`sqlparse` supports many SQL dialects loosely, but the lexer can be customized when a project uses dialect-specific keywords or operators.

The official extension docs describe configuring the singleton lexer by clearing the defaults, loading the default regex/keyword tables, and then appending custom keywords or regex rules. Use this only when formatting or parsing fails because your dialect uses syntax `sqlparse` does not know yet.

This is an advanced escape hatch. Prefer upstream defaults unless you have a narrow, repeatable dialect need.

## Configuration And Safety Notes

- There is no auth configuration. Any "setup" beyond installation is local process behavior only.
- `encoding` is optional on `split()`, `format()`, and `parse()`. If omitted, the docs say `sqlparse` assumes the input is UTF-8 or Latin-1.
- For large or untrusted SQL inputs, `sqlparse` now includes grouping limits intended to reduce DoS risk:
  - `sqlparse.engine.grouping.MAX_GROUPING_DEPTH = 100`
  - `sqlparse.engine.grouping.MAX_GROUPING_TOKENS = 10000`
- You can raise or disable those limits, but the docs explicitly warn that doing so can reintroduce denial-of-service risk for untrusted input.

Example of controlled limit tuning:

```python
import sqlparse.engine.grouping

sqlparse.engine.grouping.MAX_GROUPING_DEPTH = 200
sqlparse.engine.grouping.MAX_GROUPING_TOKENS = 50000
```

Only do that for trusted inputs that are legitimately large.

## Common Pitfalls

- Do not assume `sqlparse` validates SQL. Invalid or dialect-specific SQL can still tokenize into something that looks usable.
- `parse()` returns a tuple of `Statement` instances, not a single object.
- Token lists include whitespace and comments unless you explicitly skip them with helpers like `token_first(skip_ws=True, skip_cm=True)`.
- `strip_comments=True` does not mean "also normalize whitespace". Since `0.5.1`, some cases that previously looked cleaned up may still need `strip_whitespace=True`.
- If you override `pre-commit` hook args, include `--in-place` or the hook will not modify files.
- The docs examples still show `rev: 0.5.4` in the pre-commit snippet. For a project pinned to `0.5.5`, use `rev: 0.5.5`.
- `split()` is much safer than manual `sql.split(";")`, especially around `BEGIN ... END` style blocks.
- If you inspect identifiers, remember that aliases and real object names differ. Use `get_name()` and `get_real_name()` intentionally.

## Version-Sensitive Notes For 0.5.x

- `0.5.5` fixes DoS protection so grouping-limit failures raise `SQLParseError` instead of silently returning `None`, and it fixes splitting of `BEGIN TRANSACTION` statements.
- `0.5.4` added Python 3.14 support, top-level type annotations with a `py.typed` marker, pre-commit hook support, CLI multi-file handling, and the `--in-place` formatter flag.
- `0.5.4` also introduced documented grouping limits for token processing to harden parsing of very large inputs.
- `0.5.3` broadened protection against recursion-related DoS for deeply nested statements.
- `0.5.1` added the `compact=True` formatter option and changed some `strip_comments` behavior, so older formatting snippets can differ from current output.
- `0.5.0` dropped Python 3.5, 3.6, and 3.7 support. For `0.5.5`, PyPI declares `Requires-Python >=3.8`.

## Practical Guidance For Agents

- Use `split()` for statement boundaries, `format()` for normalization, and `parse()` only when you actually need token-level inspection.
- Prefer matching on token classes and helper methods instead of raw string slicing.
- If you need exact SQL semantics, query planning, or dialect validation, pair `sqlparse` with a database engine or a stricter parser rather than stretching it beyond its design.
