---
name: package
description: "NetworkX graph library for Python with practical guidance for graph types, algorithms, I/O, and backends"
metadata:
  languages: "python"
  versions: "3.6.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "networkx,python,graphs,algorithms,graphml,json,backends"
---

# NetworkX Python Package Guide

## What It Is

`networkx` is a pure-Python graph library for creating, mutating, analyzing, and serializing graphs. Use it when you need:

- in-memory graph data structures with node and edge attributes
- built-in graph algorithms such as shortest paths, centrality, connectivity, and traversal
- interchange with common graph formats such as edge lists, GraphML, GEXF, GML, and JSON

It does not handle authentication or service credentials because it is a local library, not a network client.

## Install

For the typical install recommended by the upstream docs:

```bash
python -m pip install "networkx[default]==3.6.1"
```

That pulls in common scientific-Python dependencies used by more of the library.

If you want the minimal pure-Python install:

```bash
python -m pip install "networkx==3.6.1"
```

If you use `uv`:

```bash
uv add "networkx[default]==3.6.1"
```

Optional extras commonly used with NetworkX:

- `pygraphviz` or `pydot` for Graphviz-based drawing and layouts
- `lxml` for GraphML support

```bash
python -m pip install pygraphviz pydot lxml
```

## Initialize And Choose A Graph Type

Import convention:

```python
import networkx as nx
```

Choose the graph class based on edge directionality and whether parallel edges matter:

- `nx.Graph()`: undirected, self-loops allowed, no parallel edges
- `nx.DiGraph()`: directed, self-loops allowed, no parallel edges
- `nx.MultiGraph()`: undirected, self-loops allowed, parallel edges allowed
- `nx.MultiDiGraph()`: directed, self-loops allowed, parallel edges allowed

Start with `Graph` or `DiGraph` unless you explicitly need multiple edges between the same node pair.

```python
import networkx as nx

G = nx.DiGraph(name="payments")
G.add_node("api", team="backend")
G.add_edge("api", "db", weight=2.5, relationship="writes")
```

Important node rules:

- nodes must be hashable Python objects
- `None` is not allowed as a node
- do not mutate a node object if its hash depends on mutable contents

## Core Usage

### Build A Graph With Attributes

```python
import networkx as nx

G = nx.Graph()

G.add_nodes_from(
    [
        ("sfo", {"city": "San Francisco"}),
        ("lax", {"city": "Los Angeles"}),
        ("las", {"city": "Las Vegas"}),
    ]
)

G.add_weighted_edges_from(
    [
        ("sfo", "lax", 337),
        ("lax", "las", 236),
        ("sfo", "las", 417),
    ],
    weight="distance_miles",
)

G.nodes["sfo"]["hub"] = True
G.edges["sfo", "lax"]["airline"] = "example"
```

### Run Algorithms

Most algorithms consume a graph plus attribute names such as `weight`.

```python
import networkx as nx

G = nx.DiGraph()
G.add_weighted_edges_from(
    [
        ("start", "a", 4),
        ("start", "b", 1),
        ("b", "a", 2),
        ("a", "finish", 1),
        ("b", "finish", 5),
    ]
)

path = nx.shortest_path(G, "start", "finish", weight="weight")
length = nx.shortest_path_length(G, "start", "finish", weight="weight")

print(path)    # ['start', 'b', 'a', 'finish']
print(length)  # 4
```

Useful patterns:

- use `G.nodes(data=True)` and `G.edges(data=True)` when attributes matter
- pass `weight="..."` explicitly for weighted algorithms
- prefer graph views for temporary filtering, and `.copy()` if you need an isolated graph you can mutate

### Convert From Tabular Data

If you already have a Pandas edge list:

```python
import networkx as nx
import pandas as pd

df = pd.DataFrame(
    [
        {"src": "a", "dst": "b", "weight": 3},
        {"src": "b", "dst": "c", "weight": 5},
    ]
)

G = nx.from_pandas_edgelist(
    df,
    source="src",
    target="dst",
    edge_attr=["weight"],
    create_using=nx.DiGraph,
)
```

## Read, Write, And Interoperate

NetworkX supports many formats out of the box, including edge lists, GraphML, GEXF, GML, Pajek, JSON, and Matrix Market.

### JSON For Web And API Interop

Use node-link JSON when you need a JSON-serializable representation:

