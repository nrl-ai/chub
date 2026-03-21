---
name: package
description: "pyparsing parser-combinator library for building PEG-style text parsers in Python"
metadata:
  languages: "python"
  versions: "3.3.2"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pyparsing,python,parsing,peg,text-processing,parser-combinators"
---

# pyparsing Python Package Guide

## What It Is

`pyparsing` is a parser-combinator library for building PEG-style grammars directly in Python code. It is a good fit for DSLs, config files, query languages, structured logs, and other cases where regular expressions become hard to maintain.

For `3.3.2`, write new code with the snake_case 3.x API: `parse_string`, `set_name`, `set_parse_action`, `DelimitedList`, `one_of`, and `using_each`. Legacy camelCase aliases still exist for compatibility in the 3.x line, but upstream documents them as deprecated and planned for removal in a future 4.0 release.

## Installation

```bash
pip install pyparsing==3.3.2
```

Common alternatives:

```bash
poetry add pyparsing==3.3.2
uv add pyparsing==3.3.2
```

Install railroad-diagram support if you need `create_diagram()` output:

```bash
pip install "pyparsing[diagrams]==3.3.2"
```

## Initialize And Set Up

There is no auth, network, or service configuration. The important setup choices are grammar style and parser behavior.

```python
import pyparsing as pp

# Upstream AI guidance recommends short namespaces for common helpers.
ppc = pp.common

# Optional: enable memoization for complex grammars with repeated backtracking.
# Call immediately after import if you decide to use it.
# pp.ParserElement.enable_packrat()

# Optional: enable bounded left recursion for genuinely left-recursive grammars.
# Do not combine with packrat.
# pp.ParserElement.enable_left_recursion()
```

Useful defaults for coding agents:

- Import as `pp`, and use `ppc = pp.common` when you need prebuilt numeric or date helpers.
- Use `parse_string(..., parse_all=True)` unless you intentionally want partial matches.
- Give important fields results names so parsed values are accessible as attributes or mapping keys.
- For nontrivial grammars, `pp.show_best_practices()` and `python -m pyparsing.ai.show_best_practices` expose upstream guidance aimed at parser authoring and testing.

## Core Usage

### Strict key-value parsing

```python
import pyparsing as pp

ppc = pp.common

identifier = pp.Word(pp.identchars, pp.identbodychars)
assignment = (
    identifier("name")
    + pp.Suppress("=")
    + ppc.integer("value")
)

result = assignment.parse_string("count=3", parse_all=True)

print(result.name)       # count
print(result["value"])   # 3
print(result.as_dict())  # {'name': 'count', 'value': 3}
```

Why this is a solid default:

- String literals such as `"="` are auto-converted to parser elements.
- `Suppress(...)` keeps punctuation out of the returned `ParseResults`.
- `pp.common.integer` converts the token to `int` for you.
- `parse_all=True` prevents silent trailing garbage.

### Delimited values and quoted strings

```python
import pyparsing as pp

identifier = pp.Word(pp.identchars, pp.identbodychars)
value = pp.common.number | pp.QuotedString('"')

pair = pp.Group(
    identifier("key")
    + pp.Suppress("=")
    + value("value")
)

config = pp.DelimitedList(pair, delim=",")

for item in config.parse_string('count=3, threshold=2.5, label="ok"', parse_all=True):
    print(item.key, item.value)
```

Reach for these building blocks first:

- `Word`, `Literal`, `Keyword`, `CaselessKeyword`
- `QuotedString`, `Regex`, `CharsNotIn`
- `Optional`, `ZeroOrMore`, `OneOrMore`, `DelimitedList`
- `Group`, `Dict`, `Suppress`, `Combine`
- `Forward` for recursive grammars
- `one_of(...)` and `infix_notation(...)` for token sets and expression grammars
- `pp.common` for numeric/date helpers with parse actions already attached

### Search, scan, and transform

```python
import pyparsing as pp

integer = pp.common.integer

print(integer.search_string("x=1 y=20 z=300").as_list())

title_case_word = pp.Word(pp.alphas)
title_case_word.set_parse_action(lambda toks: toks[0].title())
print(title_case_word.transform_string("now is the winter of our discontent"))
```

Use the right entry point:

- `parse_string(...)` for full structured inputs
- `parse_file(...)` for file contents
- `search_string(...)` to find repeated matches anywhere in larger text
- `scan_string(...)` when you also need start/end offsets
- `transform_string(...)` when parse actions should rewrite matched text

## Configuration And Global Behavior

### Whitespace

By default, pyparsing skips spaces, tabs, newlines, and carriage returns between tokens.

```python
import pyparsing as pp

pp.ParserElement.set_default_whitespace_chars(" \t")
```

That makes newlines significant. Reset to the default when you are done:

```python
pp.ParserElement.set_default_whitespace_chars(" \n\t\r")
```

