---
name: package
description: "JMESPath Python package guide for querying JSON-like data with expressions, filters, functions, and custom options"
metadata:
  languages: "python"
  versions: "1.1.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "jmespath,json,query,python,filtering"
---

# jmespath Python Package Guide

## Golden Rule

Use `jmespath` when you need to query JSON-like Python data with a declarative expression instead of writing nested loops and conditionals by hand.

- Import: `import jmespath`
- Primary APIs: `jmespath.search()` and `jmespath.compile()`
- Input shape: plain Python dict/list data, typically from `json.loads()` or API responses already decoded into Python objects
- Network/auth: none. This package does not talk to a service or manage credentials; it only evaluates expressions against local data

## Installation

```bash
pip install jmespath==1.1.0
```

For a floating install:

```bash
pip install jmespath
```

## Initialization And Setup

There is no client object, auth flow, or global configuration step.

```python
import jmespath
```

For one-off queries, call `search()` directly:

```python
import jmespath

data = {
    "foo": {
        "bar": [
            {"name": "one"},
            {"name": "two"},
        ]
    }
}

result = jmespath.search("foo.bar[-1].name", data)
print(result)  # "two"
```

For repeated evaluation of the same expression, compile once and reuse it:

```python
import jmespath

expression = jmespath.compile("foo.bar")

print(expression.search({"foo": {"bar": "baz"}}))   # "baz"
print(expression.search({"foo": {"bar": "qux"}}))   # "qux"
```

Use `compile()` when the expression string is stable and you are applying it many times in a loop or across many records.

## Core Usage

### Basic field, index, and slice access

```python
import jmespath

data = {
    "people": [
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 19},
        {"name": "Carol", "age": 41},
    ]
}

print(jmespath.search("people[0].name", data))   # "Alice"
print(jmespath.search("people[-1].name", data))  # "Carol"
print(jmespath.search("people[:2].[name, age]", data))
```

### Filters

Use a filter projection when you need to keep only array items that match a predicate.

```python
import jmespath

data = {
    "people": [
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 19},
        {"name": "Carol", "age": 41},
    ]
}

adults = jmespath.search("people[?age >= `21`].name", data)
print(adults)  # ["Alice", "Carol"]
```

Important syntax details:

- Use backticks for JSON literals such as numbers, booleans, arrays, objects, and `null`
- Use raw string literals inside single quotes for string comparisons, for example: `"tags[?Key=='Name']"`

### Projections and pipes

Projections are the main way to map over arrays.

```python
import jmespath

data = {
    "reservations": [
        {
            "instances": [
                {"type": "small", "state": {"name": "running"}},
                {"type": "medium", "state": {"name": "stopped"}},
            ]
        }
    ]
}

states = jmespath.search("reservations[].instances[].state.name", data)
print(states)  # ["running", "stopped"]
```

Use a pipe when you want the output of one expression to become the input to the next step and you want to stop the current projection before continuing:

```python
import jmespath

data = {
    "tags": [
        {"Key": "Name", "Values": ["api-a"]},
        {"Key": "Owner", "Values": ["platform"]},
    ]
}

name = jmespath.search("tags[?Key=='Name'] | [0].Values[]", data)
print(name)  # "api-a"
```

### Multiselects and reshaping

Use multiselect lists or hashes when you want to reshape the output into exactly the fields your code needs.

```python
import jmespath

data = {
    "people": [
        {"name": "Alice", "age": 30, "city": "LA"},
        {"name": "Bob", "age": 19, "city": "NYC"},
    ]
}

rows = jmespath.search("people[].{name: name, age: age}", data)
print(rows)
```

### Built-in functions

JMESPath includes built-in functions such as `contains`, `length`, `sort`, `sort_by`, `to_number`, `keys`, and `values`.

```python
import jmespath

data = {
    "items": [
        {"name": "b", "price": "12"},
        {"name": "a", "price": "7"},
        {"name": "c", "price": "20"},
    ]
}

sorted_names = jmespath.search("sort_by(items, &to_number(price))[].name", data)
print(sorted_names)  # ["a", "b", "c"]
```