```python
import json
import networkx as nx

G = nx.Graph([("A", "B")])

payload = nx.node_link_data(G)
raw_json = json.dumps(payload)

restored = nx.node_link_graph(json.loads(raw_json))
```

In `3.6.1`, the supported keyword names on `node_link_data` are `source=`, `target=`, `name=`, `key=`, `edges=`, and `nodes=`.

### GraphML

Use GraphML when you need a richer graph exchange format:

```python
import networkx as nx

G = nx.path_graph(4)
nx.write_graphml(G, "graph.graphml")

loaded = nx.read_graphml("graph.graphml")
```

Notes that matter in practice:

- `write_graphml()` uses `lxml`
- `read_graphml()` may return `MultiGraph` or `MultiDiGraph` if the file contains parallel edges
- GraphML default node and edge attributes are stored in `G.graph` and are not automatically copied onto every node or edge

## Config And Backends

NetworkX itself has no auth or credential setup.

Configuration that matters most in coding sessions is backend dispatch. NetworkX can route some operations to separately installed backends for better performance or specialized execution while keeping the same Python API.

Relevant controls from the official backend docs:

- per-call backend selection: `backend="parallel"` on supported functions
- `NETWORKX_BACKEND_PRIORITY`: backend order for dispatchable algorithm calls
- `NETWORKX_BACKEND_PRIORITY_GENERATORS`: backend order for graph-producing functions
- `NETWORKX_FALLBACK_TO_NX`: fall back to default NetworkX implementations when a backend graph reaches an unsupported function
- `NETWORKX_CACHE_CONVERTED_GRAPHS`: whether converted backend graphs are cached

Example:

```bash
NETWORKX_BACKEND_PRIORITY=cugraph python my_script.py
```

Use backend configuration only after confirming the backend is installed and supports the functions you call. Conversion and caching can improve speed, but they also add overhead and memory use.

## Common Pitfalls

- `G.add_nodes_from("spam")` adds four nodes: `"s"`, `"p"`, `"a"`, `"m"`. Use `G.add_node("spam")` for one string node.
- Nodes must be hashable, and `None` is reserved by the library for optional arguments.
- `from_pandas_edgelist()` iterates over `DataFrame.values`; rows with mixed numeric types can be coerced to floats.
- `read_graphml()` and the GraphML parser should only be used on trusted files because the parser relies on Python's standard XML library.
- GraphML default attributes live in `G.graph["node_default"]` and `G.graph["edge_default"]`; they are not pushed into every node and edge automatically.
- `G.nodes`, `G.edges`, `G.adj`, and `G.degree` are live views into the graph. Materialize them with `list(...)` or `dict(...)` if you need snapshot behavior.

## Version-Sensitive Notes For 3.6.1

- NetworkX explicitly says it does not use semantic versioning. Point releases can include minor API breakage, so pin exact versions for tooling-sensitive code instead of relying on `~=`.
- The `3.6` line replaced `random_lobster` with `random_lobster_graph` and `maybe_regular_expander` with `maybe_regular_expander_graph`.
- The deprecated `link` keyword in node-link JSON helpers expired in `3.6`; use `edges=` and `nodes=` instead.
- This guide is for `3.6.1`, which is also the stable documentation version currently published at the upstream docs URL.

## Official Sources Used For This Guide

- Install: `https://networkx.org/documentation/stable/install.html`
- Tutorial: `https://networkx.org/documentation/stable/tutorial.html`
- Graph types: `https://networkx.org/documentation/stable/reference/classes/index.html`
- Backends: `https://networkx.org/documentation/stable/reference/backends.html`
- Read/write overview: `https://networkx.org/documentation/stable/reference/readwrite/index.html`
- `read_graphml`: `https://networkx.org/documentation/stable/reference/readwrite/generated/networkx.readwrite.graphml.read_graphml.html`
- `write_graphml`: `https://networkx.org/documentation/stable/reference/readwrite/generated/networkx.readwrite.graphml.write_graphml.html`
- `node_link_data`: `https://networkx.org/documentation/stable/reference/readwrite/generated/networkx.readwrite.json_graph.node_link_data.html`
- `from_pandas_edgelist`: `https://networkx.org/documentation/stable/reference/generated/networkx.convert_matrix.from_pandas_edgelist.html`
- Releases: `https://networkx.org/documentation/stable/release/index.html`
- `3.6` release notes: `https://networkx.org/documentation/stable/release/release_3.6.html`
- PyPI: `https://pypi.org/project/networkx/`