For one expression instead of the whole process:

```python
token = pp.Word(pp.printables).leave_whitespace()
```

### Comments and ignored text

```python
import pyparsing as pp

expr = pp.Word(pp.alphas)[...].ignore(pp.c_style_comment)
```

`.ignore(...)` can be called repeatedly if your grammar needs to skip multiple comment styles.

### Inline literals

If your grammar has lots of separators, you can globally auto-suppress string literals:

```python
import pyparsing as pp

pp.ParserElement.inline_literals_using(pp.Suppress)

# ... build grammar ...

pp.ParserElement.inline_literals_using(pp.Literal)
```

This is process-wide state. Use it deliberately, especially in tests or shared libraries.

### Packrat vs left recursion

- `pp.ParserElement.enable_packrat()` adds memoization and can speed up complex grammars.
- `pp.ParserElement.enable_left_recursion()` enables bounded left recursion for left-recursive `Forward` grammars.
- Upstream documents these modes as incompatible with each other.
- Both memoization modes can skip repeated parse-action execution during backtracking, so do not depend on parse actions with order-sensitive side effects.

## Testing And Debugging

`run_tests()` is the fastest way to iterate on a small grammar:

```python
import pyparsing as pp

number = pp.common.number
number.run_tests(
    """
    100
    -3.5
    6.02e23
    """
)
```

Also useful:

- `set_name("...")` to improve parse errors
- `set_debug()` when you need to trace parser behavior
- `matches(text)` for quick boolean checks on a subexpression
- `result.dump()` while shaping `ParseResults`
- `create_diagram(...)` if you installed `pyparsing[diagrams]`

## Common Pitfalls

### Partial matches are allowed by default

`parse_string("count=3 trailing")` succeeds unless you pass `parse_all=True` or explicitly end the grammar with `StringEnd()`.

### Old examples use deprecated names

Legacy forms such as `parseString`, `setResultsName`, `delimitedList`, `enablePackrat`, and `leaveWhitespace` still appear in older blog posts and in pre-3.x code. Prefer snake_case APIs and `DelimitedList` in new code.

### `Forward` definitions are easy to get wrong

Use `<<=` for recursive assignment:

```python
expr = pp.Forward()
expr <<= ...
```

Do not replace the `Forward` with `expr = ...` after creating it, and be careful with operator precedence when combining `<<` or `<<=` with alternation.

### Tabs can break location-sensitive parse actions

`parse_string()` expands tabs before parsing so reported columns are accurate. If a parse action uses `loc` to index into the input string, either call `parse_with_tabs()` before parsing, use the full `(s, loc, toks)` signature, or expand tabs yourself first.

### Global settings leak across a process

`set_default_whitespace_chars(...)` and `inline_literals_using(...)` are global. They are convenient in standalone parsers but risky in shared libraries, test suites, or long-lived agent processes unless you isolate or reset them.

### Parse actions should not do fragile side effects

Memoization, backtracking, and left-recursion support can change when often parse actions are re-executed. Prefer pure parse actions that convert or annotate tokens rather than mutate global state, append to shared lists, or trigger I/O.

## Version-Sensitive Notes For 3.3.2

- PyPI lists `3.3.2` as the latest release, published on January 21, 2026.
- PyPI metadata says `pyparsing` requires Python `>=3.9` and publishes classifiers through Python `3.14`.
- The docs URL is still the right canonical docs root, but it currently serves a docs build labeled `PyParsing 3.3.0`, not `3.3.2`.
- The long-form "Using the pyparsing module" guide embedded in that docs site still shows its own internal revision as `3.2.0`. Treat that guide as conceptual guidance and check the API reference page for current names and signatures.
- The 3.x docs still document camelCase compatibility aliases, but upstream marks them as deprecated and slated for removal in a future 4.0 release. New code should use snake_case names only.
- Helpers you are likely to rely on in modern code, such as `DelimitedList`, `using_each`, and `show_best_practices`, are documented as additions from the 3.1 line and are available in `3.3.2`.
- If you need Python 3.8 or earlier, the upstream 3.2.x notes direct users to the `3.1` line for earlier Python 3 and to `2.4.7` for Python 2.

## Official Sources

- PyPI package page: https://pypi.org/project/pyparsing/
- PyPI release history: https://pypi.org/project/pyparsing/#history
- Docs root: https://pyparsing-docs.readthedocs.io/en/latest/
- How-to guide: https://pyparsing-docs.readthedocs.io/en/latest/HowToUsePyparsing.html
- API reference: https://pyparsing-docs.readthedocs.io/en/latest/pyparsing.html
- What changed in 3.2.x: https://pyparsing-docs.readthedocs.io/en/latest/whats_new_in_3_2.html
- Source repository: https://github.com/pyparsing/pyparsing
- Releases page: https://github.com/pyparsing/pyparsing/releases