Use `&expr` with functions like `sort_by`, `max_by`, and `min_by`. That `&` creates an expression reference; without it, the function call is wrong.

## Configuration

`jmespath.Options` controls evaluation behavior for the Python implementation.

### Preserve key order in returned mappings

If you want multiselect-hash results as `OrderedDict` instead of plain `dict`, set `dict_cls`:

```python
import collections
import jmespath

data = {"foo": {"b": "first", "a": "second"}}

result = jmespath.search(
    "{b: foo.b, a: foo.a}",
    data,
    options=jmespath.Options(dict_cls=collections.OrderedDict),
)

print(type(result))
```

### Custom functions

The Python implementation supports custom functions through `jmespath.functions.Functions` plus `jmespath.Options(custom_functions=...)`.

```python
import jmespath
from jmespath import functions

class CustomFunctions(functions.Functions):
    @functions.signature({"types": ["string"]})
    def _func_reverse(self, value):
        return value[::-1]

options = jmespath.Options(custom_functions=CustomFunctions())
result = jmespath.search("reverse(name)", {"name": "jmespath"}, options=options)
print(result)  # "htapsemj"
```

Upstream marks custom function support as experimental. Use it only when you control the execution environment and do not need cross-language portability.

## Common Pitfalls

### Missing values can disappear inside projections

When a projected subexpression returns `null`, that element is omitted from the collected result. If you expect positional alignment with the source array, do not assume a one-to-one output length after a projection.

### Quote identifiers that contain punctuation

If a key contains characters that are not valid in an unquoted identifier, quote it in the expression:

```python
import jmespath

data = {"headers": {"content-type": "application/json"}}
print(jmespath.search('headers."content-type"', data))
```

### Use the right literal syntax

- String literal in the expression language: `'Name'`
- Number / boolean / null / JSON literal: `` `21` ``, `` `true` ``, `` `null` ``

In Python source code, prefer double quotes around the whole expression when it already contains JMESPath single-quoted strings:

```python
expr = "tags[?Key=='Name']"
```

### Object wildcards are not ordered

JMESPath treats object values as unordered. If you need stable ordering, convert the data into an array before querying or sort the result explicitly.

### Ordering comparisons are numeric in the spec

For filter expressions like `[?age > \`20\`]`, the ordering operators are defined for numbers. If the source value is a numeric string, convert it with `to_number(...)` before sorting or aggregating.

### `compile()` does not validate your data shape

Compiling an expression avoids reparsing the expression string; it does not guarantee that a later input document has the keys or types you expect. Validate the data contract separately when the payload schema is uncertain.

## Version-Sensitive Notes For 1.1.0

- PyPI shows `1.1.0` released on `2026-01-22`
- PyPI lists Python requirement `>=3.9`
- Older blog posts often describe very old Python compatibility or pre-1.x behavior; prefer the current PyPI page and `jmespath.org` language/spec pages when syntax details conflict
- Custom functions remain implementation-specific and experimental in the Python package, even though the base JMESPath expression language is standardized

## Recommended Agent Workflow

1. Parse JSON into normal Python dict/list objects first.
2. Start with `jmespath.search()` for a one-off expression.
3. Switch to `jmespath.compile()` when reusing the same expression repeatedly.
4. Use multiselects to shape the output your code needs instead of post-processing large nested results.
5. Recheck filters, quoting, and numeric conversions before blaming the data source; most broken queries come from expression syntax mismatches, not from Python itself.

## Official Sources

- Docs root: https://jmespath.org/
- Tutorial: https://jmespath.org/tutorial.html
- Examples: https://jmespath.org/examples.html
- Specification: https://jmespath.org/specification.html
- PyPI package page: https://pypi.org/project/jmespath/
- Upstream repository: https://github.com/jmespath/jmespath.py
